//! [GET /_matrix/client/r0/pushrules/{scope}/{kind}/{ruleId}/enabled](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-pushrules-scope-kind-ruleid-enabled)

use ruma_api::ruma_api;

use super::RuleKind;

ruma_api! {
    metadata {
        description: "This endpoint gets whether the specified push rule is enabled.",
        method: GET,
        name: "get_pushrule_enabled",
        path: "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id/enabled",
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
        /// Whether the push rule is enabled or not.
        pub enabled: bool
    }

    error: crate::Error
}
