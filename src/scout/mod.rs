// src/scout/mod.rs

// 🟢 关键修复：使用 solana_client 库自带的 nonblocking 模块
// 这样才有 async 的 new() 方法，并且支持 tokio
use solana_client::nonblocking::pubsub_client::PubsubClient; 

use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use futures::StreamExt;
use log::{info, error, warn};
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Tensor Swap Program ID
pub const TENSOR_SWAP_PID: &str = "TSWAPaqyCSx2KABk68Shruf4Rp7Cqk7629vgix2a9p8";

#[derive(Debug, Clone)]
pub struct MinimalLog {
    pub signature: String,
    pub slot: u64,
    pub logs: Vec<String>, 
}

pub struct Scout {
    ws_url: String,
    sender: Sender<MinimalLog>,
    is_running: Arc<AtomicBool>,
}

impl Scout {
    pub fn new(ws_url: String, sender: Sender<MinimalLog>) -> Self {
        Self {
            ws_url,
            sender,
            is_running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub async fn start(self) {
        info!("🕵️ Scout module initialized. Target: Tensor Swap");

        loop {
            if !self.is_running.load(Ordering::Relaxed) {
                break;
            }

            info!("🔌 Connecting to WSS: {}...", self.ws_url);
            
            // 调用连接逻辑
            match self.connect_and_listen().await {
                Ok(_) => {
                    warn!("⚠️ Connection closed cleanly. Reconnecting in 1s...");
                    sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    error!("❌ WSS Error: {}. Retrying in 2s...", e);
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    async fn connect_and_listen(&self) -> anyhow::Result<()> {
        // 🟢 这里的 PubsubClient::new 现在是 async 的了，因为我们换了引用
        let pubsub_client = PubsubClient::new(&self.ws_url).await?;
        
        let filter = RpcTransactionLogsFilter::Mentions(vec![TENSOR_SWAP_PID.to_string()]);
        let config = RpcTransactionLogsConfig {
            commitment: Some(solana_sdk::commitment_config::CommitmentConfig::processed()),
        };

        // 订阅日志
        let (mut stream, _unsubscribe) = pubsub_client.logs_subscribe(filter, config).await?;
        info!("✅ Connected! Streaming Tensor logs...");

        while let Some(response) = stream.next().await {
            let value = response.value;
            
            if value.err.is_some() {
                continue;
            }

            let event = MinimalLog {
                signature: value.signature,
                logs: value.logs,
                slot: response.context.slot,
            };

            match self.sender.try_send(event) {
                Ok(_) => {},
                Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                    // 忽略 Full 错误，保持高速运行
                },
                Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                    return Err(anyhow::anyhow!("Channel closed"));
                }
            }
        }

        Ok(())
    }
}
