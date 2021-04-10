//! [PUT /_matrix/client/r0/rooms/{roomId}/send/{eventType}/{txnId}](https://matrix.org/docs/spec/client_server/r0.6.1#put-matrix-client-r0-rooms-roomid-send-eventtype-txnid)

use ruma_api::ruma_api;
use ruma_events::AnyMessageEventContent;
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::Outgoing;

ruma_api! {
    metadata: {
        description: "Send a message event to a room.",
        method: PUT,
        name: "create_message_event",
        path: "/_matrix/client/r0/rooms/:room_id/send/:event_type/:txn_id",
        rate_limited: false,
        authentication: AccessToken,
    }

    response: {
        /// A unique identifier for the event.
        pub event_id: EventId,
    }

    error: crate::Error
}

/// Data for a request to the `send_message_event` API endpoint.
///
/// Send a message event to a room.
#[derive(Clone, Debug, Outgoing)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[incoming_derive(!Deserialize)]
pub struct Request<'a> {
    /// The room to send the event to.
    pub room_id: &'a RoomId,

    /// The transaction ID for this event.
    ///
    /// Clients should generate an ID unique across requests with the
    /// same access token; it will be used by the server to ensure
    /// idempotency of requests.
    pub txn_id: &'a str,

    /// The event content to send.
    pub content: &'a AnyMessageEventContent,
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room id, transaction id and event content.
    pub fn new(room_id: &'a RoomId, txn_id: &'a str, content: &'a AnyMessageEventContent) -> Self {
        Self { room_id, txn_id, content }
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
        use http::header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE};
        use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
        use ruma_events::EventContent;

        let http_request = http::Request::builder()
            .method(http::Method::PUT)
            .uri(format!(
                "{}/_matrix/client/r0/rooms/{}/send/{}/{}",
                base_url.strip_suffix('/').unwrap_or(base_url),
                utf8_percent_encode(self.room_id.as_str(), NON_ALPHANUMERIC),
                utf8_percent_encode(self.content.event_type(), NON_ALPHANUMERIC),
                utf8_percent_encode(&self.txn_id, NON_ALPHANUMERIC),
            ))
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

        use ruma_events::EventContent as _;
        use serde_json::value::RawValue as RawJsonValue;

        let path_segments: Vec<&str> = request.uri().path()[1..].split('/').collect();

        let room_id = {
            let decoded =
                percent_encoding::percent_decode(path_segments[4].as_bytes()).decode_utf8()?;

            RoomId::try_from(&*decoded)?
        };

        let txn_id = percent_encoding::percent_decode(path_segments[7].as_bytes())
            .decode_utf8()?
            .into_owned();

        let content = {
            let request_body: Box<RawJsonValue> =
                serde_json::from_slice(request.body().as_slice())?;

            let event_type =
                percent_encoding::percent_decode(path_segments[6].as_bytes()).decode_utf8()?;

            AnyMessageEventContent::from_parts(&event_type, request_body)?
        };

        Ok(Self { room_id, txn_id, content })
    }
}
