//! All the profile information for a user.

use std::collections::{BTreeMap, btree_map};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[cfg(feature = "unstable-msc4262")]
use super::UserProfileUpdate;
use super::{ProfileFieldName, ProfileFieldValue, static_profile_field::StaticProfileField};

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

    /// Merges a profile that contains updates (such as from a sync response) with this
    /// profile.
    ///
    /// This operation preserves omitted values and removes null values.
    #[cfg(feature = "unstable-msc4262")]
    pub fn merge(&mut self, profile_update: UserProfileUpdate) {
        for (field, value) in profile_update {
            if value.is_null() {
                self.0.remove(&field);
            } else {
                self.0.insert(field, value);
            }
        }
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

#[cfg(test)]
#[cfg(all(feature = "unstable-msc4262", feature = "unstable-msc4426"))]
mod tests {
    use serde_json::{Value as JsonValue, json};

    use crate::{
        owned_mxc_uri,
        profile::{
            AvatarUrl, Call, CallProfileField, DisplayName, ProfileFieldValue, Status,
            StatusProfileField, UserProfile, UserProfileUpdate,
        },
    };

    #[test]
    fn merge_profile() {
        let mut profile = UserProfile::from_iter([
            ProfileFieldValue::DisplayName("Alice".to_owned()),
            ProfileFieldValue::AvatarUrl(owned_mxc_uri!("mxc://localhost/abcdef")),
            ProfileFieldValue::Status(StatusProfileField {
                text: "Working".to_owned(),
                emoji: "🧑‍💻".to_owned(),
            }),
        ]);

        let profile_update = UserProfileUpdate::from_iter([
            ("avatar_url".to_owned(), JsonValue::Null),
            ("org.matrix.msc4426.status".to_owned(), json!({ "text": "Holiday", "emoji": "🏖️"})),
            ("org.matrix.msc4426.call".to_owned(), json!({})),
        ]);

        profile.merge(profile_update.clone());

        assert_eq!(
            profile.get_static::<DisplayName>().unwrap().unwrap(),
            "Alice".to_owned(),
            "The display name should be preserved."
        );
        assert!(
            profile.get_static::<AvatarUrl>().unwrap().is_none(),
            "The avatar should be removed."
        );
        assert_eq!(
            profile.get_static::<Status>().unwrap().unwrap(),
            StatusProfileField { text: "Holiday".to_owned(), emoji: "🏖️".to_owned() },
            "The status should be updated."
        );
        assert_eq!(
            profile.get_static::<Call>().unwrap().unwrap(),
            CallProfileField { call_joined_ts: None },
            "The call indicator should be set."
        );
    }
}
