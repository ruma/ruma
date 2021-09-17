//! [PUT /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}](https://matrix.org/docs/spec/client_server/r0.6.1#put-matrix-client-r0-rooms-roomid-state-eventtype-statekey)

use ruma_api::ruma_api;
use ruma_events::{AnyStateEventContent, StateEventContent};
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::{Outgoing, Raw};
use serde_json::value::to_raw_value as to_raw_json_value;

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
        pub event_id: Box<EventId>,
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

    /// The type of event to send.
    pub event_type: &'a str,

    /// The state_key for the state to send.
    pub state_key: &'a str,

    /// The event content to send.
    pub body: Raw<AnyStateEventContent>,
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room id, state key and event content.
    ///
    /// # Errors
    ///
    /// Since `Request` stores the request body in serialized form, this function can fail if `T`s
    /// [`Serialize`][serde::Serialize] implementation can fail.
    pub fn new<T: StateEventContent>(
        room_id: &'a RoomId,
        state_key: &'a str,
        content: &'a T,
    ) -> serde_json::Result<Self> {
        Ok(Self {
            room_id,
            state_key,
            event_type: content.event_type(),
            body: Raw::from_json(to_raw_json_value(content)?),
        })
    }

    /// Creates a new `Request` with the given room id, event type, state key and raw event content.
    pub fn new_raw(
        room_id: &'a RoomId,
        event_type: &'a str,
        state_key: &'a str,
        body: Raw<AnyStateEventContent>,
    ) -> Self {
        Self { room_id, event_type, state_key, body }
    }
}

impl Response {
    /// Creates a new `Response` with the given event id.
    pub fn new(event_id: Box<EventId>) -> Self {
        Self { event_id }
    }
}

#[cfg(feature = "client")]
impl<'a> ruma_api::OutgoingRequest for Request<'a> {
    type EndpointError = crate::Error;
    type IncomingResponse = Response;

    const METADATA: ruma_api::Metadata = METADATA;

    fn try_into_http_request<T: Default + bytes::BufMut>(
        self,
        base_url: &str,
        access_token: ruma_api::SendAccessToken<'_>,
    ) -> Result<http::Request<T>, ruma_api::error::IntoHttpError> {
        use std::borrow::Cow;

        use http::header::{self, HeaderValue};
        use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

        let mut url = format!(
            "{}/_matrix/client/r0/rooms/{}/state/{}",
            base_url.strip_suffix('/').unwrap_or(base_url),
            utf8_percent_encode(self.room_id.as_str(), NON_ALPHANUMERIC),
            utf8_percent_encode(self.event_type, NON_ALPHANUMERIC),
        );

        // Last URL segment is optional, that is why this trait impl is not generated.
        if !self.state_key.is_empty() {
            url.push('/');
            url.push_str(&Cow::from(utf8_percent_encode(self.state_key, NON_ALPHANUMERIC)));
        }

        let http_request = http::Request::builder()
            .method(http::Method::PUT)
            .uri(url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!(
                    "Bearer {}",
                    access_token
                        .get_required_for_endpoint()
                        .ok_or(ruma_api::error::IntoHttpError::NeedsAuthentication)?
                ))?,
            )
            .body(ruma_serde::json_to_buf(&self.body)?)?;

        Ok(http_request)
    }
}

#[cfg(feature = "server")]
impl ruma_api::IncomingRequest for IncomingRequest {
    type EndpointError = crate::Error;
    type OutgoingResponse = Response;

    const METADATA: ruma_api::Metadata = METADATA;

    fn try_from_http_request<T: AsRef<[u8]>>(
        request: http::Request<T>,
    ) -> Result<Self, ruma_api::error::FromHttpRequestError> {
        use std::{borrow::Cow, convert::TryFrom};

        let path_segments: Vec<&str> = request.uri().path()[1..].split('/').collect();

        let room_id = {
            let decoded =
                percent_encoding::percent_decode(path_segments[4].as_bytes()).decode_utf8()?;

            RoomId::try_from(&*decoded)?
        };

        let event_type = percent_encoding::percent_decode(path_segments[6].as_bytes())
            .decode_utf8()?
            .into_owned();

        let state_key = path_segments
            .get(7)
            .map(|segment| percent_encoding::percent_decode(segment.as_bytes()).decode_utf8())
            .transpose()?
            // Last URL segment is optional, but not present is the same semantically as empty
            .unwrap_or(Cow::Borrowed(""))
            .into_owned();

        let body = serde_json::from_slice(request.body().as_ref())?;

        Ok(Self { room_id, event_type, state_key, body })
    }
}
