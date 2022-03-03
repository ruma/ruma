//! `GET /_matrix/client/*/pushrules/{scope}/{kind}/{ruleId}/enabled`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3pushrulesscopekindruleidenabled

    use ruma_common::api::ruma_api;

    use crate::push::RuleKind;

    ruma_api! {
        metadata: {
            description: "This endpoint gets whether the specified push rule is enabled.",
            method: GET,
            name: "get_pushrule_enabled",
            r0_path: "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id/enabled",
            stable_path: "/_matrix/client/v3/pushrules/:scope/:kind/:rule_id/enabled",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
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
        }

        response: {
            /// Whether the push rule is enabled or not.
            pub enabled: bool,
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
        /// Creates a new `Response` with the given enabled flag.
        pub fn new(enabled: bool) -> Self {
            Self { enabled }
        }
    }
}
