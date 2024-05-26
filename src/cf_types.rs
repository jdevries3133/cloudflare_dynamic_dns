//! Data-types we'll need to JSON-serialize or JSON-deserialize to interact
//! with the CloudFlare HTTP API.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiResponse {
    pub errors: Vec<String>,
    pub messages: Vec<String>,
    pub success: bool,
    pub result: Vec<DnsRecord>,
    pub result_info: ResultInfo,
}

#[derive(Serialize, Deserialize)]
pub struct DnsRecord {
    pub content: String,
    pub name: String,
    pub proxied: bool,
    #[serde(rename = "type")]
    pub record_type: String,
    pub created_on: String,
    pub id: String,
    pub locked: bool,
    pub modified_on: String,
    pub proxiable: bool,
    pub tags: Vec<String>,
    pub ttl: u32,
    pub zone_id: String,
    pub zone_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ResultInfo {
    pub count: u32,
    pub page: u32,
    pub per_page: u32,
    pub total_count: u32,
}
