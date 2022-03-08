//! `GET /_matrix/client/*/publicRooms`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3publicrooms

    use js_int::UInt;
    use ruma_common::{api::ruma_api, directory::PublicRoomsChunk, ServerName};

    ruma_api! {
        metadata: {
            description: "Get the list of rooms in this homeserver's public directory.",
            method: GET,
            name: "get_public_rooms",
            r0_path: "/_matrix/client/r0/publicRooms",
            stable_path: "/_matrix/client/v3/publicRooms",
            rate_limited: false,
            authentication: None,
            added: 1.0,
        }

        #[derive(Default)]
        request: {
            /// Limit for the number of results to return.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub limit: Option<UInt>,

            /// Pagination token from a previous request.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub since: Option<&'a str>,

            /// The server to fetch the public room lists from.
            ///
            /// `None` means the server this request is sent to.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub server: Option<&'a ServerName>,
        }

        response: {
            /// A paginated chunk of public rooms.
            pub chunk: Vec<PublicRoomsChunk>,

            /// A pagination token for the response.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub next_batch: Option<String>,

            /// A pagination token that allows fetching previous results.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub prev_batch: Option<String>,

            /// An estimate on the total number of public rooms, if the server has an estimate.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub total_room_count_estimate: Option<UInt>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room list chunk.
        pub fn new(chunk: Vec<PublicRoomsChunk>) -> Self {
            Self { chunk, next_batch: None, prev_batch: None, total_room_count_estimate: None }
        }
    }

    #[cfg(all(test, any(feature = "client", feature = "server")))]
    mod tests {
        use js_int::uint;

        #[cfg(feature = "client")]
        #[test]
        fn construct_request_from_refs() {
            use ruma_common::{
                api::{MatrixVersion, OutgoingRequest as _, SendAccessToken},
                server_name,
            };

            let req = super::Request {
                limit: Some(uint!(10)),
                since: Some("hello"),
                server: Some(server_name!("test.tld")),
            }
            .try_into_http_request::<Vec<u8>>(
                "https://homeserver.tld",
                SendAccessToken::IfRequired("auth_tok"),
                &[MatrixVersion::V1_1],
            )
            .unwrap();

            let uri = req.uri();
            let query = uri.query().unwrap();

            assert_eq!(uri.path(), "/_matrix/client/v3/publicRooms");
            assert!(query.contains("since=hello"));
            assert!(query.contains("limit=10"));
            assert!(query.contains("server=test.tld"));
        }

        #[cfg(feature = "server")]
        #[test]
        fn construct_response_from_refs() {
            use ruma_common::api::OutgoingResponse as _;

            let res = super::Response {
                chunk: vec![],
                next_batch: Some("next_batch_token".into()),
                prev_batch: Some("prev_batch_token".into()),
                total_room_count_estimate: Some(uint!(10)),
            }
            .try_into_http_response::<Vec<u8>>()
            .unwrap();

            assert_eq!(
                String::from_utf8_lossy(res.body()),
                r#"{"chunk":[],"next_batch":"next_batch_token","prev_batch":"prev_batch_token","total_room_count_estimate":10}"#
            );
        }
    }
}
