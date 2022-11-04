use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum LogEntryType {
    Start,
    End,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub entry_type: LogEntryType,
    pub stime: u64,
    pub task: String,
    pub note: Option<String>,
}
