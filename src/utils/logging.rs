use std::io::stderr;

use anyhow::{Context, Result};
use fern::colors::{Color, ColoredLevelConfig};
use fern::Dispatch;
use log::LevelFilter::Info;

use crate::configuration::Configuration;

pub fn initialize(configuration: &Configuration) -> Result<()> {
  let colors = ColoredLevelConfig::new()
    .trace(Color::BrightBlack)
    .debug(Color::White)
    .info(Color::Green)
    .warn(Color::Yellow)
    .error(Color::Red);

  Dispatch::new()
    .format(move |out, message, record| {
      out.finish(format_args!("[{}] {} {}", record.target(), colors.color(record.level()), message))
    })
    .level(configuration.log_level.unwrap_or(Info))
    .chain(stderr())
    .apply()
    .context("Logging initialization")?;

  Ok(())
}
