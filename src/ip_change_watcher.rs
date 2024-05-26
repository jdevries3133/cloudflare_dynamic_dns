//! Utility for monitoring our public IP address and notifying other parts
//! of the system about changes.

use anyhow::Result;
use reqwest::blocking::get;
use std::net::Ipv4Addr;

pub struct IPChangeWatcher {
    /// [Self::old_ip] is optional because it may not be known; such as on
    /// initial startup. In this case, we will initialize with [Self::old_ip]
    /// set to [Option::None], which means that [Self::did_dns_change] will
    /// always return `true`.
    old_ip: Option<Ipv4Addr>,
    current_ip: Ipv4Addr,
    public_ip_provider_service_url: String,
}

impl IPChangeWatcher {
    /// May fail if we're unable to get the current public IP from the
    /// internet.
    ///
    /// `public_ip_provider` should be the URL at which we can access an
    /// external web service which will report back our own public IP.
    /// The response from hitting this API should be our public IPv4 address
    /// in plain text. A free and public option is to pass
    /// `https://checkip.amazonaws.com`.
    pub fn new(public_ip_provider_service_url: String) -> Result<Self> {
        let current_ip =
            Self::load_current_ip(&public_ip_provider_service_url)?;
        Ok(Self {
            old_ip: None,
            current_ip,
            public_ip_provider_service_url,
        })
    }
    pub fn did_public_ip_change(&self) -> bool {
        if let Some(old_ip) = self.old_ip {
            if old_ip != self.current_ip {
                println!(
                    "IP address changed ({} => {})",
                    self.old_ip
                        .map(|i| i.to_string())
                        .unwrap_or("unknown".to_string()),
                    self.current_ip
                );
            }
            old_ip != self.current_ip
        } else {
            println!("Indicating IP change because old IP is unknown");
            true
        }
    }
    pub fn persist_current_ip(&mut self) -> Result<()> {
        self.old_ip = Some(self.current_ip);
        Ok(())
    }
    pub fn get_public_ip(&self) -> Ipv4Addr {
        self.current_ip
    }
    pub fn refresh_context(&mut self) -> Result<()> {
        self.current_ip =
            Self::load_current_ip(&self.public_ip_provider_service_url)?;
        Ok(())
    }
    fn load_current_ip(service_url: &str) -> Result<Ipv4Addr> {
        let text = get(service_url)?.text()?;
        Ok(text.trim().parse()?)
    }
}
