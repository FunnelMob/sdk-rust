//! Error types for the FunnelMob SDK.

use thiserror::Error;

/// Validation errors that can occur when creating events, revenue, or configuration.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    /// Event name cannot be empty.
    #[error("event name is required")]
    EventNameRequired,

    /// Event name exceeds the maximum length of 100 characters.
    #[error("event name exceeds 100 characters")]
    EventNameTooLong,

    /// Event name contains invalid characters or doesn't start with a letter.
    #[error("event name must start with a letter and contain only letters, numbers, and underscores")]
    EventNameInvalid,

    /// Currency code is not a valid 3-letter ISO 4217 code.
    #[error("currency must be a 3-letter ISO 4217 code")]
    InvalidCurrency,

    /// Revenue amount is negative.
    #[error("revenue amount must be a non-negative number")]
    InvalidRevenueAmount,
}

/// Top-level errors that can occur when using the FunnelMob SDK.
#[derive(Debug, Error)]
pub enum FunnelMobError {
    /// Configuration is invalid.
    #[error("configuration error: {0}")]
    Configuration(String),

    /// Validation failed.
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_display() {
        assert_eq!(
            ValidationError::EventNameRequired.to_string(),
            "event name is required"
        );
        assert_eq!(
            ValidationError::EventNameTooLong.to_string(),
            "event name exceeds 100 characters"
        );
        assert_eq!(
            ValidationError::EventNameInvalid.to_string(),
            "event name must start with a letter and contain only letters, numbers, and underscores"
        );
        assert_eq!(
            ValidationError::InvalidCurrency.to_string(),
            "currency must be a 3-letter ISO 4217 code"
        );
        assert_eq!(
            ValidationError::InvalidRevenueAmount.to_string(),
            "revenue amount must be a non-negative number"
        );
    }

    #[test]
    fn test_funnelmob_error_from_validation() {
        let validation_err = ValidationError::EventNameRequired;
        let funnelmob_err: FunnelMobError = validation_err.into();
        assert!(matches!(funnelmob_err, FunnelMobError::Validation(_)));
    }
}
