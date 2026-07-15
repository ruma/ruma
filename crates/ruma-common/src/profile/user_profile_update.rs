//! An update to the profile information for a user.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value as JsonValue;

use super::{ProfileFieldName, ProfileFieldValue, StaticProfileField};

/// An update to a user's profile.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum UserProfileUpdate {
    /// The user's profile has been updated with the included changes.
    Updated(UserProfileChanges),

    /// This user no longer needs to be tracked as they have left all shared rooms.
    Dropped,
}

impl<'d> Deserialize<'d> for UserProfileUpdate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'d>,
    {
        Option::<UserProfileChanges>::deserialize(deserializer).map(|value| match value {
            Some(changes) => Self::Updated(changes),
            None => Self::Dropped,
        })
    }
}

impl Serialize for UserProfileUpdate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Updated(changes) => serializer.serialize_some(changes),
            Self::Dropped => serializer.serialize_none(),
        }
    }
}

/// A collection of changes to be applied to a user's profile.
///
/// This type is not supposed to be used directly, but applied to an existing
/// [`UserProfile`](super::UserProfile). If a profile doesn't exist, the changes should be applied
/// to an empty one.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UserProfileChanges {
    /// Fields that have been newly set, or updated.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub updated: BTreeMap<ProfileFieldName, JsonValue>,

    /// Fields that have been removed from the profile.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed: Vec<ProfileFieldName>,
}

impl UserProfileChanges {
    /// Creates a new empty `UserProfileUpdate`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the updated value of the given [`StaticProfileField`].
    ///
    /// Returns `Ok(Some(_))` if an update to the field is included and the value was deserialized
    /// successfully, `Ok(None)` if the field update is not included, or an error if deserialization
    /// of the value failed.
    pub fn get_updated_static<F: StaticProfileField>(
        &self,
    ) -> Result<Option<F::Value>, serde_json::Error> {
        self.updated
            .get(&ProfileFieldName::from(F::NAME))
            .map(|value| serde_json::from_value(value.clone()))
            .transpose()
    }

    /// Inserts an update for the supplied profile field value.
    pub fn insert_updated_value(&mut self, value: ProfileFieldValue) {
        self.updated.insert(value.field_name(), value.value().into_owned());
    }
}
