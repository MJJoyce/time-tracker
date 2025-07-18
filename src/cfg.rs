use std::env;
use std::path::Path;

const CONFIG_KEY: &str = "TT_CONF";
const DEFAULT_CONFIG_PATH: &str = "~/.config/time-tracker/conf.toml";

const LOG_KEY: &str = "TT_LOG";
const DEFAULT_LOG_PATH: &str = "~/.time-tracker/log.csv";

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    config_file: Option<String>,
    pub task_log: String,
}

pub fn load() -> Config {
    let cfg_path = match env::var(CONFIG_KEY) {
        Ok(path) => path,
        Err(_) => DEFAULT_CONFIG_PATH.to_string(),
    };

    let conf = if Path::new(&cfg_path).is_file() {
        // TODO:
        // Implement config file parsing here
        panic!();
    } else {
        let log_path = match env::var(LOG_KEY) {
            Ok(path) => path,
            Err(_) => DEFAULT_LOG_PATH.to_string(),
        };

        Config {
            config_file: None,
            task_log: shellexpand::full(&log_path).unwrap().into(),
        }
    };

    conf
}
