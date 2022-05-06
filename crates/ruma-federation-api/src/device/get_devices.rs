//! `GET /_matrix/federation/*/user/devices/{userId}`
//!
//! Endpoint to get information about a user's devices

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#get_matrixfederationv1userdevicesuserid

    use js_int::UInt;
    use ruma_common::{
        api::ruma_api,
        encryption::{CrossSigningKey, DeviceKeys},
        serde::Raw,
        OwnedDeviceId, OwnedUserId, UserId,
    };
    use serde::{Deserialize, Serialize};

    ruma_api! {
        metadata: {
            description: "Gets information on all of the user's devices.",
            name: "get_devices",
            method: GET,
            stable_path: "/_matrix/federation/v1/user/devices/:user_id",
            rate_limited: false,
            authentication: ServerSignatures,
            added: 1.0,
        }

        request: {
            /// The user ID to retrieve devices for.
            ///
            /// Must be a user local to the receiving homeserver.
            #[ruma_api(path)]
            pub user_id: &'a UserId,
        }

        response: {
            /// The user ID devices were requested for.
            pub user_id: OwnedUserId,

            /// A unique ID for a given user_id which describes the version of the returned device list.
            ///
            /// This is matched with the `stream_id` field in `m.device_list_update` EDUs in order to
            /// incrementally update the returned device_list.
            pub stream_id: UInt,

            /// The user's devices.
            pub devices: Vec<UserDevice>,

            /// The user's master key.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub master_key: Option<Raw<CrossSigningKey>>,

            /// The users's self-signing key.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub self_signing_key: Option<Raw<CrossSigningKey>>,
        }
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user id.
        pub fn new(user_id: &'a UserId) -> Self {
            Self { user_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given user id and stream id.
        ///
        /// The device list will be empty.
        pub fn new(user_id: OwnedUserId, stream_id: UInt) -> Self {
            Self {
                user_id,
                stream_id,
                devices: Vec::new(),
                master_key: None,
                self_signing_key: None,
            }
        }
    }

    /// Information about a user's device.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct UserDevice {
        /// The device ID.
        pub device_id: OwnedDeviceId,

        /// Identity keys for the device.
        pub keys: Raw<DeviceKeys>,

        /// Optional display name for the device
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_display_name: Option<String>,
    }

    impl UserDevice {
        /// Creates a new `UserDevice` with the given device id and keys.
        pub fn new(device_id: OwnedDeviceId, keys: Raw<DeviceKeys>) -> Self {
            Self { device_id, keys, device_display_name: None }
        }
    }
}
