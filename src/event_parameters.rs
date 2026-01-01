//! Event parameters for the FunnelMob SDK.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A value that can be stored in event parameters.
///
/// Supports strings, integers, floats, and booleans.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParameterValue {
    /// A string value.
    String(String),
    /// An integer value.
    Integer(i64),
    /// A floating-point value.
    Float(f64),
    /// A boolean value.
    Boolean(bool),
}

// From implementations for ParameterValue

impl From<String> for ParameterValue {
    fn from(value: String) -> Self {
        ParameterValue::String(value)
    }
}

impl From<&str> for ParameterValue {
    fn from(value: &str) -> Self {
        ParameterValue::String(value.to_string())
    }
}

impl From<i64> for ParameterValue {
    fn from(value: i64) -> Self {
        ParameterValue::Integer(value)
    }
}

impl From<i32> for ParameterValue {
    fn from(value: i32) -> Self {
        ParameterValue::Integer(value as i64)
    }
}

impl From<f64> for ParameterValue {
    fn from(value: f64) -> Self {
        ParameterValue::Float(value)
    }
}

impl From<f32> for ParameterValue {
    fn from(value: f32) -> Self {
        ParameterValue::Float(value as f64)
    }
}

impl From<bool> for ParameterValue {
    fn from(value: bool) -> Self {
        ParameterValue::Boolean(value)
    }
}

/// A collection of custom parameters for an event.
///
/// Parameters are key-value pairs where values can be strings, integers,
/// floats, or booleans.
///
/// # Example
///
/// ```
/// use funnelmob::EventParameters;
///
/// let params = EventParameters::new()
///     .set("plan", "premium")
///     .set("trial", true)
///     .set("level", 5)
///     .set("score", 99.5);
///
/// assert!(params.has("plan"));
/// assert_eq!(params.len(), 4);
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EventParameters {
    params: HashMap<String, ParameterValue>,
}

impl EventParameters {
    /// Creates a new empty parameter collection.
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::EventParameters;
    ///
    /// let params = EventParameters::new();
    /// assert!(params.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates parameters from a HashMap.
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::{EventParameters, ParameterValue};
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("key".to_string(), ParameterValue::String("value".to_string()));
    ///
    /// let params = EventParameters::from_map(map);
    /// assert!(params.has("key"));
    /// ```
    pub fn from_map(map: HashMap<String, ParameterValue>) -> Self {
        Self { params: map }
    }

    /// Sets a parameter value. If the key already exists, the value is replaced.
    ///
    /// This method takes ownership and returns self for method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::EventParameters;
    ///
    /// let params = EventParameters::new()
    ///     .set("name", "John")
    ///     .set("active", true);
    /// ```
    pub fn set(mut self, key: impl Into<String>, value: impl Into<ParameterValue>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Sets a parameter value by mutable reference.
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::EventParameters;
    ///
    /// let mut params = EventParameters::new();
    /// params.set_mut("key", "value");
    /// ```
    pub fn set_mut(&mut self, key: impl Into<String>, value: impl Into<ParameterValue>) {
        self.params.insert(key.into(), value.into());
    }

    /// Gets a parameter value by key.
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::{EventParameters, ParameterValue};
    ///
    /// let params = EventParameters::new().set("key", "value");
    ///
    /// assert!(matches!(params.get("key"), Some(ParameterValue::String(_))));
    /// assert!(params.get("missing").is_none());
    /// ```
    pub fn get(&self, key: &str) -> Option<&ParameterValue> {
        self.params.get(key)
    }

    /// Returns true if the parameter exists.
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::EventParameters;
    ///
    /// let params = EventParameters::new().set("key", "value");
    ///
    /// assert!(params.has("key"));
    /// assert!(!params.has("missing"));
    /// ```
    pub fn has(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    /// Removes a parameter by key. Returns true if the key existed.
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::EventParameters;
    ///
    /// let mut params = EventParameters::new().set("key", "value");
    ///
    /// assert!(params.delete("key"));
    /// assert!(!params.delete("key")); // Already deleted
    /// ```
    pub fn delete(&mut self, key: &str) -> bool {
        self.params.remove(key).is_some()
    }

    /// Returns true if there are no parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::EventParameters;
    ///
    /// let params = EventParameters::new();
    /// assert!(params.is_empty());
    ///
    /// let params = EventParameters::new().set("key", "value");
    /// assert!(!params.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Returns the number of parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use funnelmob::EventParameters;
    ///
    /// let params = EventParameters::new()
    ///     .set("a", 1)
    ///     .set("b", 2);
    ///
    /// assert_eq!(params.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// Returns the underlying map, or None if empty.
    ///
    /// This is used for serialization - empty parameters are omitted from JSON.
    #[allow(dead_code)]
    pub(crate) fn to_map(&self) -> Option<&HashMap<String, ParameterValue>> {
        if self.params.is_empty() {
            None
        } else {
            Some(&self.params)
        }
    }

    /// Consumes self and returns the underlying map, or None if empty.
    #[allow(dead_code)]
    pub(crate) fn into_map(self) -> Option<HashMap<String, ParameterValue>> {
        if self.params.is_empty() {
            None
        } else {
            Some(self.params)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty() {
        let params = EventParameters::new();
        assert!(params.is_empty());
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_set_and_get_string() {
        let params = EventParameters::new().set("name", "John");

        assert_eq!(params.get("name"), Some(&ParameterValue::String("John".to_string())));
    }

    #[test]
    fn test_set_and_get_integer() {
        let params = EventParameters::new()
            .set("level_i32", 5_i32)
            .set("level_i64", 5_i64);

        assert_eq!(params.get("level_i32"), Some(&ParameterValue::Integer(5)));
        assert_eq!(params.get("level_i64"), Some(&ParameterValue::Integer(5)));
    }

    #[test]
    fn test_set_and_get_float() {
        let params = EventParameters::new()
            .set("score_f32", 99.5_f32)
            .set("score_f64", 99.5_f64);

        assert_eq!(params.get("score_f32"), Some(&ParameterValue::Float(99.5)));
        assert_eq!(params.get("score_f64"), Some(&ParameterValue::Float(99.5)));
    }

    #[test]
    fn test_set_and_get_boolean() {
        let params = EventParameters::new()
            .set("active", true)
            .set("disabled", false);

        assert_eq!(params.get("active"), Some(&ParameterValue::Boolean(true)));
        assert_eq!(params.get("disabled"), Some(&ParameterValue::Boolean(false)));
    }

    #[test]
    fn test_method_chaining() {
        let params = EventParameters::new()
            .set("plan", "premium")
            .set("trial", true)
            .set("level", 5)
            .set("score", 99.5);

        assert_eq!(params.len(), 4);
        assert!(params.has("plan"));
        assert!(params.has("trial"));
        assert!(params.has("level"));
        assert!(params.has("score"));
    }

    #[test]
    fn test_set_overwrites() {
        let params = EventParameters::new()
            .set("key", "first")
            .set("key", "second");

        assert_eq!(params.len(), 1);
        assert_eq!(params.get("key"), Some(&ParameterValue::String("second".to_string())));
    }

    #[test]
    fn test_set_mut() {
        let mut params = EventParameters::new();
        params.set_mut("key", "value");

        assert!(params.has("key"));
    }

    #[test]
    fn test_has() {
        let params = EventParameters::new().set("exists", true);

        assert!(params.has("exists"));
        assert!(!params.has("missing"));
    }

    #[test]
    fn test_delete() {
        let mut params = EventParameters::new().set("key", "value");

        assert!(params.delete("key"));
        assert!(!params.has("key"));
        assert!(!params.delete("key")); // Already deleted
    }

    #[test]
    fn test_to_map_empty() {
        let params = EventParameters::new();
        assert!(params.to_map().is_none());
    }

    #[test]
    fn test_to_map_non_empty() {
        let params = EventParameters::new().set("key", "value");
        assert!(params.to_map().is_some());
        assert_eq!(params.to_map().unwrap().len(), 1);
    }

    #[test]
    fn test_into_map() {
        let params = EventParameters::new().set("key", "value");
        let map = params.into_map();

        assert!(map.is_some());
        assert_eq!(map.unwrap().len(), 1);
    }

    #[test]
    fn test_from_map() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), ParameterValue::String("value".to_string()));

        let params = EventParameters::from_map(map);
        assert!(params.has("key"));
    }

    #[test]
    fn test_serialization() {
        let params = EventParameters::new()
            .set("string", "hello")
            .set("int", 42_i64)
            .set("float", 3.14)
            .set("bool", true);

        let map = params.to_map().unwrap();
        let json = serde_json::to_string(map).unwrap();

        // Verify all types are serialized correctly
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["string"], "hello");
        assert_eq!(parsed["int"], 42);
        assert!((parsed["float"].as_f64().unwrap() - 3.14).abs() < f64::EPSILON);
        assert_eq!(parsed["bool"], true);
    }

    #[test]
    fn test_clone_and_eq() {
        let params1 = EventParameters::new()
            .set("a", 1)
            .set("b", "two");
        let params2 = params1.clone();

        assert_eq!(params1, params2);
    }
}
