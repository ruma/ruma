//![GET /_matrix/client/r0/notification_attribute_data/global/keywords](https://github.com/matrix-org/matrix-doc/blob/rav/proposals/notification_attributes/proposals/2785-notification-attributes.md#get-_matrixclientr0notification_attribute_dataglobalkeywords)
//!
//! //! This uses the unstable prefix in [MSC2785](https://github.com/matrix-org/matrix-doc/pull/2785)
use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Get the user's current global keyword list",
        method: GET,
        name: "get_keywords",
        path: "/_matrix/client/unstable/org.matrix.msc2785/notification_attribute_data/global/keywords",
        rate_limited: true,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {}

    #[derive(Default)]
    response: {
        /// The user's current global keyword list.
        pub keywords: Vec<String>,
    }
}

impl Request {
    /// Creates a new empty `Request`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Response {
    /// Creates a new `Response` with the given list of keywords.
    pub fn new(keywords: Vec<String>) -> Self {
        Self { keywords }
    }
}
