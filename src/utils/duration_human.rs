use std::{
  fmt::{Formatter, Result as FMTResult},
  time::Duration,
};

use duration_human::DurationHuman;
use serde::{
  de::{Error as DeserializeError, Visitor},
  Deserializer,
};

struct DurationHumanVisitor;

impl<'de> Visitor<'de> for DurationHumanVisitor {
  type Value = Option<Duration>;

  fn expecting(&self, formatter: &mut Formatter) -> FMTResult {
    formatter.write_str("human readable duration formatted string")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: DeserializeError,
  {
    Ok(Some(Duration::from(&DurationHuman::try_from(value).map_err(DeserializeError::custom)?)))
  }
}

pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_str(DurationHumanVisitor)
}
