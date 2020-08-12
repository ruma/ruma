//! Module for User-Interactive Authentication API types.

use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use ruma_api::{error::ResponseDeserializationError, EndpointError};
use serde::{Deserialize, Serialize};
use serde_json::{
    from_slice as from_json_slice, to_vec as to_json_vec, value::RawValue as RawJsonValue,
    Value as JsonValue,
};

use crate::error::{Error as MatrixError, ErrorBody};

/// Additional authentication information for the user-interactive authentication API.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AuthData {
    /// Used for sending UIAA authentication requests to the homeserver directly
    /// from the client.
    DirectRequest {
        /// The login type that the client is attempting to complete.
        #[serde(rename = "type")]
        kind: String,

        /// The value of the session key given by the homeserver.
        #[serde(skip_serializing_if = "Option::is_none")]
        session: Option<String>,

        /// Parameters submitted for a particular authentication stage.
        // FIXME: RawJsonValue doesn't work here, is that a bug?
        #[serde(flatten)]
        auth_parameters: BTreeMap<String, JsonValue>,
    },

    /// Used by the client to acknowledge that the user has completed a UIAA
    /// stage through the fallback method.
    FallbackAcknowledgement {
        /// The value of the session key given by the homeserver.
        session: String,
    },
}

/// Information about available authentication flows and status for
/// User-Interactive Authenticiation API.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UiaaInfo {
    /// List of authentication flows available for this endpoint.
    pub flows: Vec<AuthFlow>,

    /// List of stages in the current flow completed by the client.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completed: Vec<String>,

    /// Authentication parameters required for the client to complete
    /// authentication.
    ///
    /// To create a `Box<RawJsonValue>`, use `serde_json::value::to_raw_value`.
    pub params: Box<RawJsonValue>,

    /// Session key for client to use to complete authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,

    /// Authentication-related errors for previous request returned by homeserver.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub auth_error: Option<ErrorBody>,
}

/// Description of steps required to authenticate via the User-Interactive
/// Authentication API.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct AuthFlow {
    /// Ordered list of stages required to complete authentication.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<String>,
}

/// Contains either a User-Interactive Authentication API response body or a
/// Matrix error.
#[derive(Clone, Debug)]
pub enum UiaaResponse {
    /// User-Interactive Authentication API response
    AuthResponse(UiaaInfo),

    /// Matrix error response
    MatrixError(MatrixError),
}

impl Display for UiaaResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthResponse(_) => write!(f, "User-Interactive Authentication required."),
            Self::MatrixError(err) => write!(f, "{}", err),
        }
    }
}

impl From<MatrixError> for UiaaResponse {
    fn from(error: MatrixError) -> Self {
        Self::MatrixError(error)
    }
}

impl EndpointError for UiaaResponse {
    fn try_from_response(
        response: http::Response<Vec<u8>>,
    ) -> Result<Self, ResponseDeserializationError> {
        if response.status() == http::StatusCode::UNAUTHORIZED {
            if let Ok(authentication_info) = from_json_slice::<UiaaInfo>(response.body()) {
                return Ok(UiaaResponse::AuthResponse(authentication_info));
            }
        }

        MatrixError::try_from_response(response).map(From::from)
    }
}

impl std::error::Error for UiaaResponse {}

impl From<UiaaResponse> for http::Response<Vec<u8>> {
    fn from(uiaa_response: UiaaResponse) -> http::Response<Vec<u8>> {
        match uiaa_response {
            UiaaResponse::AuthResponse(authentication_info) => http::Response::builder()
                .header(http::header::CONTENT_TYPE, "application/json")
                .status(&http::StatusCode::UNAUTHORIZED)
                .body(to_json_vec(&authentication_info).unwrap())
                .unwrap(),
            UiaaResponse::MatrixError(error) => http::Response::from(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use maplit::btreemap;
    use matches::assert_matches;
    use ruma_api::EndpointError;
    use serde_json::{
        from_slice as from_json_slice, from_str as from_json_str, from_value as from_json_value,
        json, to_value as to_json_value, value::to_raw_value as to_raw_json_value,
        Value as JsonValue,
    };

    use super::{AuthData, AuthFlow, UiaaInfo, UiaaResponse};
    use crate::error::{ErrorBody, ErrorKind};

    #[test]
    fn test_serialize_authentication_data_direct_request() {
        let authentication_data = AuthData::DirectRequest {
            kind: "example.type.foo".into(),
            session: Some("ZXY000".into()),
            auth_parameters: btreemap! {
                "example_credential".to_owned() => json!("verypoorsharedsecret")
            },
        };

        assert_eq!(
            json!({
                "type": "example.type.foo",
                "session": "ZXY000",
                "example_credential": "verypoorsharedsecret",
            }),
            to_json_value(authentication_data).unwrap()
        );
    }

    #[test]
    fn test_deserialize_authentication_data_direct_request() {
        let json = json!({
            "type": "example.type.foo",
            "session": "opaque_session_id",
            "example_credential": "verypoorsharedsecret",
        });

        assert_matches!(
            from_json_value::<AuthData>(json).unwrap(),
            AuthData::DirectRequest { kind, session: Some(session), auth_parameters }
            if kind == "example.type.foo"
                && session == "opaque_session_id"
                && auth_parameters == btreemap!{
                    "example_credential".to_owned() => json!("verypoorsharedsecret")
                }
        );
    }

    #[test]
    fn test_serialize_authentication_data_fallback() {
        let authentication_data = AuthData::FallbackAcknowledgement { session: "ZXY000".into() };

        assert_eq!(json!({ "session": "ZXY000" }), to_json_value(authentication_data).unwrap());
    }

    #[test]
    fn test_deserialize_authentication_data_fallback() {
        let json = json!({ "session": "opaque_session_id" });

        assert_matches!(
            from_json_value::<AuthData>(json).unwrap(),
            AuthData::FallbackAcknowledgement { session }
            if session == "opaque_session_id"
        );
    }

    #[test]
    fn test_serialize_uiaa_info() {
        let uiaa_info = UiaaInfo {
            flows: vec![AuthFlow {
                stages: vec!["m.login.password".into(), "m.login.dummy".into()],
            }],
            completed: vec!["m.login.password".into()],
            params: to_raw_json_value(&json!({
                "example.type.baz": {
                    "example_key": "foobar"
                }
            }))
            .unwrap(),
            session: None,
            auth_error: None,
        };

        let json = json!({
            "flows": [{ "stages": ["m.login.password", "m.login.dummy"] }],
            "completed": ["m.login.password"],
            "params": {
                "example.type.baz": {
                    "example_key": "foobar"
                }
            }
        });
        assert_eq!(to_json_value(uiaa_info).unwrap(), json);
    }

    #[test]
    fn test_deserialize_uiaa_info() {
        let json = json!({
            "errcode": "M_FORBIDDEN",
            "error": "Invalid password",
            "completed": ["example.type.foo"],
            "flows": [
                {
                    "stages": ["example.type.foo", "example.type.bar"]
                },
                {
                    "stages": ["example.type.foo", "example.type.baz"]
                }
            ],
            "params": {
                "example.type.baz": {
                    "example_key": "foobar"
                }
            },
            "session": "xxxxxx"
        });

        assert_matches!(
            from_json_value::<UiaaInfo>(json).unwrap(),
            UiaaInfo {
                auth_error: Some(ErrorBody {
                    kind: ErrorKind::Forbidden,
                    message: error_message,
                }),
                completed,
                flows,
                params,
                session: Some(session),
            } if error_message == "Invalid password"
                && completed == vec!["example.type.foo".to_owned()]
                && flows == vec![
                    AuthFlow {
                        stages: vec![
                            "example.type.foo".into(),
                            "example.type.bar".into(),
                        ],
                    },
                    AuthFlow {
                        stages: vec![
                            "example.type.foo".into(),
                            "example.type.baz".into(),
                        ],
                    },
                ]
                && from_json_str::<JsonValue>(params.get()).unwrap() == json!({
                    "example.type.baz": {
                        "example_key": "foobar"
                    }
                })
                && session == "xxxxxx"
        );
    }

    #[test]
    fn test_try_uiaa_response_into_http_response() {
        let uiaa_info = UiaaInfo {
            flows: vec![AuthFlow {
                stages: vec!["m.login.password".into(), "m.login.dummy".into()],
            }],
            completed: vec!["m.login.password".into()],
            params: to_raw_json_value(&json!({
                "example.type.baz": {
                    "example_key": "foobar"
                }
            }))
            .unwrap(),
            session: None,
            auth_error: None,
        };
        let uiaa_response: http::Response<Vec<u8>> = UiaaResponse::AuthResponse(uiaa_info).into();

        assert_matches!(
            from_json_slice::<UiaaInfo>(uiaa_response.body()).unwrap(),
            UiaaInfo {
                flows,
                completed,
                params,
                session: None,
                auth_error: None,
            } if flows == vec![AuthFlow {
                    stages: vec!["m.login.password".into(), "m.login.dummy".into()],
                }]
                && completed == vec!["m.login.password".to_owned()]
                && from_json_str::<JsonValue>(params.get()).unwrap() == json!({
                    "example.type.baz": {
                        "example_key": "foobar"
                    }
                })
        );
        assert_eq!(uiaa_response.status(), http::status::StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_try_uiaa_response_from_http_response() {
        let json = serde_json::to_string(&json!({
            "errcode": "M_FORBIDDEN",
            "error": "Invalid password",
            "completed": [ "example.type.foo" ],
            "flows": [
                {
                    "stages": [ "example.type.foo", "example.type.bar" ]
                },
                {
                    "stages": [ "example.type.foo", "example.type.baz" ]
                }
            ],
            "params": {
                "example.type.baz": {
                    "example_key": "foobar"
                }
            },
            "session": "xxxxxx"
        }))
        .unwrap();

        let http_response = http::Response::builder()
            .status(http::StatusCode::UNAUTHORIZED)
            .body(json.into())
            .unwrap();

        let parsed_uiaa_info = match UiaaResponse::try_from_response(http_response).unwrap() {
            UiaaResponse::AuthResponse(uiaa_info) => uiaa_info,
            _ => panic!("Expected UiaaResponse::AuthResponse"),
        };

        assert_matches!(
            parsed_uiaa_info,
            UiaaInfo {
                auth_error: Some(ErrorBody {
                    kind: ErrorKind::Forbidden,
                    message: error_message,
                }),
                completed,
                flows,
                params,
                session: Some(session),
            } if error_message == "Invalid password"
                && completed == vec!["example.type.foo".to_owned()]
                && flows == vec![
                    AuthFlow {
                        stages: vec![
                            "example.type.foo".into(),
                            "example.type.bar".into(),
                        ],
                    },
                    AuthFlow {
                        stages: vec![
                            "example.type.foo".into(),
                            "example.type.baz".into(),
                        ],
                    },
                ]
                && from_json_str::<JsonValue>(params.get()).unwrap() == json!({
                    "example.type.baz": {
                        "example_key": "foobar"
                    }
                })
                && session == "xxxxxx"
        );
    }
}
