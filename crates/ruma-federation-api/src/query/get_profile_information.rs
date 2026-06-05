//! `GET /_matrix/federation/*/query/profile`
//!
//! Get profile information, such as a display name or avatar, for a given user.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.18/server-server-api/#get_matrixfederationv1queryprofile

    use std::collections::btree_map;

    use ruma_common::{
        OwnedUserId,
        api::{request, response},
        metadata,
        profile::{ProfileFieldName, ProfileFieldValue, StaticProfileField, UserProfile},
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
        pub data: UserProfile,
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

        /// Returns the value of the given [`StaticProfileField`].
        ///
        /// Returns `Ok(Some(_))` if the field is present and the value was deserialized
        /// successfully, `Ok(None)` if the field is not set, or an error if deserialization of the
        /// value failed.
        pub fn get_static<F: StaticProfileField>(
            &self,
        ) -> Result<Option<F::Value>, serde_json::Error> {
            self.data.get_static::<F>()
        }

        /// Gets an iterator over the fields of the profile.
        pub fn iter(&self) -> btree_map::Iter<'_, String, JsonValue> {
            self.data.iter()
        }

        /// Sets a field to the given value.
        pub fn set(&mut self, field: String, value: JsonValue) {
            self.data.set(field, value);
        }
    }

    impl From<UserProfile> for Response {
        fn from(value: UserProfile) -> Self {
            Self { data: value }
        }
    }

    impl FromIterator<(String, JsonValue)> for Response {
        fn from_iter<T: IntoIterator<Item = (String, JsonValue)>>(iter: T) -> Self {
            Self { data: UserProfile::from_iter(iter) }
        }
    }

    impl FromIterator<(ProfileFieldName, JsonValue)> for Response {
        fn from_iter<T: IntoIterator<Item = (ProfileFieldName, JsonValue)>>(iter: T) -> Self {
            Self { data: UserProfile::from_iter(iter) }
        }
    }

    impl FromIterator<ProfileFieldValue> for Response {
        fn from_iter<T: IntoIterator<Item = ProfileFieldValue>>(iter: T) -> Self {
            Self { data: UserProfile::from_iter(iter) }
        }
    }

    impl Extend<(String, JsonValue)> for Response {
        fn extend<T: IntoIterator<Item = (String, JsonValue)>>(&mut self, iter: T) {
            self.data.extend(iter);
        }
    }

    impl Extend<(ProfileFieldName, JsonValue)> for Response {
        fn extend<T: IntoIterator<Item = (ProfileFieldName, JsonValue)>>(&mut self, iter: T) {
            self.data.extend(iter);
        }
    }

    impl Extend<ProfileFieldValue> for Response {
        fn extend<T: IntoIterator<Item = ProfileFieldValue>>(&mut self, iter: T) {
            self.data.extend(iter);
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
