use clap::{Args, Parser, Subcommand};

mod cfg;

#[derive(Parser)]
#[command(author, version, about)]
struct CliArgs {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Starts tracking a work task.
    Start(Start),

    /// Marks the end of a previously started task.
    End(End),

    /// Amends the task log with a new task.
    Amend(Amend),

    /// Summarizes the task log.
    Summary(Summary),
}

#[derive(Args)]
struct Start {
    /// The task name to start tracking.
    task_name: String,

    /// Optional start time for when the task occurred.
    start_time: Option<String>,

    /// An optional comment or note describing the task.
    note: Option<String>,
}

#[derive(Args)]
struct End {}

#[derive(Args)]
struct Amend {
    /// The task name to add.
    task_name: String,

    /// The duration of the task.
    duration: String,

    /// Optional start time for when the task occurred.
    start_time: Option<String>,

    /// An optional comment or note describing the task.
    note: Option<String>,
}

#[derive(Args)]
struct Summary {}

fn main() {
    let cli = CliArgs::parse();

    let conf = cfg::load();
}
