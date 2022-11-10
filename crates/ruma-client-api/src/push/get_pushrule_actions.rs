//! `GET /_matrix/client/*/pushrules/{scope}/{kind}/{ruleId}/actions`
//!
//! This endpoint get the actions for the specified push rule.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3pushrulesscopekindruleidactions

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        push::Action,
    };

    use crate::push::{RuleKind, RuleScope};

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id/actions",
            1.1 => "/_matrix/client/v3/pushrules/:scope/:kind/:rule_id/actions",
        }
    };

    /// Request type for the `get_pushrule_actions` endpoint.
    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The scope to fetch a rule from.
        #[ruma_api(path)]
        pub scope: RuleScope,

        /// The kind of rule
        #[ruma_api(path)]
        pub kind: RuleKind,

        /// The identifier for the rule.
        #[ruma_api(path)]
        pub rule_id: &'a str,
    }

    /// Response type for the `get_pushrule_actions` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The actions to perform for this rule.
        pub actions: Vec<Action>,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given scope, kind and rule ID.
        pub fn new(scope: RuleScope, kind: RuleKind, rule_id: &'a str) -> Self {
            Self { scope, kind, rule_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given actions.
        pub fn new(actions: Vec<Action>) -> Self {
            Self { actions }
        }
    }
}
