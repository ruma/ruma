//! PUT /_matrix/client/r0/directory/room/:room_alias

#![allow(clippy::exhaustive_structs)]

use bytes::BufMut;
use http::{header::CONTENT_TYPE, method::Method};
use ruma_common::{
    api::{
        error::{
            FromHttpRequestError, FromHttpResponseError, IntoHttpError, MatrixError, ServerError,
        },
        AuthScheme, EndpointError, IncomingRequest, IncomingResponse, MatrixVersion, Metadata,
        OutgoingRequest, OutgoingResponse, SendAccessToken,
    },
    OwnedRoomAliasId, OwnedRoomId,
};
use serde::{Deserialize, Serialize};

/// A request to create a new room alias.
#[derive(Debug)]
pub struct Request {
    pub room_id: OwnedRoomId,         // body
    pub room_alias: OwnedRoomAliasId, // path
}

const METADATA: Metadata = Metadata {
    description: "Add an alias to a room.",
    method: Method::PUT,
    name: "create_alias",
    unstable_path: Some("/_matrix/client/unstable/directory/room/:room_alias"),
    r0_path: Some("/_matrix/client/r0/directory/room/:room_alias"),
    stable_path: Some("/_matrix/client/v3/directory/room/:room_alias"),
    rate_limited: false,
    authentication: AuthScheme::None,
    added: Some(MatrixVersion::V1_0),
    deprecated: Some(MatrixVersion::V1_1),
    removed: Some(MatrixVersion::V1_2),
};

impl OutgoingRequest for Request {
    type EndpointError = MatrixError;
    type IncomingResponse = Response;

    const METADATA: Metadata = METADATA;

    fn try_into_http_request<T: Default + BufMut>(
        self,
        base_url: &str,
        _access_token: SendAccessToken<'_>,
        considering_versions: &'_ [MatrixVersion],
    ) -> Result<http::Request<T>, IntoHttpError> {
        let url = format!(
            "{}{}",
            base_url,
            ruma_common::api::select_path(
                considering_versions,
                &METADATA,
                Some(format_args!("/_matrix/client/unstable/directory/room/{}", self.room_alias)),
                Some(format_args!("/_matrix/client/r0/directory/room/{}", self.room_alias)),
                Some(format_args!("/_matrix/client/v3/directory/room/{}", self.room_alias)),
            )?
        );

        let request_body = RequestBody { room_id: self.room_id };

        let http_request = http::Request::builder()
            .method(METADATA.method)
            .uri(url)
            .body(ruma_common::serde::json_to_buf(&request_body)?)
            // this cannot fail because we don't give user-supplied data to any of the
            // builder methods
            .unwrap();

        Ok(http_request)
    }
}

impl IncomingRequest for Request {
    type EndpointError = MatrixError;
    type OutgoingResponse = Response;

    const METADATA: Metadata = METADATA;

    fn try_from_http_request<B, S>(
        request: http::Request<B>,
        path_args: &[S],
    ) -> Result<Self, FromHttpRequestError>
    where
        B: AsRef<[u8]>,
        S: AsRef<str>,
    {
        let (room_alias,) = serde::Deserialize::deserialize(serde::de::value::SeqDeserializer::<
            _,
            serde::de::value::Error,
        >::new(
            path_args.iter().map(::std::convert::AsRef::as_ref),
        ))?;

        let request_body: RequestBody = serde_json::from_slice(request.body().as_ref())?;

        Ok(Request { room_id: request_body.room_id, room_alias })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RequestBody {
    room_id: OwnedRoomId,
}

/// The response to a request to create a new room alias.
#[derive(Clone, Copy, Debug)]
pub struct Response;

impl IncomingResponse for Response {
    type EndpointError = MatrixError;

    fn try_from_http_response<T: AsRef<[u8]>>(
        http_response: http::Response<T>,
    ) -> Result<Self, FromHttpResponseError<MatrixError>> {
        if http_response.status().as_u16() < 400 {
            Ok(Response)
        } else {
            Err(FromHttpResponseError::Server(ServerError::Known(
                <MatrixError as EndpointError>::try_from_http_response(http_response)?,
            )))
        }
    }
}

impl OutgoingResponse for Response {
    fn try_into_http_response<T: Default + BufMut>(
        self,
    ) -> Result<http::Response<T>, IntoHttpError> {
        let response = http::Response::builder()
            .header(CONTENT_TYPE, "application/json")
            .body(ruma_common::serde::slice_to_buf(b"{}"))
            .unwrap();

        Ok(response)
    }
}
