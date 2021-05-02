//! [PUT /_matrix/client/r0/pushrules/{scope}/{kind}/{ruleId}](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-pushrules-scope-kind-ruleid)

use ruma_api::ruma_api;
use ruma_common::push::{Action, PushCondition};

use super::RuleKind;

ruma_api! {
    metadata: {
        description: "This endpoint allows the creation, modification and deletion of pushers for this user ID.",
        method: PUT,
        name: "set_pushrule",
        path: "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The scope to set the rule in. 'global' to specify global rules.
        #[ruma_api(path)]
        pub scope: &'a str,

        /// The kind of rule
        #[ruma_api(path)]
        pub kind: RuleKind,

        /// The identifier for the rule.
        #[ruma_api(path)]
        pub rule_id: &'a str,

        /// Use 'before' with a rule_id as its value to make the new rule the next-most important
        /// rule with respect to the given user defined rule.
        #[ruma_api(query)]
        pub before: Option<&'a str>,

        /// This makes the new rule the next-less important rule relative to the given user defined
        /// rule.
        #[ruma_api(query)]
        pub after: Option<&'a str>,

        /// The actions to perform when this rule is matched.
        pub actions: &'a [Action],

        /// The conditions that must hold true for an event in order for a rule to be applied to an
        /// event. A rule with no conditions always matches. Only applicable to underride and
        /// override rules, empty Vec otherwise.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub conditions: &'a [PushCondition],

        /// The glob-style pattern to match against. Only applicable to content rules.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub pattern: Option<&'a str>,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given scope, rule kind, rule ID and actions.
    pub fn new(scope: &'a str, kind: RuleKind, rule_id: &'a str, actions: &'a [Action]) -> Self {
        Self {
            scope,
            kind,
            rule_id,
            before: None,
            after: None,
            actions,
            conditions: &[],
            pattern: None,
        }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
