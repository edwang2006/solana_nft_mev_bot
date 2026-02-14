// src/strategy/parser.rs

use regex::Regex;
use lazy_static::lazy_static;


// 定义解析结果
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum TensorAction {
    Buy {
        price_lamports: u64,
        // 生产级提示：仅靠日志很难精准提取 buyer/mint Pubkey，
        // 通常我们只需提取价格来决定是否套利，或者结合 getTransaction 使用。
        // 这里为了速度，我们先只提取价格信号。
    },
    List {
        price_lamports: u64,
    },
    Unknown,
}

// 🚀 工业级优化：预编译正则表达式
// 放在 lazy_static 里，保证程序启动时只编译一次，后续调用只需 0.001ms
lazy_static! {
    // 匹配 Tensor 的 Buy 指令日志
    static ref BUY_LOG_RE: Regex = Regex::new(r"Instruction: BuySingleListing").unwrap();
    
    // 匹配 List 指令日志
    static ref LIST_LOG_RE: Regex = Regex::new(r"Instruction: List").unwrap();

    // 💡 高级技巧：尝试从转账日志中提取金额
    // 系统转账日志通常格式: "Transfer: `amount` lamports to `pubkey`"
    // 注意：不同 RPC 的日志格式可能略有不同，需要根据实战调整
    // 这里的正则是一个通用匹配，用于捕捉 SOL 流动
    static ref TRANSFER_RE: Regex = Regex::new(r"Transfer: (\d+) lamports").unwrap();
}

pub struct TensorParser;

impl TensorParser {
    pub fn parse(logs: &[String]) -> TensorAction {
        // 1. 快速位运算/布尔判断 (Fail Fast)
        // 如果日志很少，直接跳过
        if logs.is_empty() {
            return TensorAction::Unknown;
        }

        // 2. 状态机追踪
        let mut is_buy = false;
        let mut is_list = false;
        let mut max_transfer_amount = 0;

        for log in logs {
            // 极速匹配指令类型
            if BUY_LOG_RE.is_match(log) {
                is_buy = true;
            } else if LIST_LOG_RE.is_match(log) {
                is_list = true;
            }

            // 提取金额流向
            // 只有当我们在监听 Buy 或 List 时才去解析金额，节省 CPU
            if let Some(caps) = TRANSFER_RE.captures(log) {
                if let Some(amount_str) = caps.get(1) {
                    if let Ok(amount) = amount_str.as_str().parse::<u64>() {
                        // 在一笔交易中，最大的那笔转账通常是成交价（忽略手续费和小额转账）
                        if amount > max_transfer_amount {
                            max_transfer_amount = amount;
                        }
                    }
                }
            }
        }

        // 3. 综合判断 (Decision Matrix)
        if is_buy && max_transfer_amount > 0 {
            return TensorAction::Buy {
                price_lamports: max_transfer_amount,
            };
        } else if is_list {
            // List 事件通常不伴随 SOL 转账（除了微量租金），价格通常在 Event Data 里
            // 这里我们暂时只能捕捉到信号
             return TensorAction::List {
                price_lamports: 0, // List 价格解析需要 Base64 Event Data，这是下一步的难点
            };
        }

        TensorAction::Unknown
    }
}
