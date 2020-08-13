//! [POST /_matrix/client/r0/account/deactivate](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-account-deactivate)

use ruma_api::ruma_api;

use crate::r0::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

use super::ThirdPartyIdRemovalStatus;

ruma_api! {
    metadata: {
        description: "Deactivate the current user's account.",
        method: POST,
        name: "deactivate",
        path: "/_matrix/client/r0/account/deactivate",
        rate_limited: true,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {
        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData<'a>>,

        /// Identity server from which to unbind the user's third party
        /// identifier.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id_server: Option<&'a str>,
    }

    response: {
        /// Result of unbind operation.
        pub id_server_unbind_result: ThirdPartyIdRemovalStatus,
    }

    error: UiaaResponse
}

impl Request<'_> {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Response {
    /// Creates a new `Response` with the given unbind result.
    pub fn new(id_server_unbind_result: ThirdPartyIdRemovalStatus) -> Self {
        Self { id_server_unbind_result }
    }
}
