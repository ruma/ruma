//! Module for User-Interactive Authentication API types.

use std::collections::BTreeMap;

use ruma_api::{error::ResponseDeserializationError, EndpointError};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::error::{Error as MatrixError, ErrorBody};

/// Additional authentication information for the user-interactive authentication API.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[cfg_attr(test, derive(PartialEq))]
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
#[cfg_attr(test, derive(PartialEq))]
pub struct UiaaInfo {
    /// List of authentication flows available for this endpoint.
    pub flows: Vec<AuthFlow>,
    /// List of stages in the current flow completed by the client.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completed: Vec<String>,
    /// Authentication parameters required for the client to complete authentication.
    pub params: JsonValue,
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
            if let Ok(authentication_info) = serde_json::from_slice::<UiaaInfo>(response.body()) {
                return Ok(UiaaResponse::AuthResponse(authentication_info));
            }
        }

        MatrixError::try_from_response(response).map(From::from)
    }
}

impl From<UiaaResponse> for http::Response<Vec<u8>> {
    fn from(uiaa_response: UiaaResponse) -> http::Response<Vec<u8>> {
        match uiaa_response {
            UiaaResponse::AuthResponse(authentication_info) => http::Response::builder()
                .header(http::header::CONTENT_TYPE, "application/json")
                .status(&http::StatusCode::UNAUTHORIZED)
                .body(serde_json::to_vec(&authentication_info).unwrap())
                .unwrap(),
            UiaaResponse::MatrixError(error) => http::Response::from(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ruma_api::EndpointError;
    use serde_json::{
        from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
    };

    use super::{AuthData, AuthFlow, UiaaInfo, UiaaResponse};
    use crate::error::{ErrorBody, ErrorKind};

    #[test]
    fn test_serialize_authentication_data_direct_request() {
        let mut auth_parameters = BTreeMap::new();
        auth_parameters.insert(
            "example_credential".into(),
            JsonValue::String("verypoorsharedsecret".into()),
        );
        let authentication_data = AuthData::DirectRequest {
            kind: "example.type.foo".to_string(),
            session: Some("ZXY000".to_string()),
            auth_parameters,
        };

        assert_eq!(
            json!({ "type": "example.type.foo", "session": "ZXY000", "example_credential": "verypoorsharedsecret" }),
            to_json_value(authentication_data).unwrap()
        );
    }

    #[test]
    fn test_deserialize_authentication_data_direct_request() {
        let mut auth_parameters = BTreeMap::new();
        auth_parameters.insert(
            "example_credential".into(),
            JsonValue::String("verypoorsharedsecret".into()),
        );
        let authentication_data = AuthData::DirectRequest {
            kind: "example.type.foo".into(),
            session: Some("opaque_session_id".to_string()),
            auth_parameters,
        };
        let json = json!({ "type": "example.type.foo", "session": "opaque_session_id", "example_credential": "verypoorsharedsecret", });

        assert_eq!(
            from_json_value::<AuthData>(json).unwrap(),
            authentication_data
        );
    }

    #[test]
    fn test_serialize_authentication_data_fallback() {
        let authentication_data = AuthData::FallbackAcknowledgement {
            session: "ZXY000".to_string(),
        };

        assert_eq!(
            json!({ "session": "ZXY000" }),
            to_json_value(authentication_data).unwrap()
        );
    }

    #[test]
    fn test_deserialize_authentication_data_fallback() {
        let authentication_data = AuthData::FallbackAcknowledgement {
            session: "opaque_session_id".to_string(),
        };
        let json = json!({ "session": "opaque_session_id" });

        assert_eq!(
            from_json_value::<AuthData>(json).unwrap(),
            authentication_data
        );
    }

    #[test]
    fn test_serialize_uiaa_info() {
        let params = json!({
            "example.type.baz": {
                "example_key": "foobar"
            }
        });

        let uiaa_info = UiaaInfo {
            flows: vec![AuthFlow {
                stages: vec!["m.login.password".to_string(), "m.login.dummy".to_string()],
            }],
            completed: vec!["m.login.password".to_string()],
            params,
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
        });

        let uiaa_info = UiaaInfo {
            auth_error: Some(ErrorBody {
                kind: ErrorKind::Forbidden,
                message: "Invalid password".to_string(),
            }),
            completed: vec!["example.type.foo".to_string()],
            flows: vec![
                AuthFlow {
                    stages: vec![
                        "example.type.foo".to_string(),
                        "example.type.bar".to_string(),
                    ],
                },
                AuthFlow {
                    stages: vec![
                        "example.type.foo".to_string(),
                        "example.type.baz".to_string(),
                    ],
                },
            ],
            params: json!({
                "example.type.baz": {
                    "example_key": "foobar"
                }
            }),
            session: Some("xxxxxx".to_string()),
        };
        assert_eq!(from_json_value::<UiaaInfo>(json).unwrap(), uiaa_info);
    }

    #[test]
    fn test_try_uiaa_response_into_http_response() {
        let params = json!({
            "example.type.baz": {
                "example_key": "foobar"
            }
        });

        let uiaa_info = UiaaInfo {
            flows: vec![AuthFlow {
                stages: vec!["m.login.password".to_string(), "m.login.dummy".to_string()],
            }],
            completed: vec!["m.login.password".to_string()],
            params,
            session: None,
            auth_error: None,
        };
        let uiaa_response: http::Response<Vec<u8>> =
            UiaaResponse::AuthResponse(uiaa_info.clone()).into();

        assert_eq!(
            serde_json::from_slice::<UiaaInfo>(uiaa_response.body()).unwrap(),
            uiaa_info,
        );
        assert_eq!(
            uiaa_response.status(),
            http::status::StatusCode::UNAUTHORIZED
        );
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

        let uiaa_info = UiaaInfo {
            auth_error: Some(ErrorBody {
                kind: ErrorKind::Forbidden,
                message: "Invalid password".to_string(),
            }),
            completed: vec!["example.type.foo".to_string()],
            flows: vec![
                AuthFlow {
                    stages: vec![
                        "example.type.foo".to_string(),
                        "example.type.bar".to_string(),
                    ],
                },
                AuthFlow {
                    stages: vec![
                        "example.type.foo".to_string(),
                        "example.type.baz".to_string(),
                    ],
                },
            ],
            params: json!({
                "example.type.baz": {
                    "example_key": "foobar"
                }
            }),
            session: Some("xxxxxx".to_string()),
        };

        let parsed_uiaa_info = match UiaaResponse::try_from_response(http_response) {
            Ok(auth_response) => match auth_response {
                UiaaResponse::AuthResponse(uiaa_info) => Some(uiaa_info),
                _ => None,
            },
            _ => None,
        };

        assert_eq!(parsed_uiaa_info, Some(uiaa_info));
    }
}
