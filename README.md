# FunnelMob Rust SDK

A Mobile Measurement Partner (MMP) SDK for attributing app installs to advertising campaigns.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
funnelmob = "0.1.0"
```

For async support:

```toml
[dependencies]
funnelmob = { version = "0.1.0", features = ["async"] }
```

## Quick Start

```rust
use funnelmob::{FunnelMob, Configuration};

fn main() {
    // Configure the SDK
    let config = Configuration::builder("fm_live_abc123")
        .build()
        .unwrap();

    // Initialize
    let sdk = FunnelMob::new(config).unwrap();

    // Track events
    sdk.track_event("button_click").unwrap();

    // Flush before exit
    sdk.flush().unwrap();
}
```

## Configuration

```rust
use funnelmob::{Configuration, LogLevel};

let config = Configuration::builder("your_api_key")
    .server("http://localhost:3080")  // Optional: Override the API host (default: https://api.funnelmob.com)
    .platform("web")                  // Optional: defaults to detected OS
    .log_level(LogLevel::Debug)       // None, Error, Warning, Info, Debug, Verbose
    .flush_interval_ms(30000)         // Auto-flush interval (min: 1000ms)
    .max_batch_size(100)              // Events per batch (1-100)
    .build()
    .unwrap();
```

### Custom Base URL

By default the SDK calls `https://api.funnelmob.com`, appending `/v1/<endpoint>`
to each request. Override with `.server(...)` for local development:

```rust
let config = Configuration::builder("fm_test_key")
    .server("http://localhost:3080")
    .build()
    .unwrap();
```

Pass the **host root only** — the SDK appends `/v1` itself, so
`http://localhost:3080` (not `http://localhost:3080/v1`). A trailing slash is
trimmed automatically.

## Event Tracking

### Simple Events

```rust
sdk.track_event("level_complete").unwrap();
```

### Events with Revenue

```rust
use funnelmob::Revenue;

let revenue = Revenue::usd(29.99).unwrap();
sdk.track_event_with_revenue("purchase", revenue).unwrap();

// Other currencies
let eur = Revenue::eur(19.99).unwrap();
let gbp = Revenue::gbp(14.99).unwrap();
let jpy = Revenue::new(2000.0, "JPY").unwrap();
```

### Events with Parameters

```rust
use funnelmob::EventParameters;

let params = EventParameters::new()
    .set("item_id", "sku_123")
    .set("quantity", 2_i64)
    .set("price", 29.99_f64)
    .set("is_gift", false);

sdk.track_event_with_params("add_to_cart", params).unwrap();
```

### Events with Revenue and Parameters

```rust
let revenue = Revenue::usd(99.00).unwrap();
let params = EventParameters::new()
    .set("plan", "annual")
    .set("trial_days", 7_i64);

sdk.track_event_with_revenue_and_params("subscribe", revenue, params).unwrap();
```

## Standard Events

### Using Typed Methods (Recommended)

29 typed methods provide type-safe, self-documenting event tracking without needing to remember event name strings:

```rust
// Simple events
sdk.track_page_view()?;
sdk.track_add_to_cart_with_params(
    EventParameters::new().set("item_id", "SKU-123")
)?;

// Revenue events (amount + currency required)
sdk.track_purchase(29.99, "USD")?;
sdk.track_purchase_with_params(29.99, "USD",
    EventParameters::new().set("order_id", "ORD-456")
)?;
sdk.track_subscribe(9.99, "USD")?;
sdk.track_start_trial(0.0, "USD")?;
sdk.track_donate(10.0, "USD")?;

// Spend credits (amount only)
sdk.track_spent_credits(100.0)?;
```

See [docs/reference/sdk_events_reference.md](../docs/reference/sdk_events_reference.md) for the full list of 29 typed methods with platform support details.

### Using Constants

For custom event name handling or when using the generic `track_event` API:

```rust
use funnelmob::standard_events;

sdk.track_event(standard_events::FM_REGISTRATION)?;
sdk.track_event(standard_events::FM_LOGIN)?;
sdk.track_event(standard_events::FM_PURCHASE)?;
sdk.track_event(standard_events::FM_ADD_TO_CART)?;
sdk.track_event(standard_events::FM_SUBSCRIBE)?;
sdk.track_event(standard_events::FM_START_TRIAL)?;
sdk.track_event(standard_events::FM_RATE)?;
sdk.track_event(standard_events::FM_SPEND_CREDITS)?;
// ... and many more constants in the standard_events module
```

## Global Singleton

For convenience, use the singleton pattern:

```rust
use funnelmob::{FunnelMob, Configuration};

// Initialize once at startup
let config = Configuration::builder("api_key")
    .build()
    .unwrap();
FunnelMob::initialize(config).unwrap();

// Access anywhere
if let Some(sdk) = FunnelMob::shared() {
    sdk.track_event("page_view").unwrap();
}
```

## Async Support

Enable the `async` feature for async/await compatibility:

```rust
use funnelmob::{FunnelMob, Configuration, Revenue, EventParameters};

#[tokio::main]
async fn main() {
    let config = Configuration::builder("api_key")
        .build()
        .unwrap();
    let sdk = FunnelMob::new(config).unwrap();

    // Async tracking methods
    sdk.track_event_async("app_start").await.unwrap();

    sdk.track_event_with_revenue_async(
        "purchase",
        Revenue::usd(29.99).unwrap()
    ).await.unwrap();

    sdk.track_event_with_params_async(
        "signup",
        EventParameters::new().set("method", "email")
    ).await.unwrap();

    // Async flush
    sdk.flush_async().await.unwrap();

    // Async shutdown
    sdk.destroy_async().await;
}
```

## SDK Control

```rust
// Check if enabled
if sdk.is_enabled() {
    println!("SDK is tracking events");
}

// Disable tracking (e.g., for GDPR compliance)
sdk.set_enabled(false);

// Re-enable
sdk.set_enabled(true);

// Get identifiers
println!("Session ID: {}", sdk.session_id());
println!("Device ID: {}", sdk.device_id());

// Manual flush
sdk.flush().unwrap();

// Shutdown
sdk.destroy();
```

## Validation

Event names and revenue are validated:

```rust
use funnelmob::validation::{validate_event_name, validate_currency};

// Event names: letters, numbers, underscores; must start with letter; max 100 chars
assert!(validate_event_name("purchase").is_ok());
assert!(validate_event_name("level_2_complete").is_ok());
assert!(validate_event_name("2nd_level").is_err());  // Can't start with number
assert!(validate_event_name("my-event").is_err());   // No hyphens

// Currency: 3-letter uppercase ISO 4217
assert!(validate_currency("USD").is_ok());
assert!(validate_currency("usd").is_err());  // Must be uppercase
```

## Error Handling

```rust
use funnelmob::{FunnelMobError, ValidationError};

match sdk.track_event("") {
    Ok(()) => println!("Event tracked"),
    Err(FunnelMobError::Validation(ValidationError::EventNameRequired)) => {
        println!("Event name is required");
    }
    Err(e) => println!("Error: {}", e),
}
```

## Features

| Feature | Description |
|---------|-------------|
| `default` | Sync API only (ureq) |
| `async` | Adds async API (tokio + reqwest) |

## Requirements

- Rust 1.70+
- For async: tokio runtime

## License

MIT

