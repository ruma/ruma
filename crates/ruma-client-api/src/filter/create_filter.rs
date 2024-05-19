//! `POST /_matrix/client/*/user/{userId}/filter`
//!
//! Create a new filter for event retrieval.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3useruseridfilter

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedUserId,
    };

    use crate::filter::FilterDefinition;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/user/{user_id}/filter",
            1.1 => "/_matrix/client/v3/user/{user_id}/filter",
        }
    };

    /// Request type for the `create_filter` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the user uploading the filter.
        ///
        /// The access token must be authorized to make requests for this user ID.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// The filter definition.
        #[ruma_api(body)]
        pub filter: FilterDefinition,
    }

    /// Response type for the `create_filter` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The ID of the filter that was created.
        pub filter_id: String,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID and filter definition.
        pub fn new(user_id: OwnedUserId, filter: FilterDefinition) -> Self {
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
        #[cfg(feature = "server")]
        #[test]
        fn deserialize_request() {
            use ruma_common::api::IncomingRequest as _;

            use super::Request;

            let req = Request::try_from_http_request(
                http::Request::builder()
                    .method(http::Method::POST)
                    .uri("https://matrix.org/_matrix/client/r0/user/@foo:bar.com/filter")
                    .body(b"{}" as &[u8])
                    .unwrap(),
                &["@foo:bar.com"],
            )
            .unwrap();

            assert_eq!(req.user_id, "@foo:bar.com");
            assert!(req.filter.is_empty());
        }

        #[cfg(feature = "client")]
        #[test]
        fn serialize_request() {
            use ruma_common::{
                api::{MatrixVersion, OutgoingRequest, SendAccessToken},
                owned_user_id,
            };

            use crate::filter::FilterDefinition;

            let req =
                super::Request::new(owned_user_id!("@foo:bar.com"), FilterDefinition::default())
                    .try_into_http_request::<Vec<u8>>(
                        "https://matrix.org",
                        SendAccessToken::IfRequired("tok"),
                        &[MatrixVersion::V1_1],
                    )
                    .unwrap();
            assert_eq!(req.body(), b"{}");
        }
    }
}
