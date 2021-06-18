//! [GET /_matrix/client/r0/login](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-login)

use std::borrow::Cow;

use ruma_api::ruma_api;
#[cfg(feature = "unstable-pre-spec")]
use ruma_identifiers::MxcUri;
#[cfg(feature = "unstable-pre-spec")]
use ruma_serde::StringEnum;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value as JsonValue;

type JsonObject = serde_json::Map<String, JsonValue>;

ruma_api! {
    metadata: {
        description: "Gets the homeserver's supported login types to authenticate users. Clients should pick one of these and supply it as the type when logging in.",
        method: GET,
        name: "get_login_types",
        path: "/_matrix/client/r0/login",
        rate_limited: true,
        authentication: None,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// The homeserver's supported login types.
        pub flows: Vec<LoginType>,
    }

    error: crate::Error
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self
    }
}

impl Response {
    /// Creates a new `Response` with the given login types.
    pub fn new(flows: Vec<LoginType>) -> Self {
        Self { flows }
    }
}

/// An authentication mechanism.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum LoginType {
    /// A password is supplied to authenticate.
    Password(PasswordLoginType),

    /// Token-based login.
    Token(TokenLoginType),

    /// SSO-based login.
    Sso(SsoLoginType),

    /// Custom login type.
    #[doc(hidden)]
    _Custom(CustomLoginType),
}

impl LoginType {
    /// Creates a new `LoginType` with the given `login_type` string and data.
    ///
    /// Prefer to use the public variants of `LoginType` where possible; this constructor is meant
    /// be used for unsupported login types only and does not allow setting arbitrary data for
    /// supported ones.
    pub fn new(login_type: &str, data: JsonObject) -> serde_json::Result<Self> {
        fn from_json_object<T: DeserializeOwned>(obj: JsonObject) -> serde_json::Result<T> {
            serde_json::from_value(JsonValue::Object(obj))
        }

        Ok(match login_type {
            "m.login.password" => Self::Password(from_json_object(data)?),
            "m.login.token" => Self::Token(from_json_object(data)?),
            "m.login.sso" => Self::Sso(from_json_object(data)?),
            _ => Self::_Custom(CustomLoginType { type_: login_type.to_owned(), data }),
        })
    }

    /// Returns a reference to the `login_type` string.
    pub fn login_type(&self) -> &str {
        match self {
            Self::Password(_) => "m.login.password",
            Self::Token(_) => "m.login.token",
            Self::Sso(_) => "m.login.sso",
            Self::_Custom(c) => &c.type_,
        }
    }

    /// Returns the associated data.
    ///
    /// Prefer to use the public variants of `LoginType` where possible; this method is meant to
    /// be used for unsupported login types only.
    pub fn data(&self) -> Cow<'_, JsonObject> {
        fn serialize<T: Serialize>(obj: &T) -> JsonObject {
            match serde_json::to_value(obj).expect("login type serialization to succeed") {
                JsonValue::Object(obj) => obj,
                _ => panic!("all login types must serialize to objects"),
            }
        }

        match self {
            Self::Password(d) => Cow::Owned(serialize(d)),
            Self::Token(d) => Cow::Owned(serialize(d)),
            Self::Sso(d) => Cow::Owned(serialize(d)),
            Self::_Custom(c) => Cow::Borrowed(&c.data),
        }
    }
}

/// The payload for password login.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.password")]
pub struct PasswordLoginType {}

impl PasswordLoginType {
    /// Creates a new `PasswordLoginType`.
    pub fn new() -> Self {
        Self {}
    }
}

/// The payload for token-based login.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.token")]
pub struct TokenLoginType {}

impl TokenLoginType {
    /// Creates a new `PasswordLoginType`.
    pub fn new() -> Self {
        Self {}
    }
}

/// The payload for SSO login.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "m.login.sso")]
pub struct SsoLoginType {
    /// The identity provider choices.
    ///
    /// This uses the unstable prefix in
    /// [MSC2858](https://github.com/matrix-org/matrix-doc/pull/2858).
    #[cfg(feature = "unstable-pre-spec")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
    #[serde(
        default,
        rename = "org.matrix.msc2858.identity_providers",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub identity_providers: Vec<IdentityProvider>,
}

impl SsoLoginType {
    /// Creates a new `PasswordLoginType`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// An SSO login identity provider.
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IdentityProvider {
    /// The ID of the provider.
    id: String,

    /// The display name of the provider.
    name: String,

    /// The icon for the provider.
    icon: Option<MxcUri>,

    /// The brand identifier for the provider.
    brand: Option<IdentityProviderBrand>,
}

#[cfg(feature = "unstable-pre-spec")]
impl IdentityProvider {
    /// Creates an `IdentityProvider` with the given `id` and `name`.
    pub fn new(id: String, name: String) -> Self {
        Self { id, name, icon: None, brand: None }
    }
}

/// An SSO login identity provider brand identifier.
///
/// This uses the unstable prefix in
/// [MSC2858](https://github.com/matrix-org/matrix-doc/pull/2858).
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum IdentityProviderBrand {
    /// The [Apple] brand.
    ///
    /// [Apple]: https://developer.apple.com/design/human-interface-guidelines/sign-in-with-apple/overview/buttons/
    #[ruma_enum(rename = "org.matrix.apple")]
    Apple,

    /// The [Facebook](https://developers.facebook.com/docs/facebook-login/web/login-button/) brand.
    #[ruma_enum(rename = "org.matrix.facebook")]
    Facebook,

    /// The [GitHub](https://github.com/logos) brand.
    #[ruma_enum(rename = "org.matrix.github")]
    GitHub,

    /// The [GitLab](https://about.gitlab.com/press/press-kit/) brand.
    #[ruma_enum(rename = "org.matrix.gitlab")]
    GitLab,

    /// The [Google](https://developers.google.com/identity/branding-guidelines) brand.
    #[ruma_enum(rename = "org.matrix.google")]
    Google,

    /// The [Twitter] brand.
    ///
    /// [Twitter]: https://developer.twitter.com/en/docs/authentication/guides/log-in-with-twitter#tab1
    #[ruma_enum(rename = "org.matrix.twitter")]
    Twitter,

    /// A custom brand.
    #[doc(hidden)]
    _Custom(String),
}

/// A custom login payload.
#[doc(hidden)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct CustomLoginType {
    /// A custom type
    ///
    /// This field is named `type_` instead of `type` because the latter is a reserved
    /// keyword in Rust.
    #[serde(rename = "override")]
    pub type_: String,

    /// Remaining type content
    #[serde(flatten)]
    pub data: JsonObject,
}

mod login_type_serde;

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use serde::{Deserialize, Serialize};
    #[cfg(feature = "unstable-pre-spec")]
    use serde_json::to_value as to_json_value;
    use serde_json::{from_value as from_json_value, json};

    #[cfg(feature = "unstable-pre-spec")]
    use super::{IdentityProvider, IdentityProviderBrand, SsoLoginType, TokenLoginType};
    use super::{LoginType, PasswordLoginType};

    #[derive(Debug, Deserialize, Serialize)]
    struct Wrapper {
        pub flows: Vec<LoginType>,
    }

    #[test]
    fn deserialize_password_login_type() {
        assert_matches!(
            from_json_value::<Wrapper>(json!({
                "flows": [
                    { "type": "m.login.password" }
                ],
            })),
            Ok(Wrapper { flows })
            if flows.len() == 1
                && matches!(flows[0], LoginType::Password(PasswordLoginType {}))
        );
    }

    #[test]
    #[cfg(feature = "unstable-pre-spec")]
    fn deserialize_sso_login_type() {
        let mut wrapper = from_json_value::<Wrapper>(json!({
            "flows": [
                {
                    "type": "m.login.sso",
                    "org.matrix.msc2858.identity_providers": [
                        {
                            "id": "oidc-gitlab",
                            "name": "GitLab",
                            "icon": "mxc://localhost/gitlab-icon",
                            "brand": "org.matrix.gitlab"
                        },
                        {
                            "id": "custom",
                            "name": "Custom",
                        }
                    ]
                }
            ],
        }))
        .unwrap();

        let flow = wrapper.flows.pop();
        assert_matches!(wrapper.flows.as_slice(), []);

        let mut identity_providers = match flow {
            Some(LoginType::Sso(SsoLoginType { identity_providers })) => identity_providers,
            _ => panic!("unexpected enum variant: {:?}", flow),
        };

        let provider = identity_providers.pop();
        assert_matches!(
            provider,
            Some(IdentityProvider {
                id,
                name,
                icon: None,
                brand: None,
            }) if id == "custom"
                && name == "Custom"
        );

        let provider = identity_providers.pop();
        assert_matches!(
            provider,
            Some(IdentityProvider {
                id,
                name,
                icon: Some(icon),
                brand: Some(IdentityProviderBrand::GitLab),
            }) if id == "oidc-gitlab"
                && name == "GitLab"
                && icon.to_string() == "mxc://localhost/gitlab-icon"
        );
    }

    #[test]
    #[cfg(feature = "unstable-pre-spec")]
    fn serialize_sso_login_type() {
        let wrapper = to_json_value(Wrapper {
            flows: vec![
                LoginType::Token(TokenLoginType {}),
                LoginType::Sso(SsoLoginType {
                    identity_providers: vec![IdentityProvider {
                        id: "oidc-github".into(),
                        name: "GitHub".into(),
                        icon: Some("mxc://localhost/github-icon".into()),
                        brand: Some(IdentityProviderBrand::GitHub),
                    }],
                }),
            ],
        })
        .unwrap();

        assert_eq!(
            wrapper,
            json!({
                "flows": [
                    {
                        "type": "m.login.token"
                    },
                    {
                        "type": "m.login.sso",
                        "org.matrix.msc2858.identity_providers": [
                            {
                                "id": "oidc-github",
                                "name": "GitHub",
                                "icon": "mxc://localhost/github-icon",
                                "brand": "org.matrix.github"
                            },
                        ]
                    }
                ],
            })
        );
    }
}
