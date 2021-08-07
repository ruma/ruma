//! [PUT /_matrix/client/r0/notification_attribute_data/global/keywords](https://github.com/matrix-org/matrix-doc/blob/rav/proposals/notification_attributes/proposals/2785-notification-attributes.md#put-_matrixclientr0notification_attribute_dataglobalkeywords)
//!
//! //! This uses the unstable prefix in [MSC2785](https://github.com/matrix-org/matrix-doc/pull/2785)  
use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Replace the user's global keyword list",
        method: POST,
        name: "set_keywords",
        path: "/_matrix/client/unstable/org.matrix.msc2785/notification_attribute_data/global/keywords",
        rate_limited: true,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {
        /// The user's global keyword list.
        pub keywords: &'a [String],
    }

    #[derive(Default)]
    response: {}
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given keyword list.
    pub fn new(keywords: &'a [String]) -> Self {
        Self { keywords }
    }
}

impl Response {
    /// Creates a new empty `Response`.
    pub fn new() -> Self {
        Self::default()
    }
}
