//! `PUT /_matrix/client/*/profile/{userId}/avatar_url`
//!
//! Set the avatar URL of the user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#put_matrixclientv3profileuseridavatar_url

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, MxcUri, UserId,
    };

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/profile/:user_id/avatar_url",
            1.1 => "/_matrix/client/v3/profile/:user_id/avatar_url",
        }
    };

    /// Request type for the `set_avatar_url` endpoint.
    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The user whose avatar URL will be set.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The new avatar URL for the user.
        ///
        /// `None` is used to unset the avatar.
        ///
        /// If you activate the `compat` feature, this field being an empty string in JSON will
        /// result in `None` here during deserialization.
        #[cfg_attr(
            feature = "compat",
            serde(
                default,
                deserialize_with = "ruma_common::serde::empty_string_as_none",
                serialize_with = "ruma_common::serde::none_as_empty_string"
            )
        )]
        #[cfg_attr(not(feature = "compat"), serde(skip_serializing_if = "Option::is_none"))]
        pub avatar_url: Option<&'a MxcUri>,

        /// The [BlurHash](https://blurha.sh) for the avatar pointed to by `avatar_url`.
        ///
        /// This uses the unstable prefix in
        /// [MSC2448](https://github.com/matrix-org/matrix-spec-proposals/pull/2448).
        #[cfg(feature = "unstable-msc2448")]
        #[serde(rename = "xyz.amorgan.blurhash", skip_serializing_if = "Option::is_none")]
        pub blurhash: Option<&'a str>,
    }

    /// Response type for the `set_avatar_url` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user ID and avatar URL.
        pub fn new(user_id: &'a UserId, avatar_url: Option<&'a MxcUri>) -> Self {
            Self {
                user_id,
                avatar_url,
                #[cfg(feature = "unstable-msc2448")]
                blurhash: None,
            }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }

    #[cfg(all(test, feature = "server"))]
    mod tests {
        use ruma_common::api::IncomingRequest as _;

        use super::IncomingRequest;

        #[test]
        fn deserialize_unset_request() {
            let req = IncomingRequest::try_from_http_request(
                http::Request::builder()
                    .method("PUT")
                    .uri("https://bar.org/_matrix/client/r0/profile/@foo:bar.org/avatar_url")
                    .body(&[] as &[u8])
                    .unwrap(),
                &["@foo:bar.org"],
            )
            .unwrap();
            assert_eq!(req.user_id, "@foo:bar.org");
            assert_eq!(req.avatar_url, None);

            #[cfg(feature = "compat")]
            {
                let req = IncomingRequest::try_from_http_request(
                    http::Request::builder()
                        .method("PUT")
                        .uri("https://bar.org/_matrix/client/r0/profile/@foo:bar.org/avatar_url")
                        .body(serde_json::to_vec(&serde_json::json!({ "avatar_url": "" })).unwrap())
                        .unwrap(),
                    &["@foo:bar.org"],
                )
                .unwrap();
                assert_eq!(req.user_id, "@foo:bar.org");
                assert_eq!(req.avatar_url, None);
            }
        }
    }
}
