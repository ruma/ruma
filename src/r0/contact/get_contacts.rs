//! [GET /_matrix/client/r0/account/3pid](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-account-3pid)

use crate::r0::thirdparty::Medium;
use ruma_api::ruma_api;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Get a list of 3rd party contacts associated with the user's account.",
        method: GET,
        name: "get_contacts",
        path: "/_matrix/client/r0/account/3pid",
        rate_limited: false,
        requires_authentication: true,
    }

    request {}

    response {
        /// A list of third party identifiers the homeserver has associated with the user's
        /// account.
        pub threepids: Vec<ThirdPartyIdentifier>,
    }

    error: crate::Error
}

/// An identifier external to Matrix.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ThirdPartyIdentifier {
    /// The third party identifier address.
    pub address: String,
    /// The medium of third party identifier.
    pub medium: Medium,
}
