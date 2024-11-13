use std::time::Duration;

use anyhow::{Context, Result};
use figment::{
  providers::{Format, Toml},
  Figment,
};
use serde::Deserialize;

use crate::constants::CONFIG_FILE;
use crate::utils::duration_human::deserialize_duration;

#[derive(Deserialize)]
pub struct Configuration {
  #[serde(default, deserialize_with = "deserialize_duration")]
  pub interval: Option<Duration>,
}

pub fn load() -> Result<Configuration> {
  Figment::new().merge(Toml::file(CONFIG_FILE)).extract().context("Configuration load")
}
