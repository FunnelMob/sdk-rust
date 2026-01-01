//! Standard event names for the FunnelMob SDK.
//!
//! These constants provide standardized event names that are recognized
//! by the FunnelMob platform for analytics and attribution.

/// Standard event for user registration.
pub const FM_REGISTRATION: &str = "fm_registration";

/// Standard event for user login.
pub const FM_LOGIN: &str = "fm_login";

/// Standard event for completing a purchase.
pub const FM_PURCHASE: &str = "fm_purchase";

/// Standard event for adding an item to cart.
pub const FM_ADD_TO_CART: &str = "fm_add_to_cart";

/// Standard event for starting checkout.
pub const FM_CHECKOUT_START: &str = "fm_checkout_start";

/// Standard event for completing a level in a game.
pub const FM_LEVEL_COMPLETE: &str = "fm_level_complete";

/// Standard event for completing a tutorial.
pub const FM_TUTORIAL_COMPLETE: &str = "fm_tutorial_complete";

/// Standard event for subscribing to a service.
pub const FM_SUBSCRIBE: &str = "fm_subscribe";

/// Standard event for starting a free trial.
pub const FM_START_TRIAL: &str = "fm_start_trial";

/// Standard event for rating the app.
pub const FM_RATE: &str = "fm_rate";

/// Standard event for sharing content.
pub const FM_SHARE: &str = "fm_share";

/// Standard event for inviting a friend.
pub const FM_INVITE: &str = "fm_invite";

/// Standard event for achieving a goal or milestone.
pub const FM_ACHIEVEMENT: &str = "fm_achievement";

/// Standard event for spending virtual currency.
pub const FM_SPEND_CREDITS: &str = "fm_spend_credits";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::validate_event_name;

    #[test]
    fn test_all_standard_events_are_valid() {
        let events = [
            FM_REGISTRATION,
            FM_LOGIN,
            FM_PURCHASE,
            FM_ADD_TO_CART,
            FM_CHECKOUT_START,
            FM_LEVEL_COMPLETE,
            FM_TUTORIAL_COMPLETE,
            FM_SUBSCRIBE,
            FM_START_TRIAL,
            FM_RATE,
            FM_SHARE,
            FM_INVITE,
            FM_ACHIEVEMENT,
            FM_SPEND_CREDITS,
        ];

        for event in events {
            assert!(
                validate_event_name(event).is_ok(),
                "Standard event '{}' failed validation",
                event
            );
        }
    }
}
