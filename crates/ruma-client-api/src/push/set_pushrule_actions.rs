//! `PUT /_matrix/client/*/pushrules/{scope}/{kind}/{ruleId}/actions`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#put_matrixclientv3pushrulesscopekindruleidactions

    use ruma_common::{api::ruma_api, push::Action};

    use crate::push::{RuleKind, RuleScope};

    ruma_api! {
        metadata: {
            description: "This endpoint allows clients to change the actions of a push rule. This can be used to change the actions of builtin rules.",
            method: PUT,
            name: "set_pushrule_actions",
            r0_path: "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id/actions",
            stable_path: "/_matrix/client/v3/pushrules/:scope/:kind/:rule_id/actions",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The scope to fetch a rule from.
            #[ruma_api(path)]
            pub scope: RuleScope,

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
        pub fn new(
            scope: RuleScope,
            kind: RuleKind,
            rule_id: &'a str,
            actions: Vec<Action>,
        ) -> Self {
            Self { scope, kind, rule_id, actions }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
