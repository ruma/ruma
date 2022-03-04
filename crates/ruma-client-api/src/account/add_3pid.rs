//! `POST /_matrix/client/*/account/3pid/add`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3account3pidadd

    use ruma_common::{api::ruma_api, ClientSecret, SessionId};

    use crate::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

    ruma_api! {
        metadata: {
            description: "Add contact information to a user's account",
            method: POST,
            name: "add_3pid",
            r0_path: "/_matrix/client/r0/account/3pid/add",
            stable_path: "/_matrix/client/v3/account/3pid/add",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// Additional information for the User-Interactive Authentication API.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub auth: Option<AuthData<'a>>,

            /// Client-generated secret string used to protect this session.
            pub client_secret: &'a ClientSecret,

            /// The session identifier given by the identity server.
            pub sid: &'a SessionId,
        }

        #[derive(Default)]
        response: {}

        error: UiaaResponse
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given client secret and session identifier.
        pub fn new(client_secret: &'a ClientSecret, sid: &'a SessionId) -> Self {
            Self { auth: None, client_secret, sid }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
