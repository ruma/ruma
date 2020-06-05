//! [GET /_matrix/client/r0/pushrules/{scope}/{kind}/{ruleId}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-pushrules-scope-kind-ruleid)

use ruma_api::ruma_api;

use super::{PushRule, RuleKind};

ruma_api! {
    metadata {
        description: "Retrieve a single specified push rule.",
        method: GET,
        name: "get_pushrule",
        path: "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The scope to fetch rules from. 'global' to specify global rules.
        #[ruma_api(path)]
        pub scope: String,

        /// The kind of rule
        #[ruma_api(path)]
        pub kind: RuleKind,

        /// The identifier for the rule.
        #[ruma_api(path)]
        pub rule_id: String,
    }

    response {
        /// The specific push rule.
        #[ruma_api(body)]
        pub rule: PushRule
    }

    error: crate::Error
}
