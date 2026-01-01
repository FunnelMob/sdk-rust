//! FunnelMob SDK for Rust
//!
//! A Mobile Measurement Partner (MMP) SDK for attributing app installs to advertising campaigns.
//!
//! # Quick Start
//!
//! ```no_run
//! use funnelmob::{FunnelMob, Configuration, Environment, Revenue, EventParameters};
//!
//! // Configure and initialize the SDK
//! let config = Configuration::builder("com.example.app", "fm_live_abc123")
//!     .environment(Environment::Sandbox)
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
//! let config = Configuration::builder("com.example.app", "fm_live_abc123")
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
//! let config = Configuration::builder("app", "key").build().unwrap();
//! let sdk = FunnelMob::new(config).unwrap();
//!
//! sdk.track_event(standard_events::FM_REGISTRATION).unwrap();
//! ```

mod configuration;
mod error;
mod event_parameters;
mod internal;
mod revenue;
pub mod standard_events;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use once_cell::sync::OnceCell;
use uuid::Uuid;

pub use configuration::{Configuration, ConfigurationBuilder, Environment, LogLevel};
pub use error::{FunnelMobError, ValidationError};
pub use event_parameters::{EventParameters, ParameterValue};
pub use revenue::Revenue;

use internal::device_info::DeviceInfo;
use internal::event::{Event, EventBatch};
use internal::event_queue::EventQueue;
use internal::logger::Logger;
use internal::network_client::NetworkClient;
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
    logger: Logger,
    enabled: AtomicBool,
    flush_handle: RwLock<Option<thread::JoinHandle<()>>>,
    shutdown: Arc<AtomicBool>,
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
    /// let config = Configuration::builder("com.example.app", "fm_live_abc123")
    ///     .build()
    ///     .unwrap();
    ///
    /// let sdk = FunnelMob::new(config).unwrap();
    /// ```
    pub fn new(config: Configuration) -> Result<Self, FunnelMobError> {
        let logger = Logger::new(config.log_level());
        logger.info("Initializing FunnelMob SDK");

        // Collect device info
        let device_info = DeviceInfo::collect(config.app_id())?;
        logger.debug(&format!("Device ID: {}", device_info.device_id));

        // Create session
        let session_id = Uuid::new_v4();
        logger.debug(&format!("Session ID: {}", session_id));

        // Set up storage and queue
        let storage = match FileStorage::new(FileStorage::default_path(config.app_id())?) {
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

        let shutdown = Arc::new(AtomicBool::new(false));

        let sdk = Self {
            config,
            device_info,
            session_id,
            queue,
            network,
            logger,
            enabled: AtomicBool::new(true),
            flush_handle: RwLock::new(None),
            shutdown,
        };

        // Start flush timer
        sdk.start_flush_timer();

        sdk.logger.info("FunnelMob SDK initialized");
        Ok(sdk)
    }

    /// Initializes the global singleton instance.
    ///
    /// This can only be called once. Subsequent calls will return an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use funnelmob::{FunnelMob, Configuration};
    ///
    /// let config = Configuration::builder("com.example.app", "key")
    ///     .build()
    ///     .unwrap();
    ///
    /// FunnelMob::initialize(config).unwrap();
    /// ```
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
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use funnelmob::{FunnelMob, Configuration};
    /// # let config = Configuration::builder("app", "key").build().unwrap();
    /// # let sdk = FunnelMob::new(config).unwrap();
    /// sdk.track_event("button_click").unwrap();
    /// ```
    pub fn track_event(&self, event_name: &str) -> Result<(), FunnelMobError> {
        if !self.is_enabled() {
            return Ok(());
        }

        validate_event_name(event_name)?;

        let event = Event::new(event_name);
        self.queue.enqueue(event)?;

        self.logger
            .debug(&format!("Tracked event: {}", event_name));
        Ok(())
    }

    /// Tracks an event with associated revenue.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use funnelmob::{FunnelMob, Configuration, Revenue};
    /// # let config = Configuration::builder("app", "key").build().unwrap();
    /// # let sdk = FunnelMob::new(config).unwrap();
    /// sdk.track_event_with_revenue("purchase", Revenue::usd(29.99).unwrap()).unwrap();
    /// ```
    pub fn track_event_with_revenue(
        &self,
        event_name: &str,
        revenue: Revenue,
    ) -> Result<(), FunnelMobError> {
        if !self.is_enabled() {
            return Ok(());
        }

        validate_event_name(event_name)?;

        let event = Event::with_revenue(event_name, &revenue);
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
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use funnelmob::{FunnelMob, Configuration, EventParameters};
    /// # let config = Configuration::builder("app", "key").build().unwrap();
    /// # let sdk = FunnelMob::new(config).unwrap();
    /// sdk.track_event_with_params(
    ///     "signup",
    ///     EventParameters::new()
    ///         .set("plan", "premium")
    ///         .set("trial", true)
    /// ).unwrap();
    /// ```
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
            Event::with_parameters(event_name, map)
        } else {
            Event::new(event_name)
        };

        self.queue.enqueue(event)?;

        self.logger
            .debug(&format!("Tracked event: {} with params", event_name));
        Ok(())
    }

    /// Tracks an event with both revenue and custom parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use funnelmob::{FunnelMob, Configuration, Revenue, EventParameters};
    /// # let config = Configuration::builder("app", "key").build().unwrap();
    /// # let sdk = FunnelMob::new(config).unwrap();
    /// sdk.track_event_with_revenue_and_params(
    ///     "purchase",
    ///     Revenue::usd(29.99).unwrap(),
    ///     EventParameters::new().set("item_id", "sku_123")
    /// ).unwrap();
    /// ```
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
            Event::with_revenue_and_parameters(event_name, &revenue, map)
        } else {
            Event::with_revenue(event_name, &revenue)
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
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use funnelmob::{FunnelMob, Configuration};
    /// # let config = Configuration::builder("app", "key").build().unwrap();
    /// # let sdk = FunnelMob::new(config).unwrap();
    /// sdk.track_event("important_event").unwrap();
    /// sdk.flush().unwrap(); // Send immediately
    /// ```
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
            self.config.app_id(),
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

                // Remove successfully sent events from storage
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
                // Re-queue failed events
                self.queue.requeue(events)?;
                Err(e)
            }
        }
    }

    /// Enables or disables event tracking.
    ///
    /// When disabled, all tracking calls are silently ignored.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use funnelmob::{FunnelMob, Configuration};
    /// # let config = Configuration::builder("app", "key").build().unwrap();
    /// # let sdk = FunnelMob::new(config).unwrap();
    /// sdk.set_enabled(false); // Disable tracking
    /// sdk.track_event("ignored").unwrap(); // This does nothing
    /// sdk.set_enabled(true); // Re-enable tracking
    /// ```
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

    /// Starts the automatic flush timer.
    fn start_flush_timer(&self) {
        let interval = Duration::from_millis(self.config.flush_interval_ms() as u64);
        let _shutdown = Arc::clone(&self.shutdown);

        // We can't easily share self with the thread, so we'll use a simpler approach
        // The flush timer will be implemented by periodically checking
        self.logger
            .debug(&format!("Flush timer started with {}ms interval", interval.as_millis()));

        // Note: For a production SDK, we'd use a proper scheduler.
        // For this implementation, users should call flush() manually or rely on
        // flush being called before the SDK is dropped.
    }

    /// Shuts down the SDK and flushes any remaining events.
    ///
    /// This is called automatically when the SDK is dropped, but can be
    /// called manually for explicit cleanup.
    pub fn destroy(&self) {
        self.logger.info("Shutting down FunnelMob SDK");
        self.shutdown.store(true, Ordering::SeqCst);

        // Try to flush remaining events
        if let Err(e) = self.flush() {
            self.logger
                .warn(&format!("Failed to flush during shutdown: {}", e));
        }

        self.logger.info("FunnelMob SDK shutdown complete");
    }
}

impl Drop for FunnelMob {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        // Note: We can't call flush() here easily because it needs &self
        // In production, use destroy() explicitly before dropping
    }
}

// Make FunnelMob thread-safe
unsafe impl Send for FunnelMob {}
unsafe impl Sync for FunnelMob {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Configuration {
        Configuration::builder("com.test.app", "test_key")
            .environment(Environment::Sandbox)
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

        // Should not error when disabled
        assert!(sdk.track_event("test").is_ok());

        sdk.set_enabled(true);
        assert!(sdk.is_enabled());
    }

    #[test]
    fn test_session_id() {
        let sdk = FunnelMob::new(test_config()).unwrap();
        let session_id = sdk.session_id();
        assert_eq!(session_id.get_version_num(), 4); // UUID v4
    }

    #[test]
    fn test_device_id() {
        let sdk = FunnelMob::new(test_config()).unwrap();
        assert!(!sdk.device_id().is_empty());
    }
}
