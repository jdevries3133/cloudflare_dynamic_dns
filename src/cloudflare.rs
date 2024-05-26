//! Interface to the CloudFlare HTTP API

use crate::{
    cf_types::{ApiResponse, DnsRecord},
    zone_id::ZoneId,
};
use anyhow::{Error, Result};
use reqwest::{
    blocking::{Client, RequestBuilder},
    header::{HeaderName, HeaderValue},
};

pub struct CloudFlareApiClient {
    pub api_key: String,
}

impl CloudFlareApiClient {
    pub fn add_auth(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        Ok(request.header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))?,
        ))
    }
}

pub struct CloudflareZone {
    pub zone_id: ZoneId,
}

impl CloudflareZone {
    pub fn get_dns_records(
        &self,
        cf_client: &CloudFlareApiClient,
    ) -> Result<Vec<DnsRecord>> {
        let zone_id = self.zone_id.to_string();
        let http_client = Client::new();
        let request = http_client.get(format!(
            "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records",
        ));
        let request = cf_client.add_auth(request)?;
        let response = request.send()?;
        if response.status().is_success() {
            let records: ApiResponse = response.json()?;
            Ok(records.result)
        } else {
            Err(Error::msg(format!(
                "API Err(get_dns_records): stat = {} text = {}",
                response.status(),
                response.text()?
            )))
        }
    }
    pub fn save_dns_record(
        &self,
        cf_client: &CloudFlareApiClient,
        record: &DnsRecord,
    ) -> Result<()> {
        let http_client = Client::new();
        let request = http_client
            .patch(format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                self.zone_id, record.id
            ))
            .json(record);
        let request = cf_client.add_auth(request)?;
        let response = request.send()?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::msg(format!(
                "API Err(save_dns_record): stat = {} text = {}",
                response.status(),
                response.text()?
            )))
        }
    }
}
