//! [GET /_matrix/push/v1/notify](https://matrix.org/docs/spec/push_gateway/r0.1.1#post-matrix-push-v1-notify)

use ruma_api::ruma_api;

use super::{IncomingNotification, Notification};

ruma_api! {
    metadata: {
        description: "Notify a push gateway about an event or update the number of unread notifications a user has",
        name: "push",
        method: POST,
        path: "/_matrix/push/v1/notify",
        rate_limited: false,
        requires_authentication: false,
    }

    #[derive(Default)]
    #[non_exhaustive]
    request: {
        /// Information about the push notification.
        pub notification: Notification<'a>,
    }

    #[non_exhaustive]
    response: {
        /// A list of all pushkeys given in the notification request that are not valid.
        /// These could have been rejected by an upstream gateway because they have expired or
        /// have never been valid.
        pub rejected: Vec<String>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given auth chain.
    pub fn new(notification: Notification<'a>) -> Self {
        Self { notification }
    }
}
