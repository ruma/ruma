//! Module for [User-Interactive Authentication API][uiaa] types.
//!
//! [uiaa]: https://spec.matrix.org/latest/client-server-api/#user-interactive-authentication-api

use std::{borrow::Cow, fmt, marker::PhantomData};

use bytes::BufMut;
use ruma_common::{
    api::{EndpointError, OutgoingResponse, error::IntoHttpError},
    serde::StringEnum,
};
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::{from_slice as from_json_slice, value::RawValue as RawJsonValue};

use crate::{
    PrivOwnedStr,
    error::{Error as MatrixError, StandardErrorBody},
};

mod auth_data;
mod auth_params;
pub mod get_uiaa_fallback_page;

pub use self::{auth_data::*, auth_params::*};

/// The type of an authentication stage.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[non_exhaustive]
pub enum AuthType {
    /// Password-based authentication (`m.login.password`).
    #[ruma_enum(rename = "m.login.password")]
    Password,

    /// Google ReCaptcha 2.0 authentication (`m.login.recaptcha`).
    #[ruma_enum(rename = "m.login.recaptcha")]
    ReCaptcha,

    /// Email-based authentication (`m.login.email.identity`).
    #[ruma_enum(rename = "m.login.email.identity")]
    EmailIdentity,

    /// Phone number-based authentication (`m.login.msisdn`).
    #[ruma_enum(rename = "m.login.msisdn")]
    Msisdn,

    /// SSO-based authentication (`m.login.sso`).
    #[ruma_enum(rename = "m.login.sso")]
    Sso,

    /// Dummy authentication (`m.login.dummy`).
    #[ruma_enum(rename = "m.login.dummy")]
    Dummy,

    /// Registration token-based authentication (`m.login.registration_token`).
    #[ruma_enum(rename = "m.login.registration_token")]
    RegistrationToken,

    /// Terms of service (`m.login.terms`).
    ///
    /// This type is only valid during account registration.
    #[ruma_enum(rename = "m.login.terms")]
    Terms,

    /// OAuth 2.0 (`m.oauth`).
    ///
    /// This type is only valid with the cross-signing keys upload endpoint, after logging in with
    /// the OAuth 2.0 API.
    #[ruma_enum(rename = "m.oauth", alias = "org.matrix.cross_signing_reset")]
    OAuth,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// Information about available authentication flows and status for User-Interactive Authenticiation
/// API.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct UiaaInfo {
    /// List of authentication flows available for this endpoint.
    pub flows: Vec<AuthFlow>,

    /// List of stages in the current flow completed by the client.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completed: Vec<AuthType>,

    /// Authentication parameters required for the client to complete authentication.
    ///
    /// To create a `Box<RawJsonValue>`, use `serde_json::value::to_raw_value`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Box<RawJsonValue>>,

    /// Session key for client to use to complete authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,

    /// Authentication-related errors for previous request returned by homeserver.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub auth_error: Option<StandardErrorBody>,
}

impl UiaaInfo {
    /// Creates a new `UiaaInfo` with the given flows.
    pub fn new(flows: Vec<AuthFlow>) -> Self {
        Self { flows, completed: Vec::new(), params: None, session: None, auth_error: None }
    }

    /// Get the parameters for the given [`AuthType`], if they are available in the `params` object.
    ///
    /// Returns `Ok(Some(_))` if the parameters for the authentication type were found and the
    /// deserialization worked, `Ok(None)` if the parameters for the authentication type were not
    /// found, and `Err(_)` if the parameters for the authentication type were found but their
    /// deserialization failed.
    ///
    /// # Example
    ///
    /// ```
    /// # use ruma_client_api::uiaa::UiaaInfo;
    /// use ruma_client_api::uiaa::{AuthType, LoginTermsParams};
    ///
    /// # let uiaa_info = UiaaInfo::new(Vec::new());
    /// let login_terms_params = uiaa_info.params::<LoginTermsParams>(&AuthType::Terms)?;
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn params<'a, T: Deserialize<'a>>(
        &'a self,
        auth_type: &AuthType,
    ) -> Result<Option<T>, serde_json::Error> {
        struct AuthTypeVisitor<'b, T> {
            auth_type: &'b AuthType,
            _phantom: PhantomData<T>,
        }

        impl<'de, T> de::Visitor<'de> for AuthTypeVisitor<'_, T>
        where
            T: Deserialize<'de>,
        {
            type Value = Option<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a key-value map")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut params = None;

                while let Some(key) = map.next_key::<Cow<'de, str>>()? {
                    if key == self.auth_type.as_str() {
                        params = Some(map.next_value()?);
                    } else {
                        map.next_value::<de::IgnoredAny>()?;
                    }
                }

                Ok(params)
            }
        }

        let Some(params) = &self.params else {
            return Ok(None);
        };

        let mut deserializer = serde_json::Deserializer::from_str(params.get());
        deserializer.deserialize_map(AuthTypeVisitor { auth_type, _phantom: PhantomData })
    }
}

/// Description of steps required to authenticate via the User-Interactive Authentication API.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct AuthFlow {
    /// Ordered list of stages required to complete authentication.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<AuthType>,
}

impl AuthFlow {
    /// Creates a new `AuthFlow` with the given stages.
    ///
    /// To create an empty `AuthFlow`, use `AuthFlow::default()`.
    pub fn new(stages: Vec<AuthType>) -> Self {
        Self { stages }
    }
}

/// Contains either a User-Interactive Authentication API response body or a Matrix error.
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum UiaaResponse {
    /// User-Interactive Authentication API response
    AuthResponse(UiaaInfo),

    /// Matrix error response
    MatrixError(MatrixError),
}

impl fmt::Display for UiaaResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthResponse(_) => write!(f, "User-Interactive Authentication required."),
            Self::MatrixError(err) => write!(f, "{err}"),
        }
    }
}

impl From<MatrixError> for UiaaResponse {
    fn from(error: MatrixError) -> Self {
        Self::MatrixError(error)
    }
}

impl EndpointError for UiaaResponse {
    fn from_http_response<T: AsRef<[u8]>>(response: http::Response<T>) -> Self {
        if response.status() == http::StatusCode::UNAUTHORIZED
            && let Ok(uiaa_info) = from_json_slice(response.body().as_ref())
        {
            return Self::AuthResponse(uiaa_info);
        }

        Self::MatrixError(MatrixError::from_http_response(response))
    }
}

impl std::error::Error for UiaaResponse {}

impl OutgoingResponse for UiaaResponse {
    fn try_into_http_response<T: Default + BufMut>(
        self,
    ) -> Result<http::Response<T>, IntoHttpError> {
        match self {
            UiaaResponse::AuthResponse(authentication_info) => http::Response::builder()
                .header(http::header::CONTENT_TYPE, ruma_common::http_headers::APPLICATION_JSON)
                .status(http::StatusCode::UNAUTHORIZED)
                .body(ruma_common::serde::json_to_buf(&authentication_info)?)
                .map_err(Into::into),
            UiaaResponse::MatrixError(error) => error.try_into_http_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::serde::JsonObject;
    use serde_json::{from_value as from_json_value, json};

    use super::{AuthType, LoginTermsParams, UiaaInfo};

    #[test]
    fn uiaa_info_params() {
        let json = json!({
            "flows": [{
                "stages": ["m.login.terms", "m.login.email.identity", "local.custom.stage"],
            }],
            "params": {
                "local.custom.stage": {
                    "foo": "bar",
                },
                "m.login.terms": {
                    "policies": {
                        "privacy": {
                            "en-US": {
                                "name": "Privacy Policy",
                                "url": "http://matrix.local/en-US/privacy",
                            },
                            "fr-FR": {
                                "name": "Politique de confidentialité",
                                "url": "http://matrix.local/fr-FR/privacy",
                            },
                            "version": "1",
                        },
                    },
                }
            },
            "session": "abcdef",
        });

        let info = from_json_value::<UiaaInfo>(json).unwrap();

        assert_matches!(info.params::<JsonObject>(&AuthType::EmailIdentity), Ok(None));
        assert_matches!(
            info.params::<JsonObject>(&AuthType::from("local.custom.stage")),
            Ok(Some(_))
        );

        assert_matches!(info.params::<LoginTermsParams>(&AuthType::Terms), Ok(Some(params)));
        assert_eq!(params.policies.len(), 1);

        let policy = params.policies.get("privacy").unwrap();
        assert_eq!(policy.version, "1");
        assert_eq!(policy.translations.len(), 2);
        let translation = policy.translations.get("en-US").unwrap();
        assert_eq!(translation.name, "Privacy Policy");
        assert_eq!(translation.url, "http://matrix.local/en-US/privacy");
        let translation = policy.translations.get("fr-FR").unwrap();
        assert_eq!(translation.name, "Politique de confidentialité");
        assert_eq!(translation.url, "http://matrix.local/fr-FR/privacy");
    }
}
