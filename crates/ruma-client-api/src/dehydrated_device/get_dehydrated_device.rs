//! `GET /_matrix/client/*/dehydrated_device/`
//!
//! Get a dehydrated device for rehydration.

pub mod unstable {
    //! `msc3814` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3814

    use ruma_common::{
        DeviceId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        serde::Raw,
    };

    use crate::dehydrated_device::DehydratedDeviceData;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3814.v1/dehydrated_device",
        }
    }

    /// Request type for the `GET` `dehydrated_device` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {}

    /// Request type for the `GET` `dehydrated_device` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The unique ID of the device.
        pub device_id: DeviceId,
        /// Information about the device.
        pub device_data: Raw<DehydratedDeviceData>,
    }

    impl Request {
        /// Creates a new empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given device ID and device data.
        pub fn new(device_id: DeviceId, device_data: Raw<DehydratedDeviceData>) -> Self {
            Self { device_id, device_data }
        }
    }
}
