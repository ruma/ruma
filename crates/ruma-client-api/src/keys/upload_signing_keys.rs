//! `POST /_matrix/client/*/keys/device_signing/upload`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#post_matrixclientv3keysdevice_signingupload

    use ruma_common::{
        api::{request, response, Metadata},
        encryption::CrossSigningKey,
        metadata,
        serde::Raw,
    };

    use crate::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

    const METADATA: Metadata = metadata! {
        description: "Publishes cross signing keys for the user.",
        method: POST,
        name: "upload_signing_keys",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/keys/device_signing/upload",
            1.1 => "/_matrix/client/v3/keys/device_signing/upload",
        }
    };

    #[request(error = UiaaResponse)]
    #[derive(Default)]
    pub struct Request<'a> {
        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData<'a>>,

        /// The user's master key.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub master_key: Option<Raw<CrossSigningKey>>,

        /// The user's self-signing key.
        ///
        /// Must be signed with the accompanied master, or by the user's most recently uploaded
        /// master key if no master key is included in the request.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub self_signing_key: Option<Raw<CrossSigningKey>>,

        /// The user's user-signing key.
        ///
        /// Must be signed with the accompanied master, or by the user's most recently uploaded
        /// master key if no master key is included in the request.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_signing_key: Option<Raw<CrossSigningKey>>,
    }

    #[response(error = UiaaResponse)]
    #[derive(Default)]
    pub struct Response {}

    impl Request<'_> {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
