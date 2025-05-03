//! `POST /_matrix/client/*/join/{roomIdOrAlias}`
//!
//! Join a room using its ID or one of its aliases.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3joinroomidoralias

    use ruma_common::{
        api::{response, Metadata},
        metadata, OwnedRoomId, OwnedRoomOrAliasId, OwnedServerName,
    };

    use crate::membership::ThirdPartySigned;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/join/:room_id_or_alias",
            1.1 => "/_matrix/client/v3/join/:room_id_or_alias",
        }
    };

    /// Request type for the `join_room_by_id_or_alias` endpoint.
    #[derive(Clone, Debug)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Request {
        /// The room where the user should be invited.
        pub room_id_or_alias: OwnedRoomOrAliasId,

        /// The signature of a `m.third_party_invite` token to prove that this user owns a third
        /// party identity which has been invited to the room.
        pub third_party_signed: Option<ThirdPartySigned>,

        /// Optional reason for joining the room.
        pub reason: Option<String>,

        /// The servers to attempt to join the room through.
        ///
        /// One of the servers must be participating in the room.
        ///
        /// When serializing, this field is mapped to both `server_name` and `via`
        /// with identical values.
        ///
        /// When deserializing, the value is read from `via` if it's not missing or
        /// empty and `server_name` otherwise.
        pub via: Vec<OwnedServerName>,
    }

    /// Data in the request's query string.
    #[cfg_attr(feature = "client", derive(serde::Serialize))]
    #[cfg_attr(feature = "server", derive(serde::Deserialize))]
    struct RequestQuery {
        /// The servers to attempt to join the room through.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        via: Vec<OwnedServerName>,

        /// The servers to attempt to join the room through.
        ///
        /// Deprecated in Matrix >1.11 in favour of `via`.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        server_name: Vec<OwnedServerName>,
    }

    /// Data in the request's body.
    #[cfg_attr(feature = "client", derive(serde::Serialize))]
    #[cfg_attr(feature = "server", derive(serde::Deserialize))]
    struct RequestBody {
        /// The signature of a `m.third_party_invite` token to prove that this user owns a third
        /// party identity which has been invited to the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        third_party_signed: Option<ThirdPartySigned>,

        /// Optional reason for joining the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::OutgoingRequest for Request {
        type EndpointError = crate::Error;
        type IncomingResponse = Response;

        const METADATA: Metadata = METADATA;

        fn try_into_http_request<T: Default + bytes::BufMut>(
            self,
            base_url: &str,
            access_token: ruma_common::api::SendAccessToken<'_>,
            considering: &'_ ruma_common::api::SupportedVersions,
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            use http::header::{self, HeaderValue};

            // Only send `server_name` if the `via` parameter is not supported by the server.
            // `via` was introduced in Matrix 1.12.
            let server_name = if considering
                .versions
                .iter()
                .rev()
                .any(|version| version.is_superset_of(ruma_common::api::MatrixVersion::V1_12))
            {
                vec![]
            } else {
                self.via.clone()
            };

            let query_string =
                serde_html_form::to_string(RequestQuery { server_name, via: self.via })?;

            let http_request = http::Request::builder()
                .method(METADATA.method)
                .uri(METADATA.make_endpoint_url(
                    considering,
                    base_url,
                    &[&self.room_id_or_alias],
                    &query_string,
                )?)
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    header::AUTHORIZATION,
                    HeaderValue::from_str(&format!(
                        "Bearer {}",
                        access_token
                            .get_required_for_endpoint()
                            .ok_or(ruma_common::api::error::IntoHttpError::NeedsAuthentication)?
                    ))?,
                )
                .body(ruma_common::serde::json_to_buf(&RequestBody {
                    third_party_signed: self.third_party_signed,
                    reason: self.reason,
                })?)?;

            Ok(http_request)
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::IncomingRequest for Request {
        type EndpointError = crate::Error;
        type OutgoingResponse = Response;

        const METADATA: Metadata = METADATA;

        fn try_from_http_request<B, S>(
            request: http::Request<B>,
            path_args: &[S],
        ) -> Result<Self, ruma_common::api::error::FromHttpRequestError>
        where
            B: AsRef<[u8]>,
            S: AsRef<str>,
        {
            if request.method() != METADATA.method {
                return Err(ruma_common::api::error::FromHttpRequestError::MethodMismatch {
                    expected: METADATA.method,
                    received: request.method().clone(),
                });
            }

            let (room_id_or_alias,) =
                serde::Deserialize::deserialize(serde::de::value::SeqDeserializer::<
                    _,
                    serde::de::value::Error,
                >::new(
                    path_args.iter().map(::std::convert::AsRef::as_ref),
                ))?;

            let request_query: RequestQuery =
                serde_html_form::from_str(request.uri().query().unwrap_or(""))?;
            let via = if request_query.via.is_empty() {
                request_query.server_name
            } else {
                request_query.via
            };

            let body: RequestBody = serde_json::from_slice(request.body().as_ref())?;

            Ok(Self {
                room_id_or_alias,
                reason: body.reason,
                third_party_signed: body.third_party_signed,
                via,
            })
        }
    }

    /// Response type for the `join_room_by_id_or_alias` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The room that the user joined.
        pub room_id: OwnedRoomId,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID or alias ID.
        pub fn new(room_id_or_alias: OwnedRoomOrAliasId) -> Self {
            Self { room_id_or_alias, via: vec![], third_party_signed: None, reason: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room ID.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id }
        }
    }

    #[cfg(all(test, any(feature = "client", feature = "server")))]
    mod tests {
        use ruma_common::{
            api::{
                IncomingRequest as _, MatrixVersion, OutgoingRequest, SendAccessToken,
                SupportedVersions,
            },
            owned_room_id, owned_server_name,
        };

        use super::Request;

        #[cfg(feature = "client")]
        #[test]
        fn serialize_request_via_and_server_name() {
            let mut req = Request::new(owned_room_id!("!foo:b.ar").into());
            req.via = vec![owned_server_name!("f.oo")];
            let supported =
                SupportedVersions { versions: [MatrixVersion::V1_1].into(), features: Vec::new() };

            let req = req
                .try_into_http_request::<Vec<u8>>(
                    "https://matrix.org",
                    SendAccessToken::IfRequired("tok"),
                    &supported,
                )
                .unwrap();
            assert_eq!(req.uri().query(), Some("via=f.oo&server_name=f.oo"));
        }

        #[cfg(feature = "client")]
        #[test]
        fn serialize_request_only_via() {
            let mut req = Request::new(owned_room_id!("!foo:b.ar").into());
            req.via = vec![owned_server_name!("f.oo")];
            let supported =
                SupportedVersions { versions: [MatrixVersion::V1_13].into(), features: Vec::new() };

            let req = req
                .try_into_http_request::<Vec<u8>>(
                    "https://matrix.org",
                    SendAccessToken::IfRequired("tok"),
                    &supported,
                )
                .unwrap();
            assert_eq!(req.uri().query(), Some("via=f.oo"));
        }

        #[cfg(feature = "server")]
        #[test]
        fn deserialize_request_wrong_method() {
            Request::try_from_http_request(
                http::Request::builder()
                    .method(http::Method::GET)
                    .uri("https://matrix.org/_matrix/client/v3/join/!foo:b.ar?via=f.oo")
                    .body(b"{ \"reason\": \"Let me in already!\" }" as &[u8])
                    .unwrap(),
                &["!foo:b.ar"],
            )
            .expect_err("Should not deserialize request with illegal method");
        }

        #[cfg(feature = "server")]
        #[test]
        fn deserialize_request_only_via() {
            let req = Request::try_from_http_request(
                http::Request::builder()
                    .method(http::Method::POST)
                    .uri("https://matrix.org/_matrix/client/v3/join/!foo:b.ar?via=f.oo")
                    .body(b"{ \"reason\": \"Let me in already!\" }" as &[u8])
                    .unwrap(),
                &["!foo:b.ar"],
            )
            .unwrap();

            assert_eq!(req.room_id_or_alias, "!foo:b.ar");
            assert_eq!(req.reason, Some("Let me in already!".to_owned()));
            assert_eq!(req.via, vec![owned_server_name!("f.oo")]);
        }

        #[cfg(feature = "server")]
        #[test]
        fn deserialize_request_only_server_name() {
            let req = Request::try_from_http_request(
                http::Request::builder()
                    .method(http::Method::POST)
                    .uri("https://matrix.org/_matrix/client/v3/join/!foo:b.ar?server_name=f.oo")
                    .body(b"{ \"reason\": \"Let me in already!\" }" as &[u8])
                    .unwrap(),
                &["!foo:b.ar"],
            )
            .unwrap();

            assert_eq!(req.room_id_or_alias, "!foo:b.ar");
            assert_eq!(req.reason, Some("Let me in already!".to_owned()));
            assert_eq!(req.via, vec![owned_server_name!("f.oo")]);
        }

        #[cfg(feature = "server")]
        #[test]
        fn deserialize_request_via_and_server_name() {
            let req = Request::try_from_http_request(
                http::Request::builder()
                    .method(http::Method::POST)
                    .uri("https://matrix.org/_matrix/client/v3/join/!foo:b.ar?via=f.oo&server_name=b.ar")
                    .body(b"{ \"reason\": \"Let me in already!\" }" as &[u8])
                    .unwrap(),
                &["!foo:b.ar"],
            )
            .unwrap();

            assert_eq!(req.room_id_or_alias, "!foo:b.ar");
            assert_eq!(req.reason, Some("Let me in already!".to_owned()));
            assert_eq!(req.via, vec![owned_server_name!("f.oo")]);
        }
    }
}
