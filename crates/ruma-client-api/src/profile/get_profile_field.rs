//! `GET /_matrix/client/*/profile/{userId}/{key_name}`
//!
//! Get a field in the profile of the user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://github.com/matrix-org/matrix-spec-proposals/pull/4133

    use std::marker::PhantomData;

    use ruma_common::{
        api::{request, Metadata},
        metadata, OwnedUserId,
    };

    use crate::profile::{
        ProfileFieldName, ProfileFieldValue, StaticProfileField, StaticProfileFieldVisitor,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            unstable("uk.tcpip.msc4133") => "/_matrix/client/unstable/uk.tcpip.msc4133/profile/{user_id}/{field}",
            // 1.15 => "/_matrix/client/v3/profile/{user_id}/{field}",
        }
    };

    /// Request type for the `get_profile_field` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user whose profile will be fetched.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// The profile field to get.
        #[ruma_api(path)]
        pub field: ProfileFieldName,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID and field.
        pub fn new(user_id: OwnedUserId, field: ProfileFieldName) -> Self {
            Self { user_id, field }
        }

        /// Creates a new request with the given user ID and statically-known field.
        pub fn new_static<F: StaticProfileField>(user_id: OwnedUserId) -> RequestStatic<F> {
            RequestStatic::new(user_id)
        }
    }

    /// Request type for the `get_profile_field` endpoint, using a statically-known field.
    #[derive(Debug)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct RequestStatic<F: StaticProfileField> {
        /// The user whose profile will be fetched.
        pub user_id: OwnedUserId,

        /// The profile field to get.
        field: PhantomData<F>,
    }

    impl<F: StaticProfileField> RequestStatic<F> {
        /// Creates a new request with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id, field: PhantomData }
        }
    }

    impl<F: StaticProfileField> Clone for RequestStatic<F> {
        fn clone(&self) -> Self {
            Self { user_id: self.user_id.clone(), field: self.field }
        }
    }

    #[cfg(feature = "client")]
    impl<F: StaticProfileField> ruma_common::api::OutgoingRequest for RequestStatic<F> {
        type EndpointError = crate::Error;
        type IncomingResponse = ResponseStatic<F>;

        const METADATA: Metadata = METADATA;

        fn try_into_http_request<T: Default + bytes::BufMut>(
            self,
            base_url: &str,
            access_token: ruma_common::api::SendAccessToken<'_>,
            considering: &'_ ruma_common::api::SupportedVersions,
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            Request::new(self.user_id, F::NAME.into()).try_into_http_request(
                base_url,
                access_token,
                considering,
            )
        }
    }

    /// Response type for the `get_profile_field` endpoint.
    #[derive(Debug, Clone, Default)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Response {
        /// The value of the profile field.
        pub value: Option<ProfileFieldValue>,
    }

    impl Response {
        /// Creates a `Response` with the given value.
        pub fn new(value: ProfileFieldValue) -> Self {
            Self { value: Some(value) }
        }
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::IncomingResponse for Response {
        type EndpointError = crate::Error;

        fn try_from_http_response<T: AsRef<[u8]>>(
            response: http::Response<T>,
        ) -> Result<Self, ruma_common::api::error::FromHttpResponseError<Self::EndpointError>>
        {
            use ruma_common::api::EndpointError;

            use crate::profile::deserialize_profile_field_value_option;

            if response.status().as_u16() >= 400 {
                return Err(ruma_common::api::error::FromHttpResponseError::Server(
                    Self::EndpointError::from_http_response(response),
                ));
            }

            let mut de = serde_json::Deserializer::from_slice(response.body().as_ref());
            let value = deserialize_profile_field_value_option(&mut de)?;
            de.end()?;

            Ok(Self { value })
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::OutgoingResponse for Response {
        fn try_into_http_response<T: Default + bytes::BufMut>(
            self,
        ) -> Result<http::Response<T>, ruma_common::api::error::IntoHttpError> {
            use ruma_common::serde::JsonObject;

            let body = self
                .value
                .as_ref()
                .map(|value| ruma_common::serde::json_to_buf(value))
                .unwrap_or_else(||
                   // Send an empty object.
                    ruma_common::serde::json_to_buf(&JsonObject::new()))?;

            Ok(http::Response::builder()
                .status(http::StatusCode::OK)
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(body)?)
        }
    }

    /// Response type for the `get_profile_field` endpoint, using a statically-known field.
    #[derive(Debug)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ResponseStatic<F: StaticProfileField> {
        /// The value of the profile field, if it is set.
        pub value: Option<F::Value>,
    }

    impl<F: StaticProfileField> Clone for ResponseStatic<F>
    where
        F::Value: Clone,
    {
        fn clone(&self) -> Self {
            Self { value: self.value.clone() }
        }
    }

    #[cfg(feature = "client")]
    impl<F: StaticProfileField> ruma_common::api::IncomingResponse for ResponseStatic<F> {
        type EndpointError = crate::Error;

        fn try_from_http_response<T: AsRef<[u8]>>(
            response: http::Response<T>,
        ) -> Result<Self, ruma_common::api::error::FromHttpResponseError<Self::EndpointError>>
        {
            use ruma_common::api::EndpointError;
            use serde::de::Deserializer;

            if response.status().as_u16() >= 400 {
                return Err(ruma_common::api::error::FromHttpResponseError::Server(
                    Self::EndpointError::from_http_response(response),
                ));
            }

            let value = serde_json::Deserializer::from_slice(response.into_body().as_ref())
                .deserialize_map(StaticProfileFieldVisitor(PhantomData::<F>))?;

            Ok(Self { value })
        }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::{owned_mxc_uri, owned_user_id};
    use serde_json::{
        from_slice as from_json_slice, json, to_vec as to_json_vec, Value as JsonValue,
    };

    use super::v3::{Request, RequestStatic, Response};
    use crate::profile::{ProfileFieldName, ProfileFieldValue};

    #[test]
    #[cfg(feature = "client")]
    fn serialize_request() {
        use ruma_common::api::{OutgoingRequest, SendAccessToken, SupportedVersions};

        let request = Request::new(owned_user_id!("@alice:localhost"), ProfileFieldName::AvatarUrl);

        let http_request = request
            .try_into_http_request::<Vec<u8>>(
                "http://localhost/",
                SendAccessToken::Always("access_token"),
                &SupportedVersions::from_parts(&["v11".to_owned()], &Default::default()),
            )
            .unwrap();

        assert_eq!(
            http_request.uri().path(),
            "/_matrix/client/unstable/uk.tcpip.msc4133/profile/@alice:localhost/avatar_url"
        );
    }

    #[test]
    #[cfg(feature = "server")]
    fn deserialize_request() {
        use ruma_common::api::IncomingRequest;

        let request = Request::try_from_http_request(
            http::Request::get("http://localhost/_matrix/client/unstable/uk.tcpip.msc4133/profile/@alice:localhost/displayname").body(Vec::<u8>::new()).unwrap(),
                &["@alice:localhost", "displayname"],
            ).unwrap();

        assert_eq!(request.user_id, "@alice:localhost");
        assert_eq!(request.field, ProfileFieldName::DisplayName);
    }

    #[test]
    #[cfg(feature = "server")]
    fn serialize_response() {
        use ruma_common::api::OutgoingResponse;

        let response =
            Response::new(ProfileFieldValue::AvatarUrl(owned_mxc_uri!("mxc://localhost/abcdef")));

        let http_response = response.try_into_http_response::<Vec<u8>>().unwrap();

        assert_eq!(
            from_json_slice::<JsonValue>(http_response.body().as_ref()).unwrap(),
            json!({
                "avatar_url": "mxc://localhost/abcdef",
            })
        );
    }

    #[test]
    #[cfg(feature = "client")]
    fn deserialize_response() {
        use ruma_common::api::IncomingResponse;

        let body = to_json_vec(&json!({
            "custom_field": "value",
        }))
        .unwrap();

        let response = Response::try_from_http_response(http::Response::new(body)).unwrap();
        let value = response.value.unwrap();
        assert_eq!(value.field_name().as_str(), "custom_field");
        assert_eq!(value.value().as_str().unwrap(), "value");

        let empty_body = to_json_vec(&json!({})).unwrap();

        let response = Response::try_from_http_response(http::Response::new(empty_body)).unwrap();
        assert!(response.value.is_none());
    }

    /// Mock a response from the homeserver to a request of type `R` and return the given `value` as
    /// a typed response.
    #[cfg(feature = "client")]
    fn get_static_response<R: ruma_common::api::OutgoingRequest>(
        value: Option<ProfileFieldValue>,
    ) -> Result<R::IncomingResponse, ruma_common::api::error::FromHttpResponseError<R::EndpointError>>
    {
        use ruma_common::api::IncomingResponse;

        let body =
            value.map(|value| to_json_vec(&value).unwrap()).unwrap_or_else(|| b"{}".to_vec());
        R::IncomingResponse::try_from_http_response(http::Response::new(body))
    }

    #[test]
    #[cfg(feature = "client")]
    fn static_request_and_valid_response() {
        use crate::profile::AvatarUrl;

        let response = get_static_response::<RequestStatic<AvatarUrl>>(Some(
            ProfileFieldValue::AvatarUrl(owned_mxc_uri!("mxc://localhost/abcdef")),
        ))
        .unwrap();
        assert_eq!(response.value.unwrap(), "mxc://localhost/abcdef");

        let response = get_static_response::<RequestStatic<AvatarUrl>>(None).unwrap();
        assert!(response.value.is_none());
    }

    #[test]
    #[cfg(feature = "client")]
    fn static_request_and_invalid_response() {
        use crate::profile::AvatarUrl;

        get_static_response::<RequestStatic<AvatarUrl>>(Some(ProfileFieldValue::DisplayName(
            "Alice".to_owned(),
        )))
        .unwrap_err();
    }
}
