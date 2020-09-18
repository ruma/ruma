//! [PUT /_matrix/client/r0/pushrules/{scope}/{kind}/{ruleId}/enabled](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-pushrules-scope-kind-ruleid-enabled)

use ruma_api::ruma_api;

use super::RuleKind;

ruma_api! {
    metadata: {
        description: "This endpoint allows clients to enable or disable the specified push rule.",
        method: PUT,
        name: "set_pushrule_enabled",
        path: "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id/enabled",
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

        /// Whether the push rule is enabled or not.
        pub enabled: bool,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given scope, rule kind, rule ID and enabled flag.
    pub fn new(scope: &'a str, kind: RuleKind, rule_id: &'a str, enabled: bool) -> Self {
        Self { scope, kind, rule_id, enabled }
    }

    /// Creates a new `Request` to enable the given rule.
    pub fn enable(scope: &'a str, kind: RuleKind, rule_id: &'a str) -> Self {
        Self::new(scope, kind, rule_id, true)
    }

    /// Creates a new `Request` to disable the given rule.
    pub fn disable(scope: &'a str, kind: RuleKind, rule_id: &'a str) -> Self {
        Self::new(scope, kind, rule_id, false)
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
