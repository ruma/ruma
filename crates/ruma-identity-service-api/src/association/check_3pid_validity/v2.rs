//! [GET /_matrix/identity/v2/3pid/getValidated3pid](https://matrix.org/docs/spec/identity_service/r0.3.0#get-matrix-identity-v2-3pid-getvalidated3pid)

use js_int::UInt;
use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Determines if a given 3pid has been validated by a user.",
        method: GET,
        name: "check_3pid_validity",
        path: "/_matrix/identity/v2/3pid/getValidated3pid/",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The Session ID enerated by the `requestToken` call.
        #[ruma_api(query)]
        pub sid: String,

        /// The client secret passed to the `requestToken` call.
        #[ruma_api(query)]
        pub client_secret: String,

    }

    response: {
        /// The medium type of the 3pid.
        pub medium: String,

        /// The address of the 3pid being looked up.
        pub address: String,

        /// Timestamp, in milliseconds, indicating the time that the 3pid was validated.
        pub validated_at: UInt,
    }

}

impl Request {
    /// Creates a `Request` with the given Session ID and client secret.
    pub fn new(sid: String, client_secret: String) -> Self {
        Self { sid, client_secret }
    }
}

impl Response {
    /// Creates a `Response` with the given medium, address and validation timestamp.
    pub fn new(medium: String, address: String, validated_at: UInt) -> Self {
        Self { medium, address, validated_at }
    }
}
