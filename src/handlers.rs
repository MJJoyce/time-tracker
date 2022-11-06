use chrono::{NaiveDate, NaiveDateTime};
use indoc::indoc;
use lazy_static::lazy_static;
use std::io::{Error, ErrorKind};
use std::time::SystemTime;

use crate::cli::{Clear, Complete, End, Start, Status};
use crate::entry::{LogEntry, LogEntryType};
use crate::logger;

const DT_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

lazy_static! {
    static ref UNIX_EPOCH_DT: NaiveDateTime = NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0);
}

pub fn start_handler(
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

pub fn end_handler(
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

pub fn complete_handler(
    mut logger: impl logger::TTLogger + IntoIterator<Item = LogEntry> + Clone,
    task_conf: &Complete,
) -> Result<(), Box<dyn std::error::Error>> {
    let cur_t = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let dur_parts = task_conf
        .duration
        .trim()
        .split(':')
        .map(|i| i.parse())
        .collect::<Result<Vec<u64>, _>>()?;
    let duration = (60 * 60 * dur_parts[0]) + (60 * dur_parts[1]) + dur_parts[2];

    let s_time = match &task_conf.event_time {
        Some(e_time) => {
            let datetime = NaiveDateTime::parse_from_str(e_time, DT_FORMAT)?;
            datetime.timestamp() as u64
        }
        None => cur_t - duration,
    };

    let e_time = s_time + duration;

    // Check the last entry of the log to ensure that we're not going to create a invalid log file.
    //
    // If the last entry is a Start we need to compare its time with the start time of our complete
    // entry. If our complete entry occurs before the Start entry we need to insert an End entry,
    // write the complete entry, and then rewrite the Start entry.
    let last_entry = logger.clone().into_iter().last();
    let mut keep_log_valid = false;
    if let Some(entry) = &last_entry {
        if entry.entry_type == LogEntryType::Start && s_time < entry.stime {
            keep_log_valid = true;
            logger.write(LogEntry {
                entry_type: LogEntryType::End,
                stime: cur_t,
                task: None,
                note: None,
            })?;
        }
    }

    // Write the "Complete" entry Start / End entries.
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

    // Rewrite the previous Start entry if we needed to keep the log valid.
    if keep_log_valid {
        let last_entry = last_entry.unwrap();
        logger.write(LogEntry {
            entry_type: LogEntryType::Start,
            stime: cur_t,
            task: last_entry.task,
            note: last_entry.note,
        })?;
    }

    Ok(())
}

pub fn status_handler(
    logger: impl logger::TTLogger + IntoIterator<Item = LogEntry> + Clone,
    _task_conf: &Status,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(last_entry) = logger.into_iter().last() else {
        return Err(Box::new(Error::new(ErrorKind::NotFound, "Unable to located task to report status.")));
    };

    match last_entry.entry_type {
        LogEntryType::Start => {
            let dt = NaiveDateTime::from_timestamp(last_entry.stime.try_into().unwrap(), 0);

            println!(
                indoc! {"
                    Currently tracked task:
                        Task name: {}
                        Start time: {}
                        Notes: {}
                "},
                //last_entry.task.unwrap_or("<No Task Name>".to_string()),
                last_entry.task.unwrap_or_default(),
                dt,
                last_entry.note.unwrap_or_default()
            );
        }
        _ => {
            println!("Not tracking any active tasks.");
        }
    }

    Ok(())
}

pub fn clear_handler(
    logger: impl logger::TTLogger + IntoIterator<Item = LogEntry> + Clone,
    _task_conf: &Clear,
) -> Result<(), Box<dyn std::error::Error>> {
    logger.clear_log()?;
    Ok(())
}
