//! `POST /_matrix/client/*/profile/query`
//!
//! Get profile information of multiple users.

pub mod unstable {
    //! `msc4536` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4536

    use std::collections::BTreeMap;

    use ruma_common::{
        OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        profile::{ProfileFieldName, UserProfile},
    };

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("org.matrix.msc4536") => "/_matrix/client/unstable/org.matrix.msc4536/profile/query",
        }
    }

    /// Request type for the `query_profile` endpoint.
    #[request]
    pub struct Request {
        /// The user IDs to query for.
        ///
        /// The homeserver may reject requests for over 100 users.
        pub users: Vec<OwnedUserId>,

        /// List of fields to include in the response.
        ///
        /// If `None`, all profile fields will be returned.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub fields: Option<Vec<ProfileFieldName>>,
    }

    /// Response type for the `query_profile` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// The requested profile information.
        #[ruma_api(body)]
        pub profiles: BTreeMap<OwnedUserId, UserProfile>,
    }

    impl Request {
        /// Creates a new `Request` with the given user IDs.
        pub fn new(users: Vec<OwnedUserId>) -> Self {
            Self { users, fields: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given profiles.
        pub fn new(profiles: BTreeMap<OwnedUserId, UserProfile>) -> Self {
            Self { profiles }
        }
    }
}

#[cfg(all(test, feature = "client"))]
mod tests {
    use std::borrow::Cow;

    use ruma_common::{
        api::{
            IncomingResponseExt as _, MatrixVersion, OutgoingRequestExt as _, SupportedVersions,
            auth_scheme::SendAccessToken,
        },
        owned_user_id,
    };
    use serde_json::{Value as JsonValue, json};

    use super::unstable::{Request, Response};
    use crate::profile::{AvatarUrl, DisplayName, ProfileFieldName};

    #[test]
    fn serialize_request() {
        let supported = SupportedVersions {
            versions: [MatrixVersion::V1_1].into(),
            features: Default::default(),
        };
        let mut request = Request::new(vec![
            owned_user_id!("@alice:localhost"),
            owned_user_id!("@bob:localhost"),
        ]);
        request.fields = Some(vec![ProfileFieldName::DisplayName, ProfileFieldName::AvatarUrl]);

        let request: http::Request<Vec<u8>> = request
            .try_into_http_request(
                "https://homeserver.tld",
                SendAccessToken::IfRequired("auth_tok"),
                Cow::Owned(supported),
            )
            .unwrap();
        let (parts, body) = request.into_parts();

        assert_eq!(
            "https://homeserver.tld/_matrix/client/unstable/org.matrix.msc4536/profile/query",
            parts.uri
        );
        assert_eq!("POST", parts.method);
        assert_eq!(
            json!({
                "users": ["@alice:localhost", "@bob:localhost"],
                "fields": ["displayname", "avatar_url"],
            }),
            serde_json::from_slice::<JsonValue>(&body).unwrap()
        );
    }

    #[test]
    fn deserialize_response() {
        let body = json!({
            "@alice:localhost": {
                "avatar_url": "mxc://localhost/abcdef",
                "displayname": "Alice",
            },
            "@bob:localhost": {
                "displayname": "Bob",
            },
        })
        .to_string();

        let response =
            Response::try_from_http_response(http::Response::new(body.as_bytes())).unwrap();
        assert_eq!(response.profiles.len(), 2);

        let alice = &response.profiles["@alice:localhost"];
        assert_eq!(alice.get_static::<AvatarUrl>().unwrap().unwrap(), "mxc://localhost/abcdef");
        assert_eq!(alice.get_static::<DisplayName>().unwrap().unwrap(), "Alice");

        let bob = &response.profiles["@bob:localhost"];
        assert_eq!(bob.get_static::<AvatarUrl>().unwrap(), None);
        assert_eq!(bob.get_static::<DisplayName>().unwrap().unwrap(), "Bob");
    }
}
