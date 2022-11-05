use chrono::{NaiveDate, NaiveDateTime};
use lazy_static::lazy_static;

use crate::cli::Commands;

mod cfg;
mod cli;
mod entry;
mod handlers;
mod logger;

lazy_static! {
    static ref UNIX_EPOCH_DT: NaiveDateTime = NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0);
}

fn main() {
    let cli = cli::parse();
    let conf = cfg::load();

    let task_logger =
        logger::load_logger(conf.task_log).expect("Unable to parse log file. Exiting ...");

    let res = match &cli.command {
        Commands::Start(start) => handlers::start_handler(task_logger, start),
        Commands::End(end) => handlers::end_handler(task_logger, end),
        Commands::Complete(complete) => handlers::complete_handler(task_logger, complete),
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
