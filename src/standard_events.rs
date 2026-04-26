//! Standard event names for the FunnelMob SDK.
//!
//! Names mirror Meta/TikTok Standard Events verbatim so the postback layer
//! does not need to translate. `ACTIVATE_APP` is fired automatically by
//! [`crate::FunnelMob::new`] on every initialization.

/// Standard event for activating the app (every launch).
///
/// Maps to Meta `fb_mobile_activate_app` semantically; not currently
/// fanned out to Meta CAPI / TikTok Events.
pub const ACTIVATE_APP: &str = "ActivateApp";

/// Standard event for viewing a page or screen.
pub const PAGE_VIEW: &str = "PageView";

/// Standard event for viewing a content item (product, article, etc.).
pub const VIEW_CONTENT: &str = "ViewContent";

/// Standard event for performing a search.
pub const SEARCH: &str = "Search";

/// Standard event for adding an item to cart.
pub const ADD_TO_CART: &str = "AddToCart";

/// Standard event for adding an item to a wishlist.
pub const ADD_TO_WISHLIST: &str = "AddToWishlist";

/// Standard event for initiating checkout.
pub const INITIATE_CHECKOUT: &str = "InitiateCheckout";

/// Standard event for adding payment information.
pub const ADD_PAYMENT_INFO: &str = "AddPaymentInfo";

/// Standard event for completing a purchase.
pub const PURCHASE: &str = "Purchase";

/// Standard event for generating a lead.
pub const LEAD: &str = "Lead";

/// Standard event for completing registration.
pub const COMPLETE_REGISTRATION: &str = "CompleteRegistration";

/// Standard event for a contact interaction.
pub const CONTACT: &str = "Contact";

/// Standard event for scheduling an appointment.
pub const SCHEDULE: &str = "Schedule";

/// Standard event for finding a physical location.
pub const FIND_LOCATION: &str = "FindLocation";

/// Standard event for customizing a product.
pub const CUSTOMIZE_PRODUCT: &str = "CustomizeProduct";

/// Standard event for making a donation.
pub const DONATE: &str = "Donate";

/// Standard event for submitting an application.
pub const SUBMIT_APPLICATION: &str = "SubmitApplication";

/// Standard event for an application being approved.
pub const APPLICATION_APPROVAL: &str = "ApplicationApproval";

/// Standard event for downloading content.
pub const DOWNLOAD: &str = "Download";

/// Standard event for submitting a form.
pub const SUBMIT_FORM: &str = "SubmitForm";

/// Standard event for starting a free trial.
pub const START_TRIAL: &str = "StartTrial";

/// Standard event for subscribing to a service.
pub const SUBSCRIBE: &str = "Subscribe";

/// Standard event for achieving a level in a game.
pub const ACHIEVE_LEVEL: &str = "AchieveLevel";

/// Standard event for unlocking an achievement.
pub const UNLOCK_ACHIEVEMENT: &str = "UnlockAchievement";

/// Standard event for spending virtual currency.
pub const SPENT_CREDITS: &str = "SpentCredits";

/// Standard event for rating the app.
pub const RATE: &str = "Rate";

/// Standard event for completing a tutorial.
pub const COMPLETE_TUTORIAL: &str = "CompleteTutorial";

/// Standard event for a user clicking an in-app ad.
pub const IN_APP_AD_CLICK: &str = "InAppAdClick";

/// Standard event for an in-app ad impression.
pub const IN_APP_AD_IMPRESSION: &str = "InAppAdImpression";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::validate_event_name;

    #[test]
    fn test_all_standard_events_are_valid() {
        let events = [
            ACTIVATE_APP,
            PAGE_VIEW,
            VIEW_CONTENT,
            SEARCH,
            ADD_TO_CART,
            ADD_TO_WISHLIST,
            INITIATE_CHECKOUT,
            ADD_PAYMENT_INFO,
            PURCHASE,
            LEAD,
            COMPLETE_REGISTRATION,
            CONTACT,
            SCHEDULE,
            FIND_LOCATION,
            CUSTOMIZE_PRODUCT,
            DONATE,
            SUBMIT_APPLICATION,
            APPLICATION_APPROVAL,
            DOWNLOAD,
            SUBMIT_FORM,
            START_TRIAL,
            SUBSCRIBE,
            ACHIEVE_LEVEL,
            UNLOCK_ACHIEVEMENT,
            SPENT_CREDITS,
            RATE,
            COMPLETE_TUTORIAL,
            IN_APP_AD_CLICK,
            IN_APP_AD_IMPRESSION,
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
