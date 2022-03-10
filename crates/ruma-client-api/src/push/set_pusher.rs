//! `POST /_matrix/client/*/pushers/set`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3pushersset

    use ruma_common::api::ruma_api;
    use serde::{Deserialize, Serialize};

    use crate::push::{PusherData, PusherKind};

    ruma_api! {
        metadata: {
            description: "This endpoint allows the creation, modification and deletion of pushers for this user ID.",
            method: POST,
            name: "set_pusher",
            r0_path: "/_matrix/client/r0/pushers/set",
            stable_path: "/_matrix/client/v3/pushers/set",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The pusher to configure.
            #[serde(flatten)]
            pub pusher: Pusher,

            /// Controls if another pusher with the same pushkey and app id should be created.
            ///
            /// Defaults to `false`. See the spec for more details.
            #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
            pub append: bool,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl Request {
        /// Creates a new `Request` with the given pusher.
        pub fn new(pusher: Pusher) -> Self {
            Self { pusher, append: false }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }

    /// Defines a pusher.
    ///
    /// To create an instance of this type, first create a `PusherInit` and convert it via
    /// `Pusher::from` / `.into()`.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct Pusher {
        /// A unique identifier for this pusher.
        ///
        /// The maximum allowed length is 512 bytes.
        pub pushkey: String,

        /// The kind of the pusher.
        ///
        /// `None` deletes the pusher.
        pub kind: Option<PusherKind>,

        /// A reverse-DNS style identifier for the application.
        ///
        /// The maximum allowed length is 64 bytes.
        pub app_id: String,

        /// A string that will allow the user to identify what application owns this pusher.
        pub app_display_name: String,

        /// A string that will allow the user to identify what device owns this pusher.
        pub device_display_name: String,

        /// Determines which set of device specific rules this pusher executes.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub profile_tag: Option<String>,

        /// The preferred language for receiving notifications (e.g. 'en' or 'en-US')
        pub lang: String,

        /// Information for the pusher implementation itself.
        pub data: PusherData,
    }

    /// Initial set of fields of `Pusher`.
    ///
    /// This struct will not be updated even if additional fields are added to `Pusher` in a new
    /// (non-breaking) release of the Matrix specification.
    #[derive(Debug)]
    #[allow(clippy::exhaustive_structs)]
    pub struct PusherInit {
        /// A unique identifier for this pusher.
        ///
        /// The maximum allowed length is 512 bytes.
        pub pushkey: String,

        /// The kind of the pusher.
        ///
        /// `None` deletes the pusher.
        pub kind: Option<PusherKind>,

        /// A reverse-DNS style identifier for the application.
        ///
        /// The maximum allowed length is 64 bytes.
        pub app_id: String,

        /// A string that will allow the user to identify what application owns this pusher.
        pub app_display_name: String,

        /// A string that will allow the user to identify what device owns this pusher.
        pub device_display_name: String,

        /// Determines which set of device specific rules this pusher executes.
        pub profile_tag: Option<String>,

        /// The preferred language for receiving notifications (e.g. 'en' or 'en-US')
        pub lang: String,

        /// Information for the pusher implementation itself.
        pub data: PusherData,
    }

    impl From<PusherInit> for Pusher {
        fn from(init: PusherInit) -> Self {
            let PusherInit {
                pushkey,
                kind,
                app_id,
                app_display_name,
                device_display_name,
                profile_tag,
                lang,
                data,
            } = init;
            Self {
                pushkey,
                kind,
                app_id,
                app_display_name,
                device_display_name,
                profile_tag,
                lang,
                data,
            }
        }
    }
}
