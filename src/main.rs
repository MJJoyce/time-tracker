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

    /// Add a previously completed entry to the log.
    Complete(Complete),

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
struct Complete {
    /// The task name to start tracking.
    task_name: String,

    /// Duration of the task in hh:mm:ss format.
    duration: String,

    /// Event time for the entry being added. If this
    /// isn't provided the start time is calculated from the
    /// current time.
    #[clap(short, long)]
    event_time: Option<String>,

    /// An optional comment or note describing the task.
    #[clap(short, long)]
    note: Option<String>,
}

fn start_handler(
    mut logger: impl logger::TTLogger,
    task_conf: &Start,
) -> Result<(), Box<dyn std::error::Error>> {
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

fn complete_handler(
    mut logger: impl logger::TTLogger,
    task_conf: &Complete,
) -> Result<(), Box<dyn std::error::Error>> {
    let cur_t = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let dur_parts = task_conf.duration.trim().split(':').map(|i| i.parse()).collect::<Result<Vec<u64>, _>>()?;
    let duration = (60 * 60 * dur_parts[0]) + (60 * dur_parts[1]) + dur_parts[2];

    let s_time = match &task_conf.event_time {
        Some(e_time) => {
            let datetime = NaiveDateTime::parse_from_str(e_time, DT_FORMAT)?;
            datetime.timestamp() as u64
        },
        None => {
            cur_t - duration
        }
    };

    let e_time = s_time + duration;

    logger.write(LogEntry {
        entry_type: LogEntryType::Start,
        stime: s_time,
        task: Some(task_conf.task_name.clone()),
        note: task_conf.note.clone(),
    })?;

    logger.write(LogEntry {
        entry_type: LogEntryType::End,
        stime: e_time,
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
        Commands::Complete(complete) => complete_handler(task_logger, complete),
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
