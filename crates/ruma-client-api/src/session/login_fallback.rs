//! `GET /_matrix/static/client/login/` ([spec])
//!
//! Get login fallback web page.
//!
//! [spec]: https://spec.matrix.org/latest/client-server-api/#login-fallback

use ruma_common::{
    api::{request, Metadata},
    metadata, OwnedDeviceId,
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
pub struct Request {
    /// ID of the client device.
    #[ruma_api(query)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<OwnedDeviceId>,

    /// A display name to assign to the newly-created device.
    ///
    /// Ignored if `device_id` corresponds to a known device.
    #[ruma_api(query)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_device_display_name: Option<String>,
}

impl Request {
    /// Creates a new `Request` with the given auth type and session ID.
    pub fn new(
        device_id: Option<OwnedDeviceId>,
        initial_device_display_name: Option<String>,
    ) -> Self {
        Self { device_id, initial_device_display_name }
    }
}

/// Response type for the `login_fallback` endpoint.
#[derive(Debug, Clone)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Response {
    /// HTML to return to client.
    pub body: Vec<u8>,
}

impl Response {
    /// Creates a new `Response` with the given HTML body.
    pub fn new(body: Vec<u8>) -> Self {
        Self { body }
    }
}

#[cfg(feature = "server")]
impl ruma_common::api::OutgoingResponse for Response {
    fn try_into_http_response<T: Default + bytes::BufMut>(
        self,
    ) -> Result<http::Response<T>, ruma_common::api::error::IntoHttpError> {
        Ok(http::Response::builder()
            .status(http::StatusCode::OK)
            .header(http::header::CONTENT_TYPE, "text/html")
            .body(ruma_common::serde::slice_to_buf(&self.body))?)
    }
}

#[cfg(feature = "client")]
impl ruma_common::api::IncomingResponse for Response {
    type EndpointError = crate::Error;

    fn try_from_http_response<T: AsRef<[u8]>>(
        response: http::Response<T>,
    ) -> Result<Self, ruma_common::api::error::FromHttpResponseError<Self::EndpointError>> {
        use ruma_common::api::{error::FromHttpResponseError, EndpointError};

        if response.status().as_u16() >= 400 {
            return Err(FromHttpResponseError::Server(Self::EndpointError::from_http_response(
                response,
            )));
        }

        let body = response.into_body().as_ref().to_owned();
        Ok(Self { body })
    }
}
