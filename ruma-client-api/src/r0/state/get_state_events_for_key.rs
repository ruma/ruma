//! [GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}](https://matrix.org/docs/spec/client_server/r0.6.1#get-matrix-client-r0-rooms-roomid-state-eventtype-statekey)

use std::{borrow::Cow, convert::TryFrom};

use ruma_api::{
    error::{FromHttpRequestError, IntoHttpError, RequestDeserializationError},
    ruma_api, Metadata,
};
use ruma_events::EventType;
use ruma_identifiers::RoomId;
use ruma_serde::Outgoing;
use serde_json::value::RawValue as RawJsonValue;

ruma_api! {
    metadata: {
        description: "Get state events associated with a given key.",
        method: GET,
        name: "get_state_events_for_key",
        path: "/_matrix/client/r0/rooms/:room_id/state/:event_type/:state_key",
        rate_limited: false,
        authentication: AccessToken,
    }

    response: {
        /// The content of the state event.
        #[ruma_api(body)]
        pub content: Box<RawJsonValue>,
    }

    error: crate::Error
}

/// Data for a request to the `get_state_events_for_key` API endpoint.
///
/// Get state events associated with a given key.
#[derive(Clone, Debug, Outgoing)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[incoming_derive(!Deserialize)]
pub struct Request<'a> {
    /// The room to look up the state for.
    pub room_id: &'a RoomId,

    /// The type of state to look up.
    pub event_type: EventType,

    /// The key of the state to look up.
    pub state_key: &'a str,
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room ID, event type and state key.
    pub fn new(room_id: &'a RoomId, event_type: EventType, state_key: &'a str) -> Self {
        Self { room_id, event_type, state_key }
    }
}

impl Response {
    /// Creates a new `Response` with the given content.
    pub fn new(content: Box<RawJsonValue>) -> Self {
        Self { content }
    }
}

#[cfg(feature = "client")]
impl<'a> ruma_api::OutgoingRequest for Request<'a> {
    type EndpointError = crate::Error;
    type IncomingResponse = <Response as ruma_serde::Outgoing>::Incoming;

    const METADATA: Metadata = METADATA;

    fn try_into_http_request(
        self,
        base_url: &str,
        access_token: Option<&str>,
    ) -> Result<http::Request<Vec<u8>>, IntoHttpError> {
        use http::header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE};
        use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

        let mut url = format!(
            "{}/_matrix/client/r0/rooms/{}/state/{}",
            base_url.strip_suffix('/').unwrap_or(base_url),
            utf8_percent_encode(&self.room_id.to_string(), NON_ALPHANUMERIC,),
            utf8_percent_encode(&self.event_type.to_string(), NON_ALPHANUMERIC,)
        );

        if !self.state_key.is_empty() {
            url.push('/');
            url.push_str(&Cow::from(utf8_percent_encode(&self.state_key, NON_ALPHANUMERIC)));
        }

        http::Request::builder()
            .method(http::Method::GET)
            .uri(url)
            .header(CONTENT_TYPE, "application/json")
            .header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!(
                    "Bearer {}",
                    access_token.ok_or(IntoHttpError::NeedsAuthentication)?
                ))?,
            )
            .body(Vec::new())
            .map_err(Into::into)
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

        let event_type = {
            let decoded =
                match percent_encoding::percent_decode(path_segments[6].as_bytes()).decode_utf8() {
                    Ok(val) => val,
                    Err(err) => return Err(RequestDeserializationError::new(err, request).into()),
                };

            match EventType::try_from(&*decoded) {
                Ok(val) => val,
                Err(err) => return Err(RequestDeserializationError::new(err, request).into()),
            }
        };

        let state_key = {
            let decoded =
                match percent_encoding::percent_decode(path_segments[7].as_bytes()).decode_utf8() {
                    Ok(val) => val,
                    Err(err) => return Err(RequestDeserializationError::new(err, request).into()),
                };

            match String::try_from(&*decoded) {
                Ok(val) => val,
                Err(err) => return Err(RequestDeserializationError::new(err, request).into()),
            }
        };

        Ok(Self { room_id, event_type, state_key })
    }
}
