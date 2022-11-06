use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogEntryType {
    Start,
    End,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub entry_type: LogEntryType,
    pub stime: u64,
    pub task: Option<String>,
    pub note: Option<String>,
}

impl PartialEq for LogEntry {
    fn eq(&self, other: &Self) -> bool {
        self.entry_type == other.entry_type && self.stime == other.stime
    }
}

impl Eq for LogEntry {}

impl PartialOrd for LogEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LogEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.stime.cmp(&other.stime)
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub stime: u64,
    pub dur: u64,
    pub task: String,
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.stime == other.stime && self.task == other.task
    }
}

impl Eq for Task {}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.stime.cmp(&other.stime)
    }
}
