//! Endpoints for querying the server's supported feature set

use std::{borrow::Cow, collections::BTreeMap};

use maplit::btreemap;
use ruma_identifiers::RoomVersionId;
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};
use serde_json::{from_value as from_json_value, to_value as to_json_value, Value as JsonValue};

use self::iter::{CapabilitiesIter, CapabilityRef};
use crate::PrivOwnedStr;

pub mod get_capabilities;
pub mod iter;

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
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
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

impl RoomVersionStability {
    /// Creates a string slice from this `RoomVersionStability`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

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
        assert_eq!(iter_res.name(), "m.some_random_capability");
        assert_eq!(iter_res.value(), Cow::Borrowed(&json!({ "key": "value" })));

        assert!(caps_iter.next().is_none());
        Ok(())
    }
}
