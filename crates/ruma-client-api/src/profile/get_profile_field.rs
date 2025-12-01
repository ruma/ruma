//! `GET /_matrix/client/*/profile/{userId}/{key_name}`
//!
//! Get a field in the profile of the user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! Although this endpoint has a similar format to [`get_avatar_url`] and [`get_display_name`],
    //! it will only work with homeservers advertising support for the proper unstable feature or
    //! a version compatible with Matrix 1.16.
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3profileuseridkeyname
    //! [`get_avatar_url`]: crate::profile::get_avatar_url
    //! [`get_display_name`]: crate::profile::get_display_name

    use std::marker::PhantomData;

    use ruma_common::{
        OwnedUserId,
        api::{Metadata, auth_scheme::NoAuthentication, path_builder::VersionHistory},
        metadata,
    };

    use crate::profile::{ProfileFieldName, ProfileFieldValue, StaticProfileField};

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        // History valid for fields that existed in Matrix 1.0, i.e. `displayname` and `avatar_url`.
        history: {
            unstable("uk.tcpip.msc4133") => "/_matrix/client/unstable/uk.tcpip.msc4133/profile/{user_id}/{field}",
            1.0 => "/_matrix/client/r0/profile/{user_id}/{field}",
            1.1 => "/_matrix/client/v3/profile/{user_id}/{field}",
        }
    }

    /// Request type for the `get_profile_field` endpoint.
    #[derive(Clone, Debug)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Request {
        /// The user whose profile will be fetched.
        pub user_id: OwnedUserId,

        /// The profile field to get.
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

    #[cfg(feature = "client")]
    impl ruma_common::api::OutgoingRequest for Request {
        type EndpointError = crate::Error;
        type IncomingResponse = Response;

        fn try_into_http_request<T: Default + bytes::BufMut + AsRef<[u8]>>(
            self,
            base_url: &str,
            access_token: ruma_common::api::auth_scheme::SendAccessToken<'_>,
            considering: std::borrow::Cow<'_, ruma_common::api::SupportedVersions>,
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            use ruma_common::api::{auth_scheme::AuthScheme, path_builder::PathBuilder};

            let url = if self.field.existed_before_extended_profiles() {
                Self::make_endpoint_url(considering, base_url, &[&self.user_id, &self.field], "")?
            } else {
                crate::profile::EXTENDED_PROFILE_FIELD_HISTORY.make_endpoint_url(
                    considering,
                    base_url,
                    &[&self.user_id, &self.field],
                    "",
                )?
            };

            let mut http_request =
                http::Request::builder().method(Self::METHOD).uri(url).body(T::default())?;

            Self::Authentication::add_authentication(&mut http_request, access_token)?;

            Ok(http_request)
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::IncomingRequest for Request {
        type EndpointError = crate::Error;
        type OutgoingResponse = Response;

        fn try_from_http_request<B, S>(
            request: http::Request<B>,
            path_args: &[S],
        ) -> Result<Self, ruma_common::api::error::FromHttpRequestError>
        where
            B: AsRef<[u8]>,
            S: AsRef<str>,
        {
            Self::check_request_method(request.method())?;

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

    impl<F: StaticProfileField> Metadata for RequestStatic<F> {
        const METHOD: http::Method = Request::METHOD;
        const RATE_LIMITED: bool = Request::RATE_LIMITED;
        type Authentication = <Request as Metadata>::Authentication;
        type PathBuilder = <Request as Metadata>::PathBuilder;
        const PATH_BUILDER: VersionHistory = Request::PATH_BUILDER;
    }

    #[cfg(feature = "client")]
    impl<F: StaticProfileField> ruma_common::api::OutgoingRequest for RequestStatic<F> {
        type EndpointError = crate::Error;
        type IncomingResponse = ResponseStatic<F>;

        fn try_into_http_request<T: Default + bytes::BufMut + AsRef<[u8]>>(
            self,
            base_url: &str,
            access_token: ruma_common::api::auth_scheme::SendAccessToken<'_>,
            considering: std::borrow::Cow<'_, ruma_common::api::SupportedVersions>,
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

            use crate::profile::profile_field_serde::deserialize_profile_field_value_option;

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
                .header(http::header::CONTENT_TYPE, ruma_common::http_headers::APPLICATION_JSON)
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

            use crate::profile::profile_field_serde::StaticProfileFieldVisitor;

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

#[cfg(all(test, feature = "client"))]
mod tests_client {
    use ruma_common::{owned_mxc_uri, owned_user_id};
    use serde_json::{json, to_vec as to_json_vec};

    use super::v3::{Request, RequestStatic, Response};
    use crate::profile::{ProfileFieldName, ProfileFieldValue};

    #[test]
    fn serialize_request() {
        use std::borrow::Cow;

        use ruma_common::api::{OutgoingRequest, SupportedVersions, auth_scheme::SendAccessToken};

        // Profile field that existed in Matrix 1.0.
        let avatar_url_request =
            Request::new(owned_user_id!("@alice:localhost"), ProfileFieldName::AvatarUrl);

        // Matrix 1.11
        let http_request = avatar_url_request
            .clone()
            .try_into_http_request::<Vec<u8>>(
                "http://localhost/",
                SendAccessToken::None,
                Cow::Owned(SupportedVersions::from_parts(
                    &["v1.11".to_owned()],
                    &Default::default(),
                )),
            )
            .unwrap();
        assert_eq!(
            http_request.uri().path(),
            "/_matrix/client/v3/profile/@alice:localhost/avatar_url"
        );

        // Matrix 1.16
        let http_request = avatar_url_request
            .try_into_http_request::<Vec<u8>>(
                "http://localhost/",
                SendAccessToken::None,
                Cow::Owned(SupportedVersions::from_parts(
                    &["v1.16".to_owned()],
                    &Default::default(),
                )),
            )
            .unwrap();
        assert_eq!(
            http_request.uri().path(),
            "/_matrix/client/v3/profile/@alice:localhost/avatar_url"
        );

        // Profile field that didn't exist in Matrix 1.0.
        let custom_field_request =
            Request::new(owned_user_id!("@alice:localhost"), "dev.ruma.custom_field".into());

        // Matrix 1.11
        let http_request = custom_field_request
            .clone()
            .try_into_http_request::<Vec<u8>>(
                "http://localhost/",
                SendAccessToken::None,
                Cow::Owned(SupportedVersions::from_parts(
                    &["v1.11".to_owned()],
                    &Default::default(),
                )),
            )
            .unwrap();
        assert_eq!(
            http_request.uri().path(),
            "/_matrix/client/unstable/uk.tcpip.msc4133/profile/@alice:localhost/dev.ruma.custom_field"
        );

        // Matrix 1.16
        let http_request = custom_field_request
            .try_into_http_request::<Vec<u8>>(
                "http://localhost/",
                SendAccessToken::None,
                Cow::Owned(SupportedVersions::from_parts(
                    &["v1.16".to_owned()],
                    &Default::default(),
                )),
            )
            .unwrap();
        assert_eq!(
            http_request.uri().path(),
            "/_matrix/client/v3/profile/@alice:localhost/dev.ruma.custom_field"
        );
    }

    #[test]
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
    fn static_request_and_invalid_response() {
        use crate::profile::AvatarUrl;

        get_static_response::<RequestStatic<AvatarUrl>>(Some(ProfileFieldValue::DisplayName(
            "Alice".to_owned(),
        )))
        .unwrap_err();
    }
}

#[cfg(all(test, feature = "server"))]
mod tests_server {
    use ruma_common::owned_mxc_uri;
    use serde_json::{Value as JsonValue, from_slice as from_json_slice, json};

    use super::v3::{Request, Response};
    use crate::profile::{ProfileFieldName, ProfileFieldValue};

    #[test]
    fn deserialize_request() {
        use ruma_common::api::IncomingRequest;

        let request = Request::try_from_http_request(
            http::Request::get(
                "http://localhost/_matrix/client/v3/profile/@alice:localhost/displayname",
            )
            .body(Vec::<u8>::new())
            .unwrap(),
            &["@alice:localhost", "displayname"],
        )
        .unwrap();

        assert_eq!(request.user_id, "@alice:localhost");
        assert_eq!(request.field, ProfileFieldName::DisplayName);
    }

    #[test]
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
}
