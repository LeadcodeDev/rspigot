use std::fmt;
use std::net::IpAddr;

use crate::PluginHostError;

#[derive(Clone, PartialEq, Eq)]
pub struct PluginHostConfig {
    pub host: IpAddr,
    pub port: u16,
    pub token: Option<String>,
}

impl fmt::Debug for PluginHostConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluginHostConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("token", &self.token.as_ref().map(|_| "***"))
            .finish()
    }
}

impl PluginHostConfig {
    pub fn validate(&self) -> Result<(), PluginHostError> {
        if self.token.as_deref().is_none_or(str::is_empty) && !self.host.is_loopback() {
            return Err(PluginHostError::TokenRequired);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_hides_the_token() {
        let config = PluginHostConfig {
            host: IpAddr::from([0, 0, 0, 0]),
            port: 25580,
            token: Some("s3cret".to_owned()),
        };
        let shown = format!("{config:?}");
        assert!(!shown.contains("s3cret"));
        assert!(shown.contains(r#"token: Some("***")"#));
        let tokenless = PluginHostConfig {
            token: None,
            ..config
        };
        assert!(format!("{tokenless:?}").contains("token: None"));
    }
}
