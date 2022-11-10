//! `POST /_matrix/client/*/account/password`
//!
//! Change the password of the current user's account.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#post_matrixclientv3accountpassword

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    use crate::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/account/password",
            1.1 => "/_matrix/client/v3/account/password",
        }
    };

    /// Request type for the `change_password` endpoint.
    #[request(error = UiaaResponse)]
    pub struct Request<'a> {
        /// The new password for the account.
        pub new_password: &'a str,

        /// True to revoke the user's other access tokens, and their associated devices if the
        /// request succeeds.
        ///
        /// Defaults to true.
        ///
        /// When false, the server can still take advantage of the soft logout method for the
        /// user's remaining devices.
        #[serde(
            default = "ruma_common::serde::default_true",
            skip_serializing_if = "ruma_common::serde::is_true"
        )]
        pub logout_devices: bool,

        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData<'a>>,
    }

    /// Response type for the `change_password` endpoint.
    #[response(error = UiaaResponse)]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given password.
        pub fn new(new_password: &'a str) -> Self {
            Self { new_password, logout_devices: true, auth: None }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
