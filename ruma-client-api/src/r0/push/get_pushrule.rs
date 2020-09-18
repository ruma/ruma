//! [GET /_matrix/client/r0/pushrules/{scope}/{kind}/{ruleId}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-pushrules-scope-kind-ruleid)

use ruma_api::ruma_api;
use ruma_common::push::AnyPushRule;

use super::RuleKind;

ruma_api! {
    metadata: {
        description: "Retrieve a single specified push rule.",
        method: GET,
        name: "get_pushrule",
        path: "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The scope to fetch rules from. 'global' to specify global rules.
        #[ruma_api(path)]
        pub scope: &'a str,

        /// The kind of rule.
        #[ruma_api(path)]
        pub kind: RuleKind,

        /// The identifier for the rule.
        #[ruma_api(path)]
        pub rule_id: &'a str,
    }

    response: {
        /// The specific push rule.
        #[ruma_api(body)]
        pub rule: AnyPushRule,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given scope, rule kind and rule ID.
    pub fn new(scope: &'a str, kind: RuleKind, rule_id: &'a str) -> Self {
        Self { scope, kind, rule_id }
    }
}

impl Response {
    /// Creates a new `Response` with the given rule.
    pub fn new(rule: AnyPushRule) -> Self {
        Self { rule }
    }
}
