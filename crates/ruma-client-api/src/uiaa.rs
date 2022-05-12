//! Module for [User-Interactive Authentication API][uiaa] types.
//!
//! [uiaa]: https://spec.matrix.org/v1.2/client-server-api/#user-interactive-authentication-api

use std::{borrow::Cow, fmt};

use bytes::BufMut;
use ruma_common::{
    api::{
        error::{DeserializationError, IntoHttpError},
        EndpointError, OutgoingResponse,
    },
    serde::{from_raw_json_value, Incoming, JsonObject, StringEnum},
    thirdparty::Medium,
    ClientSecret, OwnedSessionId, OwnedUserId, UserId,
};
use serde::{
    de::{self, DeserializeOwned},
    Deserialize, Deserializer, Serialize,
};
use serde_json::{
    from_slice as from_json_slice, value::RawValue as RawJsonValue, Value as JsonValue,
};

use crate::{
    error::{Error as MatrixError, ErrorBody},
    PrivOwnedStr,
};

pub mod get_uiaa_fallback_page;
mod user_serde;

/// Information for one authentication stage.
///
/// To construct the custom `AuthData` variant you first have to construct [`IncomingAuthData::new`]
/// and then call [`IncomingAuthData::to_outgoing`] on it.
#[derive(Clone, Debug, Incoming, Serialize)]
#[non_exhaustive]
#[incoming_derive(!Deserialize)]
#[serde(untagged)]
pub enum AuthData<'a> {
    /// Password-based authentication (`m.login.password`).
    Password(Password<'a>),

    /// Google ReCaptcha 2.0 authentication (`m.login.recaptcha`).
    ReCaptcha(ReCaptcha<'a>),

    /// Email-based authentication (`m.login.email.identity`).
    EmailIdentity(EmailIdentity<'a>),

    /// Phone number-based authentication (`m.login.msisdn`).
    Msisdn(Msisdn<'a>),

    /// Dummy authentication (`m.login.dummy`).
    Dummy(Dummy<'a>),

    /// Registration token-based authentication (`m.login.registration_token`).
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
    pub fn auth_type(&self) -> Option<AuthType> {
        match self {
            Self::Password(_) => Some(AuthType::Password),
            Self::ReCaptcha(_) => Some(AuthType::ReCaptcha),
            Self::EmailIdentity(_) => Some(AuthType::EmailIdentity),
            Self::Msisdn(_) => Some(AuthType::Msisdn),
            Self::Dummy(_) => Some(AuthType::Dummy),
            Self::RegistrationToken(_) => Some(AuthType::RegistrationToken),
            Self::FallbackAcknowledgement(_) => None,
            Self::_Custom(c) => Some(AuthType::_Custom(PrivOwnedStr(c.auth_type.into()))),
        }
    }

    /// Returns the value of the `session` field, if it exists.
    pub fn session(&self) -> Option<&'a str> {
        match self {
            Self::Password(x) => x.session,
            Self::ReCaptcha(x) => x.session,
            Self::EmailIdentity(x) => x.session,
            Self::Msisdn(x) => x.session,
            Self::Dummy(x) => x.session,
            Self::RegistrationToken(x) => x.session,
            Self::FallbackAcknowledgement(x) => Some(x.session),
            Self::_Custom(x) => x.session,
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
                password: x.password,
                session: None,
            })),
            Self::ReCaptcha(x) => {
                Cow::Owned(serialize(ReCaptcha { response: x.response, session: None }))
            }
            Self::EmailIdentity(x) => Cow::Owned(serialize(EmailIdentity {
                thirdparty_id_creds: x.thirdparty_id_creds,
                session: None,
            })),
            Self::Msisdn(x) => Cow::Owned(serialize(Msisdn {
                thirdparty_id_creds: x.thirdparty_id_creds,
                session: None,
            })),
            Self::RegistrationToken(x) => {
                Cow::Owned(serialize(RegistrationToken { token: x.token, session: None }))
            }
            // Dummy and fallback acknowledgement have no associated data
            Self::Dummy(_) | Self::FallbackAcknowledgement(_) => Cow::Owned(JsonObject::default()),
            Self::_Custom(c) => Cow::Borrowed(c.extra),
        }
    }
}

impl IncomingAuthData {
    /// Creates a new `IncomingAuthData` with the given `auth_type` string, session and data.
    ///
    /// Prefer to use the public variants of `IncomingAuthData` where possible; this constructor is
    /// meant be used for unsupported authentication types only and does not allow setting arbitrary
    /// data for supported ones.
    ///
    /// # Errors
    ///
    /// Returns an error if the `auth_type` is known and serialization of `data` to the
    /// corresponding `IncomingAuthData` variant fails.
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
            _ => Self::_Custom(IncomingCustomAuthData {
                auth_type: auth_type.into(),
                session,
                extra: data,
            }),
        })
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
                identifier: x.identifier.to_outgoing(),
                password: &x.password,
                session: None,
            })),
            Self::ReCaptcha(x) => {
                Cow::Owned(serialize(ReCaptcha { response: &x.response, session: None }))
            }
            Self::EmailIdentity(x) => Cow::Owned(serialize(EmailIdentity {
                thirdparty_id_creds: &x.thirdparty_id_creds,
                session: None,
            })),
            Self::Msisdn(x) => Cow::Owned(serialize(Msisdn {
                thirdparty_id_creds: &x.thirdparty_id_creds,
                session: None,
            })),
            Self::RegistrationToken(x) => {
                Cow::Owned(serialize(RegistrationToken { token: &x.token, session: None }))
            }
            // Dummy and fallback acknowledgement have no associated data
            Self::Dummy(_) | Self::FallbackAcknowledgement(_) => Cow::Owned(JsonObject::default()),
            Self::_Custom(c) => Cow::Borrowed(&c.extra),
        }
    }

    /// Convert `IncomingAuthData` to `AuthData`.
    pub fn to_outgoing(&self) -> AuthData<'_> {
        match self {
            Self::Password(a) => AuthData::Password(a.to_outgoing()),
            Self::ReCaptcha(a) => AuthData::ReCaptcha(a.to_outgoing()),
            Self::EmailIdentity(a) => AuthData::EmailIdentity(a.to_outgoing()),
            Self::Msisdn(a) => AuthData::Msisdn(a.to_outgoing()),
            Self::Dummy(a) => AuthData::Dummy(a.to_outgoing()),
            Self::RegistrationToken(a) => AuthData::RegistrationToken(a.to_outgoing()),
            Self::FallbackAcknowledgement(a) => AuthData::FallbackAcknowledgement(a.to_outgoing()),
            Self::_Custom(a) => AuthData::_Custom(CustomAuthData {
                auth_type: &a.auth_type,
                session: a.session.as_deref(),
                extra: &a.extra,
            }),
        }
    }
}

impl<'de> Deserialize<'de> for IncomingAuthData {
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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
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
/// [the spec]: https://spec.matrix.org/v1.2/client-server-api/#password-based
#[derive(Clone, Debug, Incoming, Serialize)]
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

impl IncomingPassword {
    /// Convert `IncomingPassword` to `Password`.
    fn to_outgoing(&self) -> Password<'_> {
        Password {
            identifier: self.identifier.to_outgoing(),
            password: &self.password,
            session: self.session.as_deref(),
        }
    }
}

/// Data for ReCaptcha UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://spec.matrix.org/v1.2/client-server-api/#google-recaptcha
#[derive(Clone, Debug, Incoming, Serialize)]
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

impl IncomingReCaptcha {
    /// Convert `IncomingReCaptcha` to `ReCaptcha`.
    fn to_outgoing(&self) -> ReCaptcha<'_> {
        ReCaptcha { response: &self.response, session: self.session.as_deref() }
    }
}

/// Data for Email-based UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://spec.matrix.org/v1.2/client-server-api/#email-based-identity--homeserver
#[derive(Clone, Debug, Incoming, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.email.identity")]
pub struct EmailIdentity<'a> {
    /// Thirdparty identifier credentials.
    #[serde(rename = "threepid_creds")]
    pub thirdparty_id_creds: &'a ThirdpartyIdCredentials,

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<&'a str>,
}

impl IncomingEmailIdentity {
    /// Convert `IncomingEmailIdentity` to `EmailIdentity`.
    fn to_outgoing(&self) -> EmailIdentity<'_> {
        EmailIdentity {
            thirdparty_id_creds: &self.thirdparty_id_creds,
            session: self.session.as_deref(),
        }
    }
}

/// Data for phone number-based UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://spec.matrix.org/v1.2/client-server-api/#phone-numbermsisdn-based-identity--homeserver
#[derive(Clone, Debug, Incoming, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.msisdn")]
pub struct Msisdn<'a> {
    /// Thirdparty identifier credentials.
    #[serde(rename = "threepid_creds")]
    pub thirdparty_id_creds: &'a ThirdpartyIdCredentials,

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<&'a str>,
}

impl IncomingMsisdn {
    /// Convert `IncomingMsisdn` to `Msisdn`.
    fn to_outgoing(&self) -> Msisdn<'_> {
        Msisdn { thirdparty_id_creds: &self.thirdparty_id_creds, session: self.session.as_deref() }
    }
}

/// Data for dummy UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://spec.matrix.org/v1.2/client-server-api/#dummy-auth
#[derive(Clone, Debug, Default, Incoming, Serialize)]
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

impl IncomingDummy {
    /// Convert from `IncomingDummy` to `Dummy`.
    fn to_outgoing(&self) -> Dummy<'_> {
        Dummy { session: self.session.as_deref() }
    }
}

/// Data for registration token-based UIAA flow.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://spec.matrix.org/v1.2/client-server-api/#token-authenticated-registration
#[derive(Clone, Debug, Incoming, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.registration_token")]
pub struct RegistrationToken<'a> {
    /// The registration token.
    pub token: &'a str,

    /// The value of the session key given by the homeserver, if any.
    pub session: Option<&'a str>,
}

impl<'a> RegistrationToken<'a> {
    /// Creates a new `RegistrationToken` with the given token.
    pub fn new(token: &'a str) -> Self {
        Self { token, session: None }
    }
}

impl IncomingRegistrationToken {
    /// Convert from `IncomingRegistrationToken` to `RegistrationToken`.
    fn to_outgoing(&self) -> RegistrationToken<'_> {
        RegistrationToken { token: &self.token, session: self.session.as_deref() }
    }
}

/// Data for UIAA fallback acknowledgement.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://spec.matrix.org/v1.2/client-server-api/#fallback
#[derive(Clone, Debug, Incoming, Serialize)]
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

impl IncomingFallbackAcknowledgement {
    /// Convert from `IncomingFallbackAcknowledgement` to `FallbackAcknowledgement`.
    fn to_outgoing(&self) -> FallbackAcknowledgement<'_> {
        FallbackAcknowledgement { session: &self.session }
    }
}

#[doc(hidden)]
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct CustomAuthData<'a> {
    #[serde(rename = "type")]
    auth_type: &'a str,
    session: Option<&'a str>,
    #[serde(flatten)]
    extra: &'a JsonObject,
}

#[doc(hidden)]
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct IncomingCustomAuthData {
    #[serde(rename = "type")]
    auth_type: String,
    session: Option<String>,
    #[serde(flatten)]
    extra: JsonObject,
}

/// Identification information for the user.
#[derive(Clone, Debug, PartialEq, Eq, Incoming, Serialize)]
#[serde(from = "user_serde::IncomingUserIdentifier", into = "user_serde::UserIdentifier<'_>")]
#[allow(clippy::exhaustive_enums)]
pub enum UserIdentifier<'a> {
    /// Either a fully qualified Matrix user ID, or just the localpart (as part of the 'identifier'
    /// field).
    UserIdOrLocalpart(&'a str),

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

impl<'a> UserIdentifier<'a> {
    /// Creates a [`UserIdentifier::ThirdPartyId`] from an email address.
    pub fn email(address: &'a str) -> Self {
        Self::ThirdPartyId { address, medium: Medium::Email }
    }
}

impl<'a> From<&'a UserId> for UserIdentifier<'a> {
    fn from(id: &'a UserId) -> Self {
        Self::UserIdOrLocalpart(id.as_str())
    }
}

impl<'a> From<&'a OwnedUserId> for UserIdentifier<'a> {
    fn from(id: &'a OwnedUserId) -> Self {
        Self::UserIdOrLocalpart(id.as_str())
    }
}

impl IncomingUserIdentifier {
    pub(crate) fn to_outgoing(&self) -> UserIdentifier<'_> {
        match self {
            Self::UserIdOrLocalpart(id) => UserIdentifier::UserIdOrLocalpart(id),
            Self::ThirdPartyId { address, medium } => {
                UserIdentifier::ThirdPartyId { address, medium: medium.clone() }
            }
            Self::PhoneNumber { country, phone } => UserIdentifier::PhoneNumber { country, phone },
        }
    }
}

/// Credentials for third-party authentication (e.g. email / phone number).
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    ) -> Result<Self, DeserializationError> {
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
                .body(ruma_common::serde::json_to_buf(&authentication_info)?)
                .map_err(Into::into),
            UiaaResponse::MatrixError(error) => error.try_into_http_response(),
        }
    }
}
