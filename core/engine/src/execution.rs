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

impl ExecuteMessage {
    pub fn new(seq_id: u128, execution: Execution) -> Self {
        Self { seq_id, execution }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn set_execution(&mut self, execution: Execution) {
        self.execution = execution;
    }
}
