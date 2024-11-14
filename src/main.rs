#![forbid(unsafe_code)]

mod configuration;
mod constants;
mod utils;

use anyhow::Result;
use log::{info, warn};
use std::{thread::sleep, time::Duration};
use ureq::serde_json::Value;

fn main() -> Result<()> {
  let configuration = configuration::load()?;

  utils::logging::initialize(&configuration)?;

  if configuration.records.is_empty() {
    warn!("Records list is empty, nothing to do!");
    sleep(Duration::from_secs(30));
  } else {
    let interval = configuration.interval.unwrap_or(Duration::ZERO);

    let base_url =
      String::new() + "https://api.cloudflare.com/client/v4/zones/" + configuration.zone.as_str() + "/dns_records";

    info!("{}", base_url);

    dns_records(&base_url)?;

    if interval.is_zero() {
      info!("ONE SHOT");
    } else {
      loop {
        info!("{:#?}", interval);
        sleep(interval);
      }
    }
  }

  Ok(())
}

fn dns_records(url: &str) -> Result<()> {
  let response: Value = ureq::get(url).call()?.into_json()?;
  info!("{}", response);
  Ok(())
}
