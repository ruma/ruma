//! `GET /_matrix/federation/*/query/profile`
//!
//! Get profile information, such as a display name or avatar, for a given user.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.18/server-server-api/#get_matrixfederationv1queryprofile

    use std::collections::{BTreeMap, btree_map};

    use ruma_common::{
        OwnedUserId,
        api::{request, response},
        metadata,
        profile::{ProfileFieldName, ProfileFieldValue},
    };
    use serde_json::Value as JsonValue;

    use crate::authentication::ServerSignatures;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: ServerSignatures,
        path: "/_matrix/federation/v1/query/profile",
    }

    /// Request type for the `get_profile_information` endpoint.
    #[request]
    pub struct Request {
        /// User ID to query.
        #[ruma_api(query)]
        pub user_id: OwnedUserId,

        /// Profile field to query.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub field: Option<ProfileFieldName>,
    }

    impl Request {
        /// Creates a new `Request` with the given user id.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id, field: None }
        }
    }

    /// Response type for the `get_profile_information` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// The profile data.
        #[ruma_api(body)]
        data: BTreeMap<String, JsonValue>,
    }

    impl Response {
        /// Creates a new empty `Response`.
        pub fn new() -> Self {
            Self::default()
        }

        /// Returns the value of the given profile field.
        pub fn get(&self, field: &str) -> Option<&JsonValue> {
            self.data.get(field)
        }

        /// Gets an iterator over the fields of the profile.
        pub fn iter(&self) -> btree_map::Iter<'_, String, JsonValue> {
            self.data.iter()
        }

        /// Sets a field to the given value.
        pub fn set(&mut self, field: String, value: JsonValue) {
            self.data.insert(field, value);
        }
    }

    impl FromIterator<(String, JsonValue)> for Response {
        fn from_iter<T: IntoIterator<Item = (String, JsonValue)>>(iter: T) -> Self {
            Self { data: iter.into_iter().collect() }
        }
    }

    impl FromIterator<(ProfileFieldName, JsonValue)> for Response {
        fn from_iter<T: IntoIterator<Item = (ProfileFieldName, JsonValue)>>(iter: T) -> Self {
            iter.into_iter().map(|(field, value)| (field.as_str().to_owned(), value)).collect()
        }
    }

    impl FromIterator<ProfileFieldValue> for Response {
        fn from_iter<T: IntoIterator<Item = ProfileFieldValue>>(iter: T) -> Self {
            iter.into_iter().map(|value| (value.field_name(), value.value().into_owned())).collect()
        }
    }

    impl Extend<(String, JsonValue)> for Response {
        fn extend<T: IntoIterator<Item = (String, JsonValue)>>(&mut self, iter: T) {
            self.data.extend(iter);
        }
    }

    impl Extend<(ProfileFieldName, JsonValue)> for Response {
        fn extend<T: IntoIterator<Item = (ProfileFieldName, JsonValue)>>(&mut self, iter: T) {
            self.extend(iter.into_iter().map(|(field, value)| (field.as_str().to_owned(), value)));
        }
    }

    impl Extend<ProfileFieldValue> for Response {
        fn extend<T: IntoIterator<Item = ProfileFieldValue>>(&mut self, iter: T) {
            self.extend(
                iter.into_iter().map(|value| (value.field_name(), value.value().into_owned())),
            );
        }
    }

    impl IntoIterator for Response {
        type Item = (String, JsonValue);
        type IntoIter = btree_map::IntoIter<String, JsonValue>;

        fn into_iter(self) -> Self::IntoIter {
            self.data.into_iter()
        }
    }
}
