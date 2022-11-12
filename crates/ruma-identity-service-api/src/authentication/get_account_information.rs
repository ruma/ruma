//! `GET /_matrix/identity/*/account`
//!
//! Get information about what user owns the access token used in the request.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/identity-service-api/#get_matrixidentityv2account

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/identity/v2/account",
        }
    };

    /// Request type for the `get_account_information` endpoint.
    #[request]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `get_account_information` endpoint.
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
