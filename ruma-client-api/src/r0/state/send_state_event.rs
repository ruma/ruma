//! [PUT /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}](https://matrix.org/docs/spec/client_server/r0.6.1#put-matrix-client-r0-rooms-roomid-state-eventtype-statekey)

use ruma_api::ruma_api;
use ruma_events::AnyStateEventContent;
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::Outgoing;

ruma_api! {
    metadata: {
        description: "Send a state event to a room associated with a given state key.",
        method: PUT,
        name: "send_state_event",
        path: "/_matrix/client/r0/rooms/:room_id/state/:event_type/:state_key",
        rate_limited: false,
        authentication: AccessToken,
    }

    response: {
        /// A unique identifier for the event.
        pub event_id: EventId,
    }

    error: crate::Error
}

/// Data for a request to the `send_state_event` API endpoint.
///
/// Send a state event to a room associated with a given state key.
#[derive(Clone, Debug, Outgoing)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[incoming_derive(!Deserialize)]
pub struct Request<'a> {
    /// The room to set the state in.
    pub room_id: &'a RoomId,

    /// The state_key for the state to send.
    pub state_key: &'a str,

    /// The event content to send.
    pub content: &'a AnyStateEventContent,
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room id, state key and event content.
    pub fn new(room_id: &'a RoomId, state_key: &'a str, content: &'a AnyStateEventContent) -> Self {
        Self { room_id, state_key, content }
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

    const METADATA: ruma_api::Metadata = METADATA;

    fn try_into_http_request(
        self,
        base_url: &str,
        access_token: Option<&str>,
    ) -> Result<http::Request<Vec<u8>>, ruma_api::error::IntoHttpError> {
        use std::borrow::Cow;

        use http::header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE};
        use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
        use ruma_events::EventContent;

        let mut url = format!(
            "{}/_matrix/client/r0/rooms/{}/state/{}",
            base_url.strip_suffix('/').unwrap_or(base_url),
            utf8_percent_encode(self.room_id.as_str(), NON_ALPHANUMERIC),
            utf8_percent_encode(self.content.event_type(), NON_ALPHANUMERIC),
        );

        if !self.state_key.is_empty() {
            url.push('/');
            url.push_str(&Cow::from(utf8_percent_encode(&self.state_key, NON_ALPHANUMERIC)));
        }

        let http_request = http::Request::builder()
            .method(http::Method::PUT)
            .uri(url)
            .header(CONTENT_TYPE, "application/json")
            .header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!(
                    "Bearer {}",
                    access_token.ok_or(ruma_api::error::IntoHttpError::NeedsAuthentication)?
                ))?,
            )
            .body(serde_json::to_vec(&self.content)?)?;

        Ok(http_request)
    }
}

#[cfg(feature = "server")]
impl ruma_api::IncomingRequest for IncomingRequest {
    type EndpointError = crate::Error;
    type OutgoingResponse = Response;

    const METADATA: ruma_api::Metadata = METADATA;

    fn try_from_http_request(
        request: http::Request<Vec<u8>>,
    ) -> Result<Self, ruma_api::error::FromHttpRequestError> {
        use std::convert::TryFrom;

        use ruma_api::try_deserialize;
        use ruma_events::EventContent;
        use serde_json::value::RawValue as RawJsonValue;

        let path_segments: Vec<&str> = request.uri().path()[1..].split('/').collect();

        let room_id = {
            let decoded = try_deserialize!(
                request,
                percent_encoding::percent_decode(path_segments[4].as_bytes()).decode_utf8()
            );

            try_deserialize!(request, RoomId::try_from(&*decoded))
        };

        let state_key = match path_segments.get(7) {
            Some(segment) => try_deserialize!(
                request,
                percent_encoding::percent_decode(segment.as_bytes()).decode_utf8()
            )
            .into_owned(),
            None => "".into(),
        };

        let content = {
            let request_body: Box<RawJsonValue> =
                try_deserialize!(request, serde_json::from_slice(request.body().as_slice()));

            let event_type = try_deserialize!(
                request,
                percent_encoding::percent_decode(path_segments[6].as_bytes()).decode_utf8()
            );

            try_deserialize!(request, AnyStateEventContent::from_parts(&event_type, request_body))
        };

        Ok(Self { room_id, state_key, content })
    }
}
