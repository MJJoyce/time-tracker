use chrono::{Duration, NaiveDate, NaiveDateTime};
use indoc::indoc;
use itertools::Itertools;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::iter::zip;
use std::time::SystemTime;

use crate::cli::{Clear, Complete, End, Start, Status, Summary};
use crate::entry::{LogEntry, LogEntryType, Task};
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
    mut logger: impl logger::TTLogger + IntoIterator<Item = LogEntry> + Clone,
    _task_conf: &End,
) -> Result<(), Box<dyn std::error::Error>> {
    let last_entry = logger.clone().into_iter().last();

    if last_entry.is_none() || last_entry.unwrap().entry_type == LogEntryType::End {
        return Err(Box::new(Error::new(
            ErrorKind::InvalidData,
            "Not active task being tracked. Cannot mark task complete.",
        )));
    }

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
        return Err(Box::new(Error::new(
            ErrorKind::NotFound,
            "Unable to locate task for status reporting."))
        );
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
    logger: impl logger::TTLogger,
    _task_conf: &Clear,
) -> Result<(), Box<dyn std::error::Error>> {
    logger.clear_log()?;
    Ok(())
}

pub fn summary_handler(
    logger: impl logger::TTLogger + IntoIterator<Item = LogEntry> + Clone,
    _task_conf: &Summary,
) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = parse_log_into_tasks(logger);

    if tasks.len() == 0 {
        println!("Cannot generate summary for empty task list.");
        return Ok(());
    }

    let grouped_tasks = group_tasks(tasks);

    let (aggregate_stats, grouped_stats) = summarize_task_stats(&grouped_tasks);

    for (group, stats) in zip(grouped_tasks, grouped_stats) {
        println!(
            "{}",
            create_report_for_taskgroup(group, stats, &aggregate_stats)
        );
    }

    println!("{}", create_aggregate_report(aggregate_stats));

    Ok(())
}

fn parse_log_into_tasks(
    logger: impl logger::TTLogger + IntoIterator<Item = LogEntry> + Clone,
) -> Vec<Task> {
    let mut tasks = Vec::new();

    for eles in logger.into_iter().collect::<Vec<LogEntry>>().windows(2) {
        let [e1, e2] = eles else {
            eprintln!("Unable to process element window {:?}", eles);
            continue;
        };

        if e1.entry_type == LogEntryType::End {
            continue;
        }

        tasks.push(Task {
            stime: e1.stime,
            dur: e2.stime - e1.stime,
            // This seems like a dumb way of doing this ...
            task: e1.task.clone().unwrap_or_else(|| "Task".to_string()),
        });
    }

    tasks.sort();
    tasks
}

fn group_tasks(tasks: Vec<Task>) -> Vec<Vec<Task>> {
    let mut last_date = NaiveDateTime::from_timestamp(tasks[0].stime.try_into().unwrap(), 0);

    let mut grouped_tasks = Vec::new();
    let mut group = Vec::new();

    for task in tasks {
        let task_dt = NaiveDateTime::from_timestamp(task.stime.try_into().unwrap(), 0);
        //
        if task_dt.date() != last_date.date() {
            grouped_tasks.push(group);
            group = Vec::new();
            last_date = task_dt;
        }

        group.push(task);
    }

    if group.len() > 0 {
        grouped_tasks.push(group);
    }

    grouped_tasks
}

fn summarize_task_stats(
    task_groups: &Vec<Vec<Task>>,
) -> (HashMap<String, u64>, Vec<HashMap<String, u64>>) {
    let mut aggregate_stats = HashMap::new();
    for group in task_groups {
        for task in group {
            aggregate_stats
                .entry(task.task.clone())
                .and_modify(|e| *e += task.dur)
                .or_insert(task.dur);
        }
    }

    let mut grouped_stats = Vec::new();
    let mut stats = HashMap::new();
    for group in task_groups {
        for task in group {
            stats
                .entry(task.task.clone())
                .and_modify(|e| *e += task.dur)
                .or_insert(task.dur);
        }

        grouped_stats.push(stats);
        stats = HashMap::new();
    }

    (aggregate_stats, grouped_stats)
}

fn create_report_for_taskgroup(
    group: Vec<Task>,
    stats: HashMap<String, u64>,
    agg_stats: &HashMap<String, u64>,
) -> String {
    let dt = NaiveDateTime::from_timestamp(group[0].stime.try_into().unwrap(), 0);
    let mut report = format!("{} Stats\n-----------------------\n", dt.date());

    for k in stats.keys().sorted() {
        let d = Duration::seconds(stats[k] as i64);
        let t_per = stats[k] as f64 / agg_stats[k] as f64 * 100.0;
        report.push_str(&format!(
            "{}: {} ({:.2}% of task total)\n",
            k,
            format_duration(d),
            t_per
        ));
    }

    report
}

fn create_aggregate_report(agg_stats: HashMap<String, u64>) -> String {
    let mut report = "Aggregate Stats\n-----------------------\n".to_string();

    let mut total: u64 = 0;
    for v in agg_stats.values() {
        total += v;
    }

    for k in agg_stats.keys().sorted() {
        let d = Duration::seconds(agg_stats[k] as i64);
        let t_per = agg_stats[k] as f64 / total as f64 * 100.0;
        report.push_str(&format!(
            "{}: {} ({:.2}% of total time)\n",
            k,
            format_duration(d),
            t_per
        ));
    }

    report
}

fn format_duration(dur: Duration) -> String {
    let s = dur.num_seconds() % 60;
    let m = (dur.num_seconds() / 60) % 60;
    let h = (dur.num_seconds() / 60) / 60;
    format!("{}h {}m {}s", h, m, s)
}
