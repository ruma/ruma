//! [POST /_matrix/client/r0/account/3pid](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-3pid)

use ruma_api_macros::ruma_api;
use serde_derive::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Adds contact information to the user's account.",
        method: POST,
        name: "create_contact",
        path: "/_matrix/client/r0/account/3pid",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// Whether the homeserver should also bind this third party identifier to the account's
        /// Matrix ID with the passed identity server.
        ///
        /// Default to `false` if not supplied.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bind: Option<bool>,
        /// The third party credentials to associate with the account.
        pub three_pid_creds: ThreePidCredentials,
    }

    response {}
}

/// The third party credentials to associate with the account.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ThreePidCredentials {
    /// The client secret used in the session with the identity server.
    pub client_secret: String,
    /// The identity server to use.
    pub id_server: String,
    /// The session identifier given by the identity server.
    pub sid: String,
}
