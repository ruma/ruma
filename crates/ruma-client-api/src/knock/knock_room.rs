//! `POST /_matrix/client/*/knock/{roomIdOrAlias}`
//!
//! Knock on a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3knockroomidoralias

    use ruma_common::{
        OwnedRoomId, OwnedRoomOrAliasId, OwnedServerName,
        api::{auth_scheme::AccessToken, response},
        metadata,
    };

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/xyz.amorgan.knock/knock/{room_id_or_alias}",
            1.1 => "/_matrix/client/v3/knock/{room_id_or_alias}",
        }
    }

    /// Request type for the `knock_room` endpoint.
    #[derive(Clone, Debug)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Request {
        /// The room the user should knock on.
        pub room_id_or_alias: OwnedRoomOrAliasId,

        /// The reason for joining a room.
        pub reason: Option<String>,

        /// The servers to attempt to knock on the room through.
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
        /// The servers to attempt to knock on the room through.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        via: Vec<OwnedServerName>,

        /// The servers to attempt to knock on the room through.
        ///
        /// Deprecated in Matrix >1.11 in favour of `via`.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        server_name: Vec<OwnedServerName>,
    }

    /// Data in the request's body.
    #[cfg_attr(feature = "client", derive(serde::Serialize))]
    #[cfg_attr(feature = "server", derive(serde::Deserialize))]
    struct RequestBody {
        /// The reason for joining a room.
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
                .body(ruma_common::serde::json_to_buf(&RequestBody { reason: self.reason })?)?;

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

            Ok(Self { room_id_or_alias, reason: body.reason, via })
        }
    }

    /// Response type for the `knock_room` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The room that the user knocked on.
        pub room_id: OwnedRoomId,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID or alias.
        pub fn new(room_id_or_alias: OwnedRoomOrAliasId) -> Self {
            Self { room_id_or_alias, reason: None, via: vec![] }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room ID.
        pub fn new(room_id: OwnedRoomId) -> Self {
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
            owned_room_id, owned_server_name,
        };

        use super::Request;

        #[test]
        fn serialize_request_via_and_server_name() {
            let mut req = Request::new(owned_room_id!("!foo:b.ar").into());
            req.via = vec![owned_server_name!("f.oo")];
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
            let mut req = Request::new(owned_room_id!("!foo:b.ar").into());
            req.via = vec![owned_server_name!("f.oo")];
            let supported = SupportedVersions {
                versions: [MatrixVersion::V1_12].into(),
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
        use ruma_common::{api::IncomingRequest as _, owned_server_name};

        use super::Request;

        #[test]
        fn deserialize_request_wrong_method() {
            Request::try_from_http_request(
                http::Request::builder()
                    .method(http::Method::GET)
                    .uri("https://matrix.org/_matrix/client/v3/knock/!foo:b.ar?via=f.oo")
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
                    .uri("https://matrix.org/_matrix/client/v3/knock/!foo:b.ar?via=f.oo")
                    .body(b"{ \"reason\": \"Let me in already!\" }" as &[u8])
                    .unwrap(),
                &["!foo:b.ar"],
            )
            .unwrap();

            assert_eq!(req.room_id_or_alias, "!foo:b.ar");
            assert_eq!(req.reason, Some("Let me in already!".to_owned()));
            assert_eq!(req.via, vec![owned_server_name!("f.oo")]);
        }

        #[test]
        fn deserialize_request_only_server_name() {
            let req = Request::try_from_http_request(
                http::Request::builder()
                    .method(http::Method::POST)
                    .uri("https://matrix.org/_matrix/client/v3/knock/!foo:b.ar?server_name=f.oo")
                    .body(b"{ \"reason\": \"Let me in already!\" }" as &[u8])
                    .unwrap(),
                &["!foo:b.ar"],
            )
            .unwrap();

            assert_eq!(req.room_id_or_alias, "!foo:b.ar");
            assert_eq!(req.reason, Some("Let me in already!".to_owned()));
            assert_eq!(req.via, vec![owned_server_name!("f.oo")]);
        }

        #[test]
        fn deserialize_request_via_and_server_name() {
            let req = Request::try_from_http_request(
                http::Request::builder()
                    .method(http::Method::POST)
                    .uri("https://matrix.org/_matrix/client/v3/knock/!foo:b.ar?via=f.oo&server_name=b.ar")
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
