//! [PUT /_matrix/client/r0/rooms/{roomId}/send/{eventType}/{txnId}](https://matrix.org/docs/spec/client_server/r0.6.1#put-matrix-client-r0-rooms-roomid-send-eventtype-txnid)

use std::convert::TryFrom;

use ruma_api::{
    error::{
        FromHttpRequestError, FromHttpResponseError, IntoHttpError, RequestDeserializationError,
        ResponseDeserializationError, ServerError,
    },
    AuthScheme, EndpointError, Metadata,
};
use ruma_events::{AnyMessageEventContent, EventContent as _};
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::Outgoing;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

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

/// Data in the response from the `send_message_event` API endpoint.
#[derive(Clone, Debug, Outgoing)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[incoming_derive(!Deserialize)]
pub struct Response {
    /// A unique identifier for the event.
    pub event_id: EventId,
}

impl Response {
    /// Creates a new `Response` with the given event id.
    pub fn new(event_id: EventId) -> Self {
        Self { event_id }
    }
}

const METADATA: Metadata = Metadata {
    description: "Send a message event to a room.",
    method: http::Method::PUT,
    name: "send_message_event",
    path: "/_matrix/client/r0/rooms/:room_id/send/:event_type/:txn_id",
    rate_limited: false,
    authentication: AuthScheme::AccessToken,
};

/// Data in the response body.
#[derive(Debug, Deserialize, Serialize)]
struct ResponseBody {
    /// A unique identifier for the event.
    event_id: EventId,
}

impl TryFrom<Response> for http::Response<Vec<u8>> {
    type Error = IntoHttpError;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        let response = http::Response::builder()
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&ResponseBody { event_id: response.event_id })?)
            .unwrap();

        Ok(response)
    }
}

impl TryFrom<http::Response<Vec<u8>>> for Response {
    type Error = FromHttpResponseError<crate::Error>;

    fn try_from(response: http::Response<Vec<u8>>) -> Result<Self, Self::Error> {
        if response.status().as_u16() < 400 {
            let response_body: ResponseBody =
                match serde_json::from_slice(response.body().as_slice()) {
                    Ok(val) => val,
                    Err(err) => return Err(ResponseDeserializationError::new(err, response).into()),
                };

            Ok(Self { event_id: response_body.event_id })
        } else {
            match <crate::Error as EndpointError>::try_from_response(response) {
                Ok(err) => Err(ServerError::Known(err).into()),
                Err(response_err) => Err(ServerError::Unknown(response_err).into()),
            }
        }
    }
}

impl<'a> ruma_api::OutgoingRequest for Request<'a> {
    type EndpointError = crate::Error;
    type IncomingResponse = Response;

    /// Metadata for the `send_message_event` endpoint.
    const METADATA: Metadata = METADATA;

    fn try_into_http_request(
        self,
        base_url: &str,
        access_token: Option<&str>,
    ) -> Result<http::Request<Vec<u8>>, IntoHttpError> {
        use http::header::{HeaderValue, AUTHORIZATION};
        use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

        let http_request = http::Request::builder()
            .method(http::Method::PUT)
            .uri(format!(
                "{}/_matrix/client/r0/rooms/{}/send/{}/{}",
                // FIXME: Once MSRV is >= 1.45.0, switch to
                // base_url.strip_suffix('/').unwrap_or(base_url),
                match base_url.as_bytes().last() {
                    Some(b'/') => &base_url[..base_url.len() - 1],
                    _ => base_url,
                },
                utf8_percent_encode(self.room_id.as_str(), NON_ALPHANUMERIC),
                utf8_percent_encode(self.content.event_type(), NON_ALPHANUMERIC),
                utf8_percent_encode(&self.txn_id, NON_ALPHANUMERIC),
            ))
            .header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!(
                    "Bearer {}",
                    access_token.ok_or(IntoHttpError::NeedsAuthentication)?
                ))?,
            )
            .body(serde_json::to_vec(&self.content)?)?;

        Ok(http_request)
    }
}

impl ruma_api::IncomingRequest for IncomingRequest {
    type EndpointError = crate::Error;
    type OutgoingResponse = Response;

    /// Metadata for the `send_message_event` endpoint.
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

        let txn_id =
            match percent_encoding::percent_decode(path_segments[7].as_bytes()).decode_utf8() {
                Ok(val) => val.into_owned(),
                Err(err) => return Err(RequestDeserializationError::new(err, request).into()),
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

            match AnyMessageEventContent::from_parts(&event_type, request_body) {
                Ok(content) => content,
                Err(err) => return Err(RequestDeserializationError::new(err, request).into()),
            }
        };

        Ok(Self { room_id, txn_id, content })
    }
}
