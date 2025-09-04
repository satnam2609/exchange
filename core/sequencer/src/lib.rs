use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Execution {
    INSERTED,
    CANCELLED,
    FILL,
    PARTIAL(f64, u64),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ExecuteMessage {
    seq_id: u128,
    execution: Execution,
}