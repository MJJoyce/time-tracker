use std::error::Error;
use std::fs;
use std::path::Path;

use crate::entry::LogEntry;

pub trait TTLogger {
    /// Write to the TT Logger.
    fn write(&mut self, entry: LogEntry) -> Result<(), Box<dyn Error>>;

    /// Initialize the TT Logger as necessary
    fn init(&mut self) -> Result<(), Box<dyn Error>>;
}

pub struct CSVLog {
    log_loc: String,
}

impl TTLogger for CSVLog {
    fn write(&mut self, entry: LogEntry) -> Result<(), Box<dyn Error>> {
        let file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&self.log_loc)?;

        let mut wtr = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);

        wtr.serialize(entry)?;
        wtr.flush()?;
        Ok(())
    }

    fn init(&mut self) -> Result<(), Box<dyn Error>> {
        dbg!("CSV init");
        let log_path = Path::new(&self.log_loc);

        if log_path.is_file() {
            return Ok(());
        }

        match log_path.parent() {
            Some(d) => {
                if !d.is_dir() {
                    fs::create_dir_all(d)?;
                    dbg!("Creating dir {:?}", d);
                }
            }
            // We only end up here if if Path::parent() terminates in a root or
            // prefix. In that case we shouldn't need to create that directory.
            // All should be fine.
            _ => {
                dbg!("Didn't create dir {:?}", log_path);
            }
        }

        Ok(())
    }
}

pub fn load_logger(log_loc: String) -> Option<impl TTLogger> {
    match Path::new(&log_loc).extension() {
        Some(ext) => match ext.to_str() {
            Some("csv") => {
                let mut l = CSVLog { log_loc };
                match l.init() {
                    Ok(()) => Some(l),
                    Err(e) => {
                        eprintln!("CSVLog init failed: {:?}", e);
                        None
                    }
                }
            }
            _ => {
                eprintln!("Unrecognized log file format extension {:?}", ext);
                None
            }
        },
        None => {
            eprintln!(
                "Unable to extract extension from log file path {:?}",
                log_loc
            );
            None
        }
    }
}
