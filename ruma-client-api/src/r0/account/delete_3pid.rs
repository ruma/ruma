//! [POST /_matrix/client/r0/account/3pid/delete](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-account-3pid-delete)

use ruma_api::ruma_api;
use ruma_common::thirdparty::Medium;

use super::ThirdPartyIdRemovalStatus;

ruma_api! {
    metadata: {
        description: "Delete a 3PID from a user's account on an identity server.",
        method: POST,
        name: "delete_3pid",
        path: "/_matrix/client/r0/account/3pid/delete",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// Identity server to delete from.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id_server: Option<&'a str>,

        /// Medium of the 3PID to be removed.
        pub medium: Medium,

        /// Third-party address being removed.
        pub address: &'a str,
    }

    response: {
        /// Result of unbind operation.
        pub id_server_unbind_result: ThirdPartyIdRemovalStatus,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given medium and address.
    pub fn new(medium: Medium, address: &'a str) -> Self {
        Self { id_server: None, medium, address }
    }
}
