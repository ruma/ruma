//! [GET /_matrix/client/r0/notification_attribute_data/global/mentions](https://github.com/matrix-org/matrix-doc/blob/rav/proposals/notification_attributes/proposals/2785-notification-attributes.md#get-_matrixclientr0notification_attribute_dataglobalmentions)
//!
//! //! This uses the unstable prefix in [MSC2785](https://github.com/matrix-org/matrix-doc/pull/2785)
use ruma_api::ruma_api;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Get the user's current global mention settings.",
        method: GET,
        name: "get_mentions",
        path: "/_matrix/client/unstable/org.matrix.msc2785/notification_attribute_data/global/mentions",
        rate_limited: true,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {}

    #[derive(Default)]
    response: {
        /// The user's global mentions settings.
        pub mentions: Mentions,
    }
}

/// An object containing booleans which define which events should qualify for `m.mention`
/// attributes.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Mentions {
    /// Display name flag.
    pub displayname: bool,

    /// Matrix user ID flag.
    pub mxid: bool,

    /// Local part of user ID flag.
    pub localpart: bool,

    /// "@room" notification flag.
    pub room_notif: bool,
}

impl Request {
    /// Creates a new empty `Request`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Response {
    /// Creates a new `Response` with the given global mentions settings.
    pub fn new(mentions: Mentions) -> Self {
        Self { mentions }
    }
}
