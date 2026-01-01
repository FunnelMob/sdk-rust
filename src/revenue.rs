//! Revenue tracking for the FunnelMob SDK.

use crate::error::ValidationError;
use crate::internal::validation::{validate_currency, validate_revenue_amount};

/// Represents revenue associated with an event.
///
/// Revenue amounts are validated to be non-negative and currencies are normalized
/// to uppercase ISO 4217 format.
///
/// # Example
///
/// ```
/// use funnelmob::Revenue;
///
/// // Using the new constructor
/// let revenue = Revenue::new(29.99, "usd").unwrap();
/// assert_eq!(revenue.currency(), "USD"); // Normalized to uppercase
///
/// // Using convenience constructors
/// let usd_revenue = Revenue::usd(19.99).unwrap();
/// let eur_revenue = Revenue::eur(49.99).unwrap();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Revenue {
    amount: f64,
    currency: String,
}

impl Revenue {
    /// Creates a new revenue with the specified amount and currency.
    ///
    /// The currency is normalized to uppercase (e.g., "eur" becomes "EUR").
    ///
    /// # Arguments
    ///
    /// * `amount` - The revenue amount (must be non-negative)
    /// * `currency` - The ISO 4217 currency code (3 letters, case-insensitive)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The amount is negative
    /// - The currency is not a valid 3-letter code
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::Revenue;
    ///
    /// let revenue = Revenue::new(29.99, "USD").unwrap();
    /// let revenue_normalized = Revenue::new(29.99, "eur").unwrap();
    /// assert_eq!(revenue_normalized.currency(), "EUR");
    /// ```
    pub fn new(amount: f64, currency: &str) -> Result<Self, ValidationError> {
        validate_revenue_amount(amount)?;

        // Normalize currency to uppercase
        let normalized_currency = currency.to_uppercase();
        validate_currency(&normalized_currency)?;

        Ok(Self {
            amount,
            currency: normalized_currency,
        })
    }

    /// Creates a new revenue in US Dollars.
    ///
    /// # Errors
    ///
    /// Returns an error if the amount is negative.
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::Revenue;
    ///
    /// let revenue = Revenue::usd(29.99).unwrap();
    /// assert_eq!(revenue.currency(), "USD");
    /// ```
    pub fn usd(amount: f64) -> Result<Self, ValidationError> {
        Self::new(amount, "USD")
    }

    /// Creates a new revenue in Euros.
    ///
    /// # Errors
    ///
    /// Returns an error if the amount is negative.
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::Revenue;
    ///
    /// let revenue = Revenue::eur(49.99).unwrap();
    /// assert_eq!(revenue.currency(), "EUR");
    /// ```
    pub fn eur(amount: f64) -> Result<Self, ValidationError> {
        Self::new(amount, "EUR")
    }

    /// Creates a new revenue in British Pounds.
    ///
    /// # Errors
    ///
    /// Returns an error if the amount is negative.
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::Revenue;
    ///
    /// let revenue = Revenue::gbp(39.99).unwrap();
    /// assert_eq!(revenue.currency(), "GBP");
    /// ```
    pub fn gbp(amount: f64) -> Result<Self, ValidationError> {
        Self::new(amount, "GBP")
    }

    /// Returns the revenue amount.
    pub fn amount(&self) -> f64 {
        self.amount
    }

    /// Returns the currency code.
    pub fn currency(&self) -> &str {
        &self.currency
    }

    /// Returns the amount formatted as a string with exactly 2 decimal places.
    ///
    /// This is used for API serialization.
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::Revenue;
    ///
    /// let revenue = Revenue::usd(29.9).unwrap();
    /// assert_eq!(revenue.amount_string(), "29.90");
    ///
    /// let revenue = Revenue::usd(100.0).unwrap();
    /// assert_eq!(revenue.amount_string(), "100.00");
    /// ```
    pub fn amount_string(&self) -> String {
        format!("{:.2}", self.amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid() {
        let revenue = Revenue::new(29.99, "USD").unwrap();
        assert!((revenue.amount() - 29.99).abs() < f64::EPSILON);
        assert_eq!(revenue.currency(), "USD");
    }

    #[test]
    fn test_currency_normalization() {
        // Lowercase
        let revenue = Revenue::new(10.0, "eur").unwrap();
        assert_eq!(revenue.currency(), "EUR");

        // Mixed case
        let revenue = Revenue::new(10.0, "uSd").unwrap();
        assert_eq!(revenue.currency(), "USD");

        // Already uppercase
        let revenue = Revenue::new(10.0, "GBP").unwrap();
        assert_eq!(revenue.currency(), "GBP");
    }

    #[test]
    fn test_usd_helper() {
        let revenue = Revenue::usd(29.99).unwrap();
        assert!((revenue.amount() - 29.99).abs() < f64::EPSILON);
        assert_eq!(revenue.currency(), "USD");
    }

    #[test]
    fn test_eur_helper() {
        let revenue = Revenue::eur(49.99).unwrap();
        assert!((revenue.amount() - 49.99).abs() < f64::EPSILON);
        assert_eq!(revenue.currency(), "EUR");
    }

    #[test]
    fn test_gbp_helper() {
        let revenue = Revenue::gbp(39.99).unwrap();
        assert!((revenue.amount() - 39.99).abs() < f64::EPSILON);
        assert_eq!(revenue.currency(), "GBP");
    }

    #[test]
    fn test_zero_amount() {
        // Zero is valid for free items
        let revenue = Revenue::usd(0.0).unwrap();
        assert!((revenue.amount() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_large_amount() {
        let revenue = Revenue::usd(999999.99).unwrap();
        assert!((revenue.amount() - 999999.99).abs() < f64::EPSILON);
    }

    #[test]
    fn test_negative_amount() {
        let result = Revenue::new(-10.0, "USD");
        assert!(matches!(result, Err(ValidationError::InvalidRevenueAmount)));
    }

    #[test]
    fn test_invalid_currency() {
        // Too short
        let result = Revenue::new(10.0, "US");
        assert!(matches!(result, Err(ValidationError::InvalidCurrency)));

        // Too long
        let result = Revenue::new(10.0, "USDD");
        assert!(matches!(result, Err(ValidationError::InvalidCurrency)));

        // Numbers
        let result = Revenue::new(10.0, "US1");
        assert!(matches!(result, Err(ValidationError::InvalidCurrency)));
    }

    #[test]
    fn test_amount_string_formatting() {
        // Standard case
        let revenue = Revenue::usd(29.99).unwrap();
        assert_eq!(revenue.amount_string(), "29.99");

        // Trailing zero
        let revenue = Revenue::usd(29.90).unwrap();
        assert_eq!(revenue.amount_string(), "29.90");

        // Whole number
        let revenue = Revenue::usd(100.0).unwrap();
        assert_eq!(revenue.amount_string(), "100.00");

        // Zero
        let revenue = Revenue::usd(0.0).unwrap();
        assert_eq!(revenue.amount_string(), "0.00");

        // Small amount
        let revenue = Revenue::usd(0.01).unwrap();
        assert_eq!(revenue.amount_string(), "0.01");

        // Rounding (like toFixed(2) in JavaScript)
        let revenue = Revenue::usd(29.999).unwrap();
        assert_eq!(revenue.amount_string(), "30.00");

        let revenue = Revenue::usd(29.994).unwrap();
        assert_eq!(revenue.amount_string(), "29.99");
    }

    #[test]
    fn test_clone_and_eq() {
        let revenue1 = Revenue::usd(29.99).unwrap();
        let revenue2 = revenue1.clone();
        assert_eq!(revenue1, revenue2);
    }
}
