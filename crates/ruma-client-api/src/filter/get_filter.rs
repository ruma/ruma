//! `GET /_matrix/client/*/user/{userId}/filter/{filterId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3useruseridfilterfilterid

    use ruma_common::{api::ruma_api, UserId};

    use crate::filter::IncomingFilterDefinition;

    ruma_api! {
        metadata: {
            description: "Retrieve a previously created filter.",
            method: GET,
            name: "get_filter",
            r0_path: "/_matrix/client/r0/user/:user_id/filter/:filter_id",
            stable_path: "/_matrix/client/v3/user/:user_id/filter/:filter_id",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The user ID to download a filter for.
            #[ruma_api(path)]
            pub user_id: &'a UserId,

            /// The ID of the filter to download.
            #[ruma_api(path)]
            pub filter_id: &'a str,
        }

        response: {
            /// The filter definition.
            #[ruma_api(body)]
            pub filter: IncomingFilterDefinition,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user ID and filter ID.
        pub fn new(user_id: &'a UserId, filter_id: &'a str) -> Self {
            Self { user_id, filter_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given filter definition.
        pub fn new(filter: IncomingFilterDefinition) -> Self {
            Self { filter }
        }
    }

    #[cfg(all(test, any(feature = "client", feature = "server")))]
    mod tests {
        use assert_matches::assert_matches;

        #[cfg(feature = "client")]
        #[test]
        fn deserialize_response() {
            use ruma_common::api::IncomingResponse;

            assert_matches!(
                super::Response::try_from_http_response(
                    http::Response::builder().body(b"{}" as &[u8]).unwrap(),
                ),
                Ok(super::Response { filter }) if filter.is_empty()
            );
        }

        #[cfg(feature = "server")]
        #[test]
        fn serialize_response() {
            use ruma_common::api::OutgoingResponse;

            use crate::filter::IncomingFilterDefinition;

            assert_matches!(
                super::Response::new(IncomingFilterDefinition::default())
                    .try_into_http_response::<Vec<u8>>(),
                Ok(res) if res.body() == b"{}"
            );
        }
    }
}
