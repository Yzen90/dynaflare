use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct Record {
  pub id: String,
  pub name: String,
  pub content: String,
}

#[derive(Deserialize, Serialize)]
pub struct APIError {
  pub code: u32,
  pub message: String,
}

#[derive(Deserialize)]
pub struct Records {
  pub result: Vec<Record>,
  pub errors: Vec<APIError>,
}

#[derive(Deserialize)]
pub struct IP {
  pub ip: String,
}

#[derive(Deserialize, Serialize)]
pub struct BatchResult {
  pub errors: Vec<APIError>,
}

#[derive(Deserialize, Serialize)]
pub struct NewRecord {
  pub id: String,
}

#[derive(Deserialize, Serialize)]
pub struct CreateResult {
  pub posts: Vec<NewRecord>,
}

#[derive(Deserialize, Serialize)]
pub struct BatchWithCreateResult {
  pub result: CreateResult,
  pub errors: Vec<APIError>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "A")]
pub struct RecordUpdate {
  pub id: String,
  pub content: String,
}

#[derive(Serialize)]
pub enum TTL {
  #[serde(rename = "60")]
  TTL,
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "A")]
pub struct RecordCreate {
  pub name: String,
  pub content: String,
  pub ttl: TTL,
}

#[derive(Serialize)]
pub struct Batch {
  pub patches: Vec<RecordUpdate>,
}

#[derive(Serialize)]
pub struct BatchWithCreate {
  pub patches: Vec<RecordUpdate>,
  pub posts: Vec<RecordCreate>,
}
