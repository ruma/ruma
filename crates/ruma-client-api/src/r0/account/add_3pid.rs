//! [POST /_matrix/client/r0/account/3pid/add](https://matrix.org/docs/spec/client_server/r0.6.1#post-matrix-client-r0-account-3pid-add)

use ruma_api::ruma_api;
use ruma_identifiers::{ClientSecret, SessionId};

use crate::r0::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

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
