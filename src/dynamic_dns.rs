//! Main handler for performing Dynamic DNS

use crate::{
    cf_types::DnsRecord,
    cloudflare::{CloudFlareApiClient, CloudflareZone},
    ip_change_watcher::IPChangeWatcher,
};
use anyhow::Result;
use log::{error, info};
use std::net::Ipv4Addr;

/// Use the [crate::IPChangeWatcher] to see if the public IP has changed,
/// iterate over `zones_to_monitor`, and use the [crate::CloudFlareApiClient]
/// to update DNS records for each zone, if necessary.
pub fn perform_dynamic_dns(
    ip_watcher: &mut IPChangeWatcher,
    zones_to_monitor: &[CloudflareZone],
    cf_client: &CloudFlareApiClient,
) -> Result<()> {
    ip_watcher.refresh_context()?;
    if !ip_watcher.did_public_ip_change() {
        return Ok(());
    };
    for zone in zones_to_monitor.iter() {
        info!("processing zone {}", zone.zone_id);
        let mut records = zone.get_dns_records(cf_client)?;
        for mut record in records.drain(..) {
            match maybe_update_dns_record(
                &mut record,
                ip_watcher.get_public_ip(),
            ) {
                DnsRecordAction::Updated => {
                    zone.save_dns_record(cf_client, &record)?;
                    info!("Zone {} updated", zone.zone_id);
                }
                DnsRecordAction::Noop => {
                    info!("Noop for zone {}", zone.zone_id);
                }
                DnsRecordAction::Error => {
                    error!("Error updating zone {}", zone.zone_id);
                }
            }
        }
    }
    ip_watcher.persist_current_ip()?;
    Ok(())
}

#[cfg_attr(test, derive(Eq, PartialEq, Debug))]
pub enum DnsRecordAction {
    Updated,
    Noop,
    Error,
}
pub fn maybe_update_dns_record(
    record: &mut DnsRecord,
    current_ip: Ipv4Addr,
) -> DnsRecordAction {
    if record.record_type != "A" {
        return DnsRecordAction::Noop;
    }
    if record.locked {
        error!(
            "Warning: skipping record {} because it is locked",
            record.id
        );
        return DnsRecordAction::Noop;
    }
    let res = record.content.parse::<Ipv4Addr>();
    match res {
        Ok(record_ip) => {
            if record_ip != current_ip {
                record.content = current_ip.to_string();
                DnsRecordAction::Updated
            } else {
                DnsRecordAction::Noop
            }
        }
        Err(_) => DnsRecordAction::Error,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn mock_dns_record() -> DnsRecord {
        DnsRecord {
            content: "1.2.3.4".into(),
            name: "Mock Record".into(),
            proxied: true,
            record_type: "A".into(),
            created_on: "today".into(),
            id: "something".into(),
            locked: false,
            modified_on: "today".into(),
            proxiable: true,
            tags: vec![],
            ttl: 1,
            zone_id: "something".into(),
            zone_name: "something".into(),
        }
    }

    #[test]
    fn test_maybe_update_skip_updating_non_type_a() {
        let mut record = mock_dns_record();
        record.record_type = "not-type-a".into();
        let action =
            maybe_update_dns_record(&mut record, Ipv4Addr::new(4, 4, 4, 4));
        assert_eq!(action, DnsRecordAction::Noop);
    }

    /// The "maybe_update" function should mutate the record if all conditions
    /// for update are met. In this case, it should also return an action
    /// type which indicates that the record was updated.
    #[test]
    fn test_maybe_update_mutating_type_a() {
        let mut record = mock_dns_record();
        let new_ip = Ipv4Addr::new(4, 4, 4, 4);
        let res = maybe_update_dns_record(&mut record, new_ip);
        assert_eq!(record.content, new_ip.to_string());
        assert_eq!(res, DnsRecordAction::Updated);
    }

    /// If the record is locked, we will perform a Noop, even if the record
    /// is of type A, and the IPs do not match. Fwiw, the IP on the record
    /// also will not be mutated in this case.
    #[test]
    fn test_maybe_update_lock_noop() {
        let mut record = mock_dns_record();
        record.locked = true;
        let new_ip = Ipv4Addr::new(4, 4, 4, 4);
        let res = maybe_update_dns_record(&mut record, new_ip);
        assert_ne!(record.content, new_ip.to_string());
        assert_eq!(res, DnsRecordAction::Noop);
    }

    /// If, for whatever reason, the `.content` property on the CloudFlare
    /// record is not parsable as an IPv4 addr, we will return the error
    /// variabt of [DnsRecordAction]
    #[test]
    fn test_maybe_update_err() {
        let mut record = mock_dns_record();
        record.content = "This ain't no IP I ever heard of".into();
        let new_ip = Ipv4Addr::new(4, 4, 4, 4);
        let res = maybe_update_dns_record(&mut record, new_ip);
        assert_eq!(res, DnsRecordAction::Error);
    }
}
