//! `PUT /_matrix/client/*/pushrules/{scope}/{kind}/{ruleId}/actions`
//!
//! This endpoint allows clients to change the actions of a push rule. This can be used to change
//! the actions of builtin rules.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3pushrulesscopekindruleidactions

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        push::Action,
    };

    use crate::push::{RuleKind, RuleScope};

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id/actions",
            1.1 => "/_matrix/client/v3/pushrules/:scope/:kind/:rule_id/actions",
        }
    };

    /// Request type for the `set_pushrule_actions` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The scope to fetch a rule from.
        #[ruma_api(path)]
        pub scope: RuleScope,

        /// The kind of rule
        #[ruma_api(path)]
        pub kind: RuleKind,

        /// The identifier for the rule.
        #[ruma_api(path)]
        pub rule_id: String,

        /// The actions to perform for this rule
        pub actions: Vec<Action>,
    }

    /// Response type for the `set_pushrule_actions` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given scope, rule kind, rule ID and actions.
        pub fn new(
            scope: RuleScope,
            kind: RuleKind,
            rule_id: String,
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
