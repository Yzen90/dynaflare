use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Record {
  pub id: String,
  pub name: String,
  pub content: String,
}

#[derive(Deserialize)]
pub struct Records {
  pub result: Vec<Record>,
}

#[derive(Deserialize)]
pub struct IP {
  pub ip: String,
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "A")]
pub struct RecordUpdate {
  pub id: String,
  pub content: String,
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "A")]
pub struct RecordCreate {
  pub name: String,
  pub content: String,
}

#[derive(Serialize)]
pub struct Batch {
  pub patches: Option<Vec<RecordUpdate>>,
  pub posts: Option<Vec<RecordCreate>>,
}
