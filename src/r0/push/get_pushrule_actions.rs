//! [GET /_matrix/client/r0/pushrules/{scope}/{kind}/{ruleId}/actions](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-pushrules-scope-kind-ruleid-actions)

use ruma_api::ruma_api;

use super::{Action, RuleKind};

ruma_api! {
    metadata {
        description: "This endpoint get the actions for the specified push rule.",
        method: GET,
        name: "get_pushrule_actions",
        path: "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id/actions",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The scope to fetch a rule from. 'global' to specify global rules.
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
        /// The actions to perform for this rule.
        pub actions: Vec<Action>
    }

    error: crate::Error
}
