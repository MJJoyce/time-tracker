use std::error::Error;
use std::fs;
use std::path::Path;

use crate::entry::LogEntry;

pub trait TTLogger {
    /// Write to the TT Logger.
    fn write(&mut self, entry: LogEntry) -> Result<(), Box<dyn Error>>;

    /// Initialize the TT Logger as necessary
    fn init(&mut self) -> Result<(), Box<dyn Error>>;

    /// Clear the log
    fn clear_log(&self) -> Result<(), Box<dyn Error>>;
}

#[derive(Clone)]
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
        let log_path = Path::new(&self.log_loc);

        if log_path.is_file() {
            return Ok(());
        }

        if let Some(d) = log_path.parent() {
            if !d.is_dir() {
                fs::create_dir_all(d)?;
            }
        }

        Ok(())
    }

    fn clear_log(&self) -> Result<(), Box<dyn Error>> {
        fs::remove_file(&self.log_loc)?;
        Ok(())
    }
}

impl IntoIterator for CSVLog {
    type Item = LogEntry;
    type IntoIter = CSVLogIterator;

    fn into_iter(self) -> Self::IntoIter {
        let file = fs::OpenOptions::new()
            .read(true)
            .open(&self.log_loc)
            .unwrap();
        let rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(file);

        CSVLogIterator {
            file_iter: rdr.into_deserialize(),
        }
    }
}

pub struct CSVLogIterator {
    file_iter: csv::DeserializeRecordsIntoIter<fs::File, LogEntry>,
}

impl Iterator for CSVLogIterator {
    type Item = LogEntry;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: This is actual hell and makes me hate my life
        match self.file_iter.next() {
            Some(res) => match res {
                Ok(entry) => Some(entry),
                Err(e) => {
                    eprintln!("CSVLog iteration failed: {:?}", e);
                    None
                }
            },
            None => None,
        }
    }
}

pub fn load_logger(
    log_loc: String,
) -> Option<impl TTLogger + IntoIterator<Item = LogEntry> + Clone> {
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
