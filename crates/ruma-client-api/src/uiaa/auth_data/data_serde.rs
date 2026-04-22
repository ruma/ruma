//! Custom Serialize / Deserialize implementations for the authentication data types.

use std::borrow::Cow;

use as_variant::as_variant;
use ruma_common::{
    serde::{JsonObject, from_raw_json_value},
    thirdparty::Medium,
};
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::{Value as JsonValue, value::RawValue as RawJsonValue};

use super::{
    AuthData, CustomThirdPartyUserIdentifier, EmailUserIdentifier, MsisdnUserIdentifier,
    UserIdentifier,
};
use crate::uiaa::{CustomAuthData, CustomUserIdentifier};

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

        let ExtractType { auth_type } = from_raw_json_value(&json)?;

        match auth_type.as_deref() {
            Some("m.login.password") => from_raw_json_value(&json).map(Self::Password),
            Some("m.login.recaptcha") => from_raw_json_value(&json).map(Self::ReCaptcha),
            Some("m.login.email.identity") => from_raw_json_value(&json).map(Self::EmailIdentity),
            Some("m.login.msisdn") => from_raw_json_value(&json).map(Self::Msisdn),
            Some("m.login.dummy") => from_raw_json_value(&json).map(Self::Dummy),
            Some("m.login.registration_token") => {
                from_raw_json_value(&json).map(Self::RegistrationToken)
            }
            Some("m.login.terms") => from_raw_json_value(&json).map(Self::Terms),
            Some("m.oauth" | "org.matrix.cross_signing_reset") => {
                from_raw_json_value(&json).map(Self::OAuth)
            }
            None => from_raw_json_value(&json).map(Self::FallbackAcknowledgement),
            Some(_) => {
                let mut data = from_raw_json_value::<JsonObject, _>(&json)?;
                let auth_type = as_variant!(
                    data.remove("type").expect("we already checked that the type field is present"),
                    JsonValue::String
                )
                .expect("we already checked that the type is a string");
                let session = data
                    .remove("session")
                    .and_then(|session| as_variant!(session, JsonValue::String));

                Ok(Self::_Custom(CustomAuthData { auth_type, session, data }))
            }
        }
    }
}

impl<'de> Deserialize<'de> for UserIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ExtractType<'a> {
            #[serde(borrow, rename = "type")]
            identifier_type: Cow<'a, str>,
        }

        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let ExtractType { identifier_type } =
            serde_json::from_str(json.get()).map_err(de::Error::custom)?;

        match identifier_type.as_ref() {
            "m.id.user" => from_raw_json_value(&json).map(Self::Matrix),
            "m.id.phone" => from_raw_json_value(&json).map(Self::PhoneNumber),
            "m.id.thirdparty" => {
                let id: CustomThirdPartyUserIdentifier = from_raw_json_value(&json)?;
                match &id.medium {
                    Medium::Email => Ok(Self::Email(EmailUserIdentifier { address: id.address })),
                    Medium::Msisdn => Ok(Self::Msisdn(MsisdnUserIdentifier { number: id.address })),
                    _ => Ok(Self::_CustomThirdParty(id)),
                }
            }
            _ => {
                let mut data = from_raw_json_value::<JsonObject, _>(&json)?;
                let identifier_type = as_variant!(
                    data.remove("type").expect("we already checked that the type field is present"),
                    JsonValue::String
                )
                .expect("we already checked that the type is a string");

                Ok(Self::_Custom(CustomUserIdentifier { identifier_type, data }))
            }
        }
    }
}

impl Serialize for EmailUserIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Self { address } = self;

        CustomThirdPartyUserIdentifier { medium: Medium::Email, address: address.clone() }
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EmailUserIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let CustomThirdPartyUserIdentifier { medium, address } =
            CustomThirdPartyUserIdentifier::deserialize(deserializer)?;

        if medium != Medium::Email {
            return Err(de::Error::invalid_value(
                de::Unexpected::Str(medium.as_str()),
                &Medium::Email.as_str(),
            ));
        }

        Ok(Self { address })
    }
}

impl Serialize for MsisdnUserIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Self { number } = self;

        CustomThirdPartyUserIdentifier { medium: Medium::Msisdn, address: number.clone() }
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MsisdnUserIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let CustomThirdPartyUserIdentifier { medium, address } =
            CustomThirdPartyUserIdentifier::deserialize(deserializer)?;

        if medium != Medium::Msisdn {
            return Err(de::Error::invalid_value(
                de::Unexpected::Str(medium.as_str()),
                &Medium::Msisdn.as_str(),
            ));
        }

        Ok(Self { number: address })
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_let;
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{Value as JsonValue, from_value as from_json_value, json};

    use crate::uiaa::{
        AuthData, EmailUserIdentifier, MatrixUserIdentifier, MsisdnUserIdentifier,
        PhoneNumberUserIdentifier, UserIdentifier,
    };

    #[test]
    fn serialize_user_identifier() {
        assert_to_canonical_json_eq!(
            UserIdentifier::Matrix(MatrixUserIdentifier::new("@user:notareal.hs".to_owned())),
            json!({
                "type": "m.id.user",
                "user": "@user:notareal.hs",
            })
        );

        assert_to_canonical_json_eq!(
            UserIdentifier::PhoneNumber(PhoneNumberUserIdentifier::new(
                "33".to_owned(),
                "0102030405".to_owned()
            )),
            json!({
                "type": "m.id.phone",
                "country": "33",
                "phone": "0102030405",
            })
        );

        assert_to_canonical_json_eq!(
            UserIdentifier::Email(EmailUserIdentifier::new("me@myprovider.net".to_owned())),
            json!({
                "type": "m.id.thirdparty",
                "medium": "email",
                "address": "me@myprovider.net",
            })
        );

        assert_to_canonical_json_eq!(
            UserIdentifier::Msisdn(MsisdnUserIdentifier::new("330102030405".to_owned())),
            json!({
                "type": "m.id.thirdparty",
                "medium": "msisdn",
                "address": "330102030405",
            })
        );

        assert_to_canonical_json_eq!(
            UserIdentifier::third_party_id("robot".into(), "01001110".to_owned()),
            json!({
                "type": "m.id.thirdparty",
                "medium": "robot",
                "address": "01001110",
            })
        );
    }

    #[test]
    fn deserialize_user_identifier() {
        let json = json!({
            "type": "m.id.user",
            "user": "@user:notareal.hs",
        });
        assert_let!(Ok(UserIdentifier::Matrix(id)) = from_json_value(json));
        assert_eq!(id.user, "@user:notareal.hs");

        let json = json!({
            "type": "m.id.phone",
            "country": "33",
            "phone": "0102030405",
        });
        assert_let!(
            Ok(UserIdentifier::PhoneNumber(PhoneNumberUserIdentifier { country, phone })) =
                from_json_value(json)
        );
        assert_eq!(country, "33");
        assert_eq!(phone, "0102030405");

        let json = json!({
            "type": "m.id.thirdparty",
            "medium": "email",
            "address": "me@myprovider.net",
        });
        assert_let!(Ok(UserIdentifier::Email(id)) = from_json_value(json));
        assert_eq!(id.address, "me@myprovider.net");

        let json = json!({
            "type": "m.id.thirdparty",
            "medium": "msisdn",
            "address": "330102030405",
        });
        assert_let!(Ok(UserIdentifier::Msisdn(id)) = from_json_value(json));
        assert_eq!(id.number, "330102030405");

        let json = json!({
            "type": "m.id.thirdparty",
            "medium": "robot",
            "address": "01110010",
        });
        let id = from_json_value::<UserIdentifier>(json).unwrap();
        let (medium, address) = id.as_third_party_id().unwrap();
        assert_eq!(medium.as_str(), "robot");
        assert_eq!(address, "01110010");
    }

    #[test]
    fn custom_user_identifier_roundtrip() {
        let json = json!({
            "type": "local.dev.identifier",
            "foo": "bar",
        });

        let id = from_json_value::<UserIdentifier>(json.clone()).unwrap();
        assert_eq!(id.identifier_type(), "local.dev.identifier");
        let data = &*id.data();
        assert_let!(Some(JsonValue::String(foo)) = data.get("foo"));
        assert_eq!(foo, "bar");

        assert_to_canonical_json_eq!(id, json);
    }

    #[test]
    fn custom_auth_data_roundtrip() {
        let json = json!({
            "type": "local.dev.auth",
            "session": "abcdef",
            "foo": "bar",
        });

        let auth_data = from_json_value::<AuthData>(json.clone()).unwrap();
        assert_eq!(auth_data.auth_type().unwrap().as_str(), "local.dev.auth");
        assert_eq!(auth_data.session(), Some("abcdef"));
        let data = auth_data.data();
        assert_eq!(data.len(), 1);
        assert_let!(Some(JsonValue::String(foo)) = data.get("foo"));
        assert_eq!(foo, "bar");

        assert_to_canonical_json_eq!(auth_data, json);
    }
}
