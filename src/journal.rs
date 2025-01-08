use std::error::Error;
use std::io::{BufRead, BufReader, Cursor};
use std::process::Command;

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Priority {
    Emergency,
    Alert,
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    Debug,
}

// A journal entry has many more fields than this, but I don't care about most of them
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    #[serde(rename(deserialize = "PRIORITY"))]
    #[serde(deserialize_with = "deserialize_priority")]
    pub priority: Priority,

    #[serde(rename(deserialize = "_SYSTEMD_UNIT"))]
    pub unit_name: String,

    #[serde(rename(deserialize = "_SYSTEMD_USER_UNIT"))]
    pub user_unit_name: Option<String>,

    #[serde(rename(deserialize = "__REALTIME_TIMESTAMP"))]
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub timestamp: DateTime<Utc>,

    #[serde(rename(deserialize = "_HOSTNAME"))]
    pub hostname: String,

    #[serde(rename(deserialize = "_CMDLINE"))]
    pub cmdline: String,

    #[serde(rename(deserialize = "MESSAGE"))]
    pub message: String,
}

// TODO: replace with libsystemd ffi
pub fn get_journal_entries(
    unit: &str,
    user: bool,
    since: DateTime<Local>,
) -> Result<Vec<JournalEntry>, Box<dyn Error>> {
    let scope = if user { "--user" } else { "--system" };
    let cmd = Command::new("journalctl")
        .args([
            scope,
            "--unit",
            unit,
            "--since",
            since.to_rfc3339().as_str(),
            "--output",
            "json",
        ])
        .output()?;

    let cursor = Cursor::new(cmd.stdout);
    let reader = BufReader::new(cursor);

    Ok(parse_journal_entries(reader))
}

fn parse_journal_entries(reader: impl BufRead) -> Vec<JournalEntry> {
    reader
        .lines()
        .map(|l| l.unwrap())
        .map(|l| serde_json::from_str(&l))
        .filter_map(|en| en.inspect_err(|e| eprintln!("{}", e)).ok())
        .collect()
}

fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp = String::deserialize(deserializer)?;
    let timestamp = timestamp.parse::<i64>().map_err(serde::de::Error::custom)?;
    Ok(DateTime::from_timestamp_micros(timestamp).unwrap())
}

fn deserialize_priority<'de, D>(deserializer: D) -> Result<Priority, D::Error>
where
    D: Deserializer<'de>,
{
    let priority = String::deserialize(deserializer)?;
    match priority.to_lowercase().as_str() {
        "0" | "emerg" => Ok(Priority::Emergency),
        "1" | "alert" => Ok(Priority::Alert),
        "2" | "crit" => Ok(Priority::Critical),
        "3" | "err" => Ok(Priority::Error),
        "4" | "warning" => Ok(Priority::Warning),
        "5" | "notice" => Ok(Priority::Notice),
        "6" | "info" => Ok(Priority::Info),
        "7" | "debug" => Ok(Priority::Debug),
        _ => Err(serde::de::Error::custom("Invalid priority, expected one of: emerg, 0, alert, 1, crit, 2, err, 3, warning, 4, notice, 5, info, 6, debug, 7",
        )),
    }
}
