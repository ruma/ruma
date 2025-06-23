//! `GET /_matrix/client/*/profile/{userId}/{key_name}`
//!
//! Get a field in the profile of the user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://github.com/matrix-org/matrix-spec-proposals/pull/4133

    use std::marker::PhantomData;

    use ruma_common::{api::Metadata, metadata, OwnedUserId};

    use crate::profile::{
        ProfileField, ProfileFieldName, StaticProfileField, StaticProfileFieldVisitor,
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
    #[derive(Debug, Clone)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Request<F: ProfileField> {
        /// The user whose profile will be fetched.
        pub user_id: OwnedUserId,

        /// The profile field to get.
        pub field: F,
    }

    impl<F: ProfileField> Request<F> {
        /// Creates a new `Request` with the given user ID and field.
        pub fn new(user_id: OwnedUserId, field: F) -> Self {
            Self { user_id, field }
        }
    }

    impl<F: StaticProfileField> Request<PhantomData<F>> {
        /// Creates a new `Request` with the given user ID and statically-known field.
        pub fn new_static(user_id: OwnedUserId) -> Self {
            Self { user_id, field: PhantomData::<F> }
        }
    }

    #[cfg(feature = "client")]
    impl<F: StaticProfileField> ruma_common::api::OutgoingRequest for Request<PhantomData<F>> {
        type EndpointError = crate::Error;
        type IncomingResponse = Response<PhantomData<F>>;

        const METADATA: Metadata = METADATA;

        fn try_into_http_request<T: Default + bytes::BufMut>(
            self,
            base_url: &str,
            _access_token: ruma_common::api::SendAccessToken<'_>,
            considering: &'_ ruma_common::api::SupportedVersions,
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            let url = METADATA.make_endpoint_url(
                considering,
                base_url,
                &[&self.user_id, &F::NAME],
                "",
            )?;

            let http_request = http::Request::builder()
                .method(METADATA.method)
                .uri(url)
                .body(T::default())
                // this cannot fail because we don't give user-supplied data to any of the
                // builder methods
                .unwrap();

            Ok(http_request)
        }
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::OutgoingRequest for Request<ProfileFieldName> {
        type EndpointError = crate::Error;
        type IncomingResponse = Response<ProfileFieldName>;

        const METADATA: Metadata = METADATA;

        fn try_into_http_request<T: Default + bytes::BufMut>(
            self,
            base_url: &str,
            _access_token: ruma_common::api::SendAccessToken<'_>,
            considering: &'_ ruma_common::api::SupportedVersions,
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            let url = METADATA.make_endpoint_url(
                considering,
                base_url,
                &[&self.user_id, &self.field],
                "",
            )?;

            let http_request = http::Request::builder()
                .method(METADATA.method)
                .uri(url)
                .body(T::default())
                // this cannot fail because we don't give user-supplied data to any of the
                // builder methods
                .unwrap();

            Ok(http_request)
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::IncomingRequest for Request<ProfileFieldName> {
        type EndpointError = crate::Error;
        type OutgoingResponse = Response<ProfileFieldName>;

        const METADATA: Metadata = METADATA;

        fn try_from_http_request<B, S>(
            _request: http::Request<B>,
            path_args: &[S],
        ) -> Result<Self, ruma_common::api::error::FromHttpRequestError>
        where
            B: AsRef<[u8]>,
            S: AsRef<str>,
        {
            let (user_id, field) =
                serde::Deserialize::deserialize(serde::de::value::SeqDeserializer::<
                    _,
                    serde::de::value::Error,
                >::new(
                    path_args.iter().map(::std::convert::AsRef::as_ref),
                ))?;

            Ok(Self { user_id, field })
        }
    }

    /// Response type for the `get_profile_field` endpoint.
    #[derive(Debug, Clone)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Response<F: ProfileField> {
        /// The value of the profile field.
        pub value: F::Value,
    }

    impl<F: ProfileField> Response<F> {
        /// Creates a `Response` with the given value.
        pub fn new(value: F::Value) -> Self {
            Self { value }
        }
    }

    #[cfg(feature = "client")]
    impl<F: StaticProfileField> ruma_common::api::IncomingResponse for Response<PhantomData<F>> {
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

    #[cfg(feature = "client")]
    impl ruma_common::api::IncomingResponse for Response<ProfileFieldName> {
        type EndpointError = crate::Error;

        fn try_from_http_response<T: AsRef<[u8]>>(
            response: http::Response<T>,
        ) -> Result<Self, ruma_common::api::error::FromHttpResponseError<Self::EndpointError>>
        {
            use ruma_common::api::EndpointError;

            if response.status().as_u16() >= 400 {
                return Err(ruma_common::api::error::FromHttpResponseError::Server(
                    Self::EndpointError::from_http_response(response),
                ));
            }

            let value = serde_json::from_slice(response.into_body().as_ref())?;

            Ok(Self { value })
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::OutgoingResponse for Response<ProfileFieldName> {
        fn try_into_http_response<T: Default + bytes::BufMut>(
            self,
        ) -> Result<http::Response<T>, ruma_common::api::error::IntoHttpError> {
            Ok(http::Response::builder()
                .status(http::StatusCode::OK)
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(ruma_common::serde::json_to_buf(&self.value)?)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use ruma_common::{owned_mxc_uri, owned_user_id};
    use serde_json::{
        from_slice as from_json_slice, json, to_vec as to_json_vec, Value as JsonValue,
    };

    use super::v3::{Request, Response};
    use crate::profile::{ProfileFieldName, ProfileFieldValue};

    #[test]
    #[cfg(feature = "client")]
    fn serialize_request() {
        use ruma_common::api::{OutgoingRequest, SendAccessToken, SupportedVersions};

        let request = Request::<ProfileFieldName>::new(
            owned_user_id!("@alice:localhost"),
            ProfileFieldName::AvatarUrl,
        );

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
            http::Request::put("http://localhost/_matrix/client/unstable/uk.tcpip.msc4133/profile/@alice:localhost/displayname").body(Vec::<u8>::new()).unwrap(),
                &["@alice:localhost", "displayname"],
            ).unwrap();

        assert_eq!(request.user_id, "@alice:localhost");
        assert_eq!(request.field, ProfileFieldName::DisplayName);
    }

    #[test]
    #[cfg(feature = "server")]
    fn serialize_response() {
        use ruma_common::api::OutgoingResponse;

        let response = Response::<ProfileFieldName>::new(ProfileFieldValue::AvatarUrl(Some(
            owned_mxc_uri!("mxc://localhost/abcdef"),
        )));

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

        let response =
            Response::<ProfileFieldName>::try_from_http_response(http::Response::new(body))
                .unwrap();
        assert_eq!(response.value.field_name().as_str(), "custom_field");
        assert_eq!(response.value.value().as_str().unwrap(), "value");
    }

    /// Mock a response from the homeserver to a request of type `R` and return the given `value` as
    /// a typed response.
    #[cfg(feature = "client")]
    fn get_static_response<R: ruma_common::api::OutgoingRequest>(
        value: ProfileFieldValue,
    ) -> Result<R::IncomingResponse, ruma_common::api::error::FromHttpResponseError<R::EndpointError>>
    {
        use ruma_common::api::IncomingResponse;

        let body = to_json_vec(&value).unwrap();
        R::IncomingResponse::try_from_http_response(http::Response::new(body))
    }

    #[test]
    #[cfg(feature = "client")]
    fn static_request_and_valid_response() {
        use crate::profile::AvatarUrl;

        let response = get_static_response::<Request<PhantomData<AvatarUrl>>>(
            ProfileFieldValue::AvatarUrl(Some(owned_mxc_uri!("mxc://localhost/abcdef"))),
        )
        .unwrap();
        assert_eq!(response.value.unwrap(), "mxc://localhost/abcdef");
    }

    #[test]
    #[cfg(feature = "client")]
    fn static_request_and_invalid_response() {
        use crate::profile::AvatarUrl;

        get_static_response::<Request<PhantomData<AvatarUrl>>>(ProfileFieldValue::DisplayName(
            Some("Alice".to_owned()),
        ))
        .unwrap_err();
    }
}
