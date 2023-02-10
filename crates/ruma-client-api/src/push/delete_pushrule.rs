//! `DELETE /_matrix/client/*/pushrules/{scope}/{kind}/{ruleId}`
//!
//! This endpoint removes the push rule defined in the path.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#delete_matrixclientv3pushrulesscopekindruleid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    use crate::push::{RuleKind, RuleScope};

    const METADATA: Metadata = metadata! {
        method: DELETE,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id",
            1.1 => "/_matrix/client/v3/pushrules/:scope/:kind/:rule_id",
        }
    };

    /// Request type for the `delete_pushrule` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The scope to delete from.
        #[ruma_api(path)]
        pub scope: RuleScope,

        /// The kind of rule
        #[ruma_api(path)]
        pub kind: RuleKind,

        /// The identifier for the rule.
        #[ruma_api(path)]
        pub rule_id: String,
    }

    /// Response type for the `delete_pushrule` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given scope, kind and rule ID.
        pub fn new(scope: RuleScope, kind: RuleKind, rule_id: String) -> Self {
            Self { scope, kind, rule_id }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
