//! `POST /_matrix/client/*/join/{roomIdOrAlias}`
//!
//! Join a room using its ID or one of its aliases.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3joinroomidoralias

    use ruma_common::{
        RoomId, RoomOrAliasId, ServerName,
        api::{auth_scheme::AccessToken, response},
        metadata,
    };

    use crate::membership::ThirdPartySigned;

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/join/{room_id_or_alias}",
            1.1 => "/_matrix/client/v3/join/{room_id_or_alias}",
        }
    }

    /// Request type for the `join_room_by_id_or_alias` endpoint.
    #[derive(Clone, Debug)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Request {
        /// The room where the user should be invited.
        pub room_id_or_alias: RoomOrAliasId,

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
        pub via: Vec<ServerName>,
    }

    /// Data in the request's query string.
    #[cfg_attr(feature = "client", derive(serde::Serialize))]
    #[cfg_attr(feature = "server", derive(serde::Deserialize))]
    struct RequestQuery {
        /// The servers to attempt to join the room through.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        via: Vec<ServerName>,

        /// The servers to attempt to join the room through.
        ///
        /// Deprecated in Matrix >1.11 in favour of `via`.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        server_name: Vec<ServerName>,
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

        fn try_into_http_request<T: Default + bytes::BufMut + AsRef<[u8]>>(
            self,
            base_url: &str,
            access_token: ruma_common::api::auth_scheme::SendAccessToken<'_>,
            considering: std::borrow::Cow<'_, ruma_common::api::SupportedVersions>,
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            use ruma_common::api::{Metadata, auth_scheme::AuthScheme};

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

            let mut http_request = http::Request::builder()
                .method(Self::METHOD)
                .uri(Self::make_endpoint_url(
                    considering,
                    base_url,
                    &[&self.room_id_or_alias],
                    &query_string,
                )?)
                .header(http::header::CONTENT_TYPE, ruma_common::http_headers::APPLICATION_JSON)
                .body(ruma_common::serde::json_to_buf(&RequestBody {
                    third_party_signed: self.third_party_signed,
                    reason: self.reason,
                })?)?;

            Self::Authentication::add_authentication(&mut http_request, access_token).map_err(
                |error| ruma_common::api::error::IntoHttpError::Authentication(error.into()),
            )?;

            Ok(http_request)
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::IncomingRequest for Request {
        type EndpointError = crate::Error;
        type OutgoingResponse = Response;

        fn try_from_http_request<B, S>(
            request: http::Request<B>,
            path_args: &[S],
        ) -> Result<Self, ruma_common::api::error::FromHttpRequestError>
        where
            B: AsRef<[u8]>,
            S: AsRef<str>,
        {
            Self::check_request_method(request.method())?;

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
        pub room_id: RoomId,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID or alias ID.
        pub fn new(room_id_or_alias: RoomOrAliasId) -> Self {
            Self { room_id_or_alias, via: vec![], third_party_signed: None, reason: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room ID.
        pub fn new(room_id: RoomId) -> Self {
            Self { room_id }
        }
    }

    #[cfg(all(test, feature = "client"))]
    mod tests_client {
        use std::borrow::Cow;

        use ruma_common::{
            api::{
                MatrixVersion, OutgoingRequest, SupportedVersions, auth_scheme::SendAccessToken,
            },
            room_id, server_name,
        };

        use super::Request;

        #[test]
        fn serialize_request_via_and_server_name() {
            let mut req = Request::new(room_id!("!foo:b.ar").into());
            req.via = vec![server_name!("f.oo")];
            let supported = SupportedVersions {
                versions: [MatrixVersion::V1_1].into(),
                features: Default::default(),
            };

            let req = req
                .try_into_http_request::<Vec<u8>>(
                    "https://matrix.org",
                    SendAccessToken::IfRequired("tok"),
                    Cow::Owned(supported),
                )
                .unwrap();
            assert_eq!(req.uri().query(), Some("via=f.oo&server_name=f.oo"));
        }

        #[test]
        fn serialize_request_only_via() {
            let mut req = Request::new(room_id!("!foo:b.ar").into());
            req.via = vec![server_name!("f.oo")];
            let supported = SupportedVersions {
                versions: [MatrixVersion::V1_13].into(),
                features: Default::default(),
            };

            let req = req
                .try_into_http_request::<Vec<u8>>(
                    "https://matrix.org",
                    SendAccessToken::IfRequired("tok"),
                    Cow::Owned(supported),
                )
                .unwrap();
            assert_eq!(req.uri().query(), Some("via=f.oo"));
        }
    }

    #[cfg(all(test, feature = "server"))]
    mod tests_server {
        use ruma_common::{api::IncomingRequest as _, server_name};

        use super::Request;

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
            assert_eq!(req.via, vec![server_name!("f.oo")]);
        }

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
            assert_eq!(req.via, vec![server_name!("f.oo")]);
        }

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
            assert_eq!(req.via, vec![server_name!("f.oo")]);
        }
    }
}
