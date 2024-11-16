use std::io::stderr;

use anyhow::{Context, Result};
use fern::colors::{Color, ColoredLevelConfig};
use fern::Dispatch;
use log::LevelFilter::{Debug, Info};

use crate::configuration::Configuration;

pub fn initialize(configuration: &Configuration) -> Result<()> {
  let colors = ColoredLevelConfig::new()
    .trace(Color::BrightBlack)
    .debug(Color::White)
    .info(Color::Green)
    .warn(Color::Yellow)
    .error(Color::Red);

  let log_level = configuration.log_level.unwrap_or(Info);
  let deps_log_level = if log_level == Debug { Info } else { log_level };

  Dispatch::new()
    .format(move |out, message, record| {
      out.finish(format_args!("[{}] {} {}", record.target(), colors.color(record.level()), message))
    })
    .level(log_level)
    .level_for("ureq", deps_log_level)
    .level_for("rustls", deps_log_level)
    .chain(stderr())
    .apply()
    .context("Logging initialization")?;

  Ok(())
}
