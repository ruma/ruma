//! `GET /_matrix/client/*/profile/{userId}`
//!
//! Get all profile information of a user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3profileuserid

    #[cfg(feature = "unstable-msc4133")]
    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedMxcUri, OwnedUserId,
    };
    #[cfg(feature = "unstable-msc4133")]
    use serde_json::Value as JsonValue;

    #[cfg(feature = "unstable-msc4133")]
    use crate::profile::ProfileFieldName;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/client/r0/profile/{user_id}",
            1.1 => "/_matrix/client/v3/profile/{user_id}",
        }
    };

    /// Request type for the `get_profile` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user whose profile will be retrieved.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,
    }

    /// Response type for the `get_profile` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// The user's avatar URL, if set.
        ///
        /// If you activate the `compat-empty-string-null` feature, this field being an empty
        /// string in JSON will result in `None` here during deserialization.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat-empty-string-null",
            serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
        )]
        pub avatar_url: Option<OwnedMxcUri>,

        /// The user's display name, if set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>,

        /// The [BlurHash](https://blurha.sh) for the avatar pointed to by `avatar_url`.
        ///
        /// This uses the unstable prefix in
        /// [MSC2448](https://github.com/matrix-org/matrix-spec-proposals/pull/2448).
        #[cfg(feature = "unstable-msc2448")]
        #[serde(rename = "xyz.amorgan.blurhash", skip_serializing_if = "Option::is_none")]
        pub blurhash: Option<String>,

        /// The other fields of the user's profile.
        #[cfg(feature = "unstable-msc4133")]
        #[serde(flatten)]
        pub others: BTreeMap<ProfileFieldName, JsonValue>,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given avatar URL and display name.
        pub fn new(avatar_url: Option<OwnedMxcUri>, displayname: Option<String>) -> Self {
            Self {
                avatar_url,
                displayname,
                #[cfg(feature = "unstable-msc2448")]
                blurhash: None,
                #[cfg(feature = "unstable-msc4133")]
                others: BTreeMap::new(),
            }
        }
    }
}

#[cfg(all(test, feature = "unstable-msc4133"))]
mod tests {
    use ruma_common::owned_mxc_uri;
    use serde_json::{
        from_slice as from_json_slice, json, to_vec as to_json_vec, Value as JsonValue,
    };

    use super::v3::Response;

    #[test]
    #[cfg(feature = "server")]
    fn serialize_response() {
        use ruma_common::api::OutgoingResponse;

        let mut response =
            Response::new(Some(owned_mxc_uri!("mxc://localhost/abcdef")), Some("Alice".to_owned()));
        response.others.insert("custom_field".into(), "value".into());

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

        use crate::profile::ProfileFieldName;

        let body = to_json_vec(&json!({
            "avatar_url": "mxc://localhost/abcdef",
            "displayname": "Alice",
            "custom_field": "value",
        }))
        .unwrap();

        let response = Response::try_from_http_response(http::Response::new(body)).unwrap();
        assert_eq!(response.avatar_url.unwrap(), "mxc://localhost/abcdef");
        assert_eq!(response.displayname.unwrap(), "Alice");
        assert_eq!(response.others.len(), 1);
        assert_eq!(
            response.others.get(&ProfileFieldName::from("custom_field")).unwrap().as_str().unwrap(),
            "value"
        );
    }
}
