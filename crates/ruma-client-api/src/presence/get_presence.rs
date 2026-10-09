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

        /// Whether to use [MSC4532]'s revised presence states in responses (if the server supports
        /// them).
        ///
        /// Defaults to `true`.
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
        ///
        /// If the `unstable-msc4532` feature is enabled, this field is ignored during
        /// serialization, and will always have the same value as `status.msg` after
        /// deserialization.
        #[cfg_attr(
            feature = "unstable-msc4532",
            deprecated(note = "Deprecated when MSC4532 is enabled, use `status` instead")
        )]
        // required to prevent dead code warnings for the deprecated field on `ResponseBody`
        #[allow(dead_code)]
        pub status_msg: Option<String>,

        /// The status information for this user's presence.
        ///
        /// This field uses the unstable prefix defined in [MSC4532].
        ///
        /// If this field is not present at deserialization, the value of `status_msg`
        /// will be used instead.
        ///
        /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
        #[cfg(feature = "unstable-msc4532")]
        pub status: PresenceStatus,

        /// Whether the user is currently active or not.
        ///
        /// If the `unstable-msc4532` feature is enabled and `presence` is a [MSC4532]
        /// presence state, this field will be ignored and will always have the same value
        /// as [`PresenceState::currently_active`] after deserialization.
        #[cfg_attr(
            feature = "unstable-msc4532",
            deprecated(
                note = "Deprecated when MSC4532 is enabled, use `PresenceState::currently_active` instead"
            )
        )]
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
            #[allow(deprecated)]
            Self {
                presence,
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
        status_msg: Option<String>,

        /// The status information for this user's presence.
        ///
        /// This field uses the unstable prefix defined in [MSC4532].
        ///
        /// If this field is not present at deserialization, the value of `status_msg`
        /// will be used instead.
        ///
        /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
        #[serde(
            skip_serializing_if = "ruma_common::serde::is_default",
            rename = "org.continuwuity.presence_v2.msc4532.status",
            default
        )]
        #[cfg(feature = "unstable-msc4532")]
        status: PresenceStatus,

        /// Whether the user is currently active or not.
        #[serde(skip_serializing_if = "Option::is_none")]
        currently_active: Option<bool>,

        /// The length of time in milliseconds since an action was performed by the user.
        #[serde(
            with = "ruma_common::serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        last_active_ago: Option<Duration>,

        /// The user's presence state.
        presence: PresenceState,
    }

    impl<'de> Deserialize<'de> for ResponseBody {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            #[allow(deprecated, unused_variables)]
            let ResponseBodyRepr {
                status_msg,
                #[cfg(feature = "unstable-msc4532")]
                status,
                currently_active,
                last_active_ago,
                presence,
            } = ResponseBodyRepr::deserialize(deserializer)?;

            #[cfg(feature = "unstable-msc4532")]
            let status_msg =
                if status == PresenceStatus::default() { status_msg } else { status.msg };

            #[cfg(feature = "unstable-msc4532")]
            let currently_active = if presence.is_msc4532_state() {
                // ignore the provided currently_active value and use the proposal's definition
                Some(presence.currently_active())
            } else {
                currently_active
            };

            #[allow(deprecated)]
            Ok(Self {
                #[cfg(feature = "unstable-msc4532")]
                status: PresenceStatus::new(status_msg.clone()),
                status_msg,
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
            #[allow(deprecated, unused_variables)]
            let ResponseBody {
                status_msg,
                #[cfg(feature = "unstable-msc4532")]
                status,
                currently_active,
                last_active_ago,
                presence,
            } = self;

            ResponseBodyRepr {
                // If MSC4532 is enabled, set the legacy field for backwards compatibility
                #[cfg(not(feature = "unstable-msc4532"))]
                status_msg: status_msg.clone(),
                #[cfg(feature = "unstable-msc4532")]
                status_msg: status.msg.clone(),

                #[cfg(not(feature = "unstable-msc4532"))]
                currently_active: *currently_active,
                #[cfg(feature = "unstable-msc4532")]
                currently_active: Some(presence.currently_active()),

                #[cfg(feature = "unstable-msc4532")]
                status: status.clone(),
                last_active_ago: *last_active_ago,
                presence: presence.clone(),
            }
            .serialize(serializer)
        }
    }
}
