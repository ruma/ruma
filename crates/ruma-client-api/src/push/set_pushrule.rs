//! `PUT /_matrix/client/*/pushrules/{scope}/{kind}/{ruleId}`
//!
//! This endpoint allows the creation and modification of push rules for this user ID.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#put_matrixclientv3pushrulesscopekindruleid

    use ruma_common::{
        api::{response, Metadata},
        metadata,
        push::{Action, NewPushRule, PushCondition},
        serde::Incoming,
    };
    use serde::{Deserialize, Serialize};

    use crate::push::RuleScope;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushrules/:scope/:kind/:rule_id",
            1.1 => "/_matrix/client/v3/pushrules/:scope/:kind/:rule_id",
        }
    };

    /// Request type for the `set_pushrule` endpoint.
    #[derive(Clone, Debug, Incoming)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    #[incoming_derive(!Deserialize)]
    pub struct Request<'a> {
        /// The scope to set the rule in.
        pub scope: RuleScope,

        /// The rule.
        pub rule: NewPushRule,

        /// Use 'before' with a rule_id as its value to make the new rule the next-most important
        /// rule with respect to the given user defined rule.
        pub before: Option<&'a str>,

        /// This makes the new rule the next-less important rule relative to the given user defined
        /// rule.
        pub after: Option<&'a str>,
    }

    /// Response type for the `set_pushrule` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given scope and rule.
        pub fn new(scope: RuleScope, rule: NewPushRule) -> Self {
            Self { scope, rule, before: None, after: None }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }

    #[cfg(feature = "client")]
    impl<'a> ruma_common::api::OutgoingRequest for Request<'a> {
        type EndpointError = crate::Error;
        type IncomingResponse = Response;

        const METADATA: Metadata = METADATA;

        fn try_into_http_request<T: Default + bytes::BufMut>(
            self,
            base_url: &str,
            access_token: ruma_common::api::SendAccessToken<'_>,
            considering_versions: &[ruma_common::api::MatrixVersion],
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            use http::header;
            use ruma_common::serde::urlencoded;

            let query_string =
                urlencoded::to_string(RequestQuery { before: self.before, after: self.after })?;

            let url = METADATA.make_endpoint_url(
                considering_versions,
                base_url,
                &[&self.scope, &self.rule.kind(), &self.rule.rule_id()],
                &query_string,
            )?;

            let body: RequestBody = self.rule.into();

            http::Request::builder()
                .method(http::Method::GET)
                .uri(url)
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    header::AUTHORIZATION,
                    format!(
                        "Bearer {}",
                        access_token
                            .get_required_for_endpoint()
                            .ok_or(ruma_common::api::error::IntoHttpError::NeedsAuthentication)?,
                    ),
                )
                .body(ruma_common::serde::json_to_buf(&body)?)
                .map_err(Into::into)
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::IncomingRequest for IncomingRequest {
        type EndpointError = crate::Error;
        type OutgoingResponse = Response;

        const METADATA: Metadata = METADATA;

        fn try_from_http_request<B, S>(
            request: http::Request<B>,
            path_args: &[S],
        ) -> Result<Self, ruma_common::api::error::FromHttpRequestError>
        where
            B: AsRef<[u8]>,
            S: AsRef<str>,
        {
            use ruma_common::push::{
                NewConditionalPushRule, NewPatternedPushRule, NewSimplePushRule,
            };

            // Exhaustive enum to fail deserialization on unknown variants.
            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "lowercase")]
            enum RuleKind {
                Override,
                Underride,
                Sender,
                Room,
                Content,
            }

            #[derive(Deserialize)]
            struct IncomingRequestQuery {
                before: Option<String>,
                after: Option<String>,
            }

            let (scope, kind, rule_id): (RuleScope, RuleKind, String) =
                serde::Deserialize::deserialize(serde::de::value::SeqDeserializer::<
                    _,
                    serde::de::value::Error,
                >::new(
                    path_args.iter().map(::std::convert::AsRef::as_ref),
                ))?;

            let IncomingRequestQuery { before, after } =
                ruma_common::serde::urlencoded::from_str(request.uri().query().unwrap_or(""))?;

            let rule = match kind {
                RuleKind::Override => {
                    let ConditionalRequestBody { actions, conditions } =
                        serde_json::from_slice(request.body().as_ref())?;
                    NewPushRule::Override(NewConditionalPushRule::new(rule_id, conditions, actions))
                }
                RuleKind::Underride => {
                    let ConditionalRequestBody { actions, conditions } =
                        serde_json::from_slice(request.body().as_ref())?;
                    NewPushRule::Underride(NewConditionalPushRule::new(
                        rule_id, conditions, actions,
                    ))
                }
                RuleKind::Sender => {
                    let SimpleRequestBody { actions } =
                        serde_json::from_slice(request.body().as_ref())?;
                    NewPushRule::Sender(NewSimplePushRule::new(rule_id, actions))
                }
                RuleKind::Room => {
                    let SimpleRequestBody { actions } =
                        serde_json::from_slice(request.body().as_ref())?;
                    NewPushRule::Room(NewSimplePushRule::new(rule_id, actions))
                }
                RuleKind::Content => {
                    let PatternedRequestBody { actions, pattern } =
                        serde_json::from_slice(request.body().as_ref())?;
                    NewPushRule::Content(NewPatternedPushRule::new(rule_id, pattern, actions))
                }
            };

            Ok(Self { scope, rule, before, after })
        }
    }

    #[derive(Debug, Serialize)]
    struct RequestQuery<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        before: Option<&'a str>,

        #[serde(skip_serializing_if = "Option::is_none")]
        after: Option<&'a str>,
    }

    #[derive(Debug, Serialize)]
    #[serde(untagged)]
    enum RequestBody {
        Simple(SimpleRequestBody),

        Patterned(PatternedRequestBody),

        Conditional(ConditionalRequestBody),
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct SimpleRequestBody {
        actions: Vec<Action>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct PatternedRequestBody {
        actions: Vec<Action>,

        pattern: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct ConditionalRequestBody {
        actions: Vec<Action>,

        conditions: Vec<PushCondition>,
    }

    impl From<NewPushRule> for RequestBody {
        fn from(rule: NewPushRule) -> Self {
            match rule {
                NewPushRule::Override(r) => RequestBody::Conditional(ConditionalRequestBody {
                    actions: r.actions,
                    conditions: r.conditions,
                }),
                NewPushRule::Content(r) => RequestBody::Patterned(PatternedRequestBody {
                    actions: r.actions,
                    pattern: r.pattern,
                }),
                NewPushRule::Room(r) => {
                    RequestBody::Simple(SimpleRequestBody { actions: r.actions })
                }
                NewPushRule::Sender(r) => {
                    RequestBody::Simple(SimpleRequestBody { actions: r.actions })
                }
                NewPushRule::Underride(r) => RequestBody::Conditional(ConditionalRequestBody {
                    actions: r.actions,
                    conditions: r.conditions,
                }),
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("variant added to NewPushRule not covered by RequestBody"),
            }
        }
    }
}
