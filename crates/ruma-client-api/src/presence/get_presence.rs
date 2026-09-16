//! `GET /_matrix/client/*/presence/{userId}/status`
//!
//! Get presence status for this user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.19/client-server-api/#get_matrixclientv3presenceuseridstatus

    use std::time::Duration;

    #[cfg(feature = "unstable-msc4532")]
    use ruma_common::presence::PresenceStatus;
    use ruma_common::{
        OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        presence::PresenceState,
    };
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/presence/{user_id}/status",
            1.1 => "/_matrix/client/v3/presence/{user_id}/status",
        }
    }

    /// Request type for the `get_presence` endpoint.
    #[request]
    pub struct Request {
        /// The user whose presence state will be retrieved.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// Whether to use [MSC4532]'s revised presence states (if the server supports them).
        ///
        /// Defaults to `false`.
        ///
        /// This uses the unstable prefix defined in [MSC4532].
        ///
        /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
        #[cfg(feature = "unstable-msc4532")]
        #[serde(
            default,
            skip_serializing_if = "ruma_common::serde::is_default",
            rename = "org.continuwuity.presence_v2.msc4532.revised_presence"
        )]
        #[ruma_api(query)]
        pub revised_presence: bool,
    }

    /// Response type for the `get_presence` endpoint.
    #[response]
    #[ruma_api(manual_body_serde)]
    pub struct Response {
        /// The state message for this user if one was set.
        #[cfg(not(feature = "unstable-msc4532"))]
        pub status_msg: Option<String>,

        /// The status information for this user's presence.
        ///
        /// This field uses the unstable prefix defined in [MSC4532].
        ///
        /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
        #[cfg(feature = "unstable-msc4532")]
        pub status: PresenceStatus,

        /// Whether or not the user is currently active.
        pub currently_active: Option<bool>,

        /// The length of time in milliseconds since an action was performed by the user.
        pub last_active_ago: Option<Duration>,

        /// The user's presence state.
        pub presence: PresenceState,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self {
                user_id,
                #[cfg(feature = "unstable-msc4532")]
                revised_presence: true,
            }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given presence state.
        pub fn new(presence: PresenceState) -> Self {
            Self {
                presence,
                #[cfg(not(feature = "unstable-msc4532"))]
                status_msg: None,
                #[cfg(feature = "unstable-msc4532")]
                status: PresenceStatus::default(),
                currently_active: None,
                last_active_ago: None,
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    struct ResponseBodyRepr {
        /// The state message for this user if one was set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status_msg: Option<String>,

        /// The status information for this user's presence.
        ///
        /// This field uses the unstable prefix defined in [MSC4532].
        ///
        /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
        #[serde(
            skip_serializing_if = "ruma_common::serde::is_default",
            rename = "org.continuwuity.presence_v2.msc4532.status",
            default
        )]
        #[cfg(feature = "unstable-msc4532")]
        pub status: PresenceStatus,

        /// Whether or not the user is currently active.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub currently_active: Option<bool>,

        /// The length of time in milliseconds since an action was performed by the user.
        #[serde(
            with = "ruma_common::serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        pub last_active_ago: Option<Duration>,

        /// The user's presence state.
        pub presence: PresenceState,
    }

    impl<'de> Deserialize<'de> for ResponseBody {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let ResponseBodyRepr {
                status_msg,
                #[cfg(feature = "unstable-msc4532")]
                status,
                currently_active,
                last_active_ago,
                presence,
            } = ResponseBodyRepr::deserialize(deserializer)?;

            Ok(Self {
                #[cfg(not(feature = "unstable-msc4532"))]
                status_msg,
                // If MSC4532 is enabled, utilize the legacy field if that's all we have
                #[cfg(feature = "unstable-msc4532")]
                status: if status == PresenceStatus::default() {
                    PresenceStatus::new(status_msg)
                } else {
                    status
                },
                currently_active,
                last_active_ago,
                presence,
            })
        }
    }

    impl Serialize for ResponseBody {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            ResponseBodyRepr {
                // If MSC4532 is enabled, set the legacy field for backwards compatibility
                #[cfg(not(feature = "unstable-msc4532"))]
                status_msg: self.status_msg.clone(),
                #[cfg(feature = "unstable-msc4532")]
                status_msg: self.status.msg.clone(),

                #[cfg(feature = "unstable-msc4532")]
                status: self.status.clone(),
                currently_active: self.currently_active,
                last_active_ago: self.last_active_ago,
                presence: self.presence.clone(),
            }
            .serialize(serializer)
        }
    }
}
