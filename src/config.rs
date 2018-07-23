use std::env;
use std::net::IpAddr;
use stq_logging::GrayLogConfig;

use config_crate::{Config as RawConfig, ConfigError, Environment, File};

/// Service configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Listen {
    pub host: IpAddr,
    pub port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Database {
    pub dsn: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Server listen address
    pub listen: Listen,
    /// Database settings
    pub db: Database,
    /// Graylog settings
    pub graylog: Option<GrayLogConfig>,
}

static ENV_PREFIX: &'static str = "STQ_WAREHOUSES";

/// Creates new app config struct
/// #Examples
/// ```
/// use warehouses_lib::*;
///
/// let config = Config::new();
/// ```
impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = RawConfig::new();
        s.merge(File::with_name("config/base"))?;

        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or("development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // Add in settings from the environment (with a prefix of STQ_ORDERS)
        s.merge(Environment::with_prefix(ENV_PREFIX))?;

        s.try_into()
    }
}
