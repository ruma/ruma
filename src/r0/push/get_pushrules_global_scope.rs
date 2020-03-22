//! [GET /_matrix/client/r0/pushrules/global/](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-pushrules)

use std::collections::HashMap;

use ruma_api::ruma_api;

use super::{PushRule, RuleKind};

ruma_api! {
    metadata {
        description: "Retrieve all push rulesets in the global scope for this user.",
        method: GET,
        name: "get_pushrules_global_scope",
        path: "/_matrix/client/r0/pushrules/global/",
        rate_limited: false,
        requires_authentication: true,
    }

    request {}

    response {
        /// The global ruleset.
        #[ruma_api(body)]
        pub global: HashMap<RuleKind, Vec<PushRule>>,
    }

    error: crate::Error
}
