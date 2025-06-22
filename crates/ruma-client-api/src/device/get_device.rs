//! `GET /_matrix/client/*/devices/{deviceId}`
//!
//! Get a device for authenticated user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3devicesdeviceid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedDeviceId,
    };

    use crate::device::Device;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/devices/{device_id}",
            1.1 => "/_matrix/client/v3/devices/{device_id}",
        }
    };

    /// Request type for the `get_device` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The device to retrieve.
        #[ruma_api(path)]
        pub device_id: OwnedDeviceId,
    }

    /// Response type for the `get_device` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// Information about the device.
        #[ruma_api(body)]
        pub device: Device,
    }

    impl Request {
        /// Creates a new `Request` with the given device ID.
        pub fn new(device_id: OwnedDeviceId) -> Self {
            Self { device_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given device.
        pub fn new(device: Device) -> Self {
            Self { device }
        }
    }
}
