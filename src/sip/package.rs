use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageRecord {
    pub name: String,
    pub version: String,
    pub hash: String,           // “sha256:…”
    pub trust_score: f64,
    pub endorsed_by: Vec<String>,
    pub last_reviewed: String,  // ISO date
    pub source: String,         // must be a valid URI
}
