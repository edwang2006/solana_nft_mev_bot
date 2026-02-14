// src/executor/mod.rs

use solana_sdk::{
    signature::{Keypair, Signer, read_keypair_file},
    transaction::Transaction,
    pubkey::Pubkey,
    system_instruction,
    commitment_config::CommitmentConfig,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;
use log::{info, error};
use anyhow::{Result};
use crate::strategy::TradeDecision;

// Jito Tip Account (随机选择一个)
const JITO_TIP_ACCOUNT: &str = "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5";

pub struct Executor {
    rpc_client: Arc<RpcClient>,
    keypair: Keypair,
}

impl Executor {
    pub fn new(rpc_url: String, keypair_path: String) -> Result<Self> {
        info!("🔐 Loading wallet: {}", keypair_path);
        let keypair = read_keypair_file(&keypair_path)
            .map_err(|e| anyhow::anyhow!("CRITICAL: Cannot read id.json: {}", e))?;
        
        info!("✅ Wallet Ready: {} (Checking balance...)", keypair.pubkey());
        
        Ok(Self {
            rpc_client: Arc::new(RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed())),
            keypair,
        })
    }

    pub async fn execute(&self, decision: TradeDecision) {
        info!("🚀 EXECUTING: {} | Ref Tx: {}", decision.action_type, decision.signature);

        // 1. 获取 Blockhash (极速)
        let recent_blockhash = match self.rpc_client.get_latest_blockhash().await {
            Ok(b) => b,
            Err(e) => { error!("❌ Network Fail: {}", e); return; }
        };

        // 2. 构建指令 (Payload Construction)
        let mut instructions = vec![];

        // 🟢 真实盈利逻辑接入点 (Integration Point)
        // 这里的 system_instruction::transfer 是为了证明代码能跑通真实网络。
        // 如果要买 NFT，必须在这里插入 Tensor 的 Instruction Data。
        // 比如: instructions.push(tensor_program::buy_instruction(...));
        
        // 目前：发送 1000 Lamports (0.000001 SOL) 给自己，作为心跳测试
        let heartbeat_ix = system_instruction::transfer(
            &self.keypair.pubkey(),
            &self.keypair.pubkey(),
            1000, 
        );
        instructions.push(heartbeat_ix);

        // 3. Jito Bribe (必须要有小费才能防夹)
        let tip_account = Pubkey::try_from(JITO_TIP_ACCOUNT).unwrap();
        let tip_ix = system_instruction::transfer(
            &self.keypair.pubkey(),
            &tip_account,
            5000, // 0.000005 SOL 小费
        );
        instructions.push(tip_ix);

        // 4. 签名与广播
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&self.keypair.pubkey()),
            &[&self.keypair],
            recent_blockhash,
        );

        info!("🔥 BROADCASTING REAL TX...");
        match self.rpc_client.send_and_confirm_transaction(&transaction).await {
            Ok(sig) => info!("✅ TX CONFIRMED: https://solscan.io/tx/{}", sig),
            Err(e) => error!("❌ TX FAILED: {}", e),
        }
    }
}
