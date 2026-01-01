//! Validation functions for event names, currencies, and revenue amounts.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::ValidationError;

/// Regex pattern for valid event names: starts with a letter, followed by letters, numbers, or underscores.
/// Pattern: ^[a-zA-Z][a-zA-Z0-9_]*$
static EVENT_NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").expect("Invalid event name regex"));

/// Regex pattern for valid currency codes: exactly 3 uppercase ASCII letters.
/// Pattern: ^[A-Z]{3}$
static CURRENCY_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Z]{3}$").expect("Invalid currency regex"));

/// Maximum length for event names.
const MAX_EVENT_NAME_LENGTH: usize = 100;

/// Validates an event name according to the FunnelMob specification.
///
/// # Rules
/// - Must not be empty
/// - Must be 100 characters or fewer
/// - Must start with a letter (a-z, A-Z)
/// - Can only contain letters, numbers, and underscores
///
/// # Examples
///
/// ```
/// use funnelmob::validation::validate_event_name;
///
/// assert!(validate_event_name("purchase").is_ok());
/// assert!(validate_event_name("button_click").is_ok());
/// assert!(validate_event_name("step2_complete").is_ok());
///
/// assert!(validate_event_name("").is_err());           // empty
/// assert!(validate_event_name("2nd_purchase").is_err()); // starts with number
/// assert!(validate_event_name("purchase-complete").is_err()); // contains hyphen
/// ```
pub fn validate_event_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::EventNameRequired);
    }

    if name.len() > MAX_EVENT_NAME_LENGTH {
        return Err(ValidationError::EventNameTooLong);
    }

    if !EVENT_NAME_REGEX.is_match(name) {
        return Err(ValidationError::EventNameInvalid);
    }

    Ok(())
}

/// Validates a currency code according to the ISO 4217 format.
///
/// # Rules
/// - Must be exactly 3 characters
/// - Must be uppercase ASCII letters (A-Z)
///
/// Note: This validates format only, not whether the code is a real ISO 4217 currency.
///
/// # Examples
///
/// ```
/// use funnelmob::validation::validate_currency;
///
/// assert!(validate_currency("USD").is_ok());
/// assert!(validate_currency("EUR").is_ok());
///
/// assert!(validate_currency("usd").is_err()); // lowercase
/// assert!(validate_currency("US").is_err());  // too short
/// assert!(validate_currency("DOLLAR").is_err()); // too long
/// ```
pub fn validate_currency(currency: &str) -> Result<(), ValidationError> {
    if !CURRENCY_REGEX.is_match(currency) {
        return Err(ValidationError::InvalidCurrency);
    }

    Ok(())
}

/// Validates a revenue amount.
///
/// # Rules
/// - Must be non-negative (zero is valid for free items)
/// - Must be a finite number (not NaN or infinity)
///
/// # Examples
///
/// ```
/// use funnelmob::validation::validate_revenue_amount;
///
/// assert!(validate_revenue_amount(29.99).is_ok());
/// assert!(validate_revenue_amount(0.0).is_ok());  // zero is valid
/// assert!(validate_revenue_amount(999999.99).is_ok());
///
/// assert!(validate_revenue_amount(-10.0).is_err()); // negative
/// ```
pub fn validate_revenue_amount(amount: f64) -> Result<(), ValidationError> {
    if amount < 0.0 || !amount.is_finite() {
        return Err(ValidationError::InvalidRevenueAmount);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Event name tests from specs/test-cases/event-tracking.yaml

    #[test]
    fn test_valid_event_names() {
        // Valid event names
        assert!(validate_event_name("purchase").is_ok());
        assert!(validate_event_name("button_click").is_ok());
        assert!(validate_event_name("step2_complete").is_ok());
        assert!(validate_event_name("a").is_ok());
        assert!(validate_event_name("fm_registration").is_ok());
        assert!(validate_event_name("level_complete_bonus").is_ok());
        assert!(validate_event_name("A").is_ok());
        assert!(validate_event_name("Purchase").is_ok());
    }

    #[test]
    fn test_invalid_event_name_empty() {
        assert_eq!(
            validate_event_name(""),
            Err(ValidationError::EventNameRequired)
        );
    }

    #[test]
    fn test_invalid_event_name_starts_with_number() {
        assert_eq!(
            validate_event_name("2nd_purchase"),
            Err(ValidationError::EventNameInvalid)
        );
    }

    #[test]
    fn test_invalid_event_name_special_characters() {
        assert_eq!(
            validate_event_name("purchase-complete"),
            Err(ValidationError::EventNameInvalid)
        );
        assert_eq!(
            validate_event_name("purchase.complete"),
            Err(ValidationError::EventNameInvalid)
        );
        assert_eq!(
            validate_event_name("purchase complete"),
            Err(ValidationError::EventNameInvalid)
        );
        assert_eq!(
            validate_event_name("purchase@event"),
            Err(ValidationError::EventNameInvalid)
        );
    }

    #[test]
    fn test_invalid_event_name_too_long() {
        let long_name = "a".repeat(101);
        assert_eq!(
            validate_event_name(&long_name),
            Err(ValidationError::EventNameTooLong)
        );

        // Exactly 100 should be valid
        let max_name = "a".repeat(100);
        assert!(validate_event_name(&max_name).is_ok());
    }

    // Currency tests

    #[test]
    fn test_valid_currencies() {
        assert!(validate_currency("USD").is_ok());
        assert!(validate_currency("EUR").is_ok());
        assert!(validate_currency("GBP").is_ok());
        assert!(validate_currency("JPY").is_ok());
        assert!(validate_currency("CNY").is_ok());
    }

    #[test]
    fn test_invalid_currency_lowercase() {
        assert_eq!(validate_currency("usd"), Err(ValidationError::InvalidCurrency));
        assert_eq!(validate_currency("Usd"), Err(ValidationError::InvalidCurrency));
    }

    #[test]
    fn test_invalid_currency_wrong_length() {
        assert_eq!(validate_currency("US"), Err(ValidationError::InvalidCurrency));
        assert_eq!(validate_currency("USDD"), Err(ValidationError::InvalidCurrency));
        assert_eq!(validate_currency("DOLLAR"), Err(ValidationError::InvalidCurrency));
        assert_eq!(validate_currency(""), Err(ValidationError::InvalidCurrency));
    }

    // Revenue amount tests

    #[test]
    fn test_valid_revenue_amounts() {
        assert!(validate_revenue_amount(29.99).is_ok());
        assert!(validate_revenue_amount(0.0).is_ok()); // Zero is valid for free items
        assert!(validate_revenue_amount(100.0).is_ok());
        assert!(validate_revenue_amount(999999.99).is_ok());
        assert!(validate_revenue_amount(0.01).is_ok());
    }

    #[test]
    fn test_invalid_revenue_amount_negative() {
        assert_eq!(
            validate_revenue_amount(-10.0),
            Err(ValidationError::InvalidRevenueAmount)
        );
        assert_eq!(
            validate_revenue_amount(-0.01),
            Err(ValidationError::InvalidRevenueAmount)
        );
    }

    #[test]
    fn test_invalid_revenue_amount_special_values() {
        assert_eq!(
            validate_revenue_amount(f64::NAN),
            Err(ValidationError::InvalidRevenueAmount)
        );
        assert_eq!(
            validate_revenue_amount(f64::INFINITY),
            Err(ValidationError::InvalidRevenueAmount)
        );
        assert_eq!(
            validate_revenue_amount(f64::NEG_INFINITY),
            Err(ValidationError::InvalidRevenueAmount)
        );
    }
}
