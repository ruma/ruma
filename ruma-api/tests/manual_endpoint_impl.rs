//! PUT /_matrix/client/r0/directory/room/:room_alias

use std::{convert::TryFrom, ops::Deref};

use http::{header::CONTENT_TYPE, method::Method};
use ruma_identifiers::{RoomAliasId, RoomId};
use serde::{Deserialize, Serialize};

use ruma_api::{
    error::{
        FromHttpRequestError, FromHttpResponseError, IntoHttpError, RequestDeserializationError,
        ResponseDeserializationError, ServerError, Void,
    },
    AuthScheme, IncomingRequest, Metadata, Outgoing, OutgoingRequest,
};

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
}

impl TryFrom<http::Request<Vec<u8>>> for Request {
    type Error = FromHttpRequestError;

    fn try_from(request: http::Request<Vec<u8>>) -> Result<Self, Self::Error> {
        let request_body: RequestBody = match serde_json::from_slice(request.body().as_slice()) {
            Ok(body) => body,
            Err(err) => {
                return Err(RequestDeserializationError::new(err, request).into());
            }
        };
        let path_segments: Vec<&str> = request.uri().path()[1..].split('/').collect();
        Ok(Request {
            room_id: request_body.room_id,
            room_alias: {
                let segment = path_segments.get(5).unwrap().as_bytes();
                let decoded = match percent_encoding::percent_decode(segment).decode_utf8() {
                    Ok(x) => x,
                    Err(err) => return Err(RequestDeserializationError::new(err, request).into()),
                };
                match serde_json::from_str(decoded.deref()) {
                    Ok(id) => id,
                    Err(err) => return Err(RequestDeserializationError::new(err, request).into()),
                }
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

impl TryFrom<http::Response<Vec<u8>>> for Response {
    type Error = FromHttpResponseError<Void>;

    fn try_from(http_response: http::Response<Vec<u8>>) -> Result<Response, Self::Error> {
        if http_response.status().as_u16() < 400 {
            Ok(Response)
        } else {
            Err(FromHttpResponseError::Http(ServerError::Unknown(
                ResponseDeserializationError::from_response(http_response),
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
