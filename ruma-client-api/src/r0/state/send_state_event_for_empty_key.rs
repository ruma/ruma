//! [PUT /_matrix/client/r0/rooms/{roomId}/state/{eventType}](https://matrix.org/docs/spec/client_server/r0.6.1#put-matrix-client-r0-rooms-roomid-state-eventtype)

use std::convert::TryFrom;

use ruma_api::{
    error::{FromHttpRequestError, IntoHttpError, RequestDeserializationError},
    ruma_api, Metadata,
};
use ruma_events::{AnyStateEventContent, EventContent as _};
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::Outgoing;
use serde_json::value::RawValue as RawJsonValue;

ruma_api! {
    metadata: {
        description: "Send a state event to a room associated with the empty state key.",
        method: PUT,
        name: "send_state_event_for_empty_key",
        path: "/_matrix/client/r0/rooms/:room_id/state/:event_type",
        rate_limited: false,
        authentication: AccessToken,
    }

    response: {
        /// A unique identifier for the event.
        pub event_id: EventId,
    }

    error: crate::Error
}

/// Data for a request to the `send_state_event_for_empty_key` API endpoint.
///
/// Send a state event to a room associated with the empty state key.
#[derive(Clone, Debug, Outgoing)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[incoming_derive(!Deserialize)]
pub struct Request<'a> {
    /// The room to set the state in.
    pub room_id: &'a RoomId,

    /// The event content to send.
    pub content: &'a AnyStateEventContent,
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room id and event content.
    pub fn new(room_id: &'a RoomId, content: &'a AnyStateEventContent) -> Self {
        Self { room_id, content }
    }
}

impl Response {
    /// Creates a new `Response` with the given event id.
    pub fn new(event_id: EventId) -> Self {
        Self { event_id }
    }
}

#[cfg(feature = "client")]
impl<'a> ruma_api::OutgoingRequest for Request<'a> {
    type EndpointError = crate::Error;
    type IncomingResponse = Response;

    const METADATA: Metadata = METADATA;

    fn try_into_http_request(
        self,
        base_url: &str,
        access_token: Option<&str>,
    ) -> Result<http::Request<Vec<u8>>, IntoHttpError> {
        use http::header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE};
        use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

        let http_request = http::Request::builder()
            .method(http::Method::PUT)
            .uri(format!(
                "{}/_matrix/client/r0/rooms/{}/state/{}",
                base_url.strip_suffix('/').unwrap_or(base_url),
                utf8_percent_encode(self.room_id.as_str(), NON_ALPHANUMERIC),
                utf8_percent_encode(self.content.event_type(), NON_ALPHANUMERIC),
            ))
            .header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!(
                    "Bearer {}",
                    access_token.ok_or(IntoHttpError::NeedsAuthentication)?
                ))?,
            )
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&self.content)?)?;

        Ok(http_request)
    }
}

#[cfg(feature = "server")]
impl ruma_api::IncomingRequest for IncomingRequest {
    type EndpointError = crate::Error;
    type OutgoingResponse = Response;

    const METADATA: Metadata = METADATA;

    fn try_from_http_request(
        request: http::Request<Vec<u8>>,
    ) -> Result<Self, FromHttpRequestError> {
        let path_segments: Vec<&str> = request.uri().path()[1..].split('/').collect();

        let room_id = {
            let decoded =
                match percent_encoding::percent_decode(path_segments[4].as_bytes()).decode_utf8() {
                    Ok(val) => val,
                    Err(err) => return Err(RequestDeserializationError::new(err, request).into()),
                };

            match RoomId::try_from(&*decoded) {
                Ok(val) => val,
                Err(err) => return Err(RequestDeserializationError::new(err, request).into()),
            }
        };

        let content = {
            let request_body: Box<RawJsonValue> =
                match serde_json::from_slice(request.body().as_slice()) {
                    Ok(val) => val,
                    Err(err) => return Err(RequestDeserializationError::new(err, request).into()),
                };

            let event_type = {
                match percent_encoding::percent_decode(path_segments[6].as_bytes()).decode_utf8() {
                    Ok(val) => val,
                    Err(err) => return Err(RequestDeserializationError::new(err, request).into()),
                }
            };

            match AnyStateEventContent::from_parts(&event_type, request_body) {
                Ok(content) => content,
                Err(err) => return Err(RequestDeserializationError::new(err, request).into()),
            }
        };

        Ok(Self { room_id, content })
    }
}
