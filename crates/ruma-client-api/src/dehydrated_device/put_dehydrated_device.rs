//! `PUT /_matrix/client/*/dehydrated_device/`
//!
//! Uploads a dehydrated device to the homeserver.

pub mod unstable {
    //! `msc3814` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3814

    use std::collections::BTreeMap;

    use ruma_common::{
        DeviceId, OneTimeKeyId,
        api::{auth_scheme::AccessToken, request, response},
        encryption::{DeviceKeys, OneTimeKey},
        metadata,
        serde::Raw,
    };

    use crate::dehydrated_device::DehydratedDeviceData;

    metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3814.v1/dehydrated_device",
        }
    }

    /// Request type for the `PUT` `dehydrated_device` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The unique ID of the device.
        pub device_id: DeviceId,

        /// The display name of the device.
        pub initial_device_display_name: Option<String>,

        /// The data of the dehydrated device, containing the serialized and encrypted private
        /// parts of the [`DeviceKeys`].
        pub device_data: Raw<DehydratedDeviceData>,

        /// Identity keys for the dehydrated device.
        pub device_keys: Raw<DeviceKeys>,

        /// One-time public keys for "pre-key" messages.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub one_time_keys: BTreeMap<OneTimeKeyId, Raw<OneTimeKey>>,

        /// Fallback public keys for "pre-key" messages.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub fallback_keys: BTreeMap<OneTimeKeyId, Raw<OneTimeKey>>,
    }

    /// Response type for the `upload_keys` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The unique ID of the device.
        pub device_id: DeviceId,
    }

    impl Request {
        /// Creates a new Request.
        pub fn new(
            device_id: DeviceId,
            device_data: Raw<DehydratedDeviceData>,
            device_keys: Raw<DeviceKeys>,
        ) -> Self {
            Self {
                device_id,
                device_data,
                device_keys,
                initial_device_display_name: None,
                one_time_keys: Default::default(),
                fallback_keys: Default::default(),
            }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given one time key counts.
        pub fn new(device_id: DeviceId) -> Self {
            Self { device_id }
        }
    }
}
