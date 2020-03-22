//! [POST /_matrix/client/r0/account/deactivate](https://matrix.org/docs/spec/client_server/r0.6.0.html#post-matrix-client-r0-account-deactivate)

use ruma_api::ruma_api;

use super::{AuthenticationData, ThirdPartyIdRemovalStatus};

ruma_api! {
    metadata {
        description: "Deactivate the current user's account.",
        method: POST,
        name: "deactivate",
        path: "/_matrix/client/r0/account/deactivate",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthenticationData>,
        /// Identity server from which to unbind the user's third party
        /// identifier.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id_server: Option<String>,
    }

    response {
        /// Result of unbind operation.
        pub id_server_unbind_result: ThirdPartyIdRemovalStatus,
    }

    error: crate::Error
}
