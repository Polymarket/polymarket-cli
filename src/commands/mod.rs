use polymarket_client_sdk_v2::types::{Address, address};

pub(crate) const COLLATERAL_SYMBOL: &str = "pUSD";
pub(crate) const COLLATERAL_ADDRESS_STR: &str = "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB";
pub(crate) const COLLATERAL_ADDRESS: Address =
    address!("0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB");
pub(crate) const COLLATERAL_DECIMALS: u32 = 6;
pub(crate) const CONDITIONAL_TOKENS: Address =
    address!("0x4D97DCd97eC945f40cF65F87097ACe5EA0476045");
pub(crate) const CTF_EXCHANGE: Address = address!("0xE111180000d2663C0091e4f400237545B87B996B");
pub(crate) const NEG_RISK_CTF_EXCHANGE: Address =
    address!("0xe2222d279d744050d28e00520010520000310F59");
pub(crate) const NEG_RISK_ADAPTER: Address = address!("0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296");
pub(crate) const CTF_COLLATERAL_ADAPTER: Address =
    address!("0xADa100874d00e3331D00F2007a9c336a65009718");
pub(crate) const NEG_RISK_CTF_COLLATERAL_ADAPTER: Address =
    address!("0xAdA200001000ef00D07553cEE7006808F895c6F1");

pub(crate) mod proxy {
    use alloy::primitives::U256;
    use alloy::sol;
    use anyhow::{Context, Result, ensure};
    use polymarket_client_sdk_v2::POLYGON;
    use polymarket_client_sdk_v2::types::{Address, B256};

    use crate::auth;

    // Polymarket Proxy Wallet Factory interface (CallType: INVALID=0, CALL=1, DELEGATECALL=2).
    sol! {
        #[sol(rpc)]
        interface IProxyWallet {
            struct ProxyCall {
                uint8 typeCode;
                address to;
                uint256 value;
                bytes data;
            }

            function proxy(ProxyCall[] memory calls)
                external payable returns (bytes[] memory returnValues);
        }
    }

    const PROXY_FACTORY: Address =
        polymarket_client_sdk_v2::types::address!("0xaB45c5A4B0c941a2F231C04C3f49182e1A254052");

    pub(crate) fn is_proxy_mode(signature_type: Option<&str>) -> Result<bool> {
        Ok(crate::config::resolve_signature_type(signature_type)? == "proxy")
    }

    pub(crate) fn derive_proxy_address(private_key: Option<&str>) -> Result<Address> {
        let signer = auth::resolve_signer(private_key)?;
        let eoa = polymarket_client_sdk_v2::auth::Signer::address(&signer);
        polymarket_client_sdk_v2::derive_proxy_wallet(eoa, POLYGON)
            .ok_or_else(|| anyhow::anyhow!("Proxy wallet derivation not supported on this chain"))
    }

    pub(crate) async fn send_call(
        private_key: Option<&str>,
        use_proxy: bool,
        target: Address,
        calldata: Vec<u8>,
    ) -> Result<(B256, u64)> {
        use alloy::providers::Provider as _;

        let provider = auth::create_provider(private_key).await?;

        let (tx_hash, receipt) = if use_proxy {
            let factory = IProxyWallet::new(PROXY_FACTORY, &provider);
            let call = IProxyWallet::ProxyCall {
                typeCode: 1,
                to: target,
                value: U256::ZERO,
                data: calldata.into(),
            };
            let pending = factory.proxy(vec![call]).send().await?;
            let hash = *pending.tx_hash();
            (hash, pending.get_receipt().await?)
        } else {
            let tx = alloy::rpc::types::TransactionRequest::default()
                .to(target)
                .input(alloy::primitives::Bytes::from(calldata).into());
            let pending = provider.send_transaction(tx).await?;
            let hash = *pending.tx_hash();
            (hash, pending.get_receipt().await?)
        };

        ensure!(receipt.status(), "Transaction {tx_hash} reverted");
        let block_number = receipt
            .block_number
            .context("Block number not available in receipt")?;
        Ok((tx_hash, block_number))
    }
}

pub(crate) mod approve;
pub(crate) mod bridge;
pub(crate) mod clob;
pub(crate) mod comments;
pub(crate) mod ctf;
pub(crate) mod data;
pub(crate) mod events;
pub(crate) mod markets;
pub(crate) mod profiles;
pub(crate) mod series;
pub(crate) mod setup;
pub(crate) mod sports;
pub(crate) mod tags;
pub(crate) mod upgrade;
pub(crate) mod wallet;

pub(crate) fn is_numeric_id(id: &str) -> bool {
    id.parse::<u64>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    use serde_json::{Value, json};

    static RPC_ENV_LOCK: Mutex<()> = Mutex::new(());

    struct MockRpc {
        url: String,
        stopped: Arc<AtomicBool>,
        thread: Option<thread::JoinHandle<()>>,
    }

    impl MockRpc {
        fn start(status: bool) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let address = listener.local_addr().unwrap();
            let stopped = Arc::new(AtomicBool::new(false));
            let server_stopped = Arc::clone(&stopped);
            let thread = thread::spawn(move || {
                while !server_stopped.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((mut stream, _)) => handle_rpc_request(&mut stream, status),
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("mock RPC accept failed: {error}"),
                    }
                }
            });
            Self {
                url: format!("http://{address}"),
                stopped,
                thread: Some(thread),
            }
        }
    }

    impl Drop for MockRpc {
        fn drop(&mut self) {
            self.stopped.store(true, Ordering::Relaxed);
            if let Some(thread) = self.thread.take() {
                thread.join().unwrap();
            }
        }
    }

    fn handle_rpc_request(stream: &mut TcpStream, status: bool) {
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let request = read_http_request(stream);
        let request: Value = serde_json::from_slice(&request).unwrap();
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request["method"].as_str().unwrap();
        let tx_hash = format!("0x{}", "11".repeat(32));
        let block_hash = format!("0x{}", "22".repeat(32));
        let result = match method {
            "eth_chainId" => json!("0x89"),
            "net_version" => json!("137"),
            "eth_getTransactionCount" => json!("0x0"),
            "eth_estimateGas" => json!("0x5208"),
            "eth_gasPrice" | "eth_maxPriorityFeePerGas" => json!("0x3b9aca00"),
            "eth_sendRawTransaction" => json!(tx_hash),
            "eth_blockNumber" => json!("0x2a"),
            "eth_feeHistory" => json!({
                "oldestBlock": "0x21",
                "baseFeePerGas": vec!["0x3b9aca00"; 11],
                "gasUsedRatio": vec![0.5; 10],
                "reward": vec![vec!["0x3b9aca00"]; 10],
            }),
            "eth_getBlockByNumber" => json!({
                "number": "0x2a",
                "hash": block_hash,
                "parentHash": format!("0x{}", "33".repeat(32)),
                "nonce": "0x0000000000000000",
                "sha3Uncles": format!("0x{}", "00".repeat(32)),
                "logsBloom": format!("0x{}", "00".repeat(256)),
                "transactionsRoot": format!("0x{}", "00".repeat(32)),
                "stateRoot": format!("0x{}", "00".repeat(32)),
                "receiptsRoot": format!("0x{}", "00".repeat(32)),
                "miner": format!("0x{}", "00".repeat(20)),
                "difficulty": "0x0",
                "totalDifficulty": "0x0",
                "extraData": "0x",
                "size": "0x1",
                "gasLimit": "0x1c9c380",
                "gasUsed": "0x5208",
                "timestamp": "0x1",
                "transactions": [],
                "uncles": [],
                "baseFeePerGas": "0x3b9aca00",
                "mixHash": format!("0x{}", "00".repeat(32)),
            }),
            "eth_getTransactionReceipt" => json!({
                "transactionHash": tx_hash,
                "transactionIndex": "0x0",
                "blockHash": block_hash,
                "blockNumber": "0x2a",
                "from": format!("0x{}", "44".repeat(20)),
                "to": format!("0x{}", "55".repeat(20)),
                "cumulativeGasUsed": "0x5208",
                "gasUsed": "0x5208",
                "contractAddress": null,
                "logs": [],
                "logsBloom": format!("0x{}", "00".repeat(256)),
                "status": if status { "0x1" } else { "0x0" },
                "effectiveGasPrice": "0x3b9aca00",
                "type": "0x2",
            }),
            unsupported => {
                write_rpc_response(
                    stream,
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {"code": -32601, "message": format!("unsupported {unsupported}")},
                    }),
                );
                return;
            }
        };
        write_rpc_response(
            stream,
            json!({"jsonrpc": "2.0", "id": id, "result": result}),
        );
    }

    fn read_http_request(stream: &mut TcpStream) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut buffer = [0_u8; 4096];
        let header_end = loop {
            let read = stream.read(&mut buffer).unwrap();
            assert!(
                read > 0,
                "mock RPC connection closed before request completed"
            );
            bytes.extend_from_slice(&buffer[..read]);
            if let Some(end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                break end + 4;
            }
        };
        let headers = std::str::from_utf8(&bytes[..header_end]).unwrap();
        let content_length: usize = headers
            .lines()
            .find_map(|line| {
                line.split_once(':').and_then(|(name, value)| {
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse().unwrap())
                })
            })
            .unwrap();
        while bytes.len() < header_end + content_length {
            let read = stream.read(&mut buffer).unwrap();
            assert!(read > 0, "mock RPC connection closed before body completed");
            bytes.extend_from_slice(&buffer[..read]);
        }
        bytes[header_end..header_end + content_length].to_vec()
    }

    fn write_rpc_response(stream: &mut TcpStream, response: Value) {
        let body = serde_json::to_vec(&response).unwrap();
        write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
            body.len()
        )
        .unwrap();
        stream.write_all(&body).unwrap();
    }

    // The guard intentionally spans the RPC call so the process-wide environment override
    // cannot be changed while the provider is using it.
    #[allow(clippy::await_holding_lock)]
    async fn send_call_with_receipt_status(
        status: bool,
    ) -> anyhow::Result<(alloy::primitives::B256, u64)> {
        let _env_lock = RPC_ENV_LOCK.lock().unwrap();
        let rpc = MockRpc::start(status);
        // SAFETY: RPC_ENV_LOCK serializes this test's process-wide environment mutation.
        unsafe { std::env::set_var("POLYMARKET_RPC_URL", &rpc.url) };
        let result = proxy::send_call(
            Some("0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
            false,
            polymarket_client_sdk_v2::types::address!("0x0000000000000000000000000000000000000001"),
            vec![0xde, 0xad, 0xbe, 0xef],
        )
        .await;
        // SAFETY: RPC_ENV_LOCK serializes this test's process-wide environment mutation.
        unsafe { std::env::remove_var("POLYMARKET_RPC_URL") };
        result
    }

    #[tokio::test]
    async fn send_call_rejects_reverted_receipt_and_accepts_success() {
        let error = send_call_with_receipt_status(false).await.unwrap_err();
        assert!(
            error.to_string().contains("reverted"),
            "unexpected error: {error:#}"
        );

        let (_, block_number) = send_call_with_receipt_status(true).await.unwrap();
        assert_eq!(block_number, 42);
    }

    #[test]
    fn is_numeric_id_pure_digits() {
        assert!(is_numeric_id("12345"));
        assert!(is_numeric_id("0"));
    }

    #[test]
    fn is_numeric_id_rejects_non_digits() {
        assert!(!is_numeric_id("will-trump-win"));
        assert!(!is_numeric_id("0x123abc"));
        assert!(!is_numeric_id("123 456"));
    }

    #[test]
    fn is_numeric_id_rejects_empty() {
        assert!(!is_numeric_id(""));
    }
}
