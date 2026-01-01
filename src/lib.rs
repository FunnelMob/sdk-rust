//! FunnelMob SDK for Rust
//!
//! A Mobile Measurement Partner (MMP) SDK for attributing app installs to advertising campaigns.
//!
//! # Quick Start
//!
//! ```no_run
//! use funnelmob::{Configuration, Environment, Revenue, EventParameters};
//!
//! // Configure the SDK
//! let config = Configuration::builder("com.example.app", "fm_live_abc123")
//!     .environment(Environment::Sandbox)
//!     .build()
//!     .unwrap();
//!
//! // Track events (full SDK implementation coming in Phase 4)
//! // sdk.track_event("purchase").unwrap();
//! // sdk.track_event_with_revenue("purchase", Revenue::usd(29.99).unwrap()).unwrap();
//! // sdk.track_event_with_params("signup", EventParameters::new().set("plan", "premium")).unwrap();
//! ```
//!
//! # Core Types
//!
//! - [`Configuration`] - SDK configuration with builder pattern
//! - [`Revenue`] - Revenue tracking with currency normalization
//! - [`EventParameters`] - Custom event parameters (key-value pairs)
//!
//! # Validation
//!
//! The SDK validates:
//! - Event names: Must start with a letter, contain only letters/numbers/underscores, max 100 chars
//! - Currency codes: Must be 3-letter uppercase ISO 4217 codes
//! - Revenue amounts: Must be non-negative
//!
//! Invalid events are rejected with descriptive errors.

mod configuration;
mod error;
mod event_parameters;
mod internal;
mod revenue;

pub use configuration::{Configuration, ConfigurationBuilder, Environment, LogLevel};
pub use error::{FunnelMobError, ValidationError};
pub use event_parameters::{EventParameters, ParameterValue};
pub use revenue::Revenue;

// Re-export validation functions for advanced use cases
pub mod validation {
    //! Validation functions for event data.
    //!
    //! These functions are used internally by the SDK but are exposed
    //! for advanced use cases where direct validation is needed.

    pub use crate::internal::validation::{
        validate_currency, validate_event_name, validate_revenue_amount,
    };
}
