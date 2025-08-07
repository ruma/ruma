use std::str::FromStr;

use ruma_common::{OwnedUserId, UserId};
use serde::{
    de::{self, Deserialize, Deserializer, Unexpected},
    Serialize, Serializer,
};
/// A type that can be used as the `state_key` for call member state events.
/// Those state keys can be a combination of UserId and DeviceId.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::exhaustive_structs)]
pub struct CallMemberStateKey {
    key: CallMemberStateKeyEnum,
    raw: Box<str>,
}

impl CallMemberStateKey {
    /// Constructs a new CallMemberStateKey there are three possible formats:
    /// - `_{UserId}_{MemberId}` example: `_@test:user.org_DEVICE_m.call`. `member_id:
    ///   Some(DEVICE_m.call)`, `underscore: true`
    /// - `{UserId}_{MemberId}` example: `@test:user.org_DEVICE_m.call`. `member_id:
    ///   Some(DEVICE_m.call)`, `underscore: false`
    /// - `{UserId}` example: `@test:user.org`. `member_id: None`, underscore is ignored:
    ///   `underscore: false|true`
    ///
    /// The MemberId is a combination of the UserId and the session information
    /// (session.application and session.id).
    /// The session information is an opaque string that should not be parsed after creation.
    pub fn new(user_id: OwnedUserId, member_id: Option<String>, underscore: bool) -> Self {
        CallMemberStateKeyEnum::new(user_id, member_id, underscore).into()
    }

    /// Returns the user id in this state key.
    /// (This is a cheap operations. The id is already type checked on initialization. And does
    /// only returns a reference to an existing OwnedUserId.)
    ///
    /// It is recommended to not use the state key to get the user id, but rather use the `sender`
    /// field.
    pub fn user_id(&self) -> &UserId {
        match &self.key {
            CallMemberStateKeyEnum::UnderscoreMemberId(u, _) => u,
            CallMemberStateKeyEnum::MemberId(u, _) => u,
            CallMemberStateKeyEnum::User(u) => u,
        }
    }
}

impl AsRef<str> for CallMemberStateKey {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}

impl From<CallMemberStateKeyEnum> for CallMemberStateKey {
    fn from(value: CallMemberStateKeyEnum) -> Self {
        let raw = value.to_string().into();
        Self { key: value, raw }
    }
}

impl FromStr for CallMemberStateKey {
    type Err = KeyParseError;

    fn from_str(state_key: &str) -> Result<Self, Self::Err> {
        // Intentionally do not use CallMemberStateKeyEnum.into since this would reconstruct the
        // state key string.
        Ok(Self { key: CallMemberStateKeyEnum::from_str(state_key)?, raw: state_key.into() })
    }
}

impl<'de> Deserialize<'de> for CallMemberStateKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = ruma_common::serde::deserialize_cow_str(deserializer)?;
        Self::from_str(&s).map_err(|err| de::Error::invalid_value(Unexpected::Str(&s), &err))
    }
}

impl Serialize for CallMemberStateKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

/// This enum represents all possible formats for a call member event state key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CallMemberStateKeyEnum {
    UnderscoreMemberId(OwnedUserId, String),
    MemberId(OwnedUserId, String),
    User(OwnedUserId),
}

impl CallMemberStateKeyEnum {
    fn new(user_id: OwnedUserId, unique_member_id: Option<String>, underscore: bool) -> Self {
        match (unique_member_id, underscore) {
            (Some(member_id), true) => {
                CallMemberStateKeyEnum::UnderscoreMemberId(user_id, member_id)
            }
            (Some(member_id), false) => CallMemberStateKeyEnum::MemberId(user_id, member_id),
            (None, _) => CallMemberStateKeyEnum::User(user_id),
        }
    }
}

impl std::fmt::Display for CallMemberStateKeyEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            CallMemberStateKeyEnum::UnderscoreMemberId(u, d) => write!(f, "_{u}_{d}"),
            CallMemberStateKeyEnum::MemberId(u, d) => write!(f, "{u}_{d}"),
            CallMemberStateKeyEnum::User(u) => f.write_str(u.as_str()),
        }
    }
}

impl FromStr for CallMemberStateKeyEnum {
    type Err = KeyParseError;

    fn from_str(state_key: &str) -> Result<Self, Self::Err> {
        // Ignore leading underscore if present
        // (used for avoiding auth rules on @-prefixed state keys)
        let (state_key, has_underscore) = match state_key.strip_prefix('_') {
            Some(s) => (s, true),
            None => (state_key, false),
        };

        // Fail early if we cannot find the index of the ":"
        let Some(colon_idx) = state_key.find(':') else {
            return Err(KeyParseError::InvalidUser {
                user_id: state_key.to_owned(),
                error: ruma_common::IdParseError::MissingColon,
            });
        };

        let (user_id, member_id) = match state_key[colon_idx + 1..].find('_') {
            None => {
                return match UserId::parse(state_key) {
                    Ok(user_id) => {
                        if has_underscore {
                            Err(KeyParseError::LeadingUnderscoreNoMemberId)
                        } else {
                            Ok(CallMemberStateKeyEnum::new(user_id, None, has_underscore))
                        }
                    }
                    Err(err) => Err(KeyParseError::InvalidUser {
                        error: err,
                        user_id: state_key.to_owned(),
                    }),
                };
            }
            Some(suffix_idx) => {
                (&state_key[..colon_idx + 1 + suffix_idx], &state_key[colon_idx + 2 + suffix_idx..])
            }
        };

        match UserId::parse(user_id) {
            Ok(user_id) => {
                if member_id.is_empty() {
                    return Err(KeyParseError::EmptyMemberId);
                }
                Ok(CallMemberStateKeyEnum::new(user_id, Some(member_id.to_owned()), has_underscore))
            }
            Err(err) => Err(KeyParseError::InvalidUser { user_id: user_id.to_owned(), error: err }),
        }
    }
}

/// Error when trying to parse a call member state key.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum KeyParseError {
    /// The user part of the state key is invalid.
    #[error("uses a malformatted UserId in the UserId defined section.")]
    InvalidUser {
        /// The user Id that the parser thinks it should have parsed.
        user_id: String,
        /// The user Id parse error why if failed to parse it.
        error: ruma_common::IdParseError,
    },
    /// Uses a leading underscore but no trailing device id. The part after the underscore is a
    /// valid user id.
    #[error(
        "uses a leading underscore but no trailing device id. The part after the underscore is a valid user id."
    )]
    LeadingUnderscoreNoMemberId,
    /// Uses an empty memberId. (UserId with trailing underscore)
    #[error("uses an empty memberId. (UserId with trailing underscore)")]
    EmptyMemberId,
}

impl de::Expected for KeyParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "correct call member event key format. The provided string, {self})")
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ruma_common::user_id;

    use crate::call::member::{member_state_key::CallMemberStateKeyEnum, CallMemberStateKey};

    #[test]
    fn convert_state_key_enum_to_state_key() {
        let key = "_@user:domain.org_ABC";
        let state_key_enum = CallMemberStateKeyEnum::from_str(key).unwrap();
        // This generates state_key.raw from the enum
        let state_key: CallMemberStateKey = state_key_enum.into();
        // This compares state_key.raw (generated) with key (original)
        assert_eq!(state_key.as_ref(), key);
        // Compare to the from string without `CallMemberStateKeyEnum` step.
        let state_key_direct = CallMemberStateKey::new(
            user_id!("@user:domain.org").to_owned(),
            Some("ABC".to_owned()),
            true,
        );
        assert_eq!(state_key, state_key_direct);
    }

    #[test]
    fn convert_no_underscore_state_key_without_member_id() {
        let key = "@user:domain.org";
        let state_key_enum = CallMemberStateKeyEnum::from_str(key).unwrap();
        // This generates state_key.raw from the enum
        let state_key: CallMemberStateKey = state_key_enum.into();
        // This compares state_key.raw (generated) with key (original)
        assert_eq!(state_key.as_ref(), key);
        // Compare to the from string without `CallMemberStateKeyEnum` step.
        let state_key_direct =
            CallMemberStateKey::new(user_id!("@user:domain.org").to_owned(), None, false);
        assert_eq!(state_key, state_key_direct);
    }

    #[test]
    fn convert_no_underscore_state_key_with_member_id() {
        let key = "@user:domain.org_ABC_m.callTestId";
        let state_key_enum = CallMemberStateKeyEnum::from_str(key).unwrap();
        // This generates state_key.raw from the enum
        let state_key: CallMemberStateKey = state_key_enum.into();
        // This compares state_key.raw (generated) with key (original)
        assert_eq!(state_key.as_ref(), key);
        // Compare to the from string without `CallMemberStateKeyEnum` step.
        let state_key_direct = CallMemberStateKey::new(
            user_id!("@user:domain.org").to_owned(),
            Some("ABC_m.callTestId".to_owned()),
            false,
        );

        assert_eq!(state_key, state_key_direct);
    }
}
