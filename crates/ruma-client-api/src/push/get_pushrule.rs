//! `GET /_matrix/client/*/pushrules/{scope}/{kind}/{ruleId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3pushrulesscopekindruleid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    use crate::push::{PushRule, RuleKind, RuleScope};

    const METADATA: Metadata = metadata! {
        description: "Retrieve a single specified push rule.",
        method: GET,
        name: "get_pushrule",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id",
            1.1 => "/_matrix/client/v3/pushrules/:scope/:kind/:rule_id",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The scope to fetch rules from.
        #[ruma_api(path)]
        pub scope: RuleScope,

        /// The kind of rule.
        #[ruma_api(path)]
        pub kind: RuleKind,

        /// The identifier for the rule.
        #[ruma_api(path)]
        pub rule_id: &'a str,
    }

    #[response(error = crate::Error)]
    pub struct Response {
        /// The specific push rule.
        #[ruma_api(body)]
        pub rule: PushRule,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given scope, rule kind and rule ID.
        pub fn new(scope: RuleScope, kind: RuleKind, rule_id: &'a str) -> Self {
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
