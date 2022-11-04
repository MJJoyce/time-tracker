use chrono::{NaiveDate, NaiveDateTime};
use clap::{Args, Parser, Subcommand};
use lazy_static::lazy_static;

use std::time::SystemTime;

use crate::entry::{LogEntry, LogEntryType};

mod cfg;
mod entry;
mod logger;

const DT_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

lazy_static! {
    static ref UNIX_EPOCH_DT: NaiveDateTime = NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0);
}

#[derive(Parser)]
#[command(author, version, about)]
struct CliArgs {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
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

#[derive(Args, Debug, Clone)]
struct Start {
    /// The task name to start tracking.
    task_name: String,

    /// An optional comment or note describing the task.
    #[clap(short, long)]
    note: Option<String>,
}

#[derive(Args, Debug, Clone)]
struct End {}

#[derive(Args, Debug, Clone)]
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

fn start_handler(
    mut logger: impl logger::TTLogger,
    task_conf: &Start,
) -> Result<(), Box<dyn std::error::Error>> {
    // We don't need this for Start but we'll probably do something like this with Amend
    // let start_time = match &task_conf.start_time {
    //Some(t_string) => {
    //let datetime = NaiveDateTime::parse_from_str(&t_string, DT_FORMAT)?;
    //let dur = datetime.signed_duration_since(*UNIX_EPOCH_DT).num_seconds();

    //if dur < 0 {
    //Err("Invalid task start time is prior to the UNIX Epoch.")
    //} else {
    //Ok(dur as u64)
    //}
    //},
    //None => {
    //match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
    //Ok(n) => Ok(n.as_secs()),
    //Err(_) => Err("Unable to generate valid system time for task.")
    //}
    //}
    //}?;

    logger.write(LogEntry {
        entry_type: LogEntryType::Start,
        stime: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        task: Some(task_conf.task_name.clone()),
        note: task_conf.note.clone(),
    })?;

    Ok(())
}

fn end_handler(
    mut logger: impl logger::TTLogger,
    _task_conf: &End,
) -> Result<(), Box<dyn std::error::Error>> {
    logger.write(LogEntry {
        entry_type: LogEntryType::End,
        stime: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        task: None,
        note: None,
    })?;

    Ok(())
}

#[derive(Args, Debug, Clone)]
struct Summary {}

fn main() {
    let cli = CliArgs::parse();
    let conf = cfg::load();

    let task_logger =
        logger::load_logger(conf.task_log).expect("Unable to parse log file. Exiting ...");

    let res = match &cli.command {
        Commands::Start(start) => start_handler(task_logger, start),
        Commands::End(end) => end_handler(task_logger, end),
        _default => {
            panic!("Not implemented {:?}", _default);
        }
    };

    match res {
        Ok(()) => {}
        Err(e) => {
            eprintln!("tt command execution failed: {:?}", e);
        }
    }
}
