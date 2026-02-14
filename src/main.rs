// src/main.rs

// 1. 模块声明
mod config;
mod scout;
mod strategy;
mod executor;

// 2. 引入依赖
use config::Config;
use scout::{Scout, MinimalLog};
use executor::Executor;
use log::{info, error};
use tokio::sync::mpsc;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // A. 初始化日志 (显示 Info 级别)
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    info!("🚀 STARTING SOLANA MEV BOT (REAL TRADING MODE)...");
    info!("⚠️  WARNING: Real funds will be used. Ensure id.json is secure.");

    // B. 加载配置
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            error!("❌ FATAL: Config load failed: {}", e);
            return Err(e);
        }
    };

    // C. 初始化执行者 (Executor)
    // 这里会通过 Arc 包装，以便在多线程中共享同一个 RPC 连接池和钱包签名器
    let executor = match Executor::new(config.rpc_url.clone(), config.keypair_path.clone()) {
        Ok(exe) => Arc::new(exe),
        Err(e) => {
            error!("❌ FATAL: Executor init failed (Check id.json): {}", e);
            return Err(e);
        }
    };

    // D. 启动 Scout (侦察兵)
    // 通道容量设为 1000，防止高频交易时阻塞
    let (tx, mut rx) = mpsc::channel::<MinimalLog>(1000);
    let ws_url = config.ws_url.clone();
    
    tokio::spawn(async move {
        let scout = Scout::new(ws_url, tx);
        // Scout 内部有自动重连机制，通常不会返回
        scout.start().await; 
    });

    info!("✅ SYSTEM ARMED. Monitoring Tensor Swap logs...");

    // E. 主循环 (Event Loop)
    let mut last_slot = 0;

    while let Some(event) = rx.recv().await {
        // 1. 简单的防乱序处理 (Optional)
        if event.slot < last_slot {
            continue;
        }
        last_slot = event.slot;

        // 2. 策略分析 (Strategy Analysis)
        // 调用 strategy::analyze，它现在包含了 LRU 去重逻辑
        if let Some(decision) = strategy::analyze(event).await {
            
            // 3. 异步执行 (Fire & Forget)
            // 克隆 Arc 指针，将执行任务扔给后台，主线程立即去处理下一个日志
            // 这样即使 execute 需要 2 秒钟，主线程也能毫秒级响应下一个机会
            let executor_clone = executor.clone();
            
            tokio::spawn(async move {
                // 这里调用的是真实的 execute，会消耗 Gas
                executor_clone.execute(decision).await;
            });
        }
    }

    error!("❌ CRITICAL: Main log channel closed. Bot shutting down.");
    Ok(())
}
