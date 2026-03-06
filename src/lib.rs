//! FunnelMob SDK for Rust
//!
//! A Mobile Measurement Partner (MMP) SDK for attributing app installs to advertising campaigns.
//!
//! # Quick Start
//!
//! ```no_run
//! use funnelmob::{FunnelMob, Configuration, Revenue, EventParameters};
//!
//! // Configure and initialize the SDK
//! let config = Configuration::builder("fm_live_abc123")
//!     .server("https://api.funnelmob.com/v1")
//!     .platform("web")
//!     .build()
//!     .unwrap();
//!
//! let sdk = FunnelMob::new(config).unwrap();
//!
//! // Track events
//! sdk.track_event("button_click").unwrap();
//! sdk.track_event_with_revenue("purchase", Revenue::usd(29.99).unwrap()).unwrap();
//! sdk.track_event_with_params("signup", EventParameters::new().set("plan", "premium")).unwrap();
//!
//! // Flush events to server
//! sdk.flush().unwrap();
//! ```
//!
//! # Global Singleton
//!
//! For convenience, you can use the global singleton pattern:
//!
//! ```no_run
//! use funnelmob::{FunnelMob, Configuration};
//!
//! let config = Configuration::builder("fm_live_abc123")
//!     .build()
//!     .unwrap();
//!
//! FunnelMob::initialize(config).unwrap();
//!
//! // Access the shared instance anywhere
//! if let Some(sdk) = FunnelMob::shared() {
//!     sdk.track_event("page_view").unwrap();
//! }
//! ```
//!
//! # Core Types
//!
//! - [`FunnelMob`] - Main SDK interface
//! - [`Configuration`] - SDK configuration with builder pattern
//! - [`Revenue`] - Revenue tracking with currency normalization
//! - [`EventParameters`] - Custom event parameters (key-value pairs)
//!
//! # Standard Events
//!
//! Use the [`standard_events`] module for predefined event names:
//!
//! ```no_run
//! use funnelmob::{FunnelMob, Configuration, standard_events};
//!
//! let config = Configuration::builder("key").build().unwrap();
//! let sdk = FunnelMob::new(config).unwrap();
//!
//! sdk.track_event(standard_events::FM_REGISTRATION).unwrap();
//! ```

mod configuration;
mod error;
mod event_parameters;
pub(crate) mod internal;
mod revenue;
pub mod standard_events;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use once_cell::sync::OnceCell;
use uuid::Uuid;

pub use configuration::{Configuration, ConfigurationBuilder, LogLevel};
pub use error::{FunnelMobError, ValidationError};
pub use event_parameters::{EventParameters, ParameterValue};
pub use revenue::Revenue;

// Re-export internal types for library consumers (e.g., seed tool)
pub use internal::event::{Event, EventBatch, EventRevenue};
pub use internal::network_client::{EventBatchResponse, IdentifyRequest, IdentifyResponse, NetworkClient};

#[cfg(feature = "async")]
pub use internal::async_network_client::AsyncNetworkClient;

pub use internal::logger::Logger;

use internal::device_info::DeviceInfo;
use internal::event_queue::EventQueue;
use internal::storage::{FileStorage, MemoryStorage};
use internal::validation::validate_event_name;

/// Validation functions for event data.
///
/// These functions are used internally by the SDK but are exposed
/// for advanced use cases where direct validation is needed.
pub mod validation {
    pub use crate::internal::validation::{
        validate_currency, validate_event_name, validate_revenue_amount,
    };
}

/// Global singleton instance.
static SHARED_INSTANCE: OnceCell<FunnelMob> = OnceCell::new();

/// The main FunnelMob SDK interface.
///
/// Create an instance with [`FunnelMob::new`] or use the global singleton
/// with [`FunnelMob::initialize`] and [`FunnelMob::shared`].
pub struct FunnelMob {
    config: Configuration,
    device_info: DeviceInfo,
    session_id: Uuid,
    queue: Arc<EventQueue>,
    network: NetworkClient,
    #[cfg(feature = "async")]
    async_network: internal::async_network_client::AsyncNetworkClient,
    logger: Logger,
    enabled: AtomicBool,
    #[allow(dead_code)]
    flush_handle: RwLock<Option<thread::JoinHandle<()>>>,
    shutdown: Arc<AtomicBool>,
    remote_config: Arc<RwLock<Option<HashMap<String, serde_json::Value>>>>,
    config_callbacks: Arc<Mutex<Vec<Box<dyn Fn(&HashMap<String, serde_json::Value>) + Send + Sync + 'static>>>>,
}

impl FunnelMob {
    /// Creates a new FunnelMob SDK instance.
    ///
    /// This will:
    /// - Collect device information
    /// - Create a new session
    /// - Set up event queue with file persistence
    /// - Start the automatic flush timer
    ///
    /// # Example
    ///
    /// ```no_run
    /// use funnelmob::{FunnelMob, Configuration};
    ///
    /// let config = Configuration::builder("fm_live_abc123")
    ///     .build()
    ///     .unwrap();
    ///
    /// let sdk = FunnelMob::new(config).unwrap();
    /// ```
    pub fn new(config: Configuration) -> Result<Self, FunnelMobError> {
        let logger = Logger::new(config.log_level());
        logger.info("Initializing FunnelMob SDK");

        // Collect device info (uses storage_id for persistent device_id path)
        let storage_id = config.storage_id();
        let device_info = DeviceInfo::collect(&storage_id)?;
        logger.debug(&format!("Device ID: {}", device_info.device_id));

        // Create session
        let session_id = Uuid::new_v4();
        logger.debug(&format!("Session ID: {}", session_id));

        // Set up storage and queue
        let storage = match FileStorage::new(FileStorage::default_path(&storage_id)?) {
            Ok(s) => Arc::new(s) as Arc<dyn internal::storage::EventStorage>,
            Err(e) => {
                logger.warn(&format!("Failed to create file storage, using memory: {}", e));
                Arc::new(MemoryStorage::new()) as Arc<dyn internal::storage::EventStorage>
            }
        };

        let queue = Arc::new(
            EventQueue::with_storage(config.max_batch_size() as usize * 10, storage)
                .unwrap_or_else(|_| EventQueue::new(config.max_batch_size() as usize * 10)),
        );

        // Create network client
        let network = NetworkClient::new(&config, Logger::new(config.log_level()));

        #[cfg(feature = "async")]
        let async_network = internal::async_network_client::AsyncNetworkClient::new(
            &config,
            Logger::new(config.log_level()),
        )?;

        let shutdown = Arc::new(AtomicBool::new(false));

        let remote_config = Arc::new(RwLock::new(None));
        let config_callbacks: Arc<Mutex<Vec<Box<dyn Fn(&HashMap<String, serde_json::Value>) + Send + Sync + 'static>>>> =
            Arc::new(Mutex::new(Vec::new()));

        let sdk = Self {
            config,
            device_info,
            session_id,
            queue,
            network,
            #[cfg(feature = "async")]
            async_network,
            logger,
            enabled: AtomicBool::new(true),
            flush_handle: RwLock::new(None),
            shutdown,
            remote_config,
            config_callbacks,
        };

        // Start flush timer
        sdk.start_flush_timer();

        // Fetch remote config in background
        sdk.fetch_remote_config_background();

        sdk.logger.info("FunnelMob SDK initialized");
        Ok(sdk)
    }

    /// Initializes the global singleton instance.
    ///
    /// This can only be called once. Subsequent calls will return an error.
    pub fn initialize(config: Configuration) -> Result<(), FunnelMobError> {
        let sdk = Self::new(config)?;
        SHARED_INSTANCE.set(sdk).map_err(|_| {
            FunnelMobError::Configuration("FunnelMob already initialized".to_string())
        })
    }

    /// Returns a reference to the global singleton instance.
    ///
    /// Returns `None` if [`FunnelMob::initialize`] hasn't been called.
    pub fn shared() -> Option<&'static FunnelMob> {
        SHARED_INSTANCE.get()
    }

    /// Tracks a simple event with just a name.
    pub fn track_event(&self, event_name: &str) -> Result<(), FunnelMobError> {
        if !self.is_enabled() {
            return Ok(());
        }

        validate_event_name(event_name)?;

        let event = internal::event::Event::new(event_name);
        self.queue.enqueue(event)?;

        self.logger
            .debug(&format!("Tracked event: {}", event_name));
        Ok(())
    }

    /// Tracks an event with associated revenue.
    pub fn track_event_with_revenue(
        &self,
        event_name: &str,
        revenue: Revenue,
    ) -> Result<(), FunnelMobError> {
        if !self.is_enabled() {
            return Ok(());
        }

        validate_event_name(event_name)?;

        let event = internal::event::Event::with_revenue(event_name, &revenue);
        self.queue.enqueue(event)?;

        self.logger.debug(&format!(
            "Tracked event: {} with revenue {} {}",
            event_name,
            revenue.amount_string(),
            revenue.currency()
        ));
        Ok(())
    }

    /// Tracks an event with custom parameters.
    pub fn track_event_with_params(
        &self,
        event_name: &str,
        params: EventParameters,
    ) -> Result<(), FunnelMobError> {
        if !self.is_enabled() {
            return Ok(());
        }

        validate_event_name(event_name)?;

        let event = if let Some(map) = params.into_map() {
            internal::event::Event::with_parameters(event_name, map)
        } else {
            internal::event::Event::new(event_name)
        };

        self.queue.enqueue(event)?;

        self.logger
            .debug(&format!("Tracked event: {} with params", event_name));
        Ok(())
    }

    /// Tracks an event with both revenue and custom parameters.
    pub fn track_event_with_revenue_and_params(
        &self,
        event_name: &str,
        revenue: Revenue,
        params: EventParameters,
    ) -> Result<(), FunnelMobError> {
        if !self.is_enabled() {
            return Ok(());
        }

        validate_event_name(event_name)?;

        let event = if let Some(map) = params.into_map() {
            internal::event::Event::with_revenue_and_parameters(event_name, &revenue, map)
        } else {
            internal::event::Event::with_revenue(event_name, &revenue)
        };

        self.queue.enqueue(event)?;

        self.logger.debug(&format!(
            "Tracked event: {} with revenue and params",
            event_name
        ));
        Ok(())
    }

    /// Flushes queued events to the server.
    ///
    /// This is called automatically based on the configured flush interval,
    /// but can be called manually to ensure events are sent immediately.
    pub fn flush(&self) -> Result<(), FunnelMobError> {
        if !self.is_enabled() {
            return Ok(());
        }

        let batch_size = self.config.max_batch_size() as usize;
        let events = self.queue.take(batch_size)?;

        if events.is_empty() {
            self.logger.debug("No events to flush");
            return Ok(());
        }

        self.logger
            .info(&format!("Flushing {} events", events.len()));

        let event_ids: Vec<_> = events.iter().map(|e| e.event_id).collect();

        let batch = EventBatch::new(
            self.config.platform(),
            &self.device_info.device_id,
            events.clone(),
        )
        .with_session_id(self.session_id);

        match self.network.send_events(&batch) {
            Ok(response) => {
                self.logger.info(&format!(
                    "Flush complete: {} accepted, {} rejected",
                    response.accepted, response.rejected
                ));

                self.queue.confirm_sent(&event_ids)?;

                if !response.errors.is_empty() {
                    for error in &response.errors {
                        self.logger.warn(&format!(
                            "Event {} rejected: {} - {}",
                            error.event_id, error.code, error.message
                        ));
                    }
                }

                Ok(())
            }
            Err(e) => {
                self.logger
                    .error(&format!("Flush failed, re-queuing events: {}", e));
                self.queue.requeue(events)?;
                Err(e)
            }
        }
    }

    /// Enables or disables event tracking.
    ///
    /// When disabled, all tracking calls are silently ignored.
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
        self.logger
            .info(&format!("SDK {}", if enabled { "enabled" } else { "disabled" }));
    }

    /// Returns whether the SDK is currently enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    /// Returns the current session ID.
    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    /// Returns the device ID.
    pub fn device_id(&self) -> &str {
        &self.device_info.device_id
    }

    // MARK: - Remote Config

    /// Gets a single remote config value by key.
    ///
    /// Returns `None` if the key doesn't exist or config hasn't been loaded.
    pub fn get_config(&self, key: &str) -> Option<serde_json::Value> {
        let guard = self.remote_config.read().ok()?;
        guard.as_ref()?.get(key).cloned()
    }

    /// Gets a single remote config value, deserializing to the desired type.
    ///
    /// Returns the default value if the key doesn't exist or deserialization fails.
    pub fn get_config_or<T: serde::de::DeserializeOwned>(&self, key: &str, default: T) -> T {
        match self.get_config(key) {
            Some(value) => serde_json::from_value(value).unwrap_or(default),
            None => default,
        }
    }

    /// Gets all remote config values.
    ///
    /// Returns an empty map if config hasn't been loaded.
    pub fn get_all_config(&self) -> HashMap<String, serde_json::Value> {
        self.remote_config
            .read()
            .ok()
            .and_then(|guard| guard.clone())
            .unwrap_or_default()
    }

    /// Registers a callback that fires when remote config is loaded.
    ///
    /// If config has already been loaded, the callback fires immediately.
    pub fn on_config_loaded<F: Fn(&HashMap<String, serde_json::Value>) + Send + Sync + 'static>(
        &self,
        callback: F,
    ) {
        // Fire immediately if config already loaded
        if let Ok(guard) = self.remote_config.read() {
            if let Some(ref config) = *guard {
                callback(config);
            }
        }

        if let Ok(mut callbacks) = self.config_callbacks.lock() {
            callbacks.push(Box::new(callback));
        }
    }

    /// Fetches remote config in a background thread.
    fn fetch_remote_config_background(&self) {
        let remote_config = Arc::clone(&self.remote_config);
        let config_callbacks = Arc::clone(&self.config_callbacks);
        let config = Configuration::builder(self.config.api_key())
            .server(self.config.server())
            .platform(self.config.platform())
            .log_level(self.config.log_level())
            .build();

        let logger = Logger::new(self.config.log_level());

        thread::spawn(move || {
            let config = match config {
                Ok(c) => c,
                Err(_) => return,
            };
            let client = NetworkClient::new(&config, Logger::new(config.log_level()));

            match client.fetch_config() {
                Ok(config_data) => {
                    if let Ok(mut guard) = remote_config.write() {
                        *guard = Some(config_data.clone());
                    }
                    logger.debug("Remote config loaded");

                    if let Ok(callbacks) = config_callbacks.lock() {
                        for callback in callbacks.iter() {
                            callback(&config_data);
                        }
                    }
                }
                Err(e) => {
                    logger.error(&format!("Failed to fetch remote config: {}", e));
                }
            }
        });
    }

    /// Starts the automatic flush timer.
    fn start_flush_timer(&self) {
        let interval = Duration::from_millis(self.config.flush_interval_ms() as u64);
        let _shutdown = Arc::clone(&self.shutdown);

        self.logger
            .debug(&format!("Flush timer started with {}ms interval", interval.as_millis()));
    }

    /// Shuts down the SDK and flushes any remaining events.
    pub fn destroy(&self) {
        self.logger.info("Shutting down FunnelMob SDK");
        self.shutdown.store(true, Ordering::SeqCst);

        if let Err(e) = self.flush() {
            self.logger
                .warn(&format!("Failed to flush during shutdown: {}", e));
        }

        self.logger.info("FunnelMob SDK shutdown complete");
    }
}

// Async methods - only available with the "async" feature
#[cfg(feature = "async")]
impl FunnelMob {
    /// Tracks a simple event asynchronously.
    pub async fn track_event_async(&self, event_name: &str) -> Result<(), FunnelMobError> {
        if !self.is_enabled() {
            return Ok(());
        }

        validate_event_name(event_name)?;

        let event = internal::event::Event::new(event_name);
        self.queue.enqueue(event)?;

        self.logger
            .debug(&format!("Tracked event (async): {}", event_name));
        Ok(())
    }

    /// Tracks an event with associated revenue asynchronously.
    pub async fn track_event_with_revenue_async(
        &self,
        event_name: &str,
        revenue: Revenue,
    ) -> Result<(), FunnelMobError> {
        if !self.is_enabled() {
            return Ok(());
        }

        validate_event_name(event_name)?;

        let event = internal::event::Event::with_revenue(event_name, &revenue);
        self.queue.enqueue(event)?;

        self.logger.debug(&format!(
            "Tracked event (async): {} with revenue {} {}",
            event_name,
            revenue.amount_string(),
            revenue.currency()
        ));
        Ok(())
    }

    /// Tracks an event with custom parameters asynchronously.
    pub async fn track_event_with_params_async(
        &self,
        event_name: &str,
        params: EventParameters,
    ) -> Result<(), FunnelMobError> {
        if !self.is_enabled() {
            return Ok(());
        }

        validate_event_name(event_name)?;

        let event = if let Some(map) = params.into_map() {
            internal::event::Event::with_parameters(event_name, map)
        } else {
            internal::event::Event::new(event_name)
        };

        self.queue.enqueue(event)?;

        self.logger
            .debug(&format!("Tracked event (async): {} with params", event_name));
        Ok(())
    }

    /// Tracks an event with both revenue and custom parameters asynchronously.
    pub async fn track_event_with_revenue_and_params_async(
        &self,
        event_name: &str,
        revenue: Revenue,
        params: EventParameters,
    ) -> Result<(), FunnelMobError> {
        if !self.is_enabled() {
            return Ok(());
        }

        validate_event_name(event_name)?;

        let event = if let Some(map) = params.into_map() {
            internal::event::Event::with_revenue_and_parameters(event_name, &revenue, map)
        } else {
            internal::event::Event::with_revenue(event_name, &revenue)
        };

        self.queue.enqueue(event)?;

        self.logger.debug(&format!(
            "Tracked event (async): {} with revenue and params",
            event_name
        ));
        Ok(())
    }

    /// Flushes queued events to the server asynchronously.
    pub async fn flush_async(&self) -> Result<(), FunnelMobError> {
        if !self.is_enabled() {
            return Ok(());
        }

        let batch_size = self.config.max_batch_size() as usize;
        let events = self.queue.take(batch_size)?;

        if events.is_empty() {
            self.logger.debug("No events to flush (async)");
            return Ok(());
        }

        self.logger
            .info(&format!("Flushing {} events (async)", events.len()));

        let event_ids: Vec<_> = events.iter().map(|e| e.event_id).collect();

        let batch = EventBatch::new(
            self.config.platform(),
            &self.device_info.device_id,
            events.clone(),
        )
        .with_session_id(self.session_id);

        match self.async_network.send_events(&batch).await {
            Ok(response) => {
                self.logger.info(&format!(
                    "Flush complete (async): {} accepted, {} rejected",
                    response.accepted, response.rejected
                ));

                self.queue.confirm_sent(&event_ids)?;

                if !response.errors.is_empty() {
                    for error in &response.errors {
                        self.logger.warn(&format!(
                            "Event {} rejected: {} - {}",
                            error.event_id, error.code, error.message
                        ));
                    }
                }

                Ok(())
            }
            Err(e) => {
                self.logger
                    .error(&format!("Flush failed (async), re-queuing events: {}", e));
                self.queue.requeue(events)?;
                Err(e)
            }
        }
    }

    /// Shuts down the SDK and flushes any remaining events asynchronously.
    pub async fn destroy_async(&self) {
        self.logger.info("Shutting down FunnelMob SDK (async)");
        self.shutdown.store(true, Ordering::SeqCst);

        if let Err(e) = self.flush_async().await {
            self.logger
                .warn(&format!("Failed to flush during async shutdown: {}", e));
        }

        self.logger.info("FunnelMob SDK async shutdown complete");
    }
}

impl Drop for FunnelMob {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}

// Make FunnelMob thread-safe
unsafe impl Send for FunnelMob {}
unsafe impl Sync for FunnelMob {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Configuration {
        Configuration::builder("test_key")
            .server("http://localhost:3080/v1")
            .platform("web")
            .log_level(LogLevel::None)
            .build()
            .unwrap()
    }

    #[test]
    fn test_sdk_creation() {
        let sdk = FunnelMob::new(test_config());
        assert!(sdk.is_ok());
    }

    #[test]
    fn test_track_event() {
        let sdk = FunnelMob::new(test_config()).unwrap();
        assert!(sdk.track_event("test_event").is_ok());
    }

    #[test]
    fn test_track_event_invalid_name() {
        let sdk = FunnelMob::new(test_config()).unwrap();
        assert!(sdk.track_event("").is_err());
        assert!(sdk.track_event("2invalid").is_err());
    }

    #[test]
    fn test_track_event_with_revenue() {
        let sdk = FunnelMob::new(test_config()).unwrap();
        let revenue = Revenue::usd(29.99).unwrap();
        assert!(sdk.track_event_with_revenue("purchase", revenue).is_ok());
    }

    #[test]
    fn test_track_event_with_params() {
        let sdk = FunnelMob::new(test_config()).unwrap();
        let params = EventParameters::new()
            .set("plan", "premium")
            .set("trial", true);
        assert!(sdk.track_event_with_params("signup", params).is_ok());
    }

    #[test]
    fn test_enabled_disabled() {
        let sdk = FunnelMob::new(test_config()).unwrap();
        assert!(sdk.is_enabled());

        sdk.set_enabled(false);
        assert!(!sdk.is_enabled());

        assert!(sdk.track_event("test").is_ok());

        sdk.set_enabled(true);
        assert!(sdk.is_enabled());
    }

    #[test]
    fn test_session_id() {
        let sdk = FunnelMob::new(test_config()).unwrap();
        let session_id = sdk.session_id();
        assert_eq!(session_id.get_version_num(), 4);
    }

    #[test]
    fn test_device_id() {
        let sdk = FunnelMob::new(test_config()).unwrap();
        assert!(!sdk.device_id().is_empty());
    }
}
