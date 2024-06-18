//! `GET /_matrix/client/*/user/{userId}/filter/{filterId}`
//!
//! Retrieve a previously created filter.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3useruseridfilterfilterid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedUserId,
    };

    use crate::filter::FilterDefinition;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/user/{user_id}/filter/{filter_id}",
            1.1 => "/_matrix/client/v3/user/{user_id}/filter/{filter_id}",
        }
    };

    /// Request type for the `get_filter` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user ID to download a filter for.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// The ID of the filter to download.
        #[ruma_api(path)]
        pub filter_id: String,
    }

    /// Response type for the `get_filter` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The filter definition.
        #[ruma_api(body)]
        pub filter: FilterDefinition,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID and filter ID.
        pub fn new(user_id: OwnedUserId, filter_id: String) -> Self {
            Self { user_id, filter_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given filter definition.
        pub fn new(filter: FilterDefinition) -> Self {
            Self { filter }
        }
    }

    #[cfg(all(test, any(feature = "client", feature = "server")))]
    mod tests {
        #[cfg(feature = "client")]
        #[test]
        fn deserialize_response() {
            use ruma_common::api::IncomingResponse;

            let res = super::Response::try_from_http_response(
                http::Response::builder().body(b"{}" as &[u8]).unwrap(),
            )
            .unwrap();
            assert!(res.filter.is_empty());
        }

        #[cfg(feature = "server")]
        #[test]
        fn serialize_response() {
            use ruma_common::api::OutgoingResponse;

            use crate::filter::FilterDefinition;

            let res = super::Response::new(FilterDefinition::default())
                .try_into_http_response::<Vec<u8>>()
                .unwrap();
            assert_eq!(res.body(), b"{}");
        }
    }
}
