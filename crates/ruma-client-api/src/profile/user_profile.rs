//! All the profile information for a user.

use std::collections::{BTreeMap, btree_map};

use ruma_common::profile::{ProfileFieldName, ProfileFieldValue};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::profile::static_profile_field::StaticProfileField;

/// All the profile information for a user.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserProfile(BTreeMap<String, JsonValue>);

impl UserProfile {
    /// Creates a new empty `UserProfile`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the value of the given profile field.
    pub fn get(&self, field: &str) -> Option<&JsonValue> {
        self.0.get(field)
    }

    /// Returns the value of the given [`StaticProfileField`].
    ///
    /// Returns `Ok(Some(_))` if the field is present and the value was deserialized
    /// successfully, `Ok(None)` if the field is not set, or an error if deserialization of the
    /// value failed.
    pub fn get_static<F: StaticProfileField>(&self) -> Result<Option<F::Value>, serde_json::Error> {
        self.0.get(F::NAME).map(|value| serde_json::from_value(value.clone())).transpose()
    }

    /// Gets an iterator over the fields of the profile.
    pub fn iter(&self) -> btree_map::Iter<'_, String, JsonValue> {
        self.0.iter()
    }

    /// Sets a field to the given value.
    pub fn set(&mut self, field: String, value: JsonValue) {
        self.0.insert(field, value);
    }
}

impl FromIterator<(String, JsonValue)> for UserProfile {
    fn from_iter<T: IntoIterator<Item = (String, JsonValue)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl FromIterator<(ProfileFieldName, JsonValue)> for UserProfile {
    fn from_iter<T: IntoIterator<Item = (ProfileFieldName, JsonValue)>>(iter: T) -> Self {
        iter.into_iter().map(|(field, value)| (field.as_str().to_owned(), value)).collect()
    }
}

impl FromIterator<ProfileFieldValue> for UserProfile {
    fn from_iter<T: IntoIterator<Item = ProfileFieldValue>>(iter: T) -> Self {
        iter.into_iter().map(|value| (value.field_name(), value.value().into_owned())).collect()
    }
}

impl Extend<(String, JsonValue)> for UserProfile {
    fn extend<T: IntoIterator<Item = (String, JsonValue)>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl Extend<(ProfileFieldName, JsonValue)> for UserProfile {
    fn extend<T: IntoIterator<Item = (ProfileFieldName, JsonValue)>>(&mut self, iter: T) {
        self.extend(iter.into_iter().map(|(field, value)| (field.as_str().to_owned(), value)));
    }
}

impl Extend<ProfileFieldValue> for UserProfile {
    fn extend<T: IntoIterator<Item = ProfileFieldValue>>(&mut self, iter: T) {
        self.extend(iter.into_iter().map(|value| (value.field_name(), value.value().into_owned())));
    }
}

impl IntoIterator for UserProfile {
    type Item = (String, JsonValue);
    type IntoIter = btree_map::IntoIter<String, JsonValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
