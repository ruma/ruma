//! `GET /_matrix/client/*/capabilities`
//!
//! Get information about the server's supported feature set and other relevant capabilities
//! ([spec]).
//!
//! [spec]: https://spec.matrix.org/latest/client-server-api/#capabilities-negotiation

use std::{borrow::Cow, collections::BTreeMap};

use maplit::btreemap;
use ruma_common::{serde::StringEnum, RoomVersionId};
use serde::{Deserialize, Serialize};
use serde_json::{from_value as from_json_value, to_value as to_json_value, Value as JsonValue};

use self::iter::{CapabilitiesIter, CapabilityRef};
use crate::PrivOwnedStr;

pub mod iter;
pub mod v3;

/// Contains information about all the capabilities that the server supports.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Capabilities {
    /// Capability to indicate if the user can change their password.
    #[serde(
        rename = "m.change_password",
        default,
        skip_serializing_if = "ChangePasswordCapability::is_default"
    )]
    pub change_password: ChangePasswordCapability,

    /// The room versions the server supports.
    #[serde(
        rename = "m.room_versions",
        default,
        skip_serializing_if = "RoomVersionsCapability::is_default"
    )]
    pub room_versions: RoomVersionsCapability,

    /// Capability to indicate if the user can change their display name.
    #[serde(
        rename = "m.set_displayname",
        default,
        skip_serializing_if = "SetDisplayNameCapability::is_default"
    )]
    pub set_displayname: SetDisplayNameCapability,

    /// Capability to indicate if the user can change their avatar.
    #[serde(
        rename = "m.set_avatar_url",
        default,
        skip_serializing_if = "SetAvatarUrlCapability::is_default"
    )]
    pub set_avatar_url: SetAvatarUrlCapability,

    /// Capability to indicate if the user can change the third-party identifiers associated with
    /// their account.
    #[serde(
        rename = "m.3pid_changes",
        default,
        skip_serializing_if = "ThirdPartyIdChangesCapability::is_default"
    )]
    pub thirdparty_id_changes: ThirdPartyIdChangesCapability,

    /// Any other custom capabilities that the server supports outside of the specification,
    /// labeled using the Java package naming convention and stored as arbitrary JSON values.
    #[serde(flatten)]
    custom_capabilities: BTreeMap<String, JsonValue>,
}

impl Capabilities {
    /// Creates empty `Capabilities`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns the value of the given capability.
    ///
    /// Prefer to use the public fields of `Capabilities` where possible; this method is meant to be
    /// used for unsupported capabilities only.
    pub fn get(&self, capability: &str) -> Option<Cow<'_, JsonValue>> {
        fn serialize<T: Serialize>(cap: &T) -> JsonValue {
            to_json_value(cap).expect("capability serialization to succeed")
        }

        match capability {
            "m.change_password" => Some(Cow::Owned(serialize(&self.change_password))),
            "m.room_versions" => Some(Cow::Owned(serialize(&self.room_versions))),
            "m.set_displayname" => Some(Cow::Owned(serialize(&self.set_displayname))),
            "m.set_avatar_url" => Some(Cow::Owned(serialize(&self.set_avatar_url))),
            "m.3pid_changes" => Some(Cow::Owned(serialize(&self.thirdparty_id_changes))),
            _ => self.custom_capabilities.get(capability).map(Cow::Borrowed),
        }
    }

    /// Sets a capability to the given value.
    ///
    /// Prefer to use the public fields of `Capabilities` where possible; this method is meant to be
    /// used for unsupported capabilities only and does not allow setting arbitrary data for
    /// supported ones.
    pub fn set(&mut self, capability: &str, value: JsonValue) -> serde_json::Result<()> {
        match capability {
            "m.change_password" => self.change_password = from_json_value(value)?,
            "m.room_versions" => self.room_versions = from_json_value(value)?,
            "m.set_displayname" => self.set_displayname = from_json_value(value)?,
            "m.set_avatar_url" => self.set_avatar_url = from_json_value(value)?,
            "m.3pid_changes" => self.thirdparty_id_changes = from_json_value(value)?,
            _ => {
                self.custom_capabilities.insert(capability.to_owned(), value);
            }
        }

        Ok(())
    }

    /// Returns an iterator over the capabilities.
    pub fn iter(&self) -> CapabilitiesIter<'_> {
        CapabilitiesIter::new(self)
    }
}

impl<'a> IntoIterator for &'a Capabilities {
    type Item = CapabilityRef<'a>;
    type IntoIter = CapabilitiesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Information about the m.change_password capability
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ChangePasswordCapability {
    /// `true` if the user can change their password, `false` otherwise.
    pub enabled: bool,
}

impl ChangePasswordCapability {
    /// Creates a new `ChangePasswordCapability` with the given enabled flag.
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Returns whether all fields have their default value.
    pub fn is_default(&self) -> bool {
        self.enabled
    }
}

impl Default for ChangePasswordCapability {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Information about the m.room_versions capability
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomVersionsCapability {
    /// The default room version the server is using for new rooms.
    pub default: RoomVersionId,

    /// A detailed description of the room versions the server supports.
    pub available: BTreeMap<RoomVersionId, RoomVersionStability>,
}

impl RoomVersionsCapability {
    /// Creates a new `RoomVersionsCapability` with the given default room version ID and room
    /// version descriptions.
    pub fn new(
        default: RoomVersionId,
        available: BTreeMap<RoomVersionId, RoomVersionStability>,
    ) -> Self {
        Self { default, available }
    }

    /// Returns whether all fields have their default value.
    pub fn is_default(&self) -> bool {
        self.default == RoomVersionId::V1
            && self.available.len() == 1
            && self
                .available
                .get(&RoomVersionId::V1)
                .map(|stability| *stability == RoomVersionStability::Stable)
                .unwrap_or(false)
    }
}

impl Default for RoomVersionsCapability {
    fn default() -> Self {
        Self {
            default: RoomVersionId::V1,
            available: btreemap! { RoomVersionId::V1 => RoomVersionStability::Stable },
        }
    }
}

/// The stability of a room version.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
#[non_exhaustive]
pub enum RoomVersionStability {
    /// Support for the given version is stable.
    Stable,

    /// Support for the given version is unstable.
    Unstable,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// Information about the `m.set_displayname` capability
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SetDisplayNameCapability {
    /// `true` if the user can change their display name, `false` otherwise.
    pub enabled: bool,
}

impl SetDisplayNameCapability {
    /// Creates a new `SetDisplayNameCapability` with the given enabled flag.
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Returns whether all fields have their default value.
    pub fn is_default(&self) -> bool {
        self.enabled
    }
}

impl Default for SetDisplayNameCapability {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Information about the `m.set_avatar_url` capability
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SetAvatarUrlCapability {
    /// `true` if the user can change their avatar, `false` otherwise.
    pub enabled: bool,
}

impl SetAvatarUrlCapability {
    /// Creates a new `SetAvatarUrlCapability` with the given enabled flag.
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Returns whether all fields have their default value.
    pub fn is_default(&self) -> bool {
        self.enabled
    }
}

impl Default for SetAvatarUrlCapability {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Information about the `m.3pid_changes` capability
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ThirdPartyIdChangesCapability {
    /// `true` if the user can change the third-party identifiers associated with their account,
    /// `false` otherwise.
    pub enabled: bool,
}

impl ThirdPartyIdChangesCapability {
    /// Creates a new `ThirdPartyIdChangesCapability` with the given enabled flag.
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Returns whether all fields have their default value.
    pub fn is_default(&self) -> bool {
        self.enabled
    }
}

impl Default for ThirdPartyIdChangesCapability {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use assert_matches2::assert_matches;
    use serde_json::json;

    use super::Capabilities;

    #[test]
    fn capabilities_iter() -> serde_json::Result<()> {
        let mut caps = Capabilities::new();
        let custom_cap = json!({
            "key": "value",
        });
        caps.set("m.some_random_capability", custom_cap)?;
        let mut caps_iter = caps.iter();

        let iter_res = caps_iter.next().unwrap();
        assert_eq!(iter_res.name(), "m.change_password");
        assert_eq!(iter_res.value(), Cow::Borrowed(&json!({ "enabled": true })));

        let iter_res = caps_iter.next().unwrap();
        assert_eq!(iter_res.name(), "m.room_versions");
        assert_eq!(
            iter_res.value(),
            Cow::Borrowed(&json!({ "available": { "1": "stable" },"default" :"1" }))
        );

        let iter_res = caps_iter.next().unwrap();
        assert_eq!(iter_res.name(), "m.set_displayname");
        assert_eq!(iter_res.value(), Cow::Borrowed(&json!({ "enabled": true })));

        let iter_res = caps_iter.next().unwrap();
        assert_eq!(iter_res.name(), "m.set_avatar_url");
        assert_eq!(iter_res.value(), Cow::Borrowed(&json!({ "enabled": true })));

        let iter_res = caps_iter.next().unwrap();
        assert_eq!(iter_res.name(), "m.3pid_changes");
        assert_eq!(iter_res.value(), Cow::Borrowed(&json!({ "enabled": true })));

        let iter_res = caps_iter.next().unwrap();
        assert_eq!(iter_res.name(), "m.some_random_capability");
        assert_eq!(iter_res.value(), Cow::Borrowed(&json!({ "key": "value" })));

        assert_matches!(caps_iter.next(), None);
        Ok(())
    }
}
