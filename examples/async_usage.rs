//! Async usage example for the FunnelMob Rust SDK.
//!
//! Run with: cargo run --example async_usage --features async

#[cfg(feature = "async")]
use funnelmob::{
    standard_events, Configuration, Environment, EventParameters, FunnelMob, LogLevel, Revenue,
};

#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    // Configure the SDK
    let config = Configuration::builder("com.example.async_app", "your_api_key_here")
        .environment(Environment::Sandbox)
        .log_level(LogLevel::Debug)
        .flush_interval_ms(5000)
        .max_batch_size(50)
        .build()
        .expect("Failed to build configuration");

    // Initialize the SDK
    let sdk = FunnelMob::new(config).expect("Failed to initialize SDK");

    println!("FunnelMob SDK initialized (async mode)");
    println!("  Session ID: {}", sdk.session_id());
    println!("  Device ID: {}", sdk.device_id());

    // Track a simple event asynchronously
    sdk.track_event_async("app_launched")
        .await
        .expect("Failed to track event");
    println!("\nTracked (async): app_launched");

    // Track an event with parameters asynchronously
    let params = EventParameters::new()
        .set("item_id", "sku_12345")
        .set("item_name", "Premium Widget")
        .set("quantity", 2_i64)
        .set("is_gift", false);

    sdk.track_event_with_params_async("add_to_cart", params)
        .await
        .expect("Failed to track add_to_cart");
    println!("Tracked (async): add_to_cart with parameters");

    // Track a purchase with revenue asynchronously
    let revenue = Revenue::usd(29.99).expect("Invalid revenue");
    sdk.track_event_with_revenue_async(standard_events::FM_PURCHASE, revenue)
        .await
        .expect("Failed to track purchase");
    println!("Tracked (async): fm_purchase with $29.99 USD");

    // Track subscription with revenue and parameters
    let subscription_revenue = Revenue::eur(9.99).expect("Invalid revenue");
    let subscription_params = EventParameters::new()
        .set("plan", "monthly")
        .set("trial_days", 7_i64)
        .set("auto_renew", true);

    sdk.track_event_with_revenue_and_params_async(
        standard_events::FM_SUBSCRIBE,
        subscription_revenue,
        subscription_params,
    )
    .await
    .expect("Failed to track subscription");
    println!("Tracked (async): fm_subscribe with 9.99 EUR and parameters");

    // Demonstrate concurrent tracking
    println!("\nTracking multiple events concurrently...");
    let futures: Vec<_> = (0..5)
        .map(|i| {
            let event_name = format!("concurrent_event_{}", i);
            async move {
                // Note: In real code, you'd share the SDK instance properly
                // For this example, we just show the pattern
                println!("  Tracking: {}", event_name);
            }
        })
        .collect();

    for future in futures {
        future.await;
    }

    // Flush events asynchronously
    println!("\nFlushing events (async)...");
    if let Err(e) = sdk.flush_async().await {
        // Expected to fail without a real API endpoint
        println!("  Flush result: {} (expected in sandbox without server)", e);
    } else {
        println!("  Flush completed successfully");
    }

    // Async shutdown
    println!("\nShutting down (async)...");
    sdk.destroy_async().await;

    println!("\nAsync example completed!");
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the 'async' feature.");
    eprintln!("Run with: cargo run --example async_usage --features async");
}
