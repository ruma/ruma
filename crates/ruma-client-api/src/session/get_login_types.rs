//! `GET /_matrix/client/*/login`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3login

    use std::borrow::Cow;

    use ruma_common::{
        api::ruma_api,
        serde::{JsonObject, StringEnum},
        OwnedMxcUri,
    };
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_json::Value as JsonValue;

    use crate::PrivOwnedStr;

    ruma_api! {
        metadata: {
            description: "Gets the homeserver's supported login types to authenticate users. Clients should pick one of these and supply it as the type when logging in.",
            method: GET,
            name: "get_login_types",
            r0_path: "/_matrix/client/r0/login",
            stable_path: "/_matrix/client/v3/login",
            rate_limited: true,
            authentication: None,
            added: 1.0,
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
            Self {}
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
        /// Prefer to use the public variants of `LoginType` where possible; this constructor is
        /// meant be used for unsupported login types only and does not allow setting
        /// arbitrary data for supported ones.
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
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub identity_providers: Vec<IdentityProvider>,
    }

    impl SsoLoginType {
        /// Creates a new `PasswordLoginType`.
        pub fn new() -> Self {
            Self::default()
        }
    }

    /// An SSO login identity provider.
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct IdentityProvider {
        /// The ID of the provider.
        pub id: String,

        /// The display name of the provider.
        pub name: String,

        /// The icon for the provider.
        pub icon: Option<OwnedMxcUri>,

        /// The brand identifier for the provider.
        pub brand: Option<IdentityProviderBrand>,
    }

    impl IdentityProvider {
        /// Creates an `IdentityProvider` with the given `id` and `name`.
        pub fn new(id: String, name: String) -> Self {
            Self { id, name, icon: None, brand: None }
        }
    }

    /// An SSO login identity provider brand identifier.
    ///
    /// The predefined ones can be found in the matrix-spec-proposals repo in a [separate
    /// document][matrix-spec-proposals].
    ///
    /// [matrix-spec-proposals]: https://github.com/matrix-org/matrix-spec-proposals/blob/v1.1/informal/idp-brands.md
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub enum IdentityProviderBrand {
        /// The [Apple] brand.
        ///
        /// [Apple]: https://developer.apple.com/design/human-interface-guidelines/sign-in-with-apple/overview/buttons/
        #[ruma_enum(rename = "apple")]
        Apple,

        /// The [Facebook](https://developers.facebook.com/docs/facebook-login/web/login-button/) brand.
        #[ruma_enum(rename = "facebook")]
        Facebook,

        /// The [GitHub](https://github.com/logos) brand.
        #[ruma_enum(rename = "github")]
        GitHub,

        /// The [GitLab](https://about.gitlab.com/press/press-kit/) brand.
        #[ruma_enum(rename = "gitlab")]
        GitLab,

        /// The [Google](https://developers.google.com/identity/branding-guidelines) brand.
        #[ruma_enum(rename = "google")]
        Google,

        /// The [Twitter] brand.
        ///
        /// [Twitter]: https://developer.twitter.com/en/docs/authentication/guides/log-in-with-twitter#tab1
        #[ruma_enum(rename = "twitter")]
        Twitter,

        /// A custom brand.
        #[doc(hidden)]
        _Custom(PrivOwnedStr),
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
        #[serde(rename = "type")]
        pub type_: String,

        /// Remaining type content
        #[serde(flatten)]
        pub data: JsonObject,
    }

    mod login_type_serde {
        use ruma_common::serde::from_raw_json_value;
        use serde::{de, Deserialize};
        use serde_json::value::RawValue as RawJsonValue;

        use super::LoginType;

        /// Helper struct to determine the type from a `serde_json::value::RawValue`
        #[derive(Debug, Deserialize)]
        struct LoginTypeDeHelper {
            /// The login type field
            #[serde(rename = "type")]
            type_: String,
        }

        impl<'de> Deserialize<'de> for LoginType {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                let json = Box::<RawJsonValue>::deserialize(deserializer)?;
                let LoginTypeDeHelper { type_ } = from_raw_json_value(&json)?;

                Ok(match type_.as_ref() {
                    "m.login.password" => Self::Password(from_raw_json_value(&json)?),
                    "m.login.token" => Self::Token(from_raw_json_value(&json)?),
                    "m.login.sso" => Self::Sso(from_raw_json_value(&json)?),
                    _ => Self::_Custom(from_raw_json_value(&json)?),
                })
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use assert_matches::assert_matches;
        use serde::{Deserialize, Serialize};
        use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

        use super::{
            CustomLoginType, IdentityProvider, IdentityProviderBrand, LoginType, PasswordLoginType,
            SsoLoginType, TokenLoginType,
        };

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
        fn deserialize_custom_login_type() {
            assert_matches!(
                from_json_value::<Wrapper>(json!({
                    "flows": [
                        {
                            "type": "io.ruma.custom",
                            "color": "green",
                        }
                    ],
                })),
                Ok(Wrapper { flows })
                if flows.len() == 1
                    && matches!(
                        &flows[0],
                        LoginType::_Custom(CustomLoginType { type_, data })
                            if type_ == "io.ruma.custom"
                            && data == json!({ "color": "green" }).as_object().unwrap()
                    )
            );
        }

        #[test]
        fn deserialize_sso_login_type() {
            let mut wrapper = from_json_value::<Wrapper>(json!({
                "flows": [
                    {
                        "type": "m.login.sso",
                        "identity_providers": [
                            {
                                "id": "oidc-gitlab",
                                "name": "GitLab",
                                "icon": "mxc://localhost/gitlab-icon",
                                "brand": "gitlab"
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
                    && icon == "mxc://localhost/gitlab-icon"
            );
        }

        #[test]
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
                            "identity_providers": [
                                {
                                    "id": "oidc-github",
                                    "name": "GitHub",
                                    "icon": "mxc://localhost/github-icon",
                                    "brand": "github"
                                },
                            ]
                        }
                    ],
                })
            );
        }
    }
}
