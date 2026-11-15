use std::{collections::HashMap, error::Error, fmt, io::ErrorKind};

const ALPACA_KEY: &[&str] = &["ALPACA_KEY", "ALPACA_API_KEY", "APCA_API_KEY_ID"];
const ALPACA_SECRET: &[&str] = &[
    "ALPACA_SECRET",
    "ALPACA_SECRET_KEY",
    "ALPACA_API_SECRET",
    "APCA_API_SECRET_KEY",
];
const FRED_API_KEY: &[&str] = &["FRED_API_KEY"];

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConfigError {
    Missing { name: &'static str },
    UnreadableEnvFile,
    InvalidEnvFile,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { name } => {
                write!(
                    f,
                    "Missing credential: set {name} in .env or the environment"
                )
            }
            Self::UnreadableEnvFile => f.write_str("Could not read .env"),
            Self::InvalidEnvFile => f.write_str("Could not parse .env; check its KEY=value syntax"),
        }
    }
}

impl Error for ConfigError {}

pub struct Credentials {
    environment: HashMap<String, String>,
    file: HashMap<String, String>,
}

impl Credentials {
    pub fn load() -> Result<Self, ConfigError> {
        let environment = [ALPACA_KEY, ALPACA_SECRET, FRED_API_KEY]
            .concat()
            .into_iter()
            .filter_map(|name| Some((name.to_owned(), std::env::var(name).ok()?)))
            .collect();
        let file = match dotenvy::from_path_iter(".env") {
            Ok(entries) => entries
                .collect::<Result<_, _>>()
                .map_err(|_| ConfigError::InvalidEnvFile)?,
            Err(dotenvy::Error::Io(error)) if error.kind() == ErrorKind::NotFound => HashMap::new(),
            Err(_) => return Err(ConfigError::UnreadableEnvFile),
        };
        Ok(Self::new(environment, file))
    }

    pub fn new(environment: HashMap<String, String>, file: HashMap<String, String>) -> Self {
        Self { environment, file }
    }

    pub fn alpaca_key(&self) -> Result<String, ConfigError> {
        self.credential(ALPACA_KEY)
    }

    pub fn alpaca_secret(&self) -> Result<String, ConfigError> {
        self.credential(ALPACA_SECRET)
    }

    pub fn fred_api_key(&self) -> Result<String, ConfigError> {
        self.credential(FRED_API_KEY)
    }

    fn credential(&self, names: &[&'static str]) -> Result<String, ConfigError> {
        [&self.environment, &self.file]
            .into_iter()
            .find_map(|source| {
                names
                    .iter()
                    .find_map(|name| source.get(*name).filter(|v| !v.trim().is_empty()))
            })
            .cloned()
            .ok_or(ConfigError::Missing { name: names[0] })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn alpaca_key_prefers_environment_when_both_sources_set_it() {
        let credentials = Credentials::new(
            values(&[("ALPACA_KEY", "from-env")]),
            values(&[("ALPACA_KEY", "from-file")]),
        );

        let key = credentials.alpaca_key();

        assert_eq!(key.as_deref(), Ok("from-env"));
    }

    #[test]
    fn alpaca_key_reads_file_when_environment_lacks_it() {
        let credentials =
            Credentials::new(HashMap::new(), values(&[("ALPACA_API_KEY", "from-file")]));

        let key = credentials.alpaca_key();

        assert_eq!(key.as_deref(), Ok("from-file"));
    }

    #[test]
    fn alpaca_key_accepts_alias_when_primary_name_is_unset() {
        let credentials =
            Credentials::new(values(&[("APCA_API_KEY_ID", "from-alias")]), HashMap::new());

        let key = credentials.alpaca_key();

        assert_eq!(key.as_deref(), Ok("from-alias"));
    }

    #[test]
    fn alpaca_key_uses_first_alias_when_several_are_set() {
        let credentials = Credentials::new(
            values(&[("APCA_API_KEY_ID", "third"), ("ALPACA_KEY", "first")]),
            HashMap::new(),
        );

        let key = credentials.alpaca_key();

        assert_eq!(key.as_deref(), Ok("first"));
    }

    #[test]
    fn alpaca_key_skips_value_when_value_is_blank() {
        let credentials = Credentials::new(
            values(&[("ALPACA_KEY", " \t")]),
            values(&[("ALPACA_KEY", "from-file")]),
        );

        let key = credentials.alpaca_key();

        assert_eq!(key.as_deref(), Ok("from-file"));
    }

    #[test]
    fn alpaca_secret_prefers_environment_alias_when_file_sets_primary_name() {
        let credentials = Credentials::new(
            values(&[("APCA_API_SECRET_KEY", "from-env")]),
            values(&[("ALPACA_SECRET", "from-file")]),
        );

        let secret = credentials.alpaca_secret();

        assert_eq!(secret.as_deref(), Ok("from-env"));
    }

    #[test]
    fn alpaca_secret_names_primary_variable_when_every_value_is_blank() {
        let credentials = Credentials::new(
            values(&[("ALPACA_SECRET_KEY", "")]),
            values(&[("ALPACA_API_SECRET", "  ")]),
        );

        let error = credentials.alpaca_secret().unwrap_err();

        assert_eq!(
            error.to_string(),
            "Missing credential: set ALPACA_SECRET in .env or the environment"
        );
    }

    #[test]
    fn fred_api_key_reads_value_when_file_sets_it() {
        let credentials = Credentials::new(HashMap::new(), values(&[("FRED_API_KEY", "fred-key")]));

        let key = credentials.fred_api_key();

        assert_eq!(key.as_deref(), Ok("fred-key"));
    }

    #[test]
    fn fred_api_key_names_variable_when_missing() {
        let credentials = Credentials::new(values(&[("ALPACA_KEY", "alpaca-key")]), HashMap::new());

        let error = credentials.fred_api_key().unwrap_err();

        assert_eq!(
            error,
            ConfigError::Missing {
                name: "FRED_API_KEY"
            }
        );
    }
}
