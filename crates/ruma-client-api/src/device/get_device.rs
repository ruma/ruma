//! `GET /_matrix/client/*/devices/{deviceId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3devicesdeviceid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, DeviceId,
    };

    use crate::device::Device;

    const METADATA: Metadata = metadata! {
        description: "Get a device for authenticated user.",
        method: GET,
        name: "get_device",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/devices/:device_id",
            1.1 => "/_matrix/client/v3/devices/:device_id",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The device to retrieve.
        #[ruma_api(path)]
        pub device_id: &'a DeviceId,
    }

    #[response(error = crate::Error)]
    pub struct Response {
        /// Information about the device.
        #[ruma_api(body)]
        pub device: Device,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given device ID.
        pub fn new(device_id: &'a DeviceId) -> Self {
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
