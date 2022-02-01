//! [GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}](https://matrix.org/docs/spec/client_server/r0.6.1#get-matrix-client-r0-rooms-roomid-state-eventtype-statekey)

use ruma_api::ruma_api;
use ruma_events::{AnyStateEventContent, EventType};
use ruma_identifiers::RoomId;
use ruma_serde::{Outgoing, Raw};

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

impl Response {
    /// Creates a new `Response` with the given content.
    pub fn new(content: Raw<AnyStateEventContent>) -> Self {
        Self { content }
    }
}

#[cfg(feature = "client")]
impl<'a> ruma_api::OutgoingRequest for Request<'a> {
    type EndpointError = crate::Error;
    type IncomingResponse = <Response as ruma_serde::Outgoing>::Incoming;

    const METADATA: ruma_api::Metadata = METADATA;

    fn try_into_http_request<T: Default + bytes::BufMut>(
        self,
        base_url: &str,
        access_token: ruma_api::SendAccessToken<'_>,
    ) -> Result<http::Request<T>, ruma_api::error::IntoHttpError> {
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
            .body(T::default())
            .map_err(Into::into)
    }
}

#[cfg(feature = "server")]
impl ruma_api::IncomingRequest for IncomingRequest {
    type EndpointError = crate::Error;
    type OutgoingResponse = Response;

    const METADATA: ruma_api::Metadata = METADATA;

    fn try_from_http_request<T: AsRef<[u8]>, S: AsRef<str>>(
        _request: http::Request<T>,
        path_args: &[S],
    ) -> Result<Self, ruma_api::error::FromHttpRequestError> {
        let (room_id, event_type, state_key): (Box<RoomId>, EventType, String) =
            if path_args.len() == 3 {
                serde::Deserialize::deserialize(serde::de::value::SeqDeserializer::<
                    _,
                    serde::de::value::Error,
                >::new(
                    path_args.iter().map(::std::convert::AsRef::as_ref),
                ))?
            } else {
                let (a, b) = serde::Deserialize::deserialize(serde::de::value::SeqDeserializer::<
                    _,
                    serde::de::value::Error,
                >::new(
                    path_args.iter().map(::std::convert::AsRef::as_ref),
                ))?;

                (a, b, "".into())
            };

        Ok(Self { room_id, event_type, state_key })
    }
}
