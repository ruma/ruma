//! `PUT /_matrix/client/*/profile/{userId}/{key_name}`
//!
//! Set a field on the profile of the user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! Although this endpoint has a similar format to [`set_avatar_url`] and [`set_display_name`],
    //! it will only work with homeservers advertising support for the proper unstable feature or
    //! a version compatible with Matrix 1.16.
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3profileuseridkeyname
    //! [`set_avatar_url`]: crate::profile::set_avatar_url
    //! [`set_display_name`]: crate::profile::set_display_name

    use ruma_common::{
        api::{response, Metadata},
        metadata, OwnedUserId,
    };

    use crate::profile::{profile_field_serde::ProfileFieldValueVisitor, ProfileFieldValue};

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        // History valid for fields that existed in Matrix 1.0, i.e. `displayname` and `avatar_url`.
        history: {
            unstable("uk.tcpip.msc4133") => "/_matrix/client/unstable/uk.tcpip.msc4133/profile/{user_id}/{field}",
            1.0 => "/_matrix/client/r0/profile/{user_id}/{field}",
            1.1 => "/_matrix/client/v3/profile/{user_id}/{field}",
        }
    };

    /// Request type for the `set_profile_field` endpoint.
    #[derive(Debug, Clone)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Request {
        /// The user whose profile will be updated.
        pub user_id: OwnedUserId,

        /// The value of the profile field to set.
        pub value: ProfileFieldValue,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID, field and value.
        pub fn new(user_id: OwnedUserId, value: ProfileFieldValue) -> Self {
            Self { user_id, value }
        }
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::OutgoingRequest for Request {
        type EndpointError = crate::Error;
        type IncomingResponse = Response;

        const METADATA: Metadata = METADATA;

        fn try_into_http_request<T: Default + bytes::BufMut>(
            self,
            base_url: &str,
            _access_token: ruma_common::api::SendAccessToken<'_>,
            considering: &'_ ruma_common::api::SupportedVersions,
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            let field = self.value.field_name();

            let url = if field.existed_before_extended_profiles() {
                METADATA.make_endpoint_url(considering, base_url, &[&self.user_id, &field], "")?
            } else {
                crate::profile::EXTENDED_PROFILE_FIELD_HISTORY.make_endpoint_url(
                    considering,
                    base_url,
                    &[&self.user_id, &field],
                    "",
                )?
            };

            let http_request = http::Request::builder()
                .method(METADATA.method)
                .uri(url)
                .body(ruma_common::serde::json_to_buf(&self.value)?)
                // this cannot fail because we don't give user-supplied data to any of the
                // builder methods
                .unwrap();

            Ok(http_request)
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::IncomingRequest for Request {
        type EndpointError = crate::Error;
        type OutgoingResponse = Response;

        const METADATA: Metadata = METADATA;

        fn try_from_http_request<B, S>(
            request: http::Request<B>,
            path_args: &[S],
        ) -> Result<Self, ruma_common::api::error::FromHttpRequestError>
        where
            B: AsRef<[u8]>,
            S: AsRef<str>,
        {
            use serde::de::{Deserializer, Error as _};

            use crate::profile::ProfileFieldName;

            let (user_id, field): (OwnedUserId, ProfileFieldName) =
                serde::Deserialize::deserialize(serde::de::value::SeqDeserializer::<
                    _,
                    serde::de::value::Error,
                >::new(
                    path_args.iter().map(::std::convert::AsRef::as_ref),
                ))?;

            let value = serde_json::Deserializer::from_slice(request.body().as_ref())
                .deserialize_map(ProfileFieldValueVisitor(Some(field.clone())))?
                .ok_or_else(|| serde_json::Error::custom(format!("missing field `{field}`")))?;

            Ok(Request { user_id, value })
        }
    }

    /// Response type for the `set_profile_field` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::{owned_mxc_uri, owned_user_id};
    use serde_json::{
        from_slice as from_json_slice, json, to_vec as to_json_vec, Value as JsonValue,
    };

    use super::v3::Request;
    use crate::profile::ProfileFieldValue;

    #[test]
    #[cfg(feature = "client")]
    fn serialize_request() {
        use ruma_common::api::{OutgoingRequest, SendAccessToken, SupportedVersions};

        // Profile field that existed in Matrix 1.0.
        let avatar_url_request = Request::new(
            owned_user_id!("@alice:localhost"),
            ProfileFieldValue::AvatarUrl(owned_mxc_uri!("mxc://localhost/abcdef")),
        );

        // Matrix 1.11.
        let http_request = avatar_url_request
            .clone()
            .try_into_http_request::<Vec<u8>>(
                "http://localhost/",
                SendAccessToken::Always("access_token"),
                &SupportedVersions::from_parts(&["v1.11".to_owned()], &Default::default()),
            )
            .unwrap();
        assert_eq!(
            http_request.uri().path(),
            "/_matrix/client/v3/profile/@alice:localhost/avatar_url"
        );
        assert_eq!(
            from_json_slice::<JsonValue>(http_request.body().as_ref()).unwrap(),
            json!({
                "avatar_url": "mxc://localhost/abcdef",
            })
        );

        // Matrix 1.16.
        let http_request = avatar_url_request
            .try_into_http_request::<Vec<u8>>(
                "http://localhost/",
                SendAccessToken::Always("access_token"),
                &SupportedVersions::from_parts(&["v1.16".to_owned()], &Default::default()),
            )
            .unwrap();
        assert_eq!(
            http_request.uri().path(),
            "/_matrix/client/v3/profile/@alice:localhost/avatar_url"
        );
        assert_eq!(
            from_json_slice::<JsonValue>(http_request.body().as_ref()).unwrap(),
            json!({
                "avatar_url": "mxc://localhost/abcdef",
            })
        );

        // Profile field that didn't exist in Matrix 1.0.
        let custom_field_request = Request::new(
            owned_user_id!("@alice:localhost"),
            ProfileFieldValue::new("dev.ruma.custom_field", json!(true)).unwrap(),
        );

        // Matrix 1.11.
        let http_request = custom_field_request
            .clone()
            .try_into_http_request::<Vec<u8>>(
                "http://localhost/",
                SendAccessToken::Always("access_token"),
                &SupportedVersions::from_parts(&["v1.11".to_owned()], &Default::default()),
            )
            .unwrap();
        assert_eq!(
            http_request.uri().path(),
            "/_matrix/client/unstable/uk.tcpip.msc4133/profile/@alice:localhost/dev.ruma.custom_field"
        );
        assert_eq!(
            from_json_slice::<JsonValue>(http_request.body().as_ref()).unwrap(),
            json!({
                "dev.ruma.custom_field": true,
            })
        );

        // Matrix 1.16.
        let http_request = custom_field_request
            .try_into_http_request::<Vec<u8>>(
                "http://localhost/",
                SendAccessToken::Always("access_token"),
                &SupportedVersions::from_parts(&["v1.16".to_owned()], &Default::default()),
            )
            .unwrap();
        assert_eq!(
            http_request.uri().path(),
            "/_matrix/client/v3/profile/@alice:localhost/dev.ruma.custom_field"
        );
        assert_eq!(
            from_json_slice::<JsonValue>(http_request.body().as_ref()).unwrap(),
            json!({
                "dev.ruma.custom_field": true,
            })
        );
    }

    #[test]
    #[cfg(feature = "server")]
    fn deserialize_request_valid_field() {
        use ruma_common::api::IncomingRequest;

        let body = to_json_vec(&json!({
            "displayname": "Alice",
        }))
        .unwrap();

        let request = Request::try_from_http_request(
            http::Request::put(
                "http://localhost/_matrix/client/v3/profile/@alice:localhost/displayname",
            )
            .body(body)
            .unwrap(),
            &["@alice:localhost", "displayname"],
        )
        .unwrap();

        assert_eq!(request.user_id, "@alice:localhost");
        assert_matches!(request.value, ProfileFieldValue::DisplayName(display_name));
        assert_eq!(display_name, "Alice");
    }

    #[test]
    #[cfg(feature = "server")]
    fn deserialize_request_invalid_field() {
        use ruma_common::api::IncomingRequest;

        let body = to_json_vec(&json!({
            "custom_field": "value",
        }))
        .unwrap();

        Request::try_from_http_request(
            http::Request::put(
                "http://localhost/_matrix/client/v3/profile/@alice:localhost/displayname",
            )
            .body(body)
            .unwrap(),
            &["@alice:localhost", "displayname"],
        )
        .unwrap_err();
    }
}
