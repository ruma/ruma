//! Module for [User-Interactive Authentication API][uiaa] types.
//!
//! [uiaa]: https://spec.matrix.org/latest/client-server-api/#user-interactive-authentication-api

use std::{borrow::Cow, fmt};

use bytes::BufMut;
use ruma_common::{
    api::{error::IntoHttpError, EndpointError, OutgoingResponse},
    serde::{from_raw_json_value, JsonObject, StringEnum},
    thirdparty::Medium,
    ClientSecret, OwnedSessionId, OwnedUserId,
};
use serde::{
    de::{self, DeserializeOwned},
    Deserialize, Deserializer, Serialize,
};
use serde_json::{
    from_slice as from_json_slice, value::RawValue as RawJsonValue, Value as JsonValue,
};

use crate::{
    error::{Error as MatrixError, StandardErrorBody},
    PrivOwnedStr,
};

pub mod get_uiaa_fallback_page;
mod user_serde;

/// Information for one authentication stage.
#[derive(Clone, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum AuthData {
    /// Password-based authentication (`m.login.password`).
    Password(Password),

    /// Google ReCaptcha 2.0 authentication (`m.login.recaptcha`).
    ReCaptcha(ReCaptcha),

    /// Email-based authentication (`m.login.email.identity`).
    EmailIdentity(EmailIdentity),

    /// Phone number-based authentication (`m.login.msisdn`).
    Msisdn(Msisdn),

    /// Dummy authentication (`m.login.dummy`).
    Dummy(Dummy),

    /// Registration token-based authentication (`m.login.registration_token`).
    RegistrationToken(RegistrationToken),

    /// Fallback acknowledgement.
    FallbackAcknowledgement(FallbackAcknowledgement),

    #[doc(hidden)]
    _Custom(CustomAuthData),
}

impl AuthData {
    /// Creates a new `AuthData` with the given `auth_type` string, session and data.
    ///
    /// Prefer to use the public variants of `AuthData` where possible; this constructor is meant to
    /// be used for unsupported authentication types only and does not allow setting arbitrary
    /// data for supported ones.
    ///
    /// # Errors
    ///
    /// Returns an error if the `auth_type` is known and serialization of `data` to the
    /// corresponding `AuthData` variant fails.
    pub fn new(
        auth_type: &str,
        session: Option<String>,
        data: JsonObject,
    ) -> serde_json::Result<Self> {
        fn deserialize_variant<T: DeserializeOwned>(
            session: Option<String>,
            mut obj: JsonObject,
        ) -> serde_json::Result<T> {
            if let Some(session) = session {
                obj.insert("session".into(), session.into());
            }
            serde_json::from_value(JsonValue::Object(obj))
        }

        Ok(match auth_type {
            "m.login.password" => Self::Password(deserialize_variant(session, data)?),
            "m.login.recaptcha" => Self::ReCaptcha(deserialize_variant(session, data)?),
            "m.login.email.identity" => Self::EmailIdentity(deserialize_variant(session, data)?),
            "m.login.msisdn" => Self::Msisdn(deserialize_variant(session, data)?),
            "m.login.dummy" => Self::Dummy(deserialize_variant(session, data)?),
            "m.registration_token" => Self::RegistrationToken(deserialize_variant(session, data)?),
            _ => {
                Self::_Custom(CustomAuthData { auth_type: auth_type.into(), session, extra: data })
            }
        })
    }

    /// Creates a new `AuthData::FallbackAcknowledgement` with the given session key.
    pub fn fallback_acknowledgement(session: String) -> Self {
        Self::FallbackAcknowledgement(FallbackAcknowledgement::new(session))
    }

    /// Returns the value of the `type` field, if it exists.
    pub fn auth_type(&self) -> Option<AuthType> {
        match self {
            Self::Password(_) => Some(AuthType::Password),
            Self::ReCaptcha(_) => Some(AuthType::ReCaptcha),
            Self::EmailIdentity(_) => Some(AuthType::EmailIdentity),
            Self::Msisdn(_) => Some(AuthType::Msisdn),
            Self::Dummy(_) => Some(AuthType::Dummy),
            Self::RegistrationToken(_) => Some(AuthType::RegistrationToken),
            Self::FallbackAcknowledgement(_) => None,
            Self::_Custom(c) => Some(AuthType::_Custom(PrivOwnedStr(c.auth_type.as_str().into()))),
        }
    }

    /// Returns the value of the `session` field, if it exists.
    pub fn session(&self) -> Option<&str> {
        match self {
            Self::Password(x) => x.session.as_deref(),
            Self::ReCaptcha(x) => x.session.as_deref(),
            Self::EmailIdentity(x) => x.session.as_deref(),
            Self::Msisdn(x) => x.session.as_deref(),
            Self::Dummy(x) => x.session.as_deref(),
            Self::RegistrationToken(x) => x.session.as_deref(),
            Self::FallbackAcknowledgement(x) => Some(&x.session),
            Self::_Custom(x) => x.session.as_deref(),
        }
    }

    /// Returns the associated data.
    ///
    /// The returned JSON object won't contain the `type` and `session` fields, use
    /// [`.auth_type()`][Self::auth_type] / [`.session()`](Self::session) to access those.
    ///
    /// Prefer to use the public variants of `AuthData` where possible; this method is meant to be
    /// used for custom auth types only.
    pub fn data(&self) -> Cow<'_, JsonObject> {
        fn serialize<T: Serialize>(obj: T) -> JsonObject {
            match serde_json::to_value(obj).expect("auth data serialization to succeed") {
                JsonValue::Object(obj) => obj,
                _ => panic!("all auth data variants must serialize to objects"),
            }
        }

        match self {
            Self::Password(x) => Cow::Owned(serialize(Password {
                identifier: x.identifier.clone(),
                password: x.password.clone(),
                session: None,
            })),
            Self::ReCaptcha(x) => {
                Cow::Owned(serialize(ReCaptcha { response: x.response.clone(), session: None }))
            }
            Self::EmailIdentity(x) => Cow::Owned(serialize(EmailIdentity {
                thirdparty_id_creds: x.thirdparty_id_creds.clone(),
                session: None,
            })),
            Self::Msisdn(x) => Cow::Owned(serialize(Msisdn {
                thirdparty_id_creds: x.thirdparty_id_creds.clone(),
                session: None,
            })),
            Self::RegistrationToken(x) => {
                Cow::Owned(serialize(RegistrationToken { token: x.token.clone(), session: None }))
            }
            // Dummy and fallback acknowledgement have no associated data
            Self::Dummy(_) | Self::FallbackAcknowledgement(_) => Cow::Owned(JsonObject::default()),
            Self::_Custom(c) => Cow::Borrowed(&c.extra),
        }
    }
}

impl fmt::Debug for AuthData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print `Password { .. }` instead of `Password(Password { .. })`
        match self {
            Self::Password(inner) => inner.fmt(f),
            Self::ReCaptcha(inner) => inner.fmt(f),
            Self::EmailIdentity(inner) => inner.fmt(f),
            Self::Msisdn(inner) => inner.fmt(f),
            Self::Dummy(inner) => inner.fmt(f),
            Self::RegistrationToken(inner) => inner.fmt(f),
            Self::FallbackAcknowledgement(inner) => inner.fmt(f),
            Self::_Custom(inner) => inner.fmt(f),
        }
    }
}

impl<'de> Deserialize<'de> for AuthData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
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
            Some("m.login.email.identity") => from_raw_json_value(&json).map(Self::EmailIdentity),
            Some("m.login.msisdn") => from_raw_json_value(&json).map(Self::Msisdn),
            Some("m.login.dummy") => from_raw_json_value(&json).map(Self::Dummy),
            Some("m.login.registration_token") => {
                from_raw_json_value(&json).map(Self::RegistrationToken)
            }
            None => from_raw_json_value(&json).map(Self::FallbackAcknowledgement),
            Some(_) => from_raw_json_value(&json).map(Self::_Custom),
        }
    }
}

/// The type of an authentication stage.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
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

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// Data for password-based UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://spec.matrix.org/latest/client-server-api/#password-based
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.password")]
pub struct Password {
    /// One of the user's identifiers.
    pub identifier: UserIdentifier,

    /// The plaintext password.
    pub password: String,

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<String>,
}

impl Password {
    /// Creates a new `Password` with the given identifier and password.
    pub fn new(identifier: UserIdentifier, password: String) -> Self {
        Self { identifier, password, session: None }
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { identifier, password: _, session } = self;
        f.debug_struct("Password")
            .field("identifier", identifier)
            .field("session", session)
            .finish_non_exhaustive()
    }
}

/// Data for ReCaptcha UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://spec.matrix.org/latest/client-server-api/#google-recaptcha
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.recaptcha")]
pub struct ReCaptcha {
    /// The captcha response.
    pub response: String,

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<String>,
}

impl ReCaptcha {
    /// Creates a new `ReCaptcha` with the given response string.
    pub fn new(response: String) -> Self {
        Self { response, session: None }
    }
}

impl fmt::Debug for ReCaptcha {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { response: _, session } = self;
        f.debug_struct("ReCaptcha").field("session", session).finish_non_exhaustive()
    }
}

/// Data for Email-based UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://spec.matrix.org/latest/client-server-api/#email-based-identity--homeserver
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.email.identity")]
pub struct EmailIdentity {
    /// Thirdparty identifier credentials.
    #[serde(rename = "threepid_creds")]
    pub thirdparty_id_creds: ThirdpartyIdCredentials,

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<String>,
}

/// Data for phone number-based UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://spec.matrix.org/latest/client-server-api/#phone-numbermsisdn-based-identity--homeserver
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.msisdn")]
pub struct Msisdn {
    /// Thirdparty identifier credentials.
    #[serde(rename = "threepid_creds")]
    pub thirdparty_id_creds: ThirdpartyIdCredentials,

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<String>,
}

/// Data for dummy UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://spec.matrix.org/latest/client-server-api/#dummy-auth
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.dummy")]
pub struct Dummy {
    /// The value of the session key given by the homeserver, if any.
    pub session: Option<String>,
}

impl Dummy {
    /// Creates an empty `Dummy`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Data for registration token-based UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://spec.matrix.org/latest/client-server-api/#token-authenticated-registration
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.registration_token")]
pub struct RegistrationToken {
    /// The registration token.
    pub token: String,

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<String>,
}

impl RegistrationToken {
    /// Creates a new `RegistrationToken` with the given token.
    pub fn new(token: String) -> Self {
        Self { token, session: None }
    }
}

impl fmt::Debug for RegistrationToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { token: _, session } = self;
        f.debug_struct("RegistrationToken").field("session", session).finish_non_exhaustive()
    }
}

/// Data for UIAA fallback acknowledgement.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://spec.matrix.org/latest/client-server-api/#fallback
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct FallbackAcknowledgement {
    /// The value of the session key given by the homeserver.
    pub session: String,
}

impl FallbackAcknowledgement {
    /// Creates a new `FallbackAcknowledgement` with the given session key.
    pub fn new(session: String) -> Self {
        Self { session }
    }
}

#[doc(hidden)]
#[derive(Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CustomAuthData {
    #[serde(rename = "type")]
    auth_type: String,
    session: Option<String>,
    #[serde(flatten)]
    extra: JsonObject,
}

impl fmt::Debug for CustomAuthData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { auth_type, session, extra: _ } = self;
        f.debug_struct("CustomAuthData")
            .field("auth_type", auth_type)
            .field("session", session)
            .finish_non_exhaustive()
    }
}

/// Identification information for the user.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum UserIdentifier {
    /// Either a fully qualified Matrix user ID, or just the localpart (as part of the 'identifier'
    /// field).
    UserIdOrLocalpart(String),

    /// An email address.
    Email {
        /// The email address.
        address: String,
    },

    /// A phone number in the MSISDN format.
    Msisdn {
        /// The phone number according to the [E.164] numbering plan.
        ///
        /// [E.164]: https://www.itu.int/rec/T-REC-E.164-201011-I/en
        number: String,
    },

    /// A phone number as a separate country code and phone number.
    ///
    /// The homeserver will be responsible for canonicalizing this to the MSISDN format.
    PhoneNumber {
        /// The country that the phone number is from.
        ///
        /// This is a two-letter uppercase [ISO-3166-1 alpha-2] country code.
        ///
        /// [ISO-3166-1 alpha-2]: https://www.iso.org/iso-3166-country-codes.html
        country: String,

        /// The phone number.
        phone: String,
    },

    #[doc(hidden)]
    _CustomThirdParty(CustomThirdPartyId),
}

impl UserIdentifier {
    /// Creates a new `UserIdentifier` from the given third party identifier.
    pub fn third_party_id(medium: Medium, address: String) -> Self {
        match medium {
            Medium::Email => Self::Email { address },
            Medium::Msisdn => Self::Msisdn { number: address },
            _ => Self::_CustomThirdParty(CustomThirdPartyId { medium, address }),
        }
    }

    /// Get this `UserIdentifier` as a third party identifier if it is one.
    pub fn as_third_party_id(&self) -> Option<(&Medium, &str)> {
        match self {
            Self::Email { address } => Some((&Medium::Email, address)),
            Self::Msisdn { number } => Some((&Medium::Msisdn, number)),
            Self::_CustomThirdParty(CustomThirdPartyId { medium, address }) => {
                Some((medium, address))
            }
            _ => None,
        }
    }
}

impl From<OwnedUserId> for UserIdentifier {
    fn from(id: OwnedUserId) -> Self {
        Self::UserIdOrLocalpart(id.into())
    }
}

impl From<&OwnedUserId> for UserIdentifier {
    fn from(id: &OwnedUserId) -> Self {
        Self::UserIdOrLocalpart(id.as_str().to_owned())
    }
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct CustomThirdPartyId {
    medium: Medium,
    address: String,
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct IncomingCustomThirdPartyId {
    medium: Medium,
    address: String,
}

/// Credentials for third-party authentication (e.g. email / phone number).
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ThirdpartyIdCredentials {
    /// Identity server session ID.
    pub sid: OwnedSessionId,

    /// Identity server client secret.
    pub client_secret: Box<ClientSecret>,

    /// Identity server URL.
    pub id_server: String,

    /// Identity server access token.
    pub id_access_token: String,
}

impl ThirdpartyIdCredentials {
    /// Creates a new `ThirdpartyIdCredentials` with the given session ID, client secret, identity
    /// server address and access token.
    pub fn new(
        sid: OwnedSessionId,
        client_secret: Box<ClientSecret>,
        id_server: String,
        id_access_token: String,
    ) -> Self {
        Self { sid, client_secret, id_server, id_access_token }
    }
}

impl fmt::Debug for ThirdpartyIdCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { sid, client_secret: _, id_server, id_access_token } = self;
        f.debug_struct("ThirdpartyIdCredentials")
            .field("sid", sid)
            .field("id_server", id_server)
            .field("id_access_token", id_access_token)
            .finish_non_exhaustive()
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
    pub completed: Vec<AuthType>,

    /// Authentication parameters required for the client to complete authentication.
    ///
    /// To create a `Box<RawJsonValue>`, use `serde_json::value::to_raw_value`.
    pub params: Box<RawJsonValue>,

    /// Session key for client to use to complete authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,

    /// Authentication-related errors for previous request returned by homeserver.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub auth_error: Option<StandardErrorBody>,
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
        if response.status() == http::StatusCode::UNAUTHORIZED {
            if let Ok(uiaa_info) = from_json_slice(response.body().as_ref()) {
                return Self::AuthResponse(uiaa_info);
            }
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
                .header(http::header::CONTENT_TYPE, "application/json")
                .status(&http::StatusCode::UNAUTHORIZED)
                .body(ruma_common::serde::json_to_buf(&authentication_info)?)
                .map_err(Into::into),
            UiaaResponse::MatrixError(error) => error.try_into_http_response(),
        }
    }
}
