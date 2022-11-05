use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Starts tracking a work task.
    Start(Start),

    /// Marks the end of a previously started task.
    End(End),

    /// Add a previously completed entry to the log.
    Complete(Complete),

    /// Summarizes the task log.
    Summary(Summary),
}

#[derive(Args, Debug, Clone)]
pub struct Start {
    /// The task name to start tracking.
    pub task_name: String,

    /// An optional comment or note describing the task.
    #[clap(short, long)]
    pub note: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct End {}

#[derive(Args, Debug, Clone)]
pub struct Complete {
    /// The task name to start tracking.
    pub task_name: String,

    /// Duration of the task in hh:mm:ss format.
    pub duration: String,

    /// Event time for the entry being added. If this
    /// isn't provided the start time is calculated from the
    /// current time.
    #[clap(short, long)]
    pub event_time: Option<String>,

    /// An optional comment or note describing the task.
    #[clap(short, long)]
    pub note: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct Summary {}

pub fn parse() -> CliArgs {
    CliArgs::parse()
}
