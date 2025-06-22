//! `GET /_matrix/client/*/pushrules/global/{kind}/{ruleId}/actions`
//!
//! This endpoint get the actions for the specified push rule.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3pushrulesglobalkindruleidactions

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        push::Action,
    };

    use crate::push::RuleKind;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushrules/global/{kind}/{rule_id}/actions",
            1.1 => "/_matrix/client/v3/pushrules/global/{kind}/{rule_id}/actions",
        }
    };

    /// Request type for the `get_pushrule_actions` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The kind of rule
        #[ruma_api(path)]
        pub kind: RuleKind,

        /// The identifier for the rule.
        #[ruma_api(path)]
        pub rule_id: String,
    }

    /// Response type for the `get_pushrule_actions` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The actions to perform for this rule.
        pub actions: Vec<Action>,
    }

    impl Request {
        /// Creates a new `Request` with the given kind and rule ID.
        pub fn new(kind: RuleKind, rule_id: String) -> Self {
            Self { kind, rule_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given actions.
        pub fn new(actions: Vec<Action>) -> Self {
            Self { actions }
        }
    }
}
