use std::net::IpAddr;

use crate::PluginHostError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginHostConfig {
    pub host: IpAddr,
    pub port: u16,
    pub token: Option<String>,
}

impl PluginHostConfig {
    pub fn validate(&self) -> Result<(), PluginHostError> {
        if self.token.as_deref().is_none_or(str::is_empty) && !self.host.is_loopback() {
            return Err(PluginHostError::TokenRequired);
        }
        Ok(())
    }
}
