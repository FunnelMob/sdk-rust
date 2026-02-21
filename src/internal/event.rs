//! Internal event representation for the FunnelMob SDK.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::event_parameters::ParameterValue;
use crate::Revenue;

/// Internal representation of an event for serialization and storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier for this event (UUID v4).
    pub event_id: Uuid,

    /// Name of the event.
    pub event_name: String,

    /// Timestamp when the event occurred.
    pub timestamp: DateTime<Utc>,

    /// Revenue associated with this event, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revenue: Option<EventRevenue>,

    /// Custom parameters for this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, ParameterValue>>,
}

/// Revenue data for serialization (amount as string with 2 decimal places).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventRevenue {
    /// Amount as a decimal string (e.g., "29.99").
    pub amount: String,

    /// ISO 4217 currency code (uppercase).
    pub currency: String,
}

impl Event {
    /// Creates a new event with the given name.
    pub fn new(event_name: impl Into<String>) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_name: event_name.into(),
            timestamp: Utc::now(),
            revenue: None,
            parameters: None,
        }
    }

    /// Creates a new event with the given name and a custom timestamp.
    pub fn with_timestamp(event_name: impl Into<String>, timestamp: DateTime<Utc>) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_name: event_name.into(),
            timestamp,
            revenue: None,
            parameters: None,
        }
    }

    /// Creates a new event with the given name and revenue.
    pub fn with_revenue(event_name: impl Into<String>, revenue: &Revenue) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_name: event_name.into(),
            timestamp: Utc::now(),
            revenue: Some(EventRevenue::from(revenue)),
            parameters: None,
        }
    }

    /// Creates a new event with the given name and parameters.
    pub fn with_parameters(
        event_name: impl Into<String>,
        parameters: HashMap<String, ParameterValue>,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_name: event_name.into(),
            timestamp: Utc::now(),
            revenue: None,
            parameters: if parameters.is_empty() {
                None
            } else {
                Some(parameters)
            },
        }
    }

    /// Creates a new event with the given name, revenue, and parameters.
    pub fn with_revenue_and_parameters(
        event_name: impl Into<String>,
        revenue: &Revenue,
        parameters: HashMap<String, ParameterValue>,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_name: event_name.into(),
            timestamp: Utc::now(),
            revenue: Some(EventRevenue::from(revenue)),
            parameters: if parameters.is_empty() {
                None
            } else {
                Some(parameters)
            },
        }
    }

    /// Sets a custom timestamp on this event (builder pattern).
    pub fn at(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets revenue on this event (builder pattern).
    pub fn with_rev(mut self, revenue: &Revenue) -> Self {
        self.revenue = Some(EventRevenue::from(revenue));
        self
    }

    /// Sets parameters on this event (builder pattern).
    pub fn with_params(mut self, parameters: HashMap<String, ParameterValue>) -> Self {
        self.parameters = if parameters.is_empty() {
            None
        } else {
            Some(parameters)
        };
        self
    }
}

impl From<&Revenue> for EventRevenue {
    fn from(revenue: &Revenue) -> Self {
        Self {
            amount: revenue.amount_string(),
            currency: revenue.currency().to_string(),
        }
    }
}

/// Batch of events to send to the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBatch {
    /// Platform identifier (e.g., "ios", "android", "web").
    pub platform: String,

    /// Device identifier.
    pub device_id: String,

    /// Current session identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<Uuid>,

    /// Events in this batch.
    pub events: Vec<Event>,
}

impl EventBatch {
    /// Creates a new event batch.
    pub fn new(platform: impl Into<String>, device_id: impl Into<String>, events: Vec<Event>) -> Self {
        Self {
            platform: platform.into(),
            device_id: device_id.into(),
            session_id: None,
            events,
        }
    }

    /// Sets the session ID.
    pub fn with_session_id(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

/// Custom serializer for timestamps to ensure millisecond precision with Z suffix.
/// Use with `#[serde(with = "timestamp_format")]` on DateTime<Utc> fields.
pub mod timestamp_format {
    #![allow(dead_code)]
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.format(FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EventParameters;

    #[test]
    fn test_event_new() {
        let event = Event::new("purchase");

        assert_eq!(event.event_name, "purchase");
        assert!(event.revenue.is_none());
        assert!(event.parameters.is_none());
        assert_eq!(event.event_id.get_version_num(), 4);
    }

    #[test]
    fn test_event_with_revenue() {
        let revenue = Revenue::usd(29.99).unwrap();
        let event = Event::with_revenue("purchase", &revenue);

        assert_eq!(event.event_name, "purchase");
        assert!(event.revenue.is_some());
        let rev = event.revenue.unwrap();
        assert_eq!(rev.amount, "29.99");
        assert_eq!(rev.currency, "USD");
    }

    #[test]
    fn test_event_with_parameters() {
        let params = EventParameters::new()
            .set("plan", "premium")
            .set("level", 5_i64);
        let event = Event::with_parameters("signup", params.into_map().unwrap());

        assert_eq!(event.event_name, "signup");
        assert!(event.parameters.is_some());
        let params = event.parameters.unwrap();
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_event_with_empty_parameters() {
        let params = HashMap::new();
        let event = Event::with_parameters("signup", params);

        assert!(event.parameters.is_none());
    }

    #[test]
    fn test_event_with_custom_timestamp() {
        use chrono::TimeZone;
        let ts = Utc.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();
        let event = Event::with_timestamp("test", ts);
        assert_eq!(event.timestamp, ts);
    }

    #[test]
    fn test_event_builder_pattern() {
        use chrono::TimeZone;
        let ts = Utc.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();
        let revenue = Revenue::usd(9.99).unwrap();
        let event = Event::new("purchase").at(ts).with_rev(&revenue);
        assert_eq!(event.timestamp, ts);
        assert!(event.revenue.is_some());
    }

    #[test]
    fn test_event_serialization() {
        let event = Event::new("purchase");
        let json = serde_json::to_string(&event).unwrap();

        assert!(!json.contains("revenue"));
        assert!(!json.contains("parameters"));
        assert!(json.contains("event_id"));
        assert!(json.contains("event_name"));
        assert!(json.contains("timestamp"));
    }

    #[test]
    fn test_event_with_revenue_serialization() {
        let revenue = Revenue::usd(29.99).unwrap();
        let event = Event::with_revenue("purchase", &revenue);
        let json = serde_json::to_string(&event).unwrap();

        assert!(json.contains("\"amount\":\"29.99\""));
        assert!(json.contains("\"currency\":\"USD\""));
    }

    #[test]
    fn test_event_batch_serialization() {
        let events = vec![Event::new("test")];
        let batch = EventBatch::new("web", "device-123", events);
        let json = serde_json::to_string(&batch).unwrap();

        assert!(json.contains("\"platform\":\"web\""));
        assert!(json.contains("\"device_id\":\"device-123\""));
        assert!(json.contains("events"));
        assert!(!json.contains("session_id"));
    }

    #[test]
    fn test_event_batch_with_session() {
        let events = vec![Event::new("test")];
        let session_id = Uuid::new_v4();
        let batch = EventBatch::new("ios", "device-123", events)
            .with_session_id(session_id);
        let json = serde_json::to_string(&batch).unwrap();

        assert!(json.contains("session_id"));
    }

    #[test]
    fn test_event_deserialization() {
        let json = r#"{
            "event_id": "550e8400-e29b-41d4-a716-446655440000",
            "event_name": "purchase",
            "timestamp": "2025-12-21T10:30:00.000Z",
            "revenue": {
                "amount": "29.99",
                "currency": "USD"
            }
        }"#;

        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_name, "purchase");
        assert!(event.revenue.is_some());
    }

    #[test]
    fn test_timestamp_format() {
        let event = Event::new("test");
        let json = serde_json::to_string(&event).unwrap();

        assert!(json.contains("timestamp"));
    }
}
