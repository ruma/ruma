//! `POST /_matrix/client/*/user/{userId}/filter`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3useruseridfilter

    use ruma_common::{api::ruma_api, UserId};

    use crate::filter::{FilterDefinition, IncomingFilterDefinition};

    ruma_api! {
        metadata: {
            description: "Create a new filter for event retrieval.",
            method: POST,
            name: "create_filter",
            r0_path: "/_matrix/client/r0/user/:user_id/filter",
            stable_path: "/_matrix/client/v3/user/:user_id/filter",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The ID of the user uploading the filter.
            ///
            /// The access token must be authorized to make requests for this user ID.
            #[ruma_api(path)]
            pub user_id: &'a UserId,

            /// The filter definition.
            #[ruma_api(body)]
            pub filter: FilterDefinition<'a>,
        }

        response: {
            /// The ID of the filter that was created.
            pub filter_id: String,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user ID and filter definition.
        pub fn new(user_id: &'a UserId, filter: FilterDefinition<'a>) -> Self {
            Self { user_id, filter }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given filter ID.
        pub fn new(filter_id: String) -> Self {
            Self { filter_id }
        }
    }

    #[cfg(all(test, any(feature = "client", feature = "server")))]
    mod tests {
        use assert_matches::assert_matches;

        #[cfg(feature = "server")]
        #[test]
        fn deserialize_request() {
            use ruma_common::api::IncomingRequest as _;

            use super::IncomingRequest;

            let req = assert_matches!(
                IncomingRequest::try_from_http_request(
                    http::Request::builder()
                        .method(http::Method::POST)
                        .uri("https://matrix.org/_matrix/client/r0/user/@foo:bar.com/filter")
                        .body(b"{}" as &[u8])
                        .unwrap(),
                    &["@foo:bar.com"]
                ),
                Ok(req) => req
            );

            assert_eq!(req.user_id, "@foo:bar.com");
            assert!(req.filter.is_empty());
        }

        #[cfg(feature = "client")]
        #[test]
        fn serialize_request() {
            use ruma_common::{
                api::{MatrixVersion, OutgoingRequest, SendAccessToken},
                user_id,
            };

            use crate::filter::FilterDefinition;

            assert_matches!(
                super::Request::new(user_id!("@foo:bar.com"), FilterDefinition::default())
                    .try_into_http_request::<Vec<u8>>(
                        "https://matrix.org",
                        SendAccessToken::IfRequired("tok"),
                        &[MatrixVersion::V1_1]
                    ),
                Ok(res) if res.body() == b"{}"
            );
        }
    }
}
