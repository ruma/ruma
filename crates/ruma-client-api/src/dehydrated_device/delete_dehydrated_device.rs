//! `DELETE /_matrix/client/*/dehydrated_device/`
//!
//! Delete a dehydrated device.

pub mod unstable {
    //! `msc3814` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3814

    use ruma_common::{
        DeviceId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: DELETE,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3814.v1/dehydrated_device",
        }
    }

    /// Request type for the `DELETE` `dehydrated_device` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {}

    /// Request type for the `DELETE` `dehydrated_device` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The unique ID of the device that was deleted.
        pub device_id: DeviceId,
    }

    impl Request {
        /// Creates a new empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given device ID.
        pub fn new(device_id: DeviceId) -> Self {
            Self { device_id }
        }
    }
}
