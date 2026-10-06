//! PUT /_matrix/client/r0/directory/room/{room_alias}

#![allow(clippy::exhaustive_structs)]
#![allow(dead_code)]

use std::borrow::Cow;

use bytes::BufMut;
use http::method::Method;
use ruma_common::{
    OwnedRoomAliasId, OwnedRoomId,
    api::{
        EmptyBody, IncomingRequest, IncomingResponse, MatrixVersion, Metadata, OutgoingBody,
        OutgoingRequest, OutgoingResponse, SupportedVersions,
        auth_scheme::NoAuthentication,
        error::{DeserializationError, Error, IntoHttpError},
        path_builder::{StablePathSelector, VersionHistory},
    },
    serde::json_to_buf,
};
use serde::{Deserialize, Serialize};

/// A request to create a new room alias.
#[derive(Debug, Clone)]
pub struct Request {
    pub room_id: OwnedRoomId,         // body
    pub room_alias: OwnedRoomAliasId, // path
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
    type Body = RequestBody;
    type EndpointError = Error;
    type IncomingResponse = Response;

    fn try_into_http_request_inner(
        self,
        base_url: &str,
        considering: Cow<'_, SupportedVersions>,
    ) -> Result<http::Request<RequestBody>, IntoHttpError> {
        let url = Self::make_endpoint_url(considering, base_url, &[&self.room_alias], "")?;

        let request_body = RequestBody { room_id: self.room_id };

        let http_request = http::Request::builder()
            .method(Self::METHOD)
            .uri(url)
            .body(request_body)
            // this cannot fail because we don't give user-supplied data to any of the
            // builder methods
            .unwrap();

        Ok(http_request)
    }
}

impl IncomingRequest for Request {
    type EndpointError = Error;
    type OutgoingResponse = Response;

    fn try_from_http_request_inner(
        request: http::Request<&[u8]>,
        path_args: &[&str],
    ) -> Result<Self, DeserializationError> {
        let (room_alias,) = Deserialize::deserialize(serde::de::value::SeqDeserializer::<
            _,
            serde::de::value::Error,
        >::new(path_args.iter().copied()))?;

        let request_body: RequestBody = serde_json::from_slice(request.into_body())?;

        Ok(Request { room_id: request_body.room_id, room_alias })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestBody {
    room_id: OwnedRoomId,
}

impl OutgoingBody for RequestBody {
    type Error = serde_json::Error;

    fn content_type(&self) -> Option<http::HeaderValue> {
        Some(ruma_common::http_headers::APPLICATION_JSON)
    }

    fn try_into_buf<T: Default + BufMut + AsRef<[u8]>>(self) -> serde_json::Result<T> {
        json_to_buf(&self)
    }
}

/// The response to a request to create a new room alias.
#[derive(Clone, Copy, Debug)]
pub struct Response;

impl IncomingResponse for Response {
    type EndpointError = Error;

    fn try_from_http_response_inner(
        _http_response: http::Response<&[u8]>,
    ) -> Result<Self, DeserializationError> {
        Ok(Response)
    }
}

impl OutgoingResponse for Response {
    type Body = EmptyBody<false>;

    fn try_into_http_response_inner(self) -> Result<http::Response<Self::Body>, IntoHttpError> {
        Ok(http::Response::new(EmptyBody))
    }
}
