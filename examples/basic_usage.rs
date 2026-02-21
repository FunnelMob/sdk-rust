//! Basic usage example for the FunnelMob Rust SDK.
//!
//! Run with: cargo run --example basic_usage

use funnelmob::{
    standard_events, Configuration, EventParameters, FunnelMob, LogLevel, Revenue,
};

fn main() {
    // Configure the SDK
    let config = Configuration::builder("your_api_key_here")
        .server("http://localhost:3080/v1")
        .platform("web")
        .log_level(LogLevel::Debug)
        .flush_interval_ms(5000)
        .max_batch_size(50)
        .build()
        .expect("Failed to build configuration");

    // Initialize the SDK
    let sdk = FunnelMob::new(config).expect("Failed to initialize SDK");

    println!("FunnelMob SDK initialized");
    println!("  Session ID: {}", sdk.session_id());
    println!("  Device ID: {}", sdk.device_id());
    println!("  Enabled: {}", sdk.is_enabled());

    // Track a simple event
    sdk.track_event("app_launched")
        .expect("Failed to track event");
    println!("\nTracked: app_launched");

    // Track a standard registration event
    sdk.track_event(standard_events::FM_REGISTRATION)
        .expect("Failed to track registration");
    println!("Tracked: fm_registration");

    // Track an event with custom parameters
    let params = EventParameters::new()
        .set("item_id", "sku_12345")
        .set("item_name", "Premium Widget")
        .set("quantity", 2_i64)
        .set("is_gift", false);

    sdk.track_event_with_params("add_to_cart", params)
        .expect("Failed to track add_to_cart");
    println!("Tracked: add_to_cart with parameters");

    // Track a purchase event with revenue
    let revenue = Revenue::usd(29.99).expect("Invalid revenue");
    sdk.track_event_with_revenue(standard_events::FM_PURCHASE, revenue)
        .expect("Failed to track purchase");
    println!("Tracked: fm_purchase with $29.99 USD");

    // Track a subscription with revenue and parameters
    let subscription_revenue = Revenue::eur(9.99).expect("Invalid revenue");
    let subscription_params = EventParameters::new()
        .set("plan", "monthly")
        .set("trial_days", 7_i64)
        .set("auto_renew", true);

    sdk.track_event_with_revenue_and_params(
        standard_events::FM_SUBSCRIBE,
        subscription_revenue,
        subscription_params,
    )
    .expect("Failed to track subscription");
    println!("Tracked: fm_subscribe with 9.99 EUR and parameters");

    // Demonstrate revenue in different currencies
    let gbp_revenue = Revenue::gbp(19.99).expect("Invalid revenue");
    let jpy_revenue = Revenue::new(2000.0, "JPY").expect("Invalid revenue");

    sdk.track_event_with_revenue("uk_purchase", gbp_revenue)
        .expect("Failed to track UK purchase");
    sdk.track_event_with_revenue("jp_purchase", jpy_revenue)
        .expect("Failed to track JP purchase");
    println!("Tracked: purchases in GBP and JPY");

    // Demonstrate disabling/enabling the SDK
    println!("\nDisabling SDK...");
    sdk.set_enabled(false);
    println!("  Enabled: {}", sdk.is_enabled());

    sdk.track_event("ignored_event")
        .expect("Should succeed silently");
    println!("  Tracked event while disabled (silently ignored)");

    sdk.set_enabled(true);
    println!("  Re-enabled: {}", sdk.is_enabled());

    // Flush pending events
    println!("\nFlushing events...");
    if let Err(e) = sdk.flush() {
        println!("  Flush result: {} (expected without server)", e);
    } else {
        println!("  Flush completed successfully");
    }

    println!("\nExample completed!");
}
