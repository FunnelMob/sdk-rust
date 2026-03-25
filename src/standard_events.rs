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

/// Standard event for viewing a page or screen.
pub const FM_PAGE_VIEW: &str = "fm_page_view";

/// Standard event for viewing a content item (product, article, etc.).
pub const FM_VIEW_CONTENT: &str = "fm_view_content";

/// Standard event for performing a search.
pub const FM_SEARCH: &str = "fm_search";

/// Standard event for adding an item to a wishlist.
pub const FM_ADD_TO_WISHLIST: &str = "fm_add_to_wishlist";

/// Standard event for initiating checkout.
pub const FM_INITIATE_CHECKOUT: &str = "fm_initiate_checkout";

/// Standard event for adding payment information.
pub const FM_ADD_PAYMENT_INFO: &str = "fm_add_payment_info";

/// Standard event for generating a lead.
pub const FM_LEAD: &str = "fm_lead";

/// Standard event for completing registration.
pub const FM_COMPLETE_REGISTRATION: &str = "fm_complete_registration";

/// Standard event for a contact interaction.
pub const FM_CONTACT: &str = "fm_contact";

/// Standard event for scheduling an appointment.
pub const FM_SCHEDULE: &str = "fm_schedule";

/// Standard event for finding a physical location.
pub const FM_FIND_LOCATION: &str = "fm_find_location";

/// Standard event for customizing a product.
pub const FM_CUSTOMIZE_PRODUCT: &str = "fm_customize_product";

/// Standard event for making a donation.
pub const FM_DONATE: &str = "fm_donate";

/// Standard event for submitting an application.
pub const FM_SUBMIT_APPLICATION: &str = "fm_submit_application";

/// Standard event for an application being approved.
pub const FM_APPLICATION_APPROVAL: &str = "fm_application_approval";

/// Standard event for downloading content.
pub const FM_DOWNLOAD: &str = "fm_download";

/// Standard event for submitting a form.
pub const FM_SUBMIT_FORM: &str = "fm_submit_form";

/// Standard event for achieving a level in a game.
pub const FM_ACHIEVE_LEVEL: &str = "fm_achieve_level";

/// Standard event for unlocking an achievement.
pub const FM_UNLOCK_ACHIEVEMENT: &str = "fm_unlock_achievement";

/// Standard event for completing a tutorial.
pub const FM_COMPLETE_TUTORIAL: &str = "fm_complete_tutorial";

/// Standard event for activating the app (first launch after install).
pub const FM_ACTIVATE_APP: &str = "fm_activate_app";

/// Standard event for a user clicking an in-app ad.
pub const FM_IN_APP_AD_CLICK: &str = "fm_in_app_ad_click";

/// Standard event for an in-app ad impression.
pub const FM_IN_APP_AD_IMPRESSION: &str = "fm_in_app_ad_impression";

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
            FM_PAGE_VIEW,
            FM_VIEW_CONTENT,
            FM_SEARCH,
            FM_ADD_TO_WISHLIST,
            FM_INITIATE_CHECKOUT,
            FM_ADD_PAYMENT_INFO,
            FM_LEAD,
            FM_COMPLETE_REGISTRATION,
            FM_CONTACT,
            FM_SCHEDULE,
            FM_FIND_LOCATION,
            FM_CUSTOMIZE_PRODUCT,
            FM_DONATE,
            FM_SUBMIT_APPLICATION,
            FM_APPLICATION_APPROVAL,
            FM_DOWNLOAD,
            FM_SUBMIT_FORM,
            FM_ACHIEVE_LEVEL,
            FM_UNLOCK_ACHIEVEMENT,
            FM_COMPLETE_TUTORIAL,
            FM_ACTIVATE_APP,
            FM_IN_APP_AD_CLICK,
            FM_IN_APP_AD_IMPRESSION,
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
