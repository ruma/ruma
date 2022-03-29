//! `POST /_matrix/client/*/account/password`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3accountpassword

    use ruma_common::api::ruma_api;

    use crate::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

    ruma_api! {
        metadata: {
            description: "Change the password of the current user's account.",
            method: POST,
            name: "change_password",
            r0_path: "/_matrix/client/r0/account/password",
            stable_path: "/_matrix/client/v3/account/password",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The new password for the account.
            pub new_password: &'a str,

            /// True to revoke the user's other access tokens, and their associated devices if the
            /// request succeeds.
            ///
            /// Defaults to true.
            ///
            /// When false, the server can still take advantage of the soft logout method for the user's
            /// remaining devices.
            #[serde(default = "ruma_common::serde::default_true", skip_serializing_if = "ruma_common::serde::is_true")]
            pub logout_devices: bool,

            /// Additional authentication information for the user-interactive authentication API.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub auth: Option<AuthData<'a>>,
        }

        #[derive(Default)]
        response: {}

        error: UiaaResponse
    }

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
