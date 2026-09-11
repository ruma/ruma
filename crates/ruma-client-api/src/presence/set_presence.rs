//! `PUT /_matrix/client/*/presence/{userId}/status`
//!
//! Set presence status for this user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.19/client-server-api/#put_matrixclientv3presenceuseridstatus

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
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/presence/{user_id}/status",
            1.1 => "/_matrix/client/v3/presence/{user_id}/status",
        }
    }

    /// Request type for the `set_presence` endpoint.
    #[request]
    #[ruma_api(manual_body_serde)]
    pub struct Request {
        /// The user whose presence state will be updated.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// The new presence state.
        pub presence: PresenceState,

        /// The status message to attach to this state.
        #[cfg(not(feature = "unstable-msc4532"))]
        pub status_msg: Option<String>,

        /// The status information to attach to this state.
        ///
        /// This field uses the unstable prefix defined in [MSC4532].
        ///
        /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
        #[cfg(feature = "unstable-msc4532")]
        pub status: PresenceStatus,
    }

    /// Response type for the `set_presence` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given user ID and presence state.
        pub fn new(user_id: OwnedUserId, presence: PresenceState) -> Self {
            Self {
                user_id,
                presence,
                #[cfg(not(feature = "unstable-msc4532"))]
                status_msg: None,
                #[cfg(feature = "unstable-msc4532")]
                status: PresenceStatus::default(),
            }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }

    #[derive(Serialize, Deserialize)]
    struct RequestBodyRepr {
        /// The new presence state.
        pub presence: PresenceState,

        /// The status message to attach to this state.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status_msg: Option<String>,

        /// The status information to attach to this state.
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
    }

    impl<'de> Deserialize<'de> for RequestBody {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let RequestBodyRepr {
                presence,
                status_msg,
                #[cfg(feature = "unstable-msc4532")]
                status,
            } = RequestBodyRepr::deserialize(deserializer)?;

            Ok(Self {
                presence,
                #[cfg(not(feature = "unstable-msc4532"))]
                status_msg: status_msg.clone(),
                // If MSC4532 is enabled, utilize the legacy field if that's all we have
                #[cfg(feature = "unstable-msc4532")]
                status: if status == PresenceStatus::default() {
                    PresenceStatus::new(status_msg)
                } else {
                    status
                },
            })
        }
    }

    impl Serialize for RequestBody {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            RequestBodyRepr {
                presence: self.presence.clone(),

                // If MSC4532 is enabled, set the legacy field for backwards compatibility
                #[cfg(not(feature = "unstable-msc4532"))]
                status_msg: self.status_msg.clone(),
                #[cfg(feature = "unstable-msc4532")]
                status_msg: self.status.msg.clone(),

                #[cfg(feature = "unstable-msc4532")]
                status: self.status.clone(),
            }
            .serialize(serializer)
        }
    }
}
