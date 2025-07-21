//! `GET /_matrix/client/*/pushrules/global/{kind}/{ruleId}`
//!
//! Retrieve a single specified push rule.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3pushrulesglobalkindruleid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    use crate::push::{PushRule, RuleKind};

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushrules/global/{kind}/{rule_id}",
            1.1 => "/_matrix/client/v3/pushrules/global/{kind}/{rule_id}",
        }
    };

    /// Request type for the `get_pushrule` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The kind of rule.
        #[ruma_api(path)]
        pub kind: RuleKind,

        /// The identifier for the rule.
        #[ruma_api(path)]
        pub rule_id: String,
    }

    /// Response type for the `get_pushrule` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The specific push rule.
        #[ruma_api(body)]
        pub rule: PushRule,
    }

    impl Request {
        /// Creates a new `Request` with the given rule kind and rule ID.
        pub fn new(kind: RuleKind, rule_id: String) -> Self {
            Self { kind, rule_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given rule.
        pub fn new(rule: PushRule) -> Self {
            Self { rule }
        }
    }
}
