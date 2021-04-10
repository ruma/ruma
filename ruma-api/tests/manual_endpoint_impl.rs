//! PUT /_matrix/client/r0/directory/room/:room_alias

use std::convert::TryFrom;

use bytes::Buf;
use http::{header::CONTENT_TYPE, method::Method};
use ruma_api::{
    error::{FromHttpRequestError, FromHttpResponseError, IntoHttpError, ServerError, Void},
    AuthScheme, EndpointError, IncomingRequest, IncomingResponse, Metadata, OutgoingRequest,
};
use ruma_identifiers::{RoomAliasId, RoomId};
use ruma_serde::Outgoing;
use serde::{Deserialize, Serialize};

/// A request to create a new room alias.
#[derive(Debug)]
pub struct Request {
    pub room_id: RoomId,         // body
    pub room_alias: RoomAliasId, // path
}

impl Outgoing for Request {
    type Incoming = Self;
}

const METADATA: Metadata = Metadata {
    description: "Add an alias to a room.",
    method: Method::PUT,
    name: "create_alias",
    path: "/_matrix/client/r0/directory/room/:room_alias",
    rate_limited: false,
    authentication: AuthScheme::None,
};

impl OutgoingRequest for Request {
    type EndpointError = Void;
    type IncomingResponse = Response;

    const METADATA: Metadata = METADATA;

    fn try_into_http_request(
        self,
        base_url: &str,
        _access_token: Option<&str>,
    ) -> Result<http::Request<Vec<u8>>, IntoHttpError> {
        let url = (base_url.to_owned() + METADATA.path)
            .replace(":room_alias", &self.room_alias.to_string());

        let request_body = RequestBody { room_id: self.room_id };

        let http_request = http::Request::builder()
            .method(METADATA.method)
            .uri(url)
            .body(serde_json::to_vec(&request_body)?)
            // this cannot fail because we don't give user-supplied data to any of the
            // builder methods
            .unwrap();

        Ok(http_request)
    }
}

impl IncomingRequest for Request {
    type EndpointError = Void;
    type OutgoingResponse = Response;

    const METADATA: Metadata = METADATA;

    fn try_from_http_request(
        request: http::Request<Vec<u8>>,
    ) -> Result<Self, FromHttpRequestError> {
        let request_body: RequestBody = serde_json::from_slice(request.body().as_slice())?;
        let path_segments: Vec<&str> = request.uri().path()[1..].split('/').collect();

        Ok(Request {
            room_id: request_body.room_id,
            room_alias: {
                let decoded =
                    percent_encoding::percent_decode(path_segments[5].as_bytes()).decode_utf8()?;

                TryFrom::try_from(&*decoded)?
            },
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RequestBody {
    room_id: RoomId,
}

/// The response to a request to create a new room alias.
#[derive(Clone, Copy, Debug)]
pub struct Response;

impl Outgoing for Response {
    type Incoming = Self;
}

impl IncomingResponse for Response {
    type EndpointError = Void;

    fn try_from_http_response<T: Buf>(
        http_response: http::Response<T>,
    ) -> Result<Self, FromHttpResponseError<Void>> {
        if http_response.status().as_u16() < 400 {
            Ok(Response)
        } else {
            Err(FromHttpResponseError::Http(ServerError::Known(
                <Void as EndpointError>::try_from_response(http_response)?,
            )))
        }
    }
}

impl TryFrom<Response> for http::Response<Vec<u8>> {
    type Error = IntoHttpError;

    fn try_from(_: Response) -> Result<http::Response<Vec<u8>>, Self::Error> {
        let response = http::Response::builder()
            .header(CONTENT_TYPE, "application/json")
            .body(b"{}".to_vec())
            .unwrap();

        Ok(response)
    }
}
