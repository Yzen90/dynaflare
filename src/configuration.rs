use std::time::Duration;

use anyhow::{Context, Result};
use figment::{
  providers::{Format, Toml},
  Figment,
};
use log::LevelFilter;
use serde::Deserialize;

use crate::constants::CONFIG_FILE;
use crate::utils::duration_human::deserialize_duration;

#[derive(Deserialize)]
pub struct Configuration {
  #[serde(default, deserialize_with = "deserialize_duration")]
  pub interval: Option<Duration>,
  // pub api_token: String,
  pub zone: String,
  pub records: Vec<String>,
  pub log_level: Option<LevelFilter>,
}

pub fn load() -> Result<Configuration> {
  Figment::new().merge(Toml::file(CONFIG_FILE)).extract().context("Configuration load")
}
