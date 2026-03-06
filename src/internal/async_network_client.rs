//! Async HTTP network client for the FunnelMob SDK.
//!
//! This module is only available when the `async` feature is enabled.

use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::configuration::Configuration;
use crate::error::FunnelMobError;
use crate::internal::event::EventBatch;
use crate::internal::logger::Logger;
use crate::internal::network_client::{EventBatchResponse, NetworkErrorKind};

/// Async HTTP client for the FunnelMob API.
pub struct AsyncNetworkClient {
    client: Client,
    base_url: String,
    api_key: String,
    logger: Logger,
    max_retries: u32,
    base_delay_ms: u64,
}

impl AsyncNetworkClient {
    /// Creates a new async network client from configuration.
    pub fn new(config: &Configuration, logger: Logger) -> Result<Self, FunnelMobError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| FunnelMobError::Configuration(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url: config.server().to_string(),
            api_key: config.api_key().to_string(),
            logger,
            max_retries: 3,
            base_delay_ms: 1000,
        })
    }

    /// Sends a batch of events to the API asynchronously.
    ///
    /// Retries on transient failures with exponential backoff.
    pub async fn send_events(&self, batch: &EventBatch) -> Result<EventBatchResponse, FunnelMobError> {
        let url = format!("{}/events", self.base_url);
        self.post_with_retry(&url, batch).await
    }

    /// Fetches remote config from the API asynchronously.
    pub async fn fetch_config(&self) -> Result<std::collections::HashMap<String, serde_json::Value>, FunnelMobError> {
        let url = format!("{}/config", self.base_url);
        self.get_with_retry(&url).await
    }

    /// Makes a GET request with retry logic.
    async fn get_with_retry<R: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
    ) -> Result<R, FunnelMobError> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay = self.calculate_backoff(attempt);
                self.logger.debug(&format!(
                    "Retry attempt {} after {}ms delay",
                    attempt, delay
                ));
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }

            match self.do_get(url).await {
                Ok(response) => return Ok(response),
                Err((kind, msg)) => {
                    self.logger
                        .warn(&format!("Request failed (attempt {}): {}", attempt + 1, msg));

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

        let (kind, msg) = last_error.unwrap_or((NetworkErrorKind::Unknown, "Unknown error".to_string()));
        Err(FunnelMobError::Configuration(format!(
            "Network error after {} retries ({}): {}",
            self.max_retries,
            Self::error_kind_name(&kind),
            msg
        )))
    }

    /// Makes a single GET request.
    async fn do_get<R: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
    ) -> Result<R, (NetworkErrorKind, String)> {
        let response = self
            .client
            .get(url)
            .header("X-FM-API-Key", &self.api_key)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if (200..300).contains(&status) {
                    resp.json()
                        .await
                        .map_err(|e| (NetworkErrorKind::Unknown, format!("Failed to parse response: {}", e)))
                } else {
                    let body = resp.text().await.unwrap_or_default();
                    let kind = match status {
                        401 => NetworkErrorKind::Unauthorized,
                        429 => NetworkErrorKind::RateLimited,
                        400..=499 => NetworkErrorKind::ClientError,
                        500..=599 => NetworkErrorKind::ServerError,
                        _ => NetworkErrorKind::Unknown,
                    };
                    Err((kind, format!("HTTP {}: {}", status, body)))
                }
            }
            Err(e) => {
                let kind = if e.is_timeout() {
                    NetworkErrorKind::Timeout
                } else {
                    NetworkErrorKind::ConnectionError
                };
                Err((kind, e.to_string()))
            }
        }
    }

    /// Makes a POST request with retry logic.
    async fn post_with_retry<T: Serialize, R: for<'de> Deserialize<'de>>(
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
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }

            match self.do_post(url, body).await {
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
    async fn do_post<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R, (NetworkErrorKind, String)> {
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-FM-API-Key", &self.api_key)
            .json(body)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if (200..300).contains(&status) {
                    resp.json()
                        .await
                        .map_err(|e| (NetworkErrorKind::Unknown, format!("Failed to parse response: {}", e)))
                } else {
                    let body = resp.text().await.unwrap_or_default();
                    let kind = match status {
                        401 => NetworkErrorKind::Unauthorized,
                        429 => NetworkErrorKind::RateLimited,
                        400..=499 => NetworkErrorKind::ClientError,
                        500..=599 => NetworkErrorKind::ServerError,
                        _ => NetworkErrorKind::Unknown,
                    };
                    Err((kind, format!("HTTP {}: {}", status, body)))
                }
            }
            Err(e) => {
                let kind = if e.is_timeout() {
                    NetworkErrorKind::Timeout
                } else {
                    // Connection errors, DNS errors, and other transport issues
                    NetworkErrorKind::ConnectionError
                };
                Err((kind, e.to_string()))
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
        assert!(AsyncNetworkClient::should_retry(&NetworkErrorKind::RateLimited));
        assert!(AsyncNetworkClient::should_retry(&NetworkErrorKind::ServerError));
        assert!(AsyncNetworkClient::should_retry(&NetworkErrorKind::ConnectionError));
        assert!(AsyncNetworkClient::should_retry(&NetworkErrorKind::Timeout));

        assert!(!AsyncNetworkClient::should_retry(&NetworkErrorKind::Unauthorized));
        assert!(!AsyncNetworkClient::should_retry(&NetworkErrorKind::ClientError));
        assert!(!AsyncNetworkClient::should_retry(&NetworkErrorKind::Unknown));
    }

    #[test]
    fn test_error_kind_name() {
        assert_eq!(
            AsyncNetworkClient::error_kind_name(&NetworkErrorKind::Unauthorized),
            "unauthorized"
        );
        assert_eq!(
            AsyncNetworkClient::error_kind_name(&NetworkErrorKind::RateLimited),
            "rate_limited"
        );
    }
}
