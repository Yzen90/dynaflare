#![forbid(unsafe_code)]

mod configuration;
mod constants;
mod types;
mod utils;

use anyhow::{anyhow, Context, Error, Result};
use configuration::Configuration;
use log::{debug, info, warn};
use std::{thread::sleep, time::Duration};
use ureq::{Agent, AgentBuilder, Error as RequestError};

use constants::AUTHORIZATION_HEADER;
use types::*;

fn main() -> Result<()> {
  let configuration = configuration::load()?;
  utils::logging::initialize(&configuration)?;

  if configuration.records.is_empty() {
    warn!("No records provided, nothing to do!");
  } else {
    let interval = configuration.interval.unwrap_or(Duration::ZERO);
    let client = AgentBuilder::new().https_only(true).build();

    let (records_ids, mut last_ip, url) = sync_records(&client, &configuration)?;

    info!("Records IDs: {:#?}", records_ids);

    if !interval.is_zero() {
      loop {
        sleep(interval);

        let current_ip = public_ip(&client).context("Public IP retrieval")?;
        if current_ip != last_ip {
          last_ip = current_ip;

          dns_batch(
            &client,
            &url,
            &configuration,
            Batch {
              patches: records_ids.iter().map(|id| RecordUpdate { id: id.clone(), content: last_ip.clone() }).collect(),
            },
          )?;

          info!("Public IP changed, the DNS records have been updated")
        }
      }
    }
  }

  Ok(())
}

fn sync_records(client: &Agent, configuration: &Configuration) -> Result<(Vec<String>, String, String)> {
  let mut base_url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", configuration.zone);
  let dns_records_url = format!("{base_url}");

  let records_list = dns_records(&client, &dns_records_url, &configuration).context("DNS records retrieval")?;
  let last_ip = public_ip(&client).context("Public IP retrieval")?;

  let mut records = configuration.records.clone();
  let mut records_ids = Vec::<String>::with_capacity(records.len());
  let mut changed_records = Vec::<RecordUpdate>::new();
  let mut new_records = Vec::<RecordCreate>::new();

  info!("zone records: {}", records_list.len());
  info!("records: {}", records.len());

  for record in records_list {
    if let Some(index) = records.iter().position(|name| *name == record.name) {
      records_ids.push(record.id.clone());

      if record.content != last_ip {
        changed_records.push(RecordUpdate { id: record.id, content: last_ip.clone() });
      }

      records.swap_remove(index);
      if records.len() == 0 {
        break;
      }
    }
  }

  info!("new records: {}", records.len());

  for record in records {
    new_records.push(RecordCreate { name: record.clone(), content: last_ip.clone(), ttl: TTL::TTL });
  }

  base_url.push_str("/batch");
  if !new_records.is_empty() {
    records_ids.extend(dns_batch_with_create(
      &client,
      &base_url,
      &configuration,
      BatchWithCreate { patches: changed_records, posts: new_records },
    )?);
  } else if !changed_records.is_empty() {
    dns_batch(&client, &base_url, &configuration, Batch { patches: changed_records })?;
  }

  Ok((records_ids, last_ip, base_url))
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
  let data: Records = client
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
    .into_json()
    .context("Unexpected Cloudflare API response")?;

  if data.errors.is_empty() {
    Ok(data.result)
  } else {
    Err(anyhow!(flatten_error(data.errors)))
  }
}

fn dns_batch(client: &Agent, url: &str, configuration: &Configuration, batch: Batch) -> Result<()> {
  debug!("Sending DNS records batch");
  let data: BatchResult =
    client.get(url).set(AUTHORIZATION_HEADER, &configuration.api_token).send_json(batch)?.into_json()?;

  if data.errors.is_empty() {
    Ok(())
  } else {
    Err(anyhow!(flatten_error(data.errors)))
  }
}

fn dns_batch_with_create(
  client: &Agent,
  url: &str,
  configuration: &Configuration,
  batch: BatchWithCreate,
) -> Result<Vec<String>> {
  debug!("Sending DNS records batch with new records");

  let data: BatchWithCreateResult =
    client.get(url).set(AUTHORIZATION_HEADER, &configuration.api_token).send_json(batch)?.into_json()?;

  if data.errors.is_empty() {
    Ok(data.result.posts.iter().map(|record| record.id.clone()).collect())
  } else {
    Err(anyhow!(flatten_error(data.errors)))
  }
}

fn flatten_error(errors: Vec<APIError>) -> String {
  errors.iter().map(|APIError { code, message }| format!("[{code}] {message} ")).collect()
}
