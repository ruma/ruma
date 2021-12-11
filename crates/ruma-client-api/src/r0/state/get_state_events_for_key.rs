//! [GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}](https://matrix.org/docs/spec/client_server/r0.6.1#get-matrix-client-r0-rooms-roomid-state-eventtype-statekey)

use ruma_api::ruma_api;
use ruma_events::{AnyStateEventContent, EventType};
use ruma_identifiers::RoomId;
use ruma_serde::{Outgoing, Raw};
use serde::Serialize;

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
        ///
        /// Since the inner type of the `Raw` does not implement `Deserialize`, you need to use
        /// `ruma_events::RawExt` to deserialize it.
        #[ruma_api(body)]
        pub content: Raw<AnyStateEventContent>,
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

#[derive(Serialize)]
#[doc(hidden)]
#[non_exhaustive]
pub struct RequestBody {}

impl Response {
    /// Creates a new `Response` with the given content.
    pub fn new(content: Raw<AnyStateEventContent>) -> Self {
        Self { content }
    }
}

#[cfg(feature = "client")]
impl<'a> ruma_api::OutgoingRequest for Request<'a> {
    type OutgoingBody = RequestBody; // impl IntoHttpBody;
    type EndpointError = crate::Error;
    type IncomingResponse = <Response as ruma_serde::Outgoing>::Incoming;

    const METADATA: ruma_api::Metadata = METADATA;

    fn try_into_http_request(
        self,
        base_url: &str,
        access_token: ruma_api::SendAccessToken<'_>,
    ) -> Result<http::Request<Self::OutgoingBody>, ruma_api::error::IntoHttpError> {
        use std::borrow::Cow;

        use http::header;
        use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

        let mut url = format!(
            "{}/_matrix/client/r0/rooms/{}/state/{}",
            base_url.strip_suffix('/').unwrap_or(base_url),
            utf8_percent_encode(self.room_id.as_str(), NON_ALPHANUMERIC),
            utf8_percent_encode(self.event_type.as_str(), NON_ALPHANUMERIC)
        );

        if !self.state_key.is_empty() {
            url.push('/');
            url.push_str(&Cow::from(utf8_percent_encode(self.state_key, NON_ALPHANUMERIC)));
        }

        http::Request::builder()
            .method(http::Method::GET)
            .uri(url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(
                header::AUTHORIZATION,
                format!(
                    "Bearer {}",
                    access_token
                        .get_required_for_endpoint()
                        .ok_or(ruma_api::error::IntoHttpError::NeedsAuthentication)?,
                ),
            )
            .body(RequestBody {})
            .map_err(Into::into)
    }
}

#[cfg(feature = "server")]
impl ruma_api::IncomingRequest for IncomingRequest {
    type IncomingBody = ruma_api::IncomingRawHttpBody; // impl FromHttpBody<FromHttpRequestError>;
    type EndpointError = crate::Error;
    type OutgoingResponse<'a> = Response;

    const METADATA: ruma_api::Metadata = METADATA;

    fn try_from_http_request(
        request: http::Request<Self::IncomingBody>,
    ) -> Result<Self, ruma_api::error::FromHttpRequestError> {
        use std::convert::TryFrom;

        let path_segments: Vec<&str> = request.uri().path()[1..].split('/').collect();

        let room_id = {
            let decoded =
                percent_encoding::percent_decode(path_segments[4].as_bytes()).decode_utf8()?;

            Box::<RoomId>::try_from(&*decoded)?
        };

        let event_type = {
            let decoded =
                percent_encoding::percent_decode(path_segments[6].as_bytes()).decode_utf8()?;

            EventType::try_from(&*decoded)?
        };

        let state_key = match path_segments.get(7) {
            Some(segment) => {
                let decoded = percent_encoding::percent_decode(segment.as_bytes()).decode_utf8()?;

                String::try_from(&*decoded)?
            }
            None => "".into(),
        };

        let _body: ruma_api::IncomingRawHttpBody = request.into_body();

        Ok(Self { room_id, event_type, state_key })
    }
}
