//! [DELETE /_matrix/client/r0/pushrules/{scope}/{kind}/{ruleId}](https://matrix.org/docs/spec/client_server/r0.6.0#delete-matrix-client-r0-pushrules-scope-kind-ruleid)

use ruma_api::ruma_api;

use super::RuleKind;

ruma_api! {
    metadata: {
        description: "This endpoint removes the push rule defined in the path.",
        method: DELETE,
        name: "delete_pushrule",
        path: "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The scope to delete from. 'global' to specify global rules.
        #[ruma_api(path)]
        pub scope: &'a str,

        /// The kind of rule
        #[ruma_api(path)]
        pub kind: RuleKind,

        /// The identifier for the rule.
        #[ruma_api(path)]
        pub rule_id: &'a str,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given scope, kind and rule ID.
    pub fn new(scope: &'a str, kind: RuleKind, rule_id: &'a str) -> Self {
        Self { scope, kind, rule_id }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
