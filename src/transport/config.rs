use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportConfig {
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub user_agent: String,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(20),
            connect_timeout: Duration::from_secs(10),
            user_agent: concat!("gobelin/", env!("CARGO_PKG_VERSION")).to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_identifies_crate_when_unconfigured() {
        let config = TransportConfig::default();

        assert_eq!(config.timeout, Duration::from_secs(20));
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert_eq!(
            config.user_agent,
            format!("gobelin/{}", env!("CARGO_PKG_VERSION"))
        );
    }
}
