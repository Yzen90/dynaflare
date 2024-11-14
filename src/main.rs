#![forbid(unsafe_code)]

mod configuration;
mod constants;
mod types;
mod utils;

use anyhow::{Context, Error, Ok, Result};
use configuration::Configuration;
use log::{debug, info, warn};
use std::{thread::sleep, time::Duration};
use ureq::{Agent, AgentBuilder, Error as RequestError};

use constants::AUTHORIZATION_HEADER;
use types::{Batch, Record, RecordCreate, RecordUpdate, Records, IP};

fn main() -> Result<()> {
  let configuration = configuration::load()?;

  utils::logging::initialize(&configuration)?;

  if configuration.records.is_empty() {
    warn!("No records provided, nothing to do!");
  } else {
    let interval = configuration.interval.unwrap_or(Duration::ZERO);

    let client = AgentBuilder::new().https_only(true).build();
    let base_url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", configuration.zone);

    let mut record_ids = Vec::<String>::with_capacity(configuration.records.len());
    let mut last_ip: String;

    {
      let dns_records_url = format!("{base_url}?type=A");
      let records = dns_records(&client, &dns_records_url, &configuration).context("DNS records retrieval")?;
      last_ip = public_ip(&client).context("Public IP retrieval")?;

      let mut existing = Vec::<String>::new();
      let mut record_updates = Vec::<RecordUpdate>::new();
      let mut new_records = Vec::<RecordCreate>::new();

      for record in records {
        if let Some(_) = configuration.records.iter().find(|&name| *name == record.name) {
          existing.push(record.name);
          record_ids.push(record.id.clone());

          if record.content != last_ip {
            update_batch.push(RecordUpdate { id: record.id, content: last_ip.clone() });
          }
        }
      }

      for record in configuration.records {
        if let Some(_) = existing.iter().find(|&name| *name == record) {}
      }
    }

    if !interval.is_zero() {
      loop {
        sleep(interval);
        let current_ip = public_ip(&client).context("Public IP retrieval")?;

        if current_ip != last_ip {
          last_ip = current_ip;
        }
      }
    }
  }

  Ok(())
}

fn public_ip(client: &Agent) -> Result<String> {
  debug!("Retrieving public IP");
  Ok(
    client
      .get("https://api.ipify.org?format=json")
      .call()?
      .into_json::<IP>()
      .context("Unexpected ipify API response")?
      .ip,
  )
}

fn dns_records(client: &Agent, url: &str, configuration: &Configuration) -> Result<Vec<Record>> {
  debug!("Retrieving DNS records for zone: {}", configuration.zone);
  Ok(
    client
      .get(url)
      .set(AUTHORIZATION_HEADER, &configuration.api_token)
      .call()
      .map_err(|error| match error {
        RequestError::Status(code, _) => match code {
          404 => Error::new(error).context("Invalid zone"),
          400 => Error::new(error).context("Invalid API token"),
          _ => Error::new(error),
        },
        _ => Error::new(error),
      })?
      .into_json::<Records>()
      .context("Unexpected Cloudflare API response")?
      .result,
  )
}

fn dns_batch(records: Batch) -> Result<()> {
  Ok(())
}
