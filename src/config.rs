use std::{
    error::Error,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::Deserialize;

#[serde_with::serde_as]
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    /// Journal lookback (in seconds)
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[serde(default = "default_lookback")]
    pub lookback: Duration,

    pub send_from: String,
    pub send_to: Vec<String>,

    pub template: PathBuf,
    pub css: Option<PathBuf>,
}

fn default_lookback() -> Duration {
    Duration::from_secs(5 * 60)
}

pub fn load_config(config_path: &Path) -> Result<Config, Box<dyn Error>> {
    let toml = std::fs::read_to_string(config_path)?;
    let config = toml::from_str(toml.as_str())?;

    Ok(config)
}
