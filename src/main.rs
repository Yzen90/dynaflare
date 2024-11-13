#![forbid(unsafe_code)]

mod configuration;
mod constants;
mod utils;

use anyhow::{Ok, Result};

fn main() -> Result<()> {
  let configuration = configuration::load()?;

  if let Some(interval) = configuration.interval {
    println!("{:#?}", interval);
  }

  Ok(())
}
