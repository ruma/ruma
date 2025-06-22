//! `GET /_matrix/client/*/pushrules/global/{kind}/{ruleId}/enabled`
//!
//! This endpoint gets whether the specified push rule is enabled.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3pushrulesglobalkindruleidenabled

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    use crate::push::RuleKind;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushrules/global/{kind}/{rule_id}/enabled",
            1.1 => "/_matrix/client/v3/pushrules/global/{kind}/{rule_id}/enabled",
        }
    };

    /// Request type for the `get_pushrule_enabled` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The kind of rule
        #[ruma_api(path)]
        pub kind: RuleKind,

        /// The identifier for the rule.
        #[ruma_api(path)]
        pub rule_id: String,
    }

    /// Response type for the `get_pushrule_enabled` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// Whether the push rule is enabled or not.
        pub enabled: bool,
    }

    impl Request {
        /// Creates a new `Request` with the given rule kind and rule ID.
        pub fn new(kind: RuleKind, rule_id: String) -> Self {
            Self { kind, rule_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given enabled flag.
        pub fn new(enabled: bool) -> Self {
            Self { enabled }
        }
    }
}
