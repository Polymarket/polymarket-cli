use anyhow::{Context, Result};
use reqwest_middleware::ClientWithMiddleware;
use serde_json::Value;
use std::sync::Arc;
use x402_reqwest::{ReqwestWithPayments, ReqwestWithPaymentsBuild, X402Client};

const BASE_URL: &str = "https://agent.metengine.xyz";
const DEFAULT_SOLANA_RPC_URL: &str = "https://api.mainnet-beta.solana.com";

fn solana_rpc_url() -> String {
    std::env::var("SOLANA_RPC_URL").unwrap_or_else(|_| DEFAULT_SOLANA_RPC_URL.to_string())
}

/// Plain reqwest client for free endpoints (health, pricing).
pub struct FreeClient {
    inner: reqwest::Client,
}

impl FreeClient {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
        }
    }

    pub async fn get(&self, path: &str) -> Result<Value> {
        let url = format!("{BASE_URL}{path}");
        let resp = self
            .inner
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("MetEngine GET {path} failed: {e:#}"))?;
        handle_response(resp, path).await
    }
}

/// x402-paid reqwest client for paid endpoints. Solana signing is handled
/// automatically by the x402-reqwest middleware on HTTP 402 responses.
pub struct PaidClient {
    inner: ClientWithMiddleware,
}

impl PaidClient {
    pub fn new(solana_base58_key: &str) -> Result<Self> {
        let keypair = solana_keypair::Keypair::try_from_base58_string(solana_base58_key)
            .map_err(|e| anyhow::anyhow!("Invalid Solana private key: {e}"))?;
        let rpc = solana_client::nonblocking::rpc_client::RpcClient::new(solana_rpc_url());
        let scheme = x402_chain_solana::V2SolanaExactClient::new(Arc::new(keypair), Arc::new(rpc));
        let x402 = X402Client::new().register(scheme);

        let client = reqwest::Client::new().with_payments(x402).build();

        Ok(Self { inner: client })
    }

    pub async fn get<K: AsRef<str>, V: AsRef<str>>(
        &self,
        path: &str,
        query: &[(K, V)],
    ) -> Result<Value> {
        let pairs: Vec<(&str, &str)> = query
            .iter()
            .map(|(k, v)| (k.as_ref(), v.as_ref()))
            .collect();
        let url = reqwest::Url::parse_with_params(&format!("{BASE_URL}{path}"), &pairs)
            .context("Invalid query parameters")?;
        let resp = self
            .inner
            .get(url.as_str())
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("MetEngine GET {path} failed: {e:#}"))?;
        handle_response(resp, path).await
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{BASE_URL}{path}");
        let resp = self
            .inner
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("MetEngine POST {path} failed: {e:#}"))?;
        handle_response(resp, path).await
    }
}

/// Shared response handling for reqwest responses.
async fn handle_response(resp: reqwest::Response, path: &str) -> Result<Value> {
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let truncated = truncate_error(&body);
        anyhow::bail!("MetEngine API error {status} on {path}: {truncated}");
    }
    let json: Value = resp.json().await.context("Failed to parse response")?;
    Ok(unwrap_data(json))
}

fn truncate_error(body: &str) -> &str {
    let end = body
        .char_indices()
        .nth(500)
        .map(|(i, _)| i)
        .unwrap_or(body.len());
    &body[..end]
}

/// If the response has a top-level `"data"` key, return its value.
/// Otherwise return the whole response as-is.
fn unwrap_data(v: Value) -> Value {
    if let Value::Object(mut map) = v {
        map.remove("data").unwrap_or(Value::Object(map))
    } else {
        v
    }
}
