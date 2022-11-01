//! `DELETE /_matrix/client/*/pushrules/{scope}/{kind}/{ruleId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#delete_matrixclientv3pushrulesscopekindruleid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    use crate::push::{RuleKind, RuleScope};

    const METADATA: Metadata = metadata! {
        description: "This endpoint removes the push rule defined in the path.",
        method: DELETE,
        name: "delete_pushrule",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id",
            1.1 => "/_matrix/client/v3/pushrules/:scope/:kind/:rule_id",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The scope to delete from.
        #[ruma_api(path)]
        pub scope: RuleScope,

        /// The kind of rule
        #[ruma_api(path)]
        pub kind: RuleKind,

        /// The identifier for the rule.
        #[ruma_api(path)]
        pub rule_id: &'a str,
    }

    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given scope, kind and rule ID.
        pub fn new(scope: RuleScope, kind: RuleKind, rule_id: &'a str) -> Self {
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
