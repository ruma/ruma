//! `GET /_matrix/identity/*/account`
//!
//! Gets information about what user owns the access token used in the request.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/identity-service-api/#get_matrixidentityv2account

    use ruma_common::{api::ruma_api, OwnedUserId};

    ruma_api! {
        metadata: {
            description: "Gets information about what user owns the access token used in the request.",
            method: POST,
            name: "get_account_information",
            stable_path: "/_matrix/identity/v2/account",
            authentication: AccessToken,
            rate_limited: false,
            added: 1.0,
        }

        #[derive(Default)]
        request: {}

        response: {
            /// The user ID which registered the token.
            pub user_id: OwnedUserId,
        }
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given `UserId`.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id }
        }
    }
}
