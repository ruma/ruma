//! `GET /_matrix/client/*/user/{userId}/account_data/{type}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3useruseridaccount_datatype

    use ruma_common::{
        api::ruma_api, events::AnyGlobalAccountDataEventContent, serde::Raw, UserId,
    };

    ruma_api! {
        metadata: {
            description: "Gets global account data for a user.",
            name: "get_global_account_data",
            method: GET,
            r0_path: "/_matrix/client/r0/user/:user_id/account_data/:event_type",
            stable_path: "/_matrix/client/v3/user/:user_id/account_data/:event_type",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// User ID of user for whom to retrieve data.
            #[ruma_api(path)]
            pub user_id: &'a UserId,

            /// Type of data to retrieve.
            #[ruma_api(path)]
            pub event_type: &'a str,
        }

        response: {
            /// Account data content for the given type.
            ///
            /// Use [`Raw::deserialize_content`] for deserialization.
            #[ruma_api(body)]
            pub account_data: Raw<AnyGlobalAccountDataEventContent>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user ID and event type.
        pub fn new(user_id: &'a UserId, event_type: &'a str) -> Self {
            Self { user_id, event_type }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given account data.
        pub fn new(account_data: Raw<AnyGlobalAccountDataEventContent>) -> Self {
            Self { account_data }
        }
    }
}
