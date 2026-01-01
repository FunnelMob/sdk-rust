//! Configuration for the FunnelMob SDK.

use crate::error::FunnelMobError;

/// The environment to use for API requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Environment {
    /// Production environment (https://api.funnelmob.com/v1).
    #[default]
    Production,
    /// Sandbox environment for testing (https://sandbox.funnelmob.com/v1).
    Sandbox,
}

impl Environment {
    /// Returns the base URL for this environment.
    pub fn base_url(&self) -> &'static str {
        match self {
            Environment::Production => "https://api.funnelmob.com/v1",
            Environment::Sandbox => "https://sandbox.funnelmob.com/v1",
        }
    }
}

/// Log level for SDK internal logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum LogLevel {
    /// No logging.
    #[default]
    None = 0,
    /// Error messages only.
    Error = 1,
    /// Warnings and errors.
    Warning = 2,
    /// Informational messages, warnings, and errors.
    Info = 3,
    /// Debug messages and all above.
    Debug = 4,
    /// All messages including verbose output.
    Verbose = 5,
}

/// Default flush interval in milliseconds.
const DEFAULT_FLUSH_INTERVAL_MS: u32 = 30_000;

/// Minimum flush interval in milliseconds.
const MIN_FLUSH_INTERVAL_MS: u32 = 1_000;

/// Default maximum batch size.
const DEFAULT_MAX_BATCH_SIZE: u32 = 100;

/// Minimum batch size.
const MIN_BATCH_SIZE: u32 = 1;

/// Maximum batch size.
const MAX_BATCH_SIZE: u32 = 100;

/// Configuration for the FunnelMob SDK.
///
/// Use [`Configuration::builder`] to create a new configuration with the builder pattern.
///
/// # Example
///
/// ```
/// use funnelmob::{Configuration, Environment, LogLevel};
///
/// let config = Configuration::builder("com.example.app", "fm_live_abc123")
///     .environment(Environment::Sandbox)
///     .log_level(LogLevel::Debug)
///     .flush_interval_ms(60_000)
///     .max_batch_size(50)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Configuration {
    pub(crate) app_id: String,
    pub(crate) api_key: String,
    pub(crate) environment: Environment,
    pub(crate) log_level: LogLevel,
    pub(crate) flush_interval_ms: u32,
    pub(crate) max_batch_size: u32,
}

impl Configuration {
    /// Creates a new configuration builder with the required app ID and API key.
    ///
    /// # Arguments
    ///
    /// * `app_id` - Your application identifier (e.g., "com.example.app")
    /// * `api_key` - Your FunnelMob API key
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::Configuration;
    ///
    /// let config = Configuration::builder("com.example.app", "fm_live_abc123")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder(app_id: impl Into<String>, api_key: impl Into<String>) -> ConfigurationBuilder {
        ConfigurationBuilder::new(app_id, api_key)
    }

    /// Returns the base URL for API requests based on the configured environment.
    pub fn base_url(&self) -> &str {
        self.environment.base_url()
    }

    /// Returns the app ID.
    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    /// Returns the API key.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Returns the environment.
    pub fn environment(&self) -> Environment {
        self.environment
    }

    /// Returns the log level.
    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    /// Returns the flush interval in milliseconds.
    pub fn flush_interval_ms(&self) -> u32 {
        self.flush_interval_ms
    }

    /// Returns the maximum batch size.
    pub fn max_batch_size(&self) -> u32 {
        self.max_batch_size
    }
}

/// Builder for creating a [`Configuration`].
///
/// # Example
///
/// ```
/// use funnelmob::{Configuration, Environment, LogLevel};
///
/// let config = Configuration::builder("com.example.app", "fm_live_abc123")
///     .environment(Environment::Sandbox)
///     .log_level(LogLevel::Debug)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug)]
pub struct ConfigurationBuilder {
    app_id: String,
    api_key: String,
    environment: Environment,
    log_level: LogLevel,
    flush_interval_ms: u32,
    max_batch_size: u32,
}

impl ConfigurationBuilder {
    /// Creates a new builder with the required app ID and API key.
    fn new(app_id: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            api_key: api_key.into(),
            environment: Environment::default(),
            log_level: LogLevel::default(),
            flush_interval_ms: DEFAULT_FLUSH_INTERVAL_MS,
            max_batch_size: DEFAULT_MAX_BATCH_SIZE,
        }
    }

    /// Sets the environment for API requests.
    ///
    /// Default: [`Environment::Production`]
    pub fn environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
        self
    }

    /// Sets the log level for SDK internal logging.
    ///
    /// Default: [`LogLevel::None`]
    pub fn log_level(mut self, log_level: LogLevel) -> Self {
        self.log_level = log_level;
        self
    }

    /// Sets the interval in milliseconds between automatic event flushes.
    ///
    /// Values below 1000ms will be clamped to 1000ms.
    ///
    /// Default: 30000 (30 seconds)
    pub fn flush_interval_ms(mut self, interval: u32) -> Self {
        self.flush_interval_ms = interval.max(MIN_FLUSH_INTERVAL_MS);
        self
    }

    /// Sets the maximum number of events to send in a single batch.
    ///
    /// Values will be clamped to the range 1-100.
    ///
    /// Default: 100
    pub fn max_batch_size(mut self, size: u32) -> Self {
        self.max_batch_size = size.clamp(MIN_BATCH_SIZE, MAX_BATCH_SIZE);
        self
    }

    /// Builds the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the app ID or API key is empty.
    pub fn build(self) -> Result<Configuration, FunnelMobError> {
        if self.app_id.is_empty() {
            return Err(FunnelMobError::Configuration(
                "app_id is required".to_string(),
            ));
        }

        if self.api_key.is_empty() {
            return Err(FunnelMobError::Configuration(
                "api_key is required".to_string(),
            ));
        }

        Ok(Configuration {
            app_id: self.app_id,
            api_key: self.api_key,
            environment: self.environment,
            log_level: self.log_level,
            flush_interval_ms: self.flush_interval_ms,
            max_batch_size: self.max_batch_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_configuration() {
        let config = Configuration::builder("com.example.app", "fm_live_abc123")
            .build()
            .unwrap();

        assert_eq!(config.app_id(), "com.example.app");
        assert_eq!(config.api_key(), "fm_live_abc123");
        assert_eq!(config.environment(), Environment::Production);
        assert_eq!(config.log_level(), LogLevel::None);
        assert_eq!(config.flush_interval_ms(), 30_000);
        assert_eq!(config.max_batch_size(), 100);
    }

    #[test]
    fn test_full_configuration() {
        let config = Configuration::builder("com.example.app", "fm_live_abc123")
            .environment(Environment::Sandbox)
            .log_level(LogLevel::Debug)
            .flush_interval_ms(60_000)
            .max_batch_size(50)
            .build()
            .unwrap();

        assert_eq!(config.environment(), Environment::Sandbox);
        assert_eq!(config.log_level(), LogLevel::Debug);
        assert_eq!(config.flush_interval_ms(), 60_000);
        assert_eq!(config.max_batch_size(), 50);
    }

    #[test]
    fn test_empty_app_id() {
        let result = Configuration::builder("", "fm_live_abc123").build();
        assert!(matches!(result, Err(FunnelMobError::Configuration(_))));
    }

    #[test]
    fn test_empty_api_key() {
        let result = Configuration::builder("com.example.app", "").build();
        assert!(matches!(result, Err(FunnelMobError::Configuration(_))));
    }

    #[test]
    fn test_flush_interval_clamping() {
        // Below minimum
        let config = Configuration::builder("app", "key")
            .flush_interval_ms(500)
            .build()
            .unwrap();
        assert_eq!(config.flush_interval_ms(), 1000);

        // At minimum
        let config = Configuration::builder("app", "key")
            .flush_interval_ms(1000)
            .build()
            .unwrap();
        assert_eq!(config.flush_interval_ms(), 1000);

        // Above minimum
        let config = Configuration::builder("app", "key")
            .flush_interval_ms(5000)
            .build()
            .unwrap();
        assert_eq!(config.flush_interval_ms(), 5000);
    }

    #[test]
    fn test_batch_size_clamping() {
        // Below minimum
        let config = Configuration::builder("app", "key")
            .max_batch_size(0)
            .build()
            .unwrap();
        assert_eq!(config.max_batch_size(), 1);

        // At minimum
        let config = Configuration::builder("app", "key")
            .max_batch_size(1)
            .build()
            .unwrap();
        assert_eq!(config.max_batch_size(), 1);

        // In range
        let config = Configuration::builder("app", "key")
            .max_batch_size(50)
            .build()
            .unwrap();
        assert_eq!(config.max_batch_size(), 50);

        // At maximum
        let config = Configuration::builder("app", "key")
            .max_batch_size(100)
            .build()
            .unwrap();
        assert_eq!(config.max_batch_size(), 100);

        // Above maximum
        let config = Configuration::builder("app", "key")
            .max_batch_size(150)
            .build()
            .unwrap();
        assert_eq!(config.max_batch_size(), 100);
    }

    #[test]
    fn test_environment_base_urls() {
        assert_eq!(
            Environment::Production.base_url(),
            "https://api.funnelmob.com/v1"
        );
        assert_eq!(
            Environment::Sandbox.base_url(),
            "https://sandbox.funnelmob.com/v1"
        );
    }

    #[test]
    fn test_configuration_base_url() {
        let prod_config = Configuration::builder("app", "key")
            .environment(Environment::Production)
            .build()
            .unwrap();
        assert_eq!(prod_config.base_url(), "https://api.funnelmob.com/v1");

        let sandbox_config = Configuration::builder("app", "key")
            .environment(Environment::Sandbox)
            .build()
            .unwrap();
        assert_eq!(sandbox_config.base_url(), "https://sandbox.funnelmob.com/v1");
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::None < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Warning);
        assert!(LogLevel::Warning < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Verbose);
    }
}
