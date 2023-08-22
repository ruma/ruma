//! Helper module for the Serialize / Deserialize impl's for the UserIdentifier struct
//! in the parent module.

use ruma_common::{serde::from_raw_json_value, thirdparty::Medium};
use serde::{de, ser::SerializeStruct, Deserialize, Deserializer, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use super::{CustomThirdPartyId, UserIdentifier};

impl Serialize for UserIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut id;
        match self {
            Self::UserIdOrLocalpart(user) => {
                id = serializer.serialize_struct("UserIdentifier", 2)?;
                id.serialize_field("type", "m.id.user")?;
                id.serialize_field("user", user)?;
            }
            Self::PhoneNumber { country, phone } => {
                id = serializer.serialize_struct("UserIdentifier", 3)?;
                id.serialize_field("type", "m.id.phone")?;
                id.serialize_field("country", country)?;
                id.serialize_field("phone", phone)?;
            }
            Self::Email { address } => {
                id = serializer.serialize_struct("UserIdentifier", 3)?;
                id.serialize_field("type", "m.id.thirdparty")?;
                id.serialize_field("medium", &Medium::Email)?;
                id.serialize_field("address", address)?;
            }
            Self::Msisdn { number } => {
                id = serializer.serialize_struct("UserIdentifier", 3)?;
                id.serialize_field("type", "m.id.thirdparty")?;
                id.serialize_field("medium", &Medium::Msisdn)?;
                id.serialize_field("address", number)?;
            }
            Self::_CustomThirdParty(CustomThirdPartyId { medium, address }) => {
                id = serializer.serialize_struct("UserIdentifier", 3)?;
                id.serialize_field("type", "m.id.thirdparty")?;
                id.serialize_field("medium", &medium)?;
                id.serialize_field("address", address)?;
            }
        }
        id.end()
    }
}

impl<'de> Deserialize<'de> for UserIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        #[derive(Deserialize)]
        #[serde(tag = "type")]
        enum ExtractType {
            #[serde(rename = "m.id.user")]
            User,
            #[serde(rename = "m.id.phone")]
            Phone,
            #[serde(rename = "m.id.thirdparty")]
            ThirdParty,
        }

        #[derive(Deserialize)]
        struct UserIdOrLocalpart {
            user: String,
        }

        #[derive(Deserialize)]
        struct ThirdPartyId {
            medium: Medium,
            address: String,
        }

        #[derive(Deserialize)]
        struct PhoneNumber {
            country: String,
            phone: String,
        }

        let id_type = serde_json::from_str::<ExtractType>(json.get()).map_err(de::Error::custom)?;

        match id_type {
            ExtractType::User => from_raw_json_value(&json)
                .map(|user_id: UserIdOrLocalpart| Self::UserIdOrLocalpart(user_id.user)),
            ExtractType::Phone => from_raw_json_value(&json)
                .map(|nb: PhoneNumber| Self::PhoneNumber { country: nb.country, phone: nb.phone }),
            ExtractType::ThirdParty => {
                let ThirdPartyId { medium, address } = from_raw_json_value(&json)?;
                match medium {
                    Medium::Email => Ok(Self::Email { address }),
                    Medium::Msisdn => Ok(Self::Msisdn { number: address }),
                    _ => Ok(Self::_CustomThirdParty(CustomThirdPartyId { medium, address })),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use crate::uiaa::UserIdentifier;

    #[test]
    fn serialize() {
        assert_eq!(
            to_json_value(UserIdentifier::UserIdOrLocalpart("@user:notareal.hs".to_owned()))
                .unwrap(),
            json!({
                "type": "m.id.user",
                "user": "@user:notareal.hs",
            })
        );

        assert_eq!(
            to_json_value(UserIdentifier::PhoneNumber {
                country: "33".to_owned(),
                phone: "0102030405".to_owned()
            })
            .unwrap(),
            json!({
                "type": "m.id.phone",
                "country": "33",
                "phone": "0102030405",
            })
        );

        assert_eq!(
            to_json_value(UserIdentifier::Email { address: "me@myprovider.net".to_owned() })
                .unwrap(),
            json!({
                "type": "m.id.thirdparty",
                "medium": "email",
                "address": "me@myprovider.net",
            })
        );

        assert_eq!(
            to_json_value(UserIdentifier::Msisdn { number: "330102030405".to_owned() }).unwrap(),
            json!({
                "type": "m.id.thirdparty",
                "medium": "msisdn",
                "address": "330102030405",
            })
        );

        assert_eq!(
            to_json_value(UserIdentifier::third_party_id("robot".into(), "01001110".to_owned()))
                .unwrap(),
            json!({
                "type": "m.id.thirdparty",
                "medium": "robot",
                "address": "01001110",
            })
        );
    }

    #[test]
    fn deserialize() {
        let json = json!({
            "type": "m.id.user",
            "user": "@user:notareal.hs",
        });
        assert_matches!(from_json_value(json), Ok(UserIdentifier::UserIdOrLocalpart(user)));
        assert_eq!(user, "@user:notareal.hs");

        let json = json!({
            "type": "m.id.phone",
            "country": "33",
            "phone": "0102030405",
        });
        assert_matches!(from_json_value(json), Ok(UserIdentifier::PhoneNumber { country, phone }));
        assert_eq!(country, "33");
        assert_eq!(phone, "0102030405");

        let json = json!({
            "type": "m.id.thirdparty",
            "medium": "email",
            "address": "me@myprovider.net",
        });
        assert_matches!(from_json_value(json), Ok(UserIdentifier::Email { address }));
        assert_eq!(address, "me@myprovider.net");

        let json = json!({
            "type": "m.id.thirdparty",
            "medium": "msisdn",
            "address": "330102030405",
        });
        assert_matches!(from_json_value(json), Ok(UserIdentifier::Msisdn { number }));
        assert_eq!(number, "330102030405");

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
}
