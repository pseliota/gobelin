use std::time::Duration;

use super::{ClientError, executor::ReqwestExecutor, http::HttpExecutor};

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

impl TransportConfig {
    pub(crate) fn executor(&self) -> Result<Box<dyn HttpExecutor>, ClientError> {
        Ok(Box::new(ReqwestExecutor::new(self)?))
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

    #[test]
    fn executor_builds_executor_when_config_is_default() {
        let config = TransportConfig::default();

        let executor = config.executor();

        assert!(executor.is_ok());
    }

    #[test]
    fn executor_reports_initialization_when_user_agent_is_invalid() {
        let config = TransportConfig {
            user_agent: "gobelin\nagent".into(),
            ..Default::default()
        };

        let error = config.executor().err();

        assert_eq!(error, Some(ClientError::Initialization));
    }
}
