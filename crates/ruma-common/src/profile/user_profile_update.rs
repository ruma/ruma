//! An update to the profile information for a user.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::{ProfileFieldName, ProfileFieldValue, StaticProfileField};

/// An update to the profile information for a user.
///
/// This type is not supposed to be used directly, but merged into a
/// [`UserProfile`](super::UserProfile).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UserProfileUpdate {
    /// Fields that have been newly set, or updated.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub updated: BTreeMap<ProfileFieldName, JsonValue>,

    /// Fields that have been removed from the profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed: Option<Vec<ProfileFieldName>>,
}

impl UserProfileUpdate {
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
