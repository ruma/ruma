//! Types for the *m.notification_attribute_data* event.

use ruma_events_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::GlobalAccountDataEvent;

/// An event to store assignment of event notification attributes in a user's `account_data`.
pub type NotificationAttributeDataEvent =
    GlobalAccountDataEvent<NotificationAttributeDataEventContent>;

/// The payload for `NotificationAttributeDataEvent`.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.notification_attribute_data", kind = GlobalAccountData)]
pub struct NotificationAttributeDataEventContent {
    /// An array of string which form "notification keywords".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,

    /// An object containing booleans which define which events should qualify for `m.mention`
    /// attributes.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub mentions: Mentions,
}

/// An object containing booleans which define which events should qualify for `m.mention`
/// attributes.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Mentions {
    /// Display name flag.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub displayname: bool,

    /// Matrix user ID flag.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub mxid: bool,

    /// Local part of user ID flag.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub localpart: bool,

    /// "@room" notification flag.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub room_notif: bool,
}
