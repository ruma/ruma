//! `GET /_matrix/client/*/devices/{deviceId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3devicesdeviceid

    use ruma_common::{api::ruma_api, DeviceId};

    use crate::device::Device;

    ruma_api! {
        metadata: {
            description: "Get a device for authenticated user.",
            method: GET,
            name: "get_device",
            r0_path: "/_matrix/client/r0/devices/:device_id",
            stable_path: "/_matrix/client/v3/devices/:device_id",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The device to retrieve.
            #[ruma_api(path)]
            pub device_id: &'a DeviceId,
        }

        response: {
            /// Information about the device.
            #[ruma_api(body)]
            pub device: Device,
        }

        error: crate::Error
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
