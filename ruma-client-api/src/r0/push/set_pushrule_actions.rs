//! [PUT /_matrix/client/r0/pushrules/{scope}/{kind}/{ruleId}/actions](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-pushrules-scope-kind-ruleid-actions)

use ruma_api::ruma_api;
use ruma_common::push::Action;

use super::RuleKind;

ruma_api! {
    metadata: {
        description: "This endpoint allows clients to change the actions of a push rule. This can be used to change the actions of builtin rules.",
        method: PUT,
        name: "set_pushrule_actions",
        path: "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id/actions",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The scope to fetch a rule from. 'global' to specify global rules.
        #[ruma_api(path)]
        pub scope: &'a str,

        /// The kind of rule
        #[ruma_api(path)]
        pub kind: RuleKind,

        /// The identifier for the rule.
        #[ruma_api(path)]
        pub rule_id: &'a str,

        /// The actions to perform for this rule
        pub actions: Vec<Action>,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given scope, rule kind, rule ID and actions.
    pub fn new(scope: &'a str, kind: RuleKind, rule_id: &'a str, actions: Vec<Action>) -> Self {
        Self { scope, kind, rule_id, actions }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
