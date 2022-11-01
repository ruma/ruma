//! `GET /_matrix/identity/*/account`
//!
//! Gets information about what user owns the access token used in the request.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/identity-service-api/#get_matrixidentityv2account

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        description: "Gets information about what user owns the access token used in the request.",
        method: POST,
        name: "get_account_information",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/identity/v2/account",
        }
    };

    #[request]
    #[derive(Default)]
    pub struct Request {}

    #[response]
    pub struct Response {
        /// The user ID which registered the token.
        pub user_id: OwnedUserId,
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
