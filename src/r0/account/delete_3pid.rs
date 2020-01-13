//! [POST /_matrix/client/r0/account/3pid/delete](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-account-3pid-delete)

use ruma_api::ruma_api;

use super::ThirdPartyIdRemovalStatus;
use crate::r0::thirdparty::Medium;

ruma_api! {
    metadata {
        description: "Delete a 3PID from a user's account on an identity server.",
        method: POST,
        name: "delete_3pid",
        path: "/_matrix/client/r0/account/3pid/delete",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// Identity server to delete from.
        #[serde(skip_serializing_if = "Option::is_none")]
        id_server: Option<String>,
        /// Medium of the 3PID to be removed.
        medium: Medium,
        /// Third-party address being removed.
        address: String,
    }

    response {
        /// Result of unbind operation.
        id_server_unbind_result: ThirdPartyIdRemovalStatus,
    }

}
