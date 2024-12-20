#![forbid(unsafe_code)]

mod configuration;
mod constants;
mod types;
mod utils;

use anyhow::{Context, Error, Result};
use configuration::Configuration;
use log::{debug, error, info, warn};
use std::{thread::sleep, time::Duration};
use ureq::{serde_json, Agent, AgentBuilder, Error as RequestError};

use constants::*;
use types::*;

fn main() -> Result<()> {
  let configuration = configuration::load()?;
  utils::logging::initialize(&configuration)?;

  info!("DynaFlare v{}", VERSION);

  if configuration.records.is_empty() {
    warn!("No records provided, nothing to do!");
  } else {
    let interval = configuration.interval.unwrap_or(Duration::ZERO);
    let client = AgentBuilder::new().https_only(true).build();

    let (records_ids, mut last_ip, url) = sync_records(&client, &configuration)?;

    if !interval.is_zero() {
      let count_repeated_errors = configuration.count_repeated_errors.unwrap_or(false);
      let mut last_error = String::new();
      let mut error_count: usize = 0;

      loop {
        sleep(interval);

        match public_ip(&client).context("Retrieving Public IP") {
          Ok(current_ip) => {
            if current_ip != last_ip {
              match dns_batch(
                &client,
                &url,
                &configuration,
                Batch {
                  patches: records_ids
                    .iter()
                    .map(|id| RecordUpdate { id: id.clone(), content: current_ip.clone() })
                    .collect(),
                },
              )
              .context("DNS records batch operation")
              {
                Ok(()) => {
                  if error_count > 0 {
                    log_errors(None, &mut last_error, &mut error_count, &count_repeated_errors);
                  }

                  info!(
                    "Public IP changed, DNS records have been updated. Current: {} Previous: {}",
                    current_ip, last_ip
                  );

                  last_ip = current_ip;
                }
                Err(err) => log_errors(Some(err), &mut last_error, &mut error_count, &count_repeated_errors),
              };
            } else {
              if error_count == 0 {
                debug!("Public IP is {current_ip}. No records changes needed.")
              } else {
                log_errors(None, &mut last_error, &mut error_count, &count_repeated_errors);
                info!("Public IP is {current_ip}. No records changes needed.")
              }
            }
          }
          Err(err) => log_errors(Some(err), &mut last_error, &mut error_count, &count_repeated_errors),
        }
      }
    }
  }

  Ok(())
}

fn sync_records(client: &Agent, configuration: &Configuration) -> Result<(Vec<String>, String, String)> {
  let mut base_url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", configuration.zone_id);
  let dns_records_url = format!("{base_url}?type=A");

  let records_list = dns_records(&client, &dns_records_url, &configuration).context("DNS records retrieval")?;
  let current_ip = public_ip(&client).context("Public IP retrieval")?;

  let mut records = configuration.records.clone();
  let mut records_ids = Vec::<String>::with_capacity(records.len());
  let mut changed_records = Vec::<RecordUpdate>::new();
  let mut new_records = Vec::<RecordCreate>::new();

  for record in records_list {
    if let Some(index) = records.iter().position(|name| *name == record.name) {
      records_ids.push(record.id.clone());

      if record.content != current_ip {
        changed_records.push(RecordUpdate { id: record.id, content: current_ip.clone() });
      }

      records.swap_remove(index);
      if records.len() == 0 {
        break;
      }
    }
  }

  for record in records {
    new_records.push(RecordCreate { name: record.clone(), content: current_ip.clone(), ttl: DEFAULT_TTL });
  }

  let changed = changed_records.len();
  let new = new_records.len();
  base_url.push_str("/batch");

  if !new_records.is_empty() {
    records_ids.extend(
      dns_batch_with_create(
        &client,
        &base_url,
        &configuration,
        BatchWithCreate { patches: changed_records, posts: new_records },
      )
      .context("DNS records batch operation")?,
    );

    log_changes(&current_ip, new, changed);
  } else if !changed_records.is_empty() {
    dns_batch(&client, &base_url, &configuration, Batch { patches: changed_records })
      .context("DNS records batch operation")?;

    log_changes(&current_ip, new, changed);
  } else {
    info!("Public IP is {current_ip}. No records changes needed.")
  }

  Ok((records_ids, current_ip, base_url))
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
  debug!("Retrieving DNS records for zone: {}", configuration.zone_id);
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
    Err(Error::msg(flatten_error(data.errors)))
  }
}

fn dns_batch(client: &Agent, url: &str, configuration: &Configuration, batch: Batch) -> Result<()> {
  debug!("Sending DNS records batch. Data: {}", serde_json::ser::to_string(&batch)?);

  let data: BatchResult = client
    .post(url)
    .set(CONTENT_TYPE_HEADER, JSON_MIME)
    .set(AUTHORIZATION_HEADER, &configuration.api_token)
    .send_json(batch)
    .map_err(request_error)?
    .into_json()?;

  if data.errors.is_empty() {
    Ok(())
  } else {
    Err(Error::msg(flatten_error(data.errors)))
  }
}

fn dns_batch_with_create(
  client: &Agent,
  url: &str,
  configuration: &Configuration,
  batch: BatchWithCreate,
) -> Result<Vec<String>> {
  debug!("Sending DNS records batch with new records. Data: {}", serde_json::ser::to_string(&batch)?);

  let data: BatchWithCreateResult = client
    .post(url)
    .set(CONTENT_TYPE_HEADER, JSON_MIME)
    .set(AUTHORIZATION_HEADER, &configuration.api_token)
    .send_json(batch)
    .map_err(request_error)?
    .into_json()?;

  if data.errors.is_empty() {
    Ok(data.result.posts.iter().map(|record| record.id.clone()).collect())
  } else {
    Err(Error::msg(flatten_error(data.errors)))
  }
}

fn flatten_error(errors: Vec<APIError>) -> String {
  errors.iter().map(|APIError { code, message }| format!("[{code}] {message} ")).collect()
}

fn log_changes(current_ip: &String, new: usize, changed: usize) -> () {
  info!("Public IP is {current_ip}. Records created: {}, Records updated: {}.", new, changed)
}

fn request_error(error: RequestError) -> Error {
  match error {
    RequestError::Status(code, response) => {
      Error::msg(format!("[{}] {}", code, response.into_string().unwrap_or(String::new())))
    }
    _ => Error::new(error),
  }
}

fn log_errors(err: Option<Error>, last_error: &mut String, error_count: &mut usize, count: &bool) -> () {
  if let Some(err) = err {
    let current_error = format!("{:?}", err);

    if *count {
      if current_error != *last_error {
        if *error_count > 1 {
          error!("Additional {} errors of: {}", *error_count - 1, *last_error);
        }

        error!("{}", current_error);

        *error_count = 0;
        *last_error = current_error;
      }
    } else {
      error!("{}", current_error);
    }

    *error_count += 1;
  } else {
    if *count {
      if *error_count > 1 {
        error!("Additional {} errors of: {}", *error_count - 1, *last_error);
      }

      *last_error = String::new();
    }

    *error_count = 0;
  }
}
