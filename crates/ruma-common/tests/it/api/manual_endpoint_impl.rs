//! PUT /_matrix/client/r0/directory/room/{room_alias}

#![allow(clippy::exhaustive_structs)]
#![allow(dead_code)]

use std::borrow::Cow;

use bytes::BufMut;
use http::{header::CONTENT_TYPE, method::Method};
use ruma_common::{
    RoomAliasId, RoomId,
    api::{
        EndpointError, IncomingRequest, IncomingResponse, MatrixVersion, Metadata, OutgoingRequest,
        OutgoingResponse, SupportedVersions,
        auth_scheme::{NoAuthentication, SendAccessToken},
        error::{FromHttpRequestError, FromHttpResponseError, IntoHttpError, MatrixError},
        path_builder::{StablePathSelector, VersionHistory},
    },
};
use serde::{Deserialize, Serialize};

/// A request to create a new room alias.
#[derive(Debug, Clone)]
pub struct Request {
    pub room_id: RoomId,         // body
    pub room_alias: RoomAliasId, // path
}

impl Metadata for Request {
    const METHOD: Method = Method::PUT;
    const RATE_LIMITED: bool = false;
    type Authentication = NoAuthentication;
    type PathBuilder = VersionHistory;
    const PATH_BUILDER: VersionHistory = VersionHistory::new(
        &[
            (None, "/_matrix/client/unstable/directory/room/{room_alias}"),
            (
                Some("org.bar.directory"),
                "/_matrix/client/unstable/org.bar.directory/room/{room_alias}",
            ),
        ],
        &[
            (
                StablePathSelector::FeatureAndVersion {
                    feature: "org.bar.directory.stable",
                    version: MatrixVersion::V1_0,
                },
                "/_matrix/client/r0/directory/room/{room_alias}",
            ),
            (
                StablePathSelector::Version(MatrixVersion::V1_1),
                "/_matrix/client/v3/directory/room/{room_alias}",
            ),
        ],
        Some(MatrixVersion::V1_2),
        Some(MatrixVersion::V1_3),
    );
}

impl OutgoingRequest for Request {
    type EndpointError = MatrixError;
    type IncomingResponse = Response;

    fn try_into_http_request<T: Default + BufMut>(
        self,
        base_url: &str,
        _access_token: SendAccessToken<'_>,
        considering: Cow<'_, SupportedVersions>,
    ) -> Result<http::Request<T>, IntoHttpError> {
        let url = Self::make_endpoint_url(considering, base_url, &[&self.room_alias], "")?;

        let request_body = RequestBody { room_id: self.room_id };

        let http_request = http::Request::builder()
            .method(Self::METHOD)
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

    fn try_from_http_request<B, S>(
        request: http::Request<B>,
        path_args: &[S],
    ) -> Result<Self, FromHttpRequestError>
    where
        B: AsRef<[u8]>,
        S: AsRef<str>,
    {
        Self::check_request_method(request.method())?;

        let (room_alias,) = Deserialize::deserialize(serde::de::value::SeqDeserializer::<
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
    room_id: RoomId,
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
            Err(FromHttpResponseError::Server(MatrixError::from_http_response(http_response)))
        }
    }
}

impl OutgoingResponse for Response {
    fn try_into_http_response<T: Default + BufMut>(
        self,
    ) -> Result<http::Response<T>, IntoHttpError> {
        let response = http::Response::builder()
            .header(CONTENT_TYPE, ruma_common::http_headers::APPLICATION_JSON)
            .body(ruma_common::serde::slice_to_buf(b"{}"))
            .unwrap();

        Ok(response)
    }
}
