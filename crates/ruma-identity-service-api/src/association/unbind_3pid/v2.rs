//! [POST /_matrix/identity/v2/3pid/unbind](https://matrix.org/docs/spec/identity_service/r0.3.0#post-matrix-identity-v2-3pid-unbind)

use ruma_api::ruma_api;
use ruma_common::thirdparty::Medium;
use ruma_identifiers::user_id::UserId;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Remove an association between a session and a Matrix user ID.",
        method: POST,
        name: "unbind_3pid",
        path: "/_matrix/identity/v2/3pid/unbind",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The Session ID generated by the `requestToken` call.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sid: Option<&'a str>,

        /// The client secret passed to the `requestToken` call.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub client_secret: Option<&'a str>,

        /// The Matrix user ID to remove from the 3PIDs.
        pub mxid: &'a UserId,

        /// The 3PID to remove. Must match the 3PID used to generate the session if using `sid` and
        /// `client_secret` to authenticate this request.
        pub threepid:  &'a ThirdPartyId,
    }

    #[derive(Default)]
    response: {}
}

impl<'a> Request<'a> {
    /// Creates a `Request` with the given Session ID, client secret, Matrix user ID and 3PID.
    pub fn new(
        sid: Option<&'a str>,
        client_secret: Option<&'a str>,
        mxid: &'a UserId,
        threepid: &'a ThirdPartyId,
    ) -> Self {
        Self { sid, client_secret, mxid, threepid }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}

/// A 3PID to unbind.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThirdPartyId {
    /// A medium matching the medium of identifier to unbind.
    pub medium: Medium,

    /// The 3PID address to remove.
    pub address: String,
}
