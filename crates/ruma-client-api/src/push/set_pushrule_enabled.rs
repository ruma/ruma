//! `PUT /_matrix/client/*/pushrules/global/{kind}/{ruleId}/enabled`
//!
//! This endpoint allows clients to enable or disable the specified push rule.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3pushrulesglobalkindruleidenabled

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    use crate::push::RuleKind;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushrules/global/{kind}/{rule_id}/enabled",
            1.1 => "/_matrix/client/v3/pushrules/global/{kind}/{rule_id}/enabled",
        }
    };

    /// Request type for the `set_pushrule_enabled` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The kind of rule
        #[ruma_api(path)]
        pub kind: RuleKind,

        /// The identifier for the rule.
        #[ruma_api(path)]
        pub rule_id: String,

        /// Whether the push rule is enabled or not.
        pub enabled: bool,
    }

    /// Response type for the `set_pushrule_enabled` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given rule kind, rule ID and enabled flag.
        pub fn new(kind: RuleKind, rule_id: String, enabled: bool) -> Self {
            Self { kind, rule_id, enabled }
        }

        /// Creates a new `Request` to enable the given rule.
        pub fn enable(kind: RuleKind, rule_id: String) -> Self {
            Self::new(kind, rule_id, true)
        }

        /// Creates a new `Request` to disable the given rule.
        pub fn disable(kind: RuleKind, rule_id: String) -> Self {
            Self::new(kind, rule_id, false)
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
