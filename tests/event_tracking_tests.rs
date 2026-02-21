//! Event tracking tests based on shared test cases from specs/test-cases/event-tracking.yaml
//!
//! These tests ensure the Rust SDK behaves consistently with other SDKs.

use funnelmob::{
    validation::{validate_currency, validate_event_name},
    EventParameters, FunnelMob, Configuration, LogLevel, Revenue,
};

fn test_sdk() -> FunnelMob {
    let config = Configuration::builder("test_key")
        .server("http://localhost:3080/v1")
        .platform("web")
        .log_level(LogLevel::None)
        .build()
        .unwrap();
    FunnelMob::new(config).unwrap()
}

// ============================================================================
// Valid Events
// ============================================================================

#[test]
fn test_simple_event() {
    let sdk = test_sdk();
    assert!(sdk.track_event("button_click").is_ok());
}

#[test]
fn test_event_with_parameters() {
    let sdk = test_sdk();
    let params = EventParameters::new()
        .set("item_id", "sku_123")
        .set("quantity", 2_i64)
        .set("is_gift", false);
    assert!(sdk.track_event_with_params("purchase", params).is_ok());
}

#[test]
fn test_event_with_revenue() {
    let sdk = test_sdk();
    let revenue = Revenue::usd(29.99).unwrap();
    assert!(sdk.track_event_with_revenue("purchase", revenue).is_ok());
}

#[test]
fn test_event_with_revenue_and_parameters() {
    let sdk = test_sdk();
    let revenue = Revenue::eur(99.00).unwrap();
    let params = EventParameters::new()
        .set("product_name", "Premium Subscription")
        .set("duration_months", 12_i64);
    assert!(sdk
        .track_event_with_revenue_and_params("purchase", revenue, params)
        .is_ok());
}

#[test]
fn test_standard_event_registration() {
    let sdk = test_sdk();
    assert!(sdk.track_event("fm_registration").is_ok());
}

#[test]
fn test_event_name_with_underscores() {
    let sdk = test_sdk();
    assert!(sdk.track_event("level_complete_bonus").is_ok());
}

#[test]
fn test_event_name_with_numbers() {
    let sdk = test_sdk();
    assert!(sdk.track_event("step2_complete").is_ok());
}

// ============================================================================
// Invalid Events
// ============================================================================

#[test]
fn test_invalid_event_name_empty() {
    let sdk = test_sdk();
    assert!(sdk.track_event("").is_err());
}

#[test]
fn test_invalid_event_name_starts_with_number() {
    let sdk = test_sdk();
    assert!(sdk.track_event("2nd_purchase").is_err());
}

#[test]
fn test_invalid_event_name_special_chars() {
    let sdk = test_sdk();
    assert!(sdk.track_event("purchase-complete").is_err());
}

#[test]
fn test_invalid_event_name_spaces() {
    let sdk = test_sdk();
    assert!(sdk.track_event("purchase complete").is_err());
}

#[test]
fn test_invalid_event_name_too_long() {
    let sdk = test_sdk();
    let long_name =
        "this_is_a_very_long_event_name_that_exceeds_the_maximum_allowed_length_of_one_hundred_characters_total";
    assert!(long_name.len() > 100);
    assert!(sdk.track_event(long_name).is_err());
}

// ============================================================================
// Currency Normalization
// ============================================================================

#[test]
fn test_currency_lowercase_normalized() {
    let revenue = Revenue::new(10.00, "usd").unwrap();
    assert_eq!(revenue.currency(), "USD");
}

#[test]
fn test_currency_mixed_case_normalized() {
    let revenue = Revenue::new(10.00, "Eur").unwrap();
    assert_eq!(revenue.currency(), "EUR");
}

// ============================================================================
// Revenue Validation
// ============================================================================

#[test]
fn test_invalid_revenue_negative() {
    let result = Revenue::new(-10.00, "USD");
    assert!(result.is_err());
}

#[test]
fn test_invalid_currency_code() {
    let result = Revenue::new(10.00, "DOLLARS");
    assert!(result.is_err());
}

#[test]
fn test_revenue_zero_valid() {
    let sdk = test_sdk();
    let revenue = Revenue::usd(0.00).unwrap();
    assert!(sdk.track_event_with_revenue("free_trial_start", revenue).is_ok());
}

#[test]
fn test_revenue_large_amount() {
    let sdk = test_sdk();
    let revenue = Revenue::usd(999999.99).unwrap();
    assert!(sdk
        .track_event_with_revenue("enterprise_purchase", revenue)
        .is_ok());
}

// ============================================================================
// Revenue Amount Formatting
// ============================================================================

#[test]
fn test_revenue_amount_formatting() {
    let revenue = Revenue::usd(29.90).unwrap();
    assert_eq!(revenue.amount_string(), "29.90");

    let revenue = Revenue::usd(100.0).unwrap();
    assert_eq!(revenue.amount_string(), "100.00");

    let revenue = Revenue::usd(0.0).unwrap();
    assert_eq!(revenue.amount_string(), "0.00");
}

// ============================================================================
// Event Name Validation
// ============================================================================

#[test]
fn test_event_name_validation_patterns() {
    // Valid patterns
    assert!(validate_event_name("purchase").is_ok());
    assert!(validate_event_name("button_click").is_ok());
    assert!(validate_event_name("step2_complete").is_ok());
    assert!(validate_event_name("a").is_ok());
    assert!(validate_event_name("A").is_ok());
    assert!(validate_event_name("fm_registration").is_ok());

    // Invalid patterns
    assert!(validate_event_name("").is_err());
    assert!(validate_event_name("2nd_purchase").is_err());
    assert!(validate_event_name("purchase-complete").is_err());
    assert!(validate_event_name("purchase.complete").is_err());
    assert!(validate_event_name("purchase complete").is_err());
    assert!(validate_event_name("purchase@event").is_err());
}

// ============================================================================
// Currency Validation
// ============================================================================

#[test]
fn test_currency_validation_patterns() {
    assert!(validate_currency("USD").is_ok());
    assert!(validate_currency("EUR").is_ok());
    assert!(validate_currency("GBP").is_ok());
    assert!(validate_currency("JPY").is_ok());

    assert!(validate_currency("usd").is_err());
    assert!(validate_currency("US").is_err());
    assert!(validate_currency("USDD").is_err());
    assert!(validate_currency("DOLLAR").is_err());
    assert!(validate_currency("").is_err());
}

// ============================================================================
// SDK State Management
// ============================================================================

#[test]
fn test_sdk_enabled_disabled() {
    let sdk = test_sdk();

    assert!(sdk.is_enabled());

    sdk.set_enabled(false);
    assert!(!sdk.is_enabled());

    assert!(sdk.track_event("test").is_ok());

    sdk.set_enabled(true);
    assert!(sdk.is_enabled());
}

#[test]
fn test_sdk_session_and_device_id() {
    let sdk = test_sdk();

    let session_id = sdk.session_id();
    assert_eq!(session_id.get_version_num(), 4);

    assert!(!sdk.device_id().is_empty());
}
