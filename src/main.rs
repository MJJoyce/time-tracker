use crate::cli::Commands;

mod cfg;
mod cli;
mod entry;
mod handlers;
mod logger;

fn main() {
    let cli = cli::parse();
    let conf = cfg::load();

    let task_logger =
        logger::load_logger(conf.task_log).expect("Unable to parse log file. Exiting ...");

    let res = match &cli.command {
        Commands::Start(start) => handlers::start_handler(task_logger, start),
        Commands::End(end) => handlers::end_handler(task_logger, end),
        Commands::Complete(complete) => handlers::complete_handler(task_logger, complete),
        Commands::Status(status) => handlers::status_handler(task_logger, status),
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
