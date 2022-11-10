//! `GET /_matrix/static/client/login/` ([spec])
//!
//! Get login fallback web page.
//!
//! [spec]: https://spec.matrix.org/v1.4/client-server-api/#login-fallback

use ruma_common::{
    api::{request, response, Metadata},
    metadata, DeviceId,
};

const METADATA: Metadata = metadata! {
    method: GET,
    rate_limited: false,
    authentication: None,
    history: {
        1.0 => "/_matrix/static/client/login/",
    }
};

/// Request type for the `login_fallback` endpoint.
#[request(error = crate::Error)]
#[derive(Default)]
pub struct Request<'a> {
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

/// Response type for the `login_fallback` endpoint.
#[response(error = crate::Error)]
pub struct Response {
    /// HTML to return to client.
    #[ruma_api(raw_body)]
    pub body: Vec<u8>,
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
