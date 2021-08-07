//! [PUT /_matrix/client/r0/notification_attribute_data/global/mentions](PUT
//! /_matrix/client/r0/notification_attribute_data/global/mentions)
//!
//! This uses the unstable prefix in [MSC2785](https://github.com/matrix-org/matrix-doc/pull/2785)

use ruma_api::ruma_api;
use ruma_events::notification_attribute_data::Mentions;

ruma_api! {
    metadata: {
        description: "Set the user's global mentions settings",
        method: POST,
        name: "set_mentions",
        path: "/_matrix/client/unstable/org.matrix.msc2785/notification_attribute_data/global/mentions",
        rate_limited: true,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {
        /// The user's global mentions settings.
        pub mentions: Mentions,
    }

    #[derive(Default)]
    response: {}
}

impl Request {
    /// Creates a `Request` with the given global mentions settings.
    pub fn new(mentions: Mentions) -> Self {
        Self { mentions }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self::default()
    }
}
