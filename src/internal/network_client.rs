//! HTTP network client for the FunnelMob SDK.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::configuration::Configuration;
use crate::error::FunnelMobError;
use crate::internal::event::EventBatch;
use crate::internal::logger::Logger;

/// Response from the events API.
#[derive(Debug, Clone, Deserialize)]
pub struct EventBatchResponse {
    /// Number of events accepted.
    pub accepted: i32,

    /// Number of events rejected.
    pub rejected: i32,

    /// Errors for rejected events.
    #[serde(default)]
    pub errors: Vec<EventError>,
}

/// Error for a rejected event.
#[derive(Debug, Clone, Deserialize)]
pub struct EventError {
    /// Event ID that was rejected.
    pub event_id: String,

    /// Error code.
    pub code: String,

    /// Human-readable error message.
    pub message: String,
}

/// Session registration request.
#[derive(Debug, Clone, Serialize)]
pub struct SessionRequest {
    pub app_id: String,
    pub device_id: String,
    pub session_id: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_first_session: Option<bool>,
}

/// Response from the session API.
#[derive(Debug, Clone, Deserialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub server_timestamp: String,
}

/// Network error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkErrorKind {
    /// Invalid API key (401).
    Unauthorized,
    /// Rate limit exceeded (429).
    RateLimited,
    /// Client error (4xx).
    ClientError,
    /// Server error (5xx).
    ServerError,
    /// Network/connection error.
    ConnectionError,
    /// Timeout.
    Timeout,
    /// Unknown error.
    Unknown,
}

/// Synchronous HTTP client for the FunnelMob API.
pub struct NetworkClient {
    base_url: String,
    api_key: String,
    logger: Logger,
    max_retries: u32,
    base_delay_ms: u64,
}

impl NetworkClient {
    /// Creates a new network client from configuration.
    pub fn new(config: &Configuration, logger: Logger) -> Self {
        Self {
            base_url: config.base_url().to_string(),
            api_key: config.api_key().to_string(),
            logger,
            max_retries: 3,
            base_delay_ms: 1000,
        }
    }

    /// Sends a batch of events to the API.
    ///
    /// Retries on transient failures with exponential backoff.
    pub fn send_events(&self, batch: &EventBatch) -> Result<EventBatchResponse, FunnelMobError> {
        let url = format!("{}/events", self.base_url);
        self.post_with_retry(&url, batch)
    }

    /// Registers a new session.
    pub fn register_session(
        &self,
        request: &SessionRequest,
    ) -> Result<SessionResponse, FunnelMobError> {
        let url = format!("{}/session", self.base_url);
        self.post_with_retry(&url, request)
    }

    /// Makes a POST request with retry logic.
    fn post_with_retry<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R, FunnelMobError> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay = self.calculate_backoff(attempt);
                self.logger.debug(&format!(
                    "Retry attempt {} after {}ms delay",
                    attempt, delay
                ));
                std::thread::sleep(Duration::from_millis(delay));
            }

            match self.do_post(url, body) {
                Ok(response) => return Ok(response),
                Err((kind, msg)) => {
                    self.logger
                        .warn(&format!("Request failed (attempt {}): {}", attempt + 1, msg));

                    // Don't retry on client errors (except rate limiting)
                    if !Self::should_retry(&kind) {
                        return Err(FunnelMobError::Configuration(format!(
                            "Network error ({}): {}",
                            Self::error_kind_name(&kind),
                            msg
                        )));
                    }

                    last_error = Some((kind, msg));
                }
            }
        }

        // All retries exhausted
        let (kind, msg) = last_error.unwrap_or((NetworkErrorKind::Unknown, "Unknown error".to_string()));
        Err(FunnelMobError::Configuration(format!(
            "Network error after {} retries ({}): {}",
            self.max_retries,
            Self::error_kind_name(&kind),
            msg
        )))
    }

    /// Makes a single POST request.
    fn do_post<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R, (NetworkErrorKind, String)> {
        let response = ureq::post(url)
            .set("Content-Type", "application/json")
            .set("X-FM-API-Key", &self.api_key)
            .timeout(Duration::from_secs(30))
            .send_json(body);

        match response {
            Ok(resp) => {
                let status = resp.status();
                if (200..300).contains(&status) {
                    resp.into_json()
                        .map_err(|e| (NetworkErrorKind::Unknown, format!("Failed to parse response: {}", e)))
                } else {
                    // Unexpected success status
                    Err((NetworkErrorKind::Unknown, format!("Unexpected status: {}", status)))
                }
            }
            Err(ureq::Error::Status(status, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                let kind = match status {
                    401 => NetworkErrorKind::Unauthorized,
                    429 => NetworkErrorKind::RateLimited,
                    400..=499 => NetworkErrorKind::ClientError,
                    500..=599 => NetworkErrorKind::ServerError,
                    _ => NetworkErrorKind::Unknown,
                };
                Err((kind, format!("HTTP {}: {}", status, body)))
            }
            Err(ureq::Error::Transport(transport)) => {
                let kind = match transport.kind() {
                    ureq::ErrorKind::Io => {
                        // Check if it's a timeout by looking at the message
                        if transport.to_string().to_lowercase().contains("timeout") {
                            NetworkErrorKind::Timeout
                        } else {
                            NetworkErrorKind::ConnectionError
                        }
                    }
                    ureq::ErrorKind::ConnectionFailed | ureq::ErrorKind::Dns => {
                        NetworkErrorKind::ConnectionError
                    }
                    _ => NetworkErrorKind::ConnectionError,
                };
                Err((kind, transport.to_string()))
            }
        }
    }

    /// Determines if a request should be retried based on error kind.
    fn should_retry(kind: &NetworkErrorKind) -> bool {
        matches!(
            kind,
            NetworkErrorKind::RateLimited
                | NetworkErrorKind::ServerError
                | NetworkErrorKind::ConnectionError
                | NetworkErrorKind::Timeout
        )
    }

    /// Calculates exponential backoff delay.
    fn calculate_backoff(&self, attempt: u32) -> u64 {
        // Exponential backoff: base * 2^attempt with jitter
        let base_delay = self.base_delay_ms * 2u64.pow(attempt.saturating_sub(1));

        // Add some jitter (0-25% of delay)
        let jitter = (base_delay as f64 * 0.25 * rand_float()) as u64;

        // Cap at 30 seconds
        (base_delay + jitter).min(30_000)
    }

    /// Returns a human-readable name for the error kind.
    fn error_kind_name(kind: &NetworkErrorKind) -> &'static str {
        match kind {
            NetworkErrorKind::Unauthorized => "unauthorized",
            NetworkErrorKind::RateLimited => "rate_limited",
            NetworkErrorKind::ClientError => "client_error",
            NetworkErrorKind::ServerError => "server_error",
            NetworkErrorKind::ConnectionError => "connection_error",
            NetworkErrorKind::Timeout => "timeout",
            NetworkErrorKind::Unknown => "unknown",
        }
    }
}

/// Simple pseudo-random float between 0 and 1 for jitter.
fn rand_float() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (nanos % 1000) as f64 / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_retry() {
        assert!(NetworkClient::should_retry(&NetworkErrorKind::RateLimited));
        assert!(NetworkClient::should_retry(&NetworkErrorKind::ServerError));
        assert!(NetworkClient::should_retry(&NetworkErrorKind::ConnectionError));
        assert!(NetworkClient::should_retry(&NetworkErrorKind::Timeout));

        assert!(!NetworkClient::should_retry(&NetworkErrorKind::Unauthorized));
        assert!(!NetworkClient::should_retry(&NetworkErrorKind::ClientError));
        assert!(!NetworkClient::should_retry(&NetworkErrorKind::Unknown));
    }

    #[test]
    fn test_error_kind_name() {
        assert_eq!(
            NetworkClient::error_kind_name(&NetworkErrorKind::Unauthorized),
            "unauthorized"
        );
        assert_eq!(
            NetworkClient::error_kind_name(&NetworkErrorKind::RateLimited),
            "rate_limited"
        );
    }

    #[test]
    fn test_calculate_backoff() {
        let config = Configuration::builder("app", "key").build().unwrap();
        let client = NetworkClient::new(&config, Logger::default());

        // First retry should be ~1000ms (base delay)
        let delay1 = client.calculate_backoff(1);
        assert!(delay1 >= 1000 && delay1 <= 1250); // With jitter

        // Second retry should be ~2000ms
        let delay2 = client.calculate_backoff(2);
        assert!(delay2 >= 2000 && delay2 <= 2500);

        // Third retry should be ~4000ms
        let delay3 = client.calculate_backoff(3);
        assert!(delay3 >= 4000 && delay3 <= 5000);
    }

    #[test]
    fn test_backoff_cap() {
        let config = Configuration::builder("app", "key").build().unwrap();
        let client = NetworkClient::new(&config, Logger::default());

        // Very high attempt should be capped at 30 seconds
        let delay = client.calculate_backoff(10);
        assert!(delay <= 30_000);
    }
}
