//! [PUT /_matrix/client/r0/pushrules/{scope}/{kind}/{ruleId}/actions](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-pushrules-scope-kind-ruleid-actions)

use ruma_api::ruma_api;

use super::{Action, RuleKind};

ruma_api! {
    metadata {
        description: "This endpoint allows clients to change the actions of a push rule. This can be used to change the actions of builtin rules.",
        method: PUT,
        name: "set_pushrule_actions",
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

        /// The actions to perform for this rule
        pub actions: Vec<Action>
    }

    response {}

    error: crate::Error
}
