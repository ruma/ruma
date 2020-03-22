//! [GET /_matrix/client/r0/pushrules/](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-pushrules)

use std::collections::HashMap;

use ruma_api::ruma_api;

use super::{PushRule, RuleKind};

ruma_api! {
    metadata {
        description: "Retrieve all push rulesets for this user.",
        method: GET,
        name: "get_pushrules_all",
        path: "/_matrix/client/r0/pushrules/",
        rate_limited: false,
        requires_authentication: true,
    }

    request {}

    response {
        /// The global ruleset
        pub global: HashMap<RuleKind, Vec<PushRule>>
    }

    error: crate::Error
}
