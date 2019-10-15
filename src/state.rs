use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use crate::util;
use chrono::{Utc, DateTime, TimeZone};

#[derive(Deserialize, Serialize)]
pub struct State {
    pub last_version_check_time: DateTime<Utc>,
}

impl State {
    pub fn read() -> Self {
        match fs::read_to_string(state_file_path()) {
            Ok(c) => {
                toml::from_str(&c).expect("valid state")
            }
            _ => Self {
                last_version_check_time: Utc.timestamp(0, 0),
            },
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let content = toml::to_string(&self).expect("encoded TOML string");
        let sf = state_file_path();
        fs::write(&sf, content).map_err(|e| format!("Cannot write file: {}", e))?;
        util::set_permissions(&sf, 0o600);
        Ok(())
    }
}

const STATE_FILE_PATH: &str = "~/.local/share/awscredx/state.toml";

fn state_file_path() -> PathBuf {
    util::path_to_absolute(STATE_FILE_PATH)
}