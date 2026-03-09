use std::collections::{HashMap, HashSet};
use std::env;
use std::io::Write as _;
use std::process::{Command, Stdio};

use alloy::dyn_abi::eip712::TypedData;
use alloy::primitives::{Address, B256, ChainId, Signature, U256};
use alloy::signers::local::PrivateKeySigner;
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// --- CWP Protocol Types ---

#[derive(Debug, Deserialize)]
struct CwpInfo {
    name: String,
}

#[derive(Debug, Deserialize)]
struct AccountEntry {
    address: String,
}

#[derive(Debug, Deserialize)]
struct AccountsResponse {
    accounts: Vec<AccountEntry>,
}

#[derive(Debug, Serialize)]
struct SignHashInput {
    account: String,
    hash: String,
}

#[derive(Debug, Deserialize)]
struct SignatureResponse {
    signature: String,
}

#[derive(Serialize)]
struct CwpTransaction {
    to: String,
    data: String,
    value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    gas: Option<String>,
}

#[derive(Serialize)]
struct SendTransactionInput {
    account: String,
    chain: String,
    transaction: CwpTransaction,
}

#[derive(Deserialize)]
struct SendTransactionResponse {
    #[serde(rename = "transactionHash")]
    transaction_hash: String,
}

// --- CWP Protocol Client ---

fn cwp_exec(binary: &str, operation: &str, input: Option<&serde_json::Value>) -> Result<serde_json::Value> {
    let mut cmd = Command::new(binary);
    cmd.arg(operation)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().with_context(|| format!("Failed to spawn {binary}"))?;

    if let Some(data) = input {
        if let Some(mut stdin) = child.stdin.take() {
            let json = serde_json::to_vec(data)?;
            stdin.write_all(&json)?;
        }
    } else {
        drop(child.stdin.take());
    }

    let output = child
        .wait_with_output()
        .context("Failed to read CWP output")?;

    if cfg!(debug_assertions) {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        if !stderr_str.is_empty() {
            eprintln!("[debug] {binary} {operation} stderr: {stderr_str}");
        }
    }

    match output.status.code() {
        Some(0) => {}
        Some(2) => bail!("CWP operation '{operation}' not supported by {binary}"),
        Some(3) => bail!("User rejected the signing request"),
        Some(4) => bail!("CWP operation timed out"),
        Some(5) => bail!("Wallet not connected"),
        Some(code) => bail!("CWP binary {binary} exited with code {code}"),
        None => bail!("CWP binary {binary} was terminated by signal"),
    }

    let stdout = String::from_utf8(output.stdout)
        .context("CWP output is not valid UTF-8")?;
    serde_json::from_str(stdout.trim())
        .context("CWP output is not valid JSON")
}

// --- Discovery ---

pub struct CwpProvider {
    pub binary: String,
    pub name: String,
}

pub fn discover() -> Vec<CwpProvider> {
    let path_var = match env::var("PATH") {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let mut seen = HashSet::new();
    let mut providers = Vec::new();

    for dir in env::split_paths(&path_var) {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if !name_str.starts_with("wallet-") {
                continue;
            }

            if !seen.insert(name_str.to_string()) {
                continue;
            }

            let binary = name_str.to_string();

            // Try to get info
            let info = match Command::new(&binary)
                .arg("info")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
            {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    serde_json::from_str::<CwpInfo>(stdout.trim()).ok()
                }
                _ => None,
            };

            let display_name = info
                .map(|i| i.name)
                .unwrap_or_else(|| name_str.strip_prefix("wallet-").unwrap_or(&name_str).to_string());

            providers.push(CwpProvider {
                binary,
                name: display_name,
            });
        }
    }

    providers
}

// --- CwpSigner ---

#[derive(Clone)]
pub struct CwpSigner {
    binary: String,
    address: Address,
    chain_id: Option<ChainId>,
}

impl CwpSigner {
    pub fn new(binary: &str, address: Address, chain_id: Option<ChainId>) -> Self {
        Self {
            binary: binary.to_string(),
            address,
            chain_id,
        }
    }

    fn sign_hash_blocking(&self, hash: &B256) -> Result<Signature> {
        let input = serde_json::to_value(SignHashInput {
            account: self.address.to_checksum(None),
            hash: format!("{hash}"),
        })?;

        let output = cwp_exec(&self.binary, "sign-hash", Some(&input))?;
        let resp: SignatureResponse =
            serde_json::from_value(output).context("Invalid sign-hash response")?;

        parse_signature(&resp.signature)
    }

    fn send_transaction_blocking(
        &self,
        to: Address,
        data: Vec<u8>,
        value: U256,
        gas: Option<u64>,
    ) -> Result<B256> {
        let chain = format!(
            "eip155:{}",
            self.chain_id.unwrap_or(137)
        );
        let input = serde_json::to_value(SendTransactionInput {
            account: self.address.to_checksum(None),
            chain,
            transaction: CwpTransaction {
                to: to.to_checksum(None),
                data: format!("0x{}", alloy::primitives::hex::encode(&data)),
                value: format!("{value}"),
                gas: gas.map(|g| format!("{g}")),
            },
        })?;

        let output = cwp_exec(&self.binary, "send-transaction", Some(&input))?;
        let resp: SendTransactionResponse =
            serde_json::from_value(output).context("Invalid send-transaction response")?;

        resp.transaction_hash
            .parse()
            .context("Invalid transaction hash from CWP")
    }

    pub async fn send_transaction(
        &self,
        to: Address,
        data: Vec<u8>,
        value: U256,
        gas: Option<u64>,
    ) -> Result<B256> {
        let signer = self.clone();
        tokio::task::spawn_blocking(move || {
            signer.send_transaction_blocking(to, data, value, gas)
        })
        .await
        .context("send_transaction task panicked")?
    }

    fn sign_typed_data_blocking(&self, typed_data: &TypedData) -> Result<Signature> {
        let mut typed_data_json = serde_json::to_value(typed_data)?;
        normalize_typed_data_for_wallet(&mut typed_data_json);
        let input = serde_json::json!({
            "account": self.address.to_checksum(None),
            "typedData": typed_data_json,
        });

        let output = cwp_exec(&self.binary, "sign-typed-data", Some(&input))?;
        let resp: SignatureResponse =
            serde_json::from_value(output).context("Invalid sign-typed-data response")?;

        parse_signature(&resp.signature)
    }
}

fn parse_signature(hex_sig: &str) -> Result<Signature> {
    let sig: alloy::primitives::Bytes = hex_sig.parse().context("Invalid signature hex")?;
    if sig.len() != 65 {
        bail!("Expected 65-byte signature, got {} bytes", sig.len());
    }

    Signature::from_raw(&sig).map_err(|e| anyhow::anyhow!("Invalid signature: {e}"))
}

#[async_trait]
impl alloy::signers::Signer for CwpSigner {
    fn address(&self) -> Address {
        self.address
    }

    fn chain_id(&self) -> Option<ChainId> {
        self.chain_id
    }

    fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        self.chain_id = chain_id;
    }

    async fn sign_hash(&self, hash: &B256) -> alloy::signers::Result<Signature> {
        let hash = *hash;
        let signer = self.clone();

        tokio::task::spawn_blocking(move || signer.sign_hash_blocking(&hash))
            .await
            .map_err(|e| alloy::signers::Error::Other(Box::new(e)))?
            .map_err(cwp_err)
    }

    async fn sign_dynamic_typed_data(
        &self,
        payload: &TypedData,
    ) -> alloy::signers::Result<Signature> {
        let payload = payload.clone();
        let signer = self.clone();

        tokio::task::spawn_blocking(move || signer.sign_typed_data_blocking(&payload))
            .await
            .map_err(|e| alloy::signers::Error::Other(Box::new(e)))?
            .map_err(cwp_err)
    }
}

fn cwp_err(e: anyhow::Error) -> alloy::signers::Error {
    alloy::signers::Error::Other(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
    )))
}

pub fn connect(binary: &str) -> Result<CwpSigner> {
    let output = cwp_exec(binary, "accounts", None)?;
    let resp: AccountsResponse =
        serde_json::from_value(output).context("Invalid accounts response")?;

    let first = resp
        .accounts
        .first()
        .context("No accounts returned by CWP provider")?;

    let address: Address = first
        .address
        .parse()
        .context("Invalid address from CWP provider")?;

    Ok(CwpSigner::new(binary, address, None))
}

// --- Typed Data Normalization ---
// Alloy serializes uint256 as "0x..." hex and address as lowercase.
// Wallets (MetaMask, Zerion via WalletConnect) expect decimal strings for
// uint256 and checksummed addresses in eth_signTypedData_v4 payloads.

fn normalize_typed_data_for_wallet(typed_data: &mut serde_json::Value) {
    // Build a type lookup from the "types" field: { "ClobAuth": {"address": "address", "nonce": "uint256", ...} }
    let type_map = build_type_map(typed_data);

    // Normalize domain values (chainId is uint256)
    if let Some(domain) = typed_data.get_mut("domain") {
        normalize_object(domain, &eip712_domain_types(), &type_map);
    }

    // Normalize message values using the primaryType
    let primary_type = typed_data
        .get("primaryType")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if let Some(message) = typed_data.get_mut("message") {
        if let Some(field_types) = type_map.get(&primary_type) {
            normalize_object(message, field_types, &type_map);
        }
    }
}

fn eip712_domain_types() -> HashMap<String, String> {
    [
        ("name", "string"),
        ("version", "string"),
        ("chainId", "uint256"),
        ("verifyingContract", "address"),
        ("salt", "bytes32"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

fn build_type_map(
    typed_data: &serde_json::Value,
) -> HashMap<String, HashMap<String, String>> {
    let mut map = HashMap::new();
    if let Some(types) = typed_data.get("types").and_then(|t| t.as_object()) {
        for (type_name, fields) in types {
            let mut field_map = HashMap::new();
            if let Some(fields) = fields.as_array() {
                for field in fields {
                    if let (Some(name), Some(ty)) = (
                        field.get("name").and_then(|n| n.as_str()),
                        field.get("type").and_then(|t| t.as_str()),
                    ) {
                        field_map.insert(name.to_string(), ty.to_string());
                    }
                }
            }
            map.insert(type_name.clone(), field_map);
        }
    }
    map
}

fn normalize_object(
    obj: &mut serde_json::Value,
    field_types: &HashMap<String, String>,
    type_map: &HashMap<String, HashMap<String, String>>,
) {
    if let Some(map) = obj.as_object_mut() {
        for (field_name, value) in map.iter_mut() {
            if let Some(sol_type) = field_types.get(field_name.as_str()) {
                normalize_value(value, sol_type, type_map);
            }
        }
    }
}

fn normalize_value(
    value: &mut serde_json::Value,
    sol_type: &str,
    type_map: &HashMap<String, HashMap<String, String>>,
) {
    if sol_type.starts_with("uint") || sol_type.starts_with("int") {
        // Convert hex "0x..." to decimal string.
        // Note: uses U256 for both uint and int types. Polymarket only uses uint256;
        // proper signed int support would need I256 if int types are ever used.
        if let Some(s) = value.as_str() {
            if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                if let Ok(n) = U256::from_str_radix(hex, 16) {
                    *value = serde_json::Value::String(n.to_string());
                }
            }
        }
    } else if sol_type == "address" {
        // Convert to checksummed address
        if let Some(s) = value.as_str() {
            if let Ok(addr) = s.parse::<Address>() {
                *value = serde_json::Value::String(addr.to_checksum(None));
            }
        }
    } else if let Some(nested_fields) = type_map.get(sol_type) {
        // Recurse into nested struct types
        normalize_object(value, nested_fields, type_map);
    }
}

// --- PolySigner ---
// Custom enum replacing Either<PrivateKeySigner, CwpSigner>.
// Unlike Either's Signer impl which only forwards sign_hash,
// this forwards ALL Signer methods including sign_dynamic_typed_data.

pub enum PolySigner {
    Local(PrivateKeySigner),
    Cwp(CwpSigner),
}

#[async_trait]
impl alloy::signers::Signer for PolySigner {
    fn address(&self) -> Address {
        match self {
            Self::Local(s) => s.address(),
            Self::Cwp(s) => s.address(),
        }
    }

    fn chain_id(&self) -> Option<ChainId> {
        match self {
            Self::Local(s) => s.chain_id(),
            Self::Cwp(s) => s.chain_id(),
        }
    }

    fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        match self {
            Self::Local(s) => s.set_chain_id(chain_id),
            Self::Cwp(s) => s.set_chain_id(chain_id),
        }
    }

    async fn sign_hash(&self, hash: &B256) -> alloy::signers::Result<Signature> {
        match self {
            Self::Local(s) => s.sign_hash(hash).await,
            Self::Cwp(s) => s.sign_hash(hash).await,
        }
    }

    async fn sign_dynamic_typed_data(
        &self,
        payload: &TypedData,
    ) -> alloy::signers::Result<Signature> {
        match self {
            Self::Local(s) => s.sign_dynamic_typed_data(payload).await,
            Self::Cwp(s) => s.sign_dynamic_typed_data(payload).await,
        }
    }
}
