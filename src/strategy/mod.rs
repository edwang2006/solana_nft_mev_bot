// src/strategy/mod.rs
pub mod parser;

use crate::scout::MinimalLog;
use parser::{TensorParser, TensorAction};
use log::{info, error};
use lru::LruCache;
use std::sync::Mutex;
use std::num::NonZeroUsize;

#[allow(dead_code)]
#[derive(Debug)]
pub struct TradeDecision {
    pub action_type: String, 
    pub price_lamports: u64,
    pub signature: String,
}

// 🛡️ 工业级防护：去重缓存
// 使用 lazy_static 维护一个全局的缓存，记录最近处理过的 1000 个签名
lazy_static::lazy_static! {
    static ref SIG_CACHE: Mutex<LruCache<String, ()>> = Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap()));
}

pub async fn analyze(event: MinimalLog) -> Option<TradeDecision> {
    // 1. 去重检查 (Deduplication)
    {
        let mut cache = SIG_CACHE.lock().unwrap();
        if cache.contains(&event.signature) {
            // 如果这个签名已经处理过，直接忽略，防止重复执行
            return None;
        }
        cache.put(event.signature.clone(), ());
    }

    // 2. 解析日志
    let result = std::panic::catch_unwind(|| {
        TensorParser::parse(&event.logs)
    });

    match result {
        Ok(action) => match action {
            TensorAction::Buy { price_lamports } => {
                // 3. 盈利逻辑 (Profit Logic)
                // ⚠️ 严正提示：监听到 'Buy' 意味着货已经没了。
                // 这里的逻辑是 "Follow Trend" (趋势跟随) 或者 "Test Fire" (测试开火)。
                // 真正生产环境你需要监听 'List'。
                
                if price_lamports > 0 {
                    info!("⚡️ MARKET ACTIVITY | Valid Trade: {:.4} SOL | Tx: ...{}", 
                        price_lamports as f64 / 1e9, 
                        &event.signature[..8]
                    );
                    
                    return Some(TradeDecision {
                        action_type: "TEST_BUY".to_string(),
                        price_lamports,
                        signature: event.signature,
                    });
                }
                None
            },
            _ => None,
        },
        Err(_) => {
            error!("🔥 Strategy Panic recovered.");
            None
        }
    }
}
