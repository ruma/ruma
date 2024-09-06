//! `POST /_matrix/client/*/knock/{roomIdOrAlias}`
//!
//! Knock on a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3knockroomidoralias

    use ruma_common::{
        api::{response, Metadata},
        metadata, OwnedRoomId, OwnedRoomOrAliasId, OwnedServerName,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/xyz.amorgan.knock/knock/:room_id_or_alias",
            1.1 => "/_matrix/client/v3/knock/:room_id_or_alias",
        }
    };

    /// Request type for the `knock_room` endpoint.
    #[derive(Clone, Debug)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct Request {
        /// The room the user should knock on.
        // #[ruma_api(path)]
        pub room_id_or_alias: OwnedRoomOrAliasId,

        /// The reason for joining a room.
        // #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,

        /// The servers to attempt to knock on the room through.
        ///
        /// One of the servers must be participating in the room.
        // #[ruma_api(query)]
        // #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
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
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        #[deprecated = "Since Matrix 1.12, clients should use `knock::knock_room::v3::RequestQuery::via`."]
        server_name: Vec<OwnedServerName>,
    }

    /// Data in the request's body.
    #[cfg_attr(feature = "client", derive(serde::Serialize))]
    #[cfg_attr(feature = "server", derive(serde::Deserialize))]
    struct RequestBody {
        /// The reason for joining a room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::OutgoingRequest for Request {
        type EndpointError = crate::Error;
        type IncomingResponse = Response;

        const METADATA: Metadata = METADATA;

        #[allow(deprecated)]
        fn try_into_http_request<T: Default + bytes::BufMut>(
            self,
            base_url: &str,
            access_token: ruma_common::api::SendAccessToken<'_>,
            considering_versions: &'_ [ruma_common::api::MatrixVersion],
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            use http::header::{self, HeaderValue};

            let query_string = serde_html_form::to_string(RequestQuery {
                server_name: self.via.clone(),
                via: self.via,
            })?;

            let http_request = http::Request::builder()
                .method(METADATA.method)
                .uri(METADATA.make_endpoint_url(
                    considering_versions,
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
                .body(ruma_common::serde::json_to_buf(&RequestBody { reason: self.reason })?)?;

            Ok(http_request)
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::IncomingRequest for Request {
        type EndpointError = crate::Error;
        type OutgoingResponse = Response;

        const METADATA: Metadata = METADATA;

        #[allow(deprecated)]
        fn try_from_http_request<B, S>(
            request: http::Request<B>,
            path_args: &[S],
        ) -> Result<Self, ruma_common::api::error::FromHttpRequestError>
        where
            B: AsRef<[u8]>,
            S: AsRef<str>,
        {
            // let (room_id_or_alias): (OwnedRoomOrAliasId) =
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

    #[cfg(all(test, any(feature = "client", feature = "server")))]
    mod tests {
        use ruma_common::{
            api::{IncomingRequest as _, MatrixVersion, OutgoingRequest, SendAccessToken},
            owned_room_id, owned_server_name,
        };

        use super::Request;

        #[cfg(feature = "client")]
        #[test]
        fn serialize_request() {
            let mut req = Request::new(owned_room_id!("!foo:b.ar").into());
            req.via = vec![owned_server_name!("f.oo")];
            let req = req
                .try_into_http_request::<Vec<u8>>(
                    "https://matrix.org",
                    SendAccessToken::IfRequired("tok"),
                    &[MatrixVersion::V1_1],
                )
                .unwrap();
            assert_eq!(req.uri().query(), Some("via=f.oo&server_name=f.oo"));
        }

        #[cfg(feature = "server")]
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

        #[cfg(feature = "server")]
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

        #[cfg(feature = "server")]
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
