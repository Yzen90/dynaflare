#![forbid(unsafe_code)]

mod configuration;
mod constants;
mod utils;

use log::{info, warn};
use std::{thread::sleep, time::Duration};

use anyhow::{Ok, Result};

fn main() -> Result<()> {
  let configuration = configuration::load()?;

  utils::logging::initialize(&configuration)?;

  if configuration.records.is_empty() {
    warn!("Records list is empty, nothing to do!");
    sleep(Duration::from_secs(30));
  } else {
    let interval = configuration.interval.unwrap_or(Duration::ZERO);

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
