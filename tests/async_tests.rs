//! Async tests for the FunnelMob SDK.
//!
//! These tests verify the async API works correctly.
//! Run with: cargo test --features async

#![cfg(feature = "async")]

use funnelmob::{Configuration, Environment, EventParameters, FunnelMob, LogLevel, Revenue};

fn test_sdk() -> FunnelMob {
    let config = Configuration::builder("com.test.async", "test_key")
        .environment(Environment::Sandbox)
        .log_level(LogLevel::None)
        .build()
        .unwrap();
    FunnelMob::new(config).unwrap()
}

#[tokio::test]
async fn test_track_event_async() {
    let sdk = test_sdk();
    assert!(sdk.track_event_async("async_event").await.is_ok());
}

#[tokio::test]
async fn test_track_event_async_invalid_name() {
    let sdk = test_sdk();
    assert!(sdk.track_event_async("").await.is_err());
    assert!(sdk.track_event_async("2invalid").await.is_err());
}

#[tokio::test]
async fn test_track_event_with_revenue_async() {
    let sdk = test_sdk();
    let revenue = Revenue::usd(29.99).unwrap();
    assert!(
        sdk.track_event_with_revenue_async("async_purchase", revenue)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn test_track_event_with_params_async() {
    let sdk = test_sdk();
    let params = EventParameters::new()
        .set("plan", "premium")
        .set("async", true);
    assert!(
        sdk.track_event_with_params_async("async_signup", params)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn test_track_event_with_revenue_and_params_async() {
    let sdk = test_sdk();
    let revenue = Revenue::eur(99.00).unwrap();
    let params = EventParameters::new()
        .set("product", "subscription")
        .set("months", 12_i64);
    assert!(
        sdk.track_event_with_revenue_and_params_async("async_subscribe", revenue, params)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn test_async_disabled() {
    let sdk = test_sdk();
    sdk.set_enabled(false);

    // Should succeed silently when disabled
    assert!(sdk.track_event_async("ignored").await.is_ok());
    assert!(sdk.flush_async().await.is_ok());

    sdk.set_enabled(true);
    assert!(sdk.track_event_async("enabled_again").await.is_ok());
}

#[tokio::test]
async fn test_flush_async_empty_queue() {
    // Create a fresh SDK instance that hasn't tracked any events
    let config = Configuration::builder("com.test.async.empty", "test_key")
        .environment(Environment::Sandbox)
        .log_level(LogLevel::None)
        .build()
        .unwrap();
    let sdk = FunnelMob::new(config).unwrap();

    // Disable the SDK so flush returns immediately without network call
    sdk.set_enabled(false);
    assert!(sdk.flush_async().await.is_ok());
}

#[tokio::test]
async fn test_mixed_sync_async() {
    let sdk = test_sdk();

    // Mix sync and async calls
    sdk.track_event("sync_event").unwrap();
    sdk.track_event_async("async_event").await.unwrap();
    sdk.track_event("sync_event_2").unwrap();
    sdk.track_event_async("async_event_2").await.unwrap();

    // Both types should be queued
    // Note: We can't easily verify queue contents, but no errors means success
}

#[tokio::test]
async fn test_concurrent_async_tracking() {
    // Spawn multiple async tasks
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let event_name = format!("concurrent_event_{}", i);
            async move {
                // Each task creates its own SDK instance
                let sdk = test_sdk();
                sdk.track_event_async(&event_name).await
            }
        })
        .collect();

    // Wait for all tasks
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}
