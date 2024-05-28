//! Uses `checkip.amazonaws.com` or a similar service to monitor the public IP
//! address and update selected CloudFlare zones upon a public IP address
//! change.

use anyhow::Result;
use log::info;
use std::{env, time::Duration};

mod cf_types;
mod cloudflare;
mod dynamic_dns;
mod ip_change_watcher;
mod zone_id;

use cloudflare::{CloudFlareApiClient, CloudflareZone};
use dynamic_dns::perform_dynamic_dns;
use ip_change_watcher::IPChangeWatcher;
use zone_id::ZoneId;

/// In an infinite loop, monitor our public IP address and update CloudFlare
/// DNS records if our public IP changes.
fn main() -> Result<()> {
    env_logger::init();
    let poll_interval = Duration::from_secs(300);
    let cf_client = CloudFlareApiClient {
        api_key: env::var("CLOUDFLARE_API_KEY")
            .expect("CLOUDFLARE_API_KEY is present in the environment"),
    };
    let zones_to_monitor = [
        CloudflareZone {
            zone_id: ZoneId::new("0e3c65bfec2453a4ae85a492c09dbe5c")?,
        },
        CloudflareZone {
            zone_id: ZoneId::new("191b0dc2fa5002fd4dfe517a430b78f2")?,
        },
        CloudflareZone {
            zone_id: ZoneId::new("2407484328fca8298a86e6d80abf9b70")?,
        },
        CloudflareZone {
            zone_id: ZoneId::new("30ed3e88cd9a56e0eb2b326a63500f4e")?,
        },
        CloudflareZone {
            zone_id: ZoneId::new("556ec9676ad2cbb97aca9ed6821abe2e")?,
        },
        CloudflareZone {
            zone_id: ZoneId::new("895f3cf17bce5139a2d76efbc5c7037f")?,
        },
        CloudflareZone {
            zone_id: ZoneId::new("a7ff542a3fed09794c18d244dd088c3d")?,
        },
        CloudflareZone {
            zone_id: ZoneId::new("e0771f82634175079c0df7e614a868db")?,
        },
    ];
    let mut ip_watcher =
        IPChangeWatcher::new("https://checkip.amazonaws.com".to_string())?;
    loop {
        info!("Beginning dynamic DNS check");
        perform_dynamic_dns(
            &mut ip_watcher,
            &zones_to_monitor[..],
            &cf_client,
        )?;
        info!("Dynamic DNS check is done");
        std::thread::sleep(poll_interval);
    }
}
