// src/config.rs
use dotenv::dotenv;
use std::env;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub ws_url: String,
    pub keypair_path: String, // 🟢 补回了这个字段
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv().ok();

        let config = Config {
            rpc_url: env::var("RPC_URL").expect("❌ RPC_URL missing in .env"),
            ws_url: env::var("WS_URL").expect("❌ WS_URL missing in .env"),
            // 🟢 读取路径，如果没有设置则报错
            keypair_path: env::var("KEYPAIR_PATH").expect("❌ KEYPAIR_PATH missing in .env"),
        };

        Ok(config)
    }
}
