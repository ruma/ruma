//! [POST /_matrix/client/r0/account/password](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-password)

use ruma_api_macros::ruma_api;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Change the password of the current user's account.",
        method: POST,
        name: "change_password",
        path: "/_matrix/client/r0/account/password",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The new password for the account.
        pub new_password: String,
        // TODO: missing `auth` field
    }

    response {}
}
