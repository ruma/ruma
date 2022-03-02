//! `GET /_matrix/client/*/pushrules/{scope}/{kind}/{ruleId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3pushrulesscopekindruleid

    use ruma_common::api::ruma_api;

    use crate::push::{PushRule, RuleKind};

    ruma_api! {
        metadata: {
            description: "Retrieve a single specified push rule.",
            method: GET,
            name: "get_pushrule",
            r0_path: "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id",
            stable_path: "/_matrix/client/v3/pushrules/:scope/:kind/:rule_id",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
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
            pub rule: PushRule,
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
        pub fn new(rule: PushRule) -> Self {
            Self { rule }
        }
    }
}
