//! Module for [User-Interactive Authentication API][uiaa] types.
//!
//! [uiaa]: https://matrix.org/docs/spec/client_server/r0.6.1#user-interactive-authentication-api

use std::{borrow::Cow, collections::BTreeMap, fmt};

use bytes::BufMut;
use ruma_api::{
    error::{IntoHttpError, ResponseDeserializationError},
    EndpointError, OutgoingResponse,
};
use ruma_common::thirdparty::Medium;
use ruma_identifiers::{ClientSecret, SessionId};
use ruma_serde::Outgoing;
use serde::{
    de::{self, DeserializeOwned},
    Deserialize, Deserializer, Serialize,
};
use serde_json::{
    from_slice as from_json_slice, value::RawValue as RawJsonValue, Value as JsonValue,
};

use crate::error::{Error as MatrixError, ErrorBody};

pub mod authorize_fallback;
mod user_serde;

/// Additional authentication information for the user-interactive authentication API.
#[derive(Clone, Debug, Outgoing, Serialize)]
#[non_exhaustive]
#[incoming_derive(!Deserialize)]
#[serde(untagged)]
pub enum AuthData<'a> {
    /// Password-based authentication (`m.login.password`).
    Password(Password<'a>),

    /// Google ReCaptcha 2.0 authentication (`m.login.recaptcha`).
    ReCaptcha(ReCaptcha<'a>),

    /// Token-based authentication (`m.login.token`).
    Token(Token<'a>),

    /// OAuth2-based authentication (`m.login.oauth2`).
    OAuth2(OAuth2<'a>),

    /// Email-based authentication (`m.login.email.identity`).
    EmailIdentity(EmailIdentity<'a>),

    /// Phone number-based authentication (`m.login.msisdn`).
    Msisdn(Msisdn<'a>),

    /// Dummy authentication (`m.login.dummy`).
    Dummy(Dummy<'a>),

    /// Registration token-based authentication (`org.matrix.msc3231.login.registration_token`)
    #[cfg(feature = "unstable-pre-spec")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
    RegistrationToken(RegistrationToken<'a>),

    /// Fallback acknowledgement.
    FallbackAcknowledgement(FallbackAcknowledgement<'a>),

    #[doc(hidden)]
    _Custom(CustomAuthData<'a>),
}

impl<'a> AuthData<'a> {
    /// Creates a new `AuthData::FallbackAcknowledgement` with the given session key.
    pub fn fallback_acknowledgement(session: &'a str) -> Self {
        Self::FallbackAcknowledgement(FallbackAcknowledgement::new(session))
    }

    /// Returns the value of the `type` field, if it exists.
    pub fn auth_type(&self) -> Option<&'a str> {
        match self {
            Self::Password(_) => Some("m.login.password"),
            Self::ReCaptcha(_) => Some("m.login.recaptcha"),
            Self::Token(_) => Some("m.login.token"),
            Self::OAuth2(_) => Some("m.login.oauth2"),
            Self::EmailIdentity(_) => Some("m.login.email.identity"),
            Self::Msisdn(_) => Some("m.login.msisdn"),
            Self::Dummy(_) => Some("m.login.dummy"),
            #[cfg(feature = "unstable-pre-spec")]
            Self::RegistrationToken(_) => Some("org.matrix.msc3231.login.registration_token"),
            Self::FallbackAcknowledgement(_) => None,
            Self::_Custom(c) => Some(c.auth_type),
        }
    }

    /// Returns the value of the `session` field, if it exists.
    pub fn session(&self) -> Option<&'a str> {
        match self {
            Self::Password(x) => x.session,
            Self::ReCaptcha(x) => x.session,
            Self::Token(x) => x.session,
            Self::OAuth2(x) => x.session,
            Self::EmailIdentity(x) => x.session,
            Self::Msisdn(x) => x.session,
            Self::Dummy(x) => x.session,
            #[cfg(feature = "unstable-pre-spec")]
            Self::RegistrationToken(x) => x.session,
            Self::FallbackAcknowledgement(x) => Some(x.session),
            Self::_Custom(x) => x.session,
        }
    }
}

impl IncomingAuthData {
    /// Returns the value of the `type` field, if it exists.
    pub fn auth_type(&self) -> Option<&str> {
        match self {
            Self::Password(_) => Some("m.login.password"),
            Self::ReCaptcha(_) => Some("m.login.recaptcha"),
            Self::Token(_) => Some("m.login.token"),
            Self::OAuth2(_) => Some("m.login.oauth2"),
            Self::EmailIdentity(_) => Some("m.login.email.identity"),
            Self::Msisdn(_) => Some("m.login.msisdn"),
            Self::Dummy(_) => Some("m.login.dummy"),
            #[cfg(feature = "unstable-pre-spec")]
            Self::RegistrationToken(_) => Some("org.matrix.msc3231.login.registration_token"),
            Self::FallbackAcknowledgement(_) => None,
            Self::_Custom(c) => Some(&c.auth_type),
        }
    }

    /// Returns the value of the `session` field, if it exists.
    pub fn session(&self) -> Option<&str> {
        match self {
            Self::Password(x) => x.session.as_deref(),
            Self::ReCaptcha(x) => x.session.as_deref(),
            Self::Token(x) => x.session.as_deref(),
            Self::OAuth2(x) => x.session.as_deref(),
            Self::EmailIdentity(x) => x.session.as_deref(),
            Self::Msisdn(x) => x.session.as_deref(),
            Self::Dummy(x) => x.session.as_deref(),
            #[cfg(feature = "unstable-pre-spec")]
            Self::RegistrationToken(x) => x.session.as_deref(),
            Self::FallbackAcknowledgement(x) => Some(&x.session),
            Self::_Custom(x) => x.session.as_deref(),
        }
    }
}

impl<'de> Deserialize<'de> for IncomingAuthData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        fn from_raw_json_value<T: DeserializeOwned, E: de::Error>(
            raw: &RawJsonValue,
        ) -> Result<T, E> {
            serde_json::from_str(raw.get()).map_err(E::custom)
        }

        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        #[derive(Deserialize)]
        struct ExtractType<'a> {
            #[serde(borrow, rename = "type")]
            auth_type: Option<Cow<'a, str>>,
        }

        let auth_type = serde_json::from_str::<ExtractType<'_>>(json.get())
            .map_err(de::Error::custom)?
            .auth_type;

        match auth_type.as_deref() {
            Some("m.login.password") => from_raw_json_value(&json).map(Self::Password),
            Some("m.login.recaptcha") => from_raw_json_value(&json).map(Self::ReCaptcha),
            Some("m.login.token") => from_raw_json_value(&json).map(Self::Token),
            Some("m.login.oauth2") => from_raw_json_value(&json).map(Self::OAuth2),
            Some("m.login.email.identity") => from_raw_json_value(&json).map(Self::EmailIdentity),
            Some("m.login.msisdn") => from_raw_json_value(&json).map(Self::Msisdn),
            Some("m.login.dummy") => from_raw_json_value(&json).map(Self::Dummy),
            #[cfg(feature = "unstable-pre-spec")]
            Some("org.matrix.msc3231.login.registration_token") => {
                from_raw_json_value(&json).map(Self::RegistrationToken)
            }
            None => from_raw_json_value(&json).map(Self::FallbackAcknowledgement),
            Some(_) => from_raw_json_value(&json).map(Self::_Custom),
        }
    }
}

/// Data for password-based UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://matrix.org/docs/spec/client_server/r0.6.1#password-based
#[derive(Clone, Debug, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.password")]
pub struct Password<'a> {
    /// One of the user's identifiers.
    pub identifier: UserIdentifier<'a>,

    /// The plaintext password.
    pub password: &'a str,

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<&'a str>,
}

impl<'a> Password<'a> {
    /// Creates a new `Password` with the given identifier and password.
    pub fn new(identifier: UserIdentifier<'a>, password: &'a str) -> Self {
        Self { identifier, password, session: None }
    }
}

/// Data for ReCaptcha UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://matrix.org/docs/spec/client_server/r0.6.1#google-recaptcha
#[derive(Clone, Debug, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.recaptcha")]
pub struct ReCaptcha<'a> {
    /// The captcha response.
    pub response: &'a str,

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<&'a str>,
}

impl<'a> ReCaptcha<'a> {
    /// Creates a new `ReCaptcha` with the given response string.
    pub fn new(response: &'a str) -> Self {
        Self { response, session: None }
    }
}

/// Data for token-based UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://matrix.org/docs/spec/client_server/r0.6.1#token-based
#[derive(Clone, Debug, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.token")]
pub struct Token<'a> {
    /// The login token.
    pub token: &'a str,

    /// The transaction ID.
    pub txn_id: &'a str,

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<&'a str>,
}

impl<'a> Token<'a> {
    /// Creates a new `Token` with the given token and transaction ID.
    pub fn new(token: &'a str, txn_id: &'a str) -> Self {
        Self { token, txn_id, session: None }
    }
}

/// Data for OAuth2-based UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://matrix.org/docs/spec/client_server/r0.6.1#oauth2-based
#[derive(Clone, Debug, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.oauth2")]
pub struct OAuth2<'a> {
    /// Authorization Request URI or service selection URI.
    pub uri: &'a str,

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<&'a str>,
}

impl<'a> OAuth2<'a> {
    /// Creates a new `OAuth2` with the given URI.
    pub fn new(uri: &'a str) -> Self {
        Self { uri, session: None }
    }
}

/// Data for Email-based UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://matrix.org/docs/spec/client_server/r0.6.1#email-based-identity-homeserver
#[derive(Clone, Debug, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.email.identity")]
pub struct EmailIdentity<'a> {
    /// Thirdparty identifier credentials.
    #[serde(rename = "threepidCreds")]
    pub thirdparty_id_creds: &'a [ThirdpartyIdCredentials<'a>],

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<&'a str>,
}

/// Data for phone number-based UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://matrix.org/docs/spec/client_server/r0.6.1#phone-number-msisdn-based-identity-homeserver
#[derive(Clone, Debug, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.msisdn")]
pub struct Msisdn<'a> {
    /// Thirdparty identifier credentials.
    #[serde(rename = "threepidCreds")]
    pub thirdparty_id_creds: &'a [ThirdpartyIdCredentials<'a>],

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<&'a str>,
}

/// Data for dummy UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://matrix.org/docs/spec/client_server/r0.6.1#dummy-auth
#[derive(Clone, Debug, Default, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.dummy")]
pub struct Dummy<'a> {
    /// The value of the session key given by the homeserver, if any.
    pub session: Option<&'a str>,
}

impl Dummy<'_> {
    /// Creates an empty `Dummy`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Data for registration token-based UIAA flow.
///
/// See [MSC3231] for how to use this.
///
/// [MSC3231]: https://github.com/matrix-org/matrix-doc/pull/3231
#[derive(Clone, Debug, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
#[serde(tag = "type", rename = "org.matrix.msc3231.login.registration_token")]
pub struct RegistrationToken<'a> {
    /// The registration token.
    pub token: &'a str,

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<&'a str>,
}

#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
impl<'a> RegistrationToken<'a> {
    /// Creates a new `RegistrationToken` with the given token.
    pub fn new(token: &'a str) -> Self {
        Self { token, session: None }
    }
}

/// Data for UIAA fallback acknowledgement.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://matrix.org/docs/spec/client_server/r0.6.1#fallback
#[derive(Clone, Debug, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct FallbackAcknowledgement<'a> {
    /// The value of the session key given by the homeserver.
    pub session: &'a str,
}

impl<'a> FallbackAcknowledgement<'a> {
    /// Creates a new `FallbackAcknowledgement` with the given session key.
    pub fn new(session: &'a str) -> Self {
        Self { session }
    }
}

#[doc(hidden)]
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct CustomAuthData<'a> {
    #[serde(rename = "type")]
    pub auth_type: &'a str,
    pub session: Option<&'a str>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, JsonValue>,
}

#[doc(hidden)]
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct IncomingCustomAuthData {
    #[serde(rename = "type")]
    pub auth_type: String,
    pub session: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, JsonValue>,
}

impl Outgoing for CustomAuthData<'_> {
    type Incoming = IncomingCustomAuthData;
}

/// Identification information for the user.
#[derive(Clone, Debug, PartialEq, Eq, Outgoing, Serialize)]
#[serde(from = "user_serde::IncomingUserIdentifier", into = "user_serde::UserIdentifier<'_>")]
#[allow(clippy::exhaustive_enums)]
pub enum UserIdentifier<'a> {
    /// Either a fully qualified Matrix user ID, or just the localpart (as part of the 'identifier'
    /// field).
    MatrixId(&'a str),

    /// Third party identifier (as part of the 'identifier' field).
    ThirdPartyId {
        /// Third party identifier for the user.
        address: &'a str,

        /// The medium of the identifier.
        medium: Medium,
    },

    /// Same as third-party identification with medium == msisdn, but with a non-canonicalised
    /// phone number.
    PhoneNumber {
        /// The country that the phone number is from.
        country: &'a str,

        /// The phone number.
        phone: &'a str,
    },
}

/// Credentials for thirdparty authentification (e.g. email / phone number).
#[derive(Clone, Debug, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ThirdpartyIdCredentials<'a> {
    /// Identity server session ID.
    pub sid: &'a SessionId,

    /// Identity server client secret.
    pub client_secret: &'a ClientSecret,

    /// Identity server URL.
    pub id_server: &'a str,

    /// Identity server access token.
    pub id_access_token: &'a str,
}

impl<'a> ThirdpartyIdCredentials<'a> {
    /// Creates a new `ThirdpartyIdCredentials` with the given session ID, client secret, identity
    /// server address and access token.
    pub fn new(
        sid: &'a SessionId,
        client_secret: &'a ClientSecret,
        id_server: &'a str,
        id_access_token: &'a str,
    ) -> Self {
        Self { sid, client_secret, id_server, id_access_token }
    }
}

/// Information about available authentication flows and status for User-Interactive Authenticiation
/// API.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UiaaInfo {
    /// List of authentication flows available for this endpoint.
    pub flows: Vec<AuthFlow>,

    /// List of stages in the current flow completed by the client.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completed: Vec<String>,

    /// Authentication parameters required for the client to complete authentication.
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

impl UiaaInfo {
    /// Creates a new `UiaaInfo` with the given flows and parameters.
    pub fn new(flows: Vec<AuthFlow>, params: Box<RawJsonValue>) -> Self {
        Self { flows, completed: Vec::new(), params, session: None, auth_error: None }
    }
}

/// Description of steps required to authenticate via the User-Interactive Authentication API.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AuthFlow {
    /// Ordered list of stages required to complete authentication.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<String>,
}

impl AuthFlow {
    /// Creates a new `AuthFlow` with the given stages.
    ///
    /// To create an empty `AuthFlow`, use `AuthFlow::default()`.
    pub fn new(stages: Vec<String>) -> Self {
        Self { stages }
    }
}

/// Contains either a User-Interactive Authentication API response body or a Matrix error.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
    fn try_from_http_response<T: AsRef<[u8]>>(
        response: http::Response<T>,
    ) -> Result<Self, ResponseDeserializationError> {
        if response.status() == http::StatusCode::UNAUTHORIZED {
            Ok(UiaaResponse::AuthResponse(from_json_slice(response.body().as_ref())?))
        } else {
            MatrixError::try_from_http_response(response).map(From::from)
        }
    }
}

impl std::error::Error for UiaaResponse {}

impl OutgoingResponse for UiaaResponse {
    fn try_into_http_response<T: Default + BufMut>(
        self,
    ) -> Result<http::Response<T>, IntoHttpError> {
        match self {
            UiaaResponse::AuthResponse(authentication_info) => http::Response::builder()
                .header(http::header::CONTENT_TYPE, "application/json")
                .status(&http::StatusCode::UNAUTHORIZED)
                .body(ruma_serde::json_to_buf(&authentication_info)?)
                .map_err(Into::into),
            UiaaResponse::MatrixError(error) => error.try_into_http_response(),
        }
    }
}
