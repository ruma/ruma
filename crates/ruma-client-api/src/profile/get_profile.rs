//! `GET /_matrix/client/*/profile/{userId}`
//!
//! Get all profile information of a user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3profileuserid

    use std::collections::{BTreeMap, btree_map};

    use ruma_common::{
        UserId,
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
    };
    use serde_json::Value as JsonValue;

    use crate::profile::{ProfileFieldName, ProfileFieldValue, StaticProfileField};

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.0 => "/_matrix/client/r0/profile/{user_id}",
            1.1 => "/_matrix/client/v3/profile/{user_id}",
        }
    }

    /// Request type for the `get_profile` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user whose profile will be retrieved.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    /// Response type for the `get_profile` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// The profile data.
        #[ruma_api(body)]
        data: BTreeMap<String, JsonValue>,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID.
        pub fn new(user_id: UserId) -> Self {
            Self { user_id }
        }
    }

    impl Response {
        /// Creates a new empty `Response`.
        pub fn new() -> Self {
            Self::default()
        }

        /// Returns the value of the given capability.
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
            self.data.get(F::NAME).map(|value| serde_json::from_value(value.clone())).transpose()
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::v3::Response;

    #[test]
    #[cfg(feature = "server")]
    fn serialize_response() {
        use ruma_common::{api::OutgoingResponse, mxc_uri};
        use serde_json::{Value as JsonValue, from_slice as from_json_slice};

        use crate::profile::ProfileFieldValue;

        let response = [
            ProfileFieldValue::AvatarUrl(mxc_uri!("mxc://localhost/abcdef")),
            ProfileFieldValue::DisplayName("Alice".to_owned()),
            ProfileFieldValue::new("custom_field", "value".into()).unwrap(),
        ]
        .into_iter()
        .collect::<Response>();

        let http_response = response.try_into_http_response::<Vec<u8>>().unwrap();

        assert_eq!(
            from_json_slice::<JsonValue>(http_response.body().as_ref()).unwrap(),
            json!({
                "avatar_url": "mxc://localhost/abcdef",
                "displayname": "Alice",
                "custom_field": "value",
            })
        );
    }

    #[test]
    #[cfg(feature = "client")]
    fn deserialize_response() {
        use ruma_common::api::IncomingResponse;
        use serde_json::to_vec as to_json_vec;

        use crate::profile::{AvatarUrl, DisplayName};

        // Values are set.
        let body = to_json_vec(&json!({
            "avatar_url": "mxc://localhost/abcdef",
            "displayname": "Alice",
            "custom_field": "value",
        }))
        .unwrap();

        let response = Response::try_from_http_response(http::Response::new(body)).unwrap();
        assert_eq!(response.get_static::<AvatarUrl>().unwrap().unwrap(), "mxc://localhost/abcdef");
        assert_eq!(response.get_static::<DisplayName>().unwrap().unwrap(), "Alice");
        assert_eq!(response.get("custom_field").unwrap().as_str().unwrap(), "value");

        // Values are missing or null.
        let body = to_json_vec(&json!({
            "custom_field": null,
        }))
        .unwrap();

        let response = Response::try_from_http_response(http::Response::new(body)).unwrap();
        assert_eq!(response.get_static::<AvatarUrl>().unwrap(), None);
        assert_eq!(response.get_static::<DisplayName>().unwrap(), None);
        assert!(response.get("custom_field").unwrap().is_null());
    }
}
