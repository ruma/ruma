//! PUT /_matrix/client/r0/directory/room/:room_alias

#![allow(clippy::exhaustive_structs)]

use http::{header::CONTENT_TYPE, method::Method};
use ruma_common::{
    api::{
        error::{FromHttpRequestError, FromHttpResponseError, IntoHttpError, MatrixError},
        AuthScheme, IncomingRequest, IncomingResponse, IntoHttpBody, MatrixVersion, Metadata,
        OutgoingRequest, OutgoingResponse, SendAccessToken, TryFromHttpBody, VersionHistory,
    },
    OwnedRoomAliasId, OwnedRoomId,
};
use serde::{Deserialize, Serialize};

/// A request to create a new room alias.
#[derive(Debug, Clone)]
pub struct Request {
    pub room_id: OwnedRoomId,         // body
    pub room_alias: OwnedRoomAliasId, // path
}

const METADATA: Metadata = Metadata {
    method: Method::PUT,
    rate_limited: false,
    authentication: AuthScheme::None,
    history: VersionHistory::new(
        &["/_matrix/client/unstable/directory/room/:room_alias"],
        &[
            (MatrixVersion::V1_0, "/_matrix/client/r0/directory/room/:room_alias"),
            (MatrixVersion::V1_1, "/_matrix/client/v3/directory/room/:room_alias"),
        ],
        Some(MatrixVersion::V1_2),
        Some(MatrixVersion::V1_3),
    ),
};

impl OutgoingRequest for Request {
    type OutgoingBody = impl IntoHttpBody;
    type EndpointError = MatrixError;
    type IncomingResponse = Response;

    const METADATA: Metadata = METADATA;

    fn try_into_http_request(
        self,
        base_url: &str,
        _access_token: SendAccessToken<'_>,
        considering_versions: &'_ [MatrixVersion],
    ) -> Result<http::Request<Self::OutgoingBody>, IntoHttpError> {
        let url =
            METADATA.make_endpoint_url(considering_versions, base_url, &[&self.room_alias], "")?;

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
    type IncomingBody = impl TryFromHttpBody<FromHttpRequestError>;
    type EndpointError = MatrixError;
    type OutgoingResponse = Response;

    const METADATA: Metadata = METADATA;

    fn try_from_http_request<S>(
        request: http::Request<Self::IncomingBody>,
        path_args: &[S],
    ) -> Result<Self, FromHttpRequestError>
    where
        S: AsRef<str>,
    {
        let (room_alias,) = serde::Deserialize::deserialize(serde::de::value::SeqDeserializer::<
            _,
            serde::de::value::Error,
        >::new(
            path_args.iter().map(::std::convert::AsRef::as_ref),
        ))?;

        let request_body: RequestBody = request.into_body();
        Ok(Request { room_id: request_body.room_id, room_alias })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestBody {
    room_id: OwnedRoomId,
}

/// The response to a request to create a new room alias.
#[derive(Clone, Copy, Debug)]
pub struct Response;

#[doc(hidden)]
#[derive(Serialize)]
pub struct ResponseBody {}

impl<Error> TryFromHttpBody<Error> for ResponseBody {
    fn from_buf(_body: &[u8]) -> Result<Self, Error> {
        Ok(Self {})
    }
}

impl IncomingResponse for Response {
    type IncomingBody = impl TryFromHttpBody<FromHttpResponseError<Self::EndpointError>>;
    type EndpointError = MatrixError;

    fn try_from_http_response(
        http_response: http::Response<Self::IncomingBody>,
    ) -> Result<Self, FromHttpResponseError<MatrixError>> {
        let _body: ResponseBody = http_response.into_body();
        Ok(Response)
    }
}

impl OutgoingResponse for Response {
    type OutgoingBody = impl IntoHttpBody;

    fn try_into_http_response(self) -> Result<http::Response<Self::OutgoingBody>, IntoHttpError> {
        let response = http::Response::builder()
            .header(CONTENT_TYPE, "application/json")
            .body(ResponseBody {})
            .unwrap();

        Ok(response)
    }
}
