//! `PUT /_matrix/client/*/devices/{deviceId}`
//!
//! Update metadata for a device.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3devicesdeviceid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedDeviceId,
    };

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/devices/{device_id}",
            1.1 => "/_matrix/client/v3/devices/{device_id}",
        }
    };

    /// Request type for the `update_device` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The device to update.
        #[ruma_api(path)]
        pub device_id: OwnedDeviceId,

        /// The new display name for this device.
        ///
        /// If this is `None`, the display name won't be changed.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub display_name: Option<String>,
    }

    /// Response type for the `update_device` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given device ID.
        pub fn new(device_id: OwnedDeviceId) -> Self {
            Self { device_id, display_name: None }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
