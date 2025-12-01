//! `PUT /_matrix/client/*/pushrules/global/{kind}/{ruleId}`
//!
//! This endpoint allows the creation and modification of push rules for this user ID.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3pushrulesglobalkindruleid

    use ruma_common::{
        api::{auth_scheme::AccessToken, response},
        metadata,
        push::{Action, NewPushRule, PushCondition},
    };

    metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushrules/global/{kind}/{rule_id}",
            1.1 => "/_matrix/client/v3/pushrules/global/{kind}/{rule_id}",
        }
    }

    /// Request type for the `set_pushrule` endpoint.
    #[derive(Clone, Debug)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Request {
        /// The rule.
        pub rule: NewPushRule,

        /// Use 'before' with a rule_id as its value to make the new rule the next-most important
        /// rule with respect to the given user defined rule.
        pub before: Option<String>,

        /// This makes the new rule the next-less important rule relative to the given user defined
        /// rule.
        pub after: Option<String>,
    }

    /// Response type for the `set_pushrule` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given rule.
        pub fn new(rule: NewPushRule) -> Self {
            Self { rule, before: None, after: None }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::OutgoingRequest for Request {
        type EndpointError = crate::Error;
        type IncomingResponse = Response;

        fn try_into_http_request<T: Default + bytes::BufMut + AsRef<[u8]>>(
            self,
            base_url: &str,
            access_token: ruma_common::api::auth_scheme::SendAccessToken<'_>,
            considering: std::borrow::Cow<'_, ruma_common::api::SupportedVersions>,
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            use ruma_common::api::{Metadata, auth_scheme::AuthScheme};

            fn serialize_rule<T: Default + bytes::BufMut>(
                rule: NewPushRule,
            ) -> serde_json::Result<T> {
                match rule {
                    NewPushRule::Override(r) | NewPushRule::Underride(r) => {
                        let body =
                            ConditionalRequestBody { actions: r.actions, conditions: r.conditions };
                        ruma_common::serde::json_to_buf(&body)
                    }
                    NewPushRule::Content(r) => {
                        let body = PatternedRequestBody { actions: r.actions, pattern: r.pattern };
                        ruma_common::serde::json_to_buf(&body)
                    }
                    NewPushRule::Room(r) => {
                        let body = SimpleRequestBody { actions: r.actions };
                        ruma_common::serde::json_to_buf(&body)
                    }
                    NewPushRule::Sender(r) => {
                        let body = SimpleRequestBody { actions: r.actions };
                        ruma_common::serde::json_to_buf(&body)
                    }
                    #[cfg(not(ruma_unstable_exhaustive_types))]
                    _ => unreachable!(
                        "variant added to NewPushRule not serializable to request body"
                    ),
                }
            }

            let query_string = serde_html_form::to_string(RequestQuery {
                before: self.before,
                after: self.after,
            })?;

            let url = Self::make_endpoint_url(
                considering,
                base_url,
                &[&self.rule.kind(), &self.rule.rule_id()],
                &query_string,
            )?;

            let mut http_request = http::Request::builder()
                .method(Self::METHOD)
                .uri(url)
                .header(http::header::CONTENT_TYPE, ruma_common::http_headers::APPLICATION_JSON)
                .body(serialize_rule(self.rule)?)?;

            Self::Authentication::add_authentication(&mut http_request, access_token).map_err(
                |error| ruma_common::api::error::IntoHttpError::Authentication(error.into()),
            )?;

            Ok(http_request)
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::IncomingRequest for Request {
        type EndpointError = crate::Error;
        type OutgoingResponse = Response;

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
            #[derive(Debug, serde::Deserialize)]
            #[serde(rename_all = "lowercase")]
            enum RuleKind {
                Override,
                Underride,
                Sender,
                Room,
                Content,
            }

            Self::check_request_method(request.method())?;

            let (kind, rule_id): (RuleKind, String) =
                serde::Deserialize::deserialize(serde::de::value::SeqDeserializer::<
                    _,
                    serde::de::value::Error,
                >::new(
                    path_args.iter().map(::std::convert::AsRef::as_ref),
                ))?;

            let RequestQuery { before, after } =
                serde_html_form::from_str(request.uri().query().unwrap_or(""))?;

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
                    let rule_id = rule_id.try_into()?;
                    NewPushRule::Sender(NewSimplePushRule::new(rule_id, actions))
                }
                RuleKind::Room => {
                    let SimpleRequestBody { actions } =
                        serde_json::from_slice(request.body().as_ref())?;
                    let rule_id = rule_id.try_into()?;
                    NewPushRule::Room(NewSimplePushRule::new(rule_id, actions))
                }
                RuleKind::Content => {
                    let PatternedRequestBody { actions, pattern } =
                        serde_json::from_slice(request.body().as_ref())?;
                    NewPushRule::Content(NewPatternedPushRule::new(rule_id, pattern, actions))
                }
            };

            Ok(Self { rule, before, after })
        }
    }

    #[derive(Debug)]
    #[cfg_attr(feature = "client", derive(serde::Serialize))]
    #[cfg_attr(feature = "server", derive(serde::Deserialize))]
    struct RequestQuery {
        #[serde(skip_serializing_if = "Option::is_none")]
        before: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        after: Option<String>,
    }

    #[derive(Debug)]
    #[cfg_attr(feature = "client", derive(serde::Serialize))]
    #[cfg_attr(feature = "server", derive(serde::Deserialize))]
    struct SimpleRequestBody {
        actions: Vec<Action>,
    }

    #[derive(Debug)]
    #[cfg_attr(feature = "client", derive(serde::Serialize))]
    #[cfg_attr(feature = "server", derive(serde::Deserialize))]
    struct PatternedRequestBody {
        actions: Vec<Action>,

        pattern: String,
    }

    #[derive(Debug)]
    #[cfg_attr(feature = "client", derive(serde::Serialize))]
    #[cfg_attr(feature = "server", derive(serde::Deserialize))]
    struct ConditionalRequestBody {
        actions: Vec<Action>,

        conditions: Vec<PushCondition>,
    }
}
