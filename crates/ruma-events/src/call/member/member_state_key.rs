use std::str::FromStr;

use ruma_common::{DeviceId, OwnedDeviceId, OwnedUserId, UserId};
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
    /// - `_{UserId}_{DeviceId}` example: `_@test:user.org_DEVICE`. `device_id: Some`, `underscore:
    ///   true`
    /// - `{UserId}_{DeviceId}` example: `@test:user.org_DEVICE`. `device_id: Some`, `underscore:
    ///   false`
    /// - `{UserId}` example: `@test:user.org`. `device_id: None`, underscore is ignored:
    ///   `underscore: false|true`
    ///
    /// Dependent on the parameters the correct CallMemberStateKey will be constructed.
    pub fn new(user_id: OwnedUserId, device_id: Option<OwnedDeviceId>, underscore: bool) -> Self {
        CallMemberStateKeyEnum::new(user_id, device_id, underscore).into()
    }

    /// Returns the user id in this state key.
    /// (This is a cheap operations. The id is already type checked on initialization. And does
    /// only returns a reference to an existing OwnedUserId.)
    pub fn user_id(&self) -> &UserId {
        match &self.key {
            CallMemberStateKeyEnum::UnderscoreUserDevice(u, _) => u,
            CallMemberStateKeyEnum::UserDevice(u, _) => u,
            CallMemberStateKeyEnum::User(u) => u,
        }
    }

    /// Returns the device id in this state key (if available)
    /// (This is a cheap operations. The id is already type checked on initialization. And does
    /// only returns a reference to an existing OwnedDeviceId.)
    pub fn device_id(&self) -> Option<&DeviceId> {
        match &self.key {
            CallMemberStateKeyEnum::UnderscoreUserDevice(_, d) => Some(d),
            CallMemberStateKeyEnum::UserDevice(_, d) => Some(d),
            CallMemberStateKeyEnum::User(_) => None,
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
    UnderscoreUserDevice(OwnedUserId, OwnedDeviceId),
    UserDevice(OwnedUserId, OwnedDeviceId),
    User(OwnedUserId),
}

impl CallMemberStateKeyEnum {
    fn new(user_id: OwnedUserId, device_id: Option<OwnedDeviceId>, underscore: bool) -> Self {
        match (device_id, underscore) {
            (Some(device_id), true) => {
                CallMemberStateKeyEnum::UnderscoreUserDevice(user_id, device_id)
            }
            (Some(device_id), false) => CallMemberStateKeyEnum::UserDevice(user_id, device_id),
            (None, _) => CallMemberStateKeyEnum::User(user_id),
        }
    }
}

impl std::fmt::Display for CallMemberStateKeyEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            CallMemberStateKeyEnum::UnderscoreUserDevice(u, d) => write!(f, "_{u}_{d}"),
            CallMemberStateKeyEnum::UserDevice(u, d) => write!(f, "{u}_{d}"),
            CallMemberStateKeyEnum::User(u) => f.write_str(u.as_str()),
        }
    }
}

impl FromStr for CallMemberStateKeyEnum {
    type Err = KeyParseError;

    fn from_str(state_key: &str) -> Result<Self, Self::Err> {
        // Ignore leading underscore if present
        // (used for avoiding auth rules on @-prefixed state keys)
        let (state_key, underscore) = match state_key.strip_prefix('_') {
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

        let (user_id, device_id) = match state_key[colon_idx + 1..].find('_') {
            None => {
                return match UserId::parse(state_key) {
                    Ok(user_id) => {
                        if underscore {
                            Err(KeyParseError::LeadingUnderscoreNoDevice)
                        } else {
                            Ok(CallMemberStateKeyEnum::new(user_id, None, underscore))
                        }
                    }
                    Err(err) => Err(KeyParseError::InvalidUser {
                        error: err,
                        user_id: state_key.to_owned(),
                    }),
                }
            }
            Some(suffix_idx) => {
                (&state_key[..colon_idx + 1 + suffix_idx], &state_key[colon_idx + 2 + suffix_idx..])
            }
        };

        match (UserId::parse(user_id), OwnedDeviceId::from(device_id)) {
            (Ok(user_id), device_id) => {
                if device_id.as_str().is_empty() {
                    return Err(KeyParseError::EmptyDevice);
                };
                Ok(CallMemberStateKeyEnum::new(user_id, Some(device_id), underscore))
            }
            (Err(err), _) => {
                Err(KeyParseError::InvalidUser { user_id: user_id.to_owned(), error: err })
            }
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
    #[error("uses a leading underscore but no trailing device id. The part after the underscore is a valid user id.")]
    LeadingUnderscoreNoDevice,
    /// Uses an empty device id. (UserId with trailing underscore)
    #[error("uses an empty device id. (UserId with trailing underscore)")]
    EmptyDevice,
}

impl de::Expected for KeyParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "correct call member event key format. The provided string, {})", self)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::call::member::{member_state_key::CallMemberStateKeyEnum, CallMemberStateKey};

    #[test]
    fn convert_state_key_enum_to_state_key() {
        let key = "_@user:domain.org_DEVICE";
        let state_key_enum = CallMemberStateKeyEnum::from_str(key).unwrap();
        // This generates state_key.raw from the enum
        let state_key: CallMemberStateKey = state_key_enum.into();
        // This compares state_key.raw (generated) with key (original)
        assert_eq!(state_key.as_ref(), key);
        // Compare to the from string without `CallMemberStateKeyEnum` step.
        let state_key_direct = CallMemberStateKey::from_str(state_key.as_ref()).unwrap();
        assert_eq!(state_key, state_key_direct);
    }
}
