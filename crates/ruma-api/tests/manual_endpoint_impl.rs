//! PUT /_matrix/client/r0/directory/room/:room_alias

// #![feature(type_alias_impl_trait)]
#![allow(clippy::exhaustive_structs)]

use std::convert::TryFrom;

use http::{header::CONTENT_TYPE, method::Method};
use ruma_api::{
    error::{FromHttpRequestError, FromHttpResponseError, IntoHttpError, MatrixError},
    AuthScheme, FromHttpBody, IncomingRequest, IncomingResponse, Metadata, OutgoingRequest,
    OutgoingResponse, SendAccessToken,
};
use ruma_identifiers::{RoomAliasId, RoomId};
use ruma_serde::Outgoing;
use serde::{Deserialize, Serialize};

/// A request to create a new room alias.
#[derive(Debug)]
pub struct Request {
    pub room_id: Box<RoomId>,         // body
    pub room_alias: Box<RoomAliasId>, // path
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
    type OutgoingBody = RequestBody;
    type EndpointError = MatrixError;
    type IncomingResponse = Response;

    const METADATA: Metadata = METADATA;

    fn try_into_http_request(
        self,
        base_url: &str,
        _access_token: SendAccessToken<'_>,
    ) -> Result<http::Request<Self::OutgoingBody>, IntoHttpError> {
        let url = (base_url.to_owned() + METADATA.path)
            .replace(":room_alias", &self.room_alias.to_string());

        let request_body = RequestBody { room_id: self.room_id };

        let http_request = http::Request::builder()
            .method(METADATA.method)
            .uri(url)
            .body(request_body)
            // this cannot fail because we don't give user-supplied data to any of the
            // builder methods
            .unwrap();

        Ok(http_request)
    }
}

impl IncomingRequest for Request {
    type IncomingBody = RequestBody; // impl FromHttpBody<FromHttpRequestError>;
    type EndpointError = MatrixError;
    type OutgoingResponse = Response;

    const METADATA: Metadata = METADATA;

    fn try_from_http_request(
        request: http::Request<Self::IncomingBody>,
    ) -> Result<Self, FromHttpRequestError> {
        let path_segments: Vec<&str> = request.uri().path()[1..].split('/').collect();
        let room_alias = {
            let decoded =
                percent_encoding::percent_decode(path_segments[5].as_bytes()).decode_utf8()?;

            TryFrom::try_from(&*decoded)?
        };

        let request_body: RequestBody = request.into_body();
        Ok(Request { room_id: request_body.room_id, room_alias })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestBody {
    room_id: Box<RoomId>,
}

/// The response to a request to create a new room alias.
#[derive(Clone, Copy, Debug)]
pub struct Response;

impl Outgoing for Response {
    type Incoming = Self;
}

#[doc(hidden)]
#[derive(Serialize)]
pub struct ResponseBody {}

impl<Error> FromHttpBody<Error> for ResponseBody {
    fn from_buf(_body: &[u8]) -> Result<Self, Error> {
        Ok(Self {})
    }
}

impl IncomingResponse for Response {
    type IncomingBody = ResponseBody; //impl FromHttpBody<FromHttpResponseError<Self::EndpointError>>;
    type EndpointError = MatrixError;

    fn try_from_http_response(
        http_response: http::Response<Self::IncomingBody>,
    ) -> Result<Self, FromHttpResponseError<MatrixError>> {
        let _body: ResponseBody = http_response.into_body();
        Ok(Response)
    }
}

impl OutgoingResponse for Response {
    type OutgoingBody = ResponseBody; // impl IntoHttpBody;

    fn try_into_http_response(self) -> Result<http::Response<Self::OutgoingBody>, IntoHttpError> {
        let response = http::Response::builder()
            .header(CONTENT_TYPE, "application/json")
            .body(ResponseBody {})
            .unwrap();

        Ok(response)
    }
}
