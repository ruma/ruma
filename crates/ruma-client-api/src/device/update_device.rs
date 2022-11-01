//! `PUT /_matrix/client/*/devices/{deviceId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#put_matrixclientv3devicesdeviceid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, DeviceId,
    };

    const METADATA: Metadata = metadata! {
        description: "Update metadata for a device.",
        method: PUT,
        name: "update_device",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/devices/:device_id",
            1.1 => "/_matrix/client/v3/devices/:device_id",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The device to update.
        #[ruma_api(path)]
        pub device_id: &'a DeviceId,

        /// The new display name for this device.
        ///
        /// If this is `None`, the display name won't be changed.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub display_name: Option<&'a str>,
    }

    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given device ID.
        pub fn new(device_id: &'a DeviceId) -> Self {
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
