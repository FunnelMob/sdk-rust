//! Configuration for the FunnelMob SDK.

use crate::error::FunnelMobError;

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

/// Default server URL.
const DEFAULT_SERVER: &str = "https://api.funnelmob.com/v1";

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
/// use funnelmob::{Configuration, LogLevel};
///
/// let config = Configuration::builder("fm_live_abc123")
///     .server("https://api.funnelmob.com/v1")
///     .platform("web")
///     .log_level(LogLevel::Debug)
///     .flush_interval_ms(60_000)
///     .max_batch_size(50)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Configuration {
    pub(crate) api_key: String,
    pub(crate) server: String,
    pub(crate) platform: String,
    pub(crate) log_level: LogLevel,
    pub(crate) flush_interval_ms: u32,
    pub(crate) max_batch_size: u32,
}

impl Configuration {
    /// Creates a new configuration builder with the required API key.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your FunnelMob API key
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::Configuration;
    ///
    /// let config = Configuration::builder("fm_live_abc123")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder(api_key: impl Into<String>) -> ConfigurationBuilder {
        ConfigurationBuilder::new(api_key)
    }

    /// Returns the server base URL for API requests.
    pub fn server(&self) -> &str {
        &self.server
    }

    /// Returns the platform identifier (e.g., "ios", "android", "web").
    pub fn platform(&self) -> &str {
        &self.platform
    }

    /// Returns the API key.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Returns a storage identifier derived from the API key.
    ///
    /// Uses the last 8 characters of the API key (or the full key if shorter)
    /// as a stable, filesystem-safe identifier for per-key storage paths.
    pub fn storage_id(&self) -> String {
        let key = &self.api_key;
        let suffix = if key.len() > 8 {
            &key[key.len() - 8..]
        } else {
            key
        };
        suffix
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect()
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
/// use funnelmob::{Configuration, LogLevel};
///
/// let config = Configuration::builder("fm_live_abc123")
///     .server("http://localhost:3080/v1")
///     .platform("web")
///     .log_level(LogLevel::Debug)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug)]
pub struct ConfigurationBuilder {
    api_key: String,
    server: String,
    platform: String,
    log_level: LogLevel,
    flush_interval_ms: u32,
    max_batch_size: u32,
}

impl ConfigurationBuilder {
    /// Creates a new builder with the required API key.
    fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            server: DEFAULT_SERVER.to_string(),
            platform: detect_platform(),
            log_level: LogLevel::default(),
            flush_interval_ms: DEFAULT_FLUSH_INTERVAL_MS,
            max_batch_size: DEFAULT_MAX_BATCH_SIZE,
        }
    }

    /// Sets the server base URL for API requests.
    ///
    /// Default: `https://api.funnelmob.com/v1`
    pub fn server(mut self, server: impl Into<String>) -> Self {
        self.server = server.into();
        self
    }

    /// Sets the platform identifier sent with each event batch.
    ///
    /// Default: auto-detected from the OS (e.g., "linux", "macos", "windows").
    /// For the seed tool, override with "ios", "android", or "web".
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = platform.into();
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
    /// Returns an error if the API key or server URL is empty.
    pub fn build(self) -> Result<Configuration, FunnelMobError> {
        if self.api_key.is_empty() {
            return Err(FunnelMobError::Configuration(
                "api_key is required".to_string(),
            ));
        }

        if self.server.is_empty() {
            return Err(FunnelMobError::Configuration(
                "server URL is required".to_string(),
            ));
        }

        if self.platform.is_empty() {
            return Err(FunnelMobError::Configuration(
                "platform is required".to_string(),
            ));
        }

        Ok(Configuration {
            api_key: self.api_key,
            server: self.server,
            platform: self.platform,
            log_level: self.log_level,
            flush_interval_ms: self.flush_interval_ms,
            max_batch_size: self.max_batch_size,
        })
    }
}

/// Auto-detect platform from the current OS.
fn detect_platform() -> String {
    #[cfg(target_os = "macos")]
    {
        "macos".to_string()
    }
    #[cfg(target_os = "linux")]
    {
        "linux".to_string()
    }
    #[cfg(target_os = "windows")]
    {
        "windows".to_string()
    }
    #[cfg(target_os = "ios")]
    {
        "ios".to_string()
    }
    #[cfg(target_os = "android")]
    {
        "android".to_string()
    }
    #[cfg(not(any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows",
        target_os = "ios",
        target_os = "android"
    )))]
    {
        std::env::consts::OS.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_configuration() {
        let config = Configuration::builder("fm_live_abc123")
            .build()
            .unwrap();

        assert_eq!(config.api_key(), "fm_live_abc123");
        assert_eq!(config.server(), DEFAULT_SERVER);
        assert!(!config.platform().is_empty());
        assert_eq!(config.log_level(), LogLevel::None);
        assert_eq!(config.flush_interval_ms(), 30_000);
        assert_eq!(config.max_batch_size(), 100);
    }

    #[test]
    fn test_full_configuration() {
        let config = Configuration::builder("fm_live_abc123")
            .server("http://localhost:3080/v1")
            .platform("web")
            .log_level(LogLevel::Debug)
            .flush_interval_ms(60_000)
            .max_batch_size(50)
            .build()
            .unwrap();

        assert_eq!(config.server(), "http://localhost:3080/v1");
        assert_eq!(config.platform(), "web");
        assert_eq!(config.log_level(), LogLevel::Debug);
        assert_eq!(config.flush_interval_ms(), 60_000);
        assert_eq!(config.max_batch_size(), 50);
    }

    #[test]
    fn test_empty_api_key() {
        let result = Configuration::builder("").build();
        assert!(matches!(result, Err(FunnelMobError::Configuration(_))));
    }

    #[test]
    fn test_empty_server() {
        let result = Configuration::builder("key").server("").build();
        assert!(matches!(result, Err(FunnelMobError::Configuration(_))));
    }

    #[test]
    fn test_empty_platform() {
        let result = Configuration::builder("key").platform("").build();
        assert!(matches!(result, Err(FunnelMobError::Configuration(_))));
    }

    #[test]
    fn test_storage_id() {
        let config = Configuration::builder("fm_prod_abcdefghij123456")
            .build()
            .unwrap();
        let storage_id = config.storage_id();
        assert_eq!(storage_id.len(), 8);
        assert!(storage_id.chars().all(|c| c.is_alphanumeric() || c == '_'));
    }

    #[test]
    fn test_flush_interval_clamping() {
        let config = Configuration::builder("key")
            .flush_interval_ms(500)
            .build()
            .unwrap();
        assert_eq!(config.flush_interval_ms(), 1000);

        let config = Configuration::builder("key")
            .flush_interval_ms(1000)
            .build()
            .unwrap();
        assert_eq!(config.flush_interval_ms(), 1000);

        let config = Configuration::builder("key")
            .flush_interval_ms(5000)
            .build()
            .unwrap();
        assert_eq!(config.flush_interval_ms(), 5000);
    }

    #[test]
    fn test_batch_size_clamping() {
        let config = Configuration::builder("key")
            .max_batch_size(0)
            .build()
            .unwrap();
        assert_eq!(config.max_batch_size(), 1);

        let config = Configuration::builder("key")
            .max_batch_size(50)
            .build()
            .unwrap();
        assert_eq!(config.max_batch_size(), 50);

        let config = Configuration::builder("key")
            .max_batch_size(100)
            .build()
            .unwrap();
        assert_eq!(config.max_batch_size(), 100);

        let config = Configuration::builder("key")
            .max_batch_size(150)
            .build()
            .unwrap();
        assert_eq!(config.max_batch_size(), 100);
    }

    #[test]
    fn test_default_server_url() {
        let config = Configuration::builder("key").build().unwrap();
        assert_eq!(config.server(), "https://api.funnelmob.com/v1");
    }

    #[test]
    fn test_custom_server_url() {
        let config = Configuration::builder("key")
            .server("http://localhost:3080/v1")
            .build()
            .unwrap();
        assert_eq!(config.server(), "http://localhost:3080/v1");
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
