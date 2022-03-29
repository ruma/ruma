//! `GET /_matrix/static/client/login/` (fallback, [spec])
//!
//! [spec]: https://spec.matrix.org/v1.2/client-server-api/#login-fallback

use ruma_common::{api::ruma_api, DeviceId};

ruma_api! {
    metadata: {
        description: "Get login fallback web page.",
        method: GET,
        name: "login_fallback",
        stable_path: "/_matrix/static/client/login/",
        rate_limited: false,
        authentication: None,
        added: 1.0,
    }

    #[derive(Default)]
    request: {
        /// ID of the client device.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_id: Option<&'a DeviceId>,

        /// A display name to assign to the newly-created device.
        ///
        /// Ignored if `device_id` corresponds to a known device.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub initial_device_display_name: Option<&'a str>,
    }

    response: {
        /// HTML to return to client.
        #[ruma_api(raw_body)]
        pub body: Vec<u8>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given auth type and session ID.
    pub fn new(
        device_id: Option<&'a DeviceId>,
        initial_device_display_name: Option<&'a str>,
    ) -> Self {
        Self { device_id, initial_device_display_name }
    }
}

impl Response {
    /// Creates a new `Response` with the given HTML body.
    pub fn new(body: Vec<u8>) -> Self {
        Self { body }
    }
}
