//! `GET /_matrix/client/*/dehydrated_device/{device_id}/events`
//!
//! Get to-device events for a dehydrated device.

pub mod unstable {
    //! `msc3814` ([MSC])
    //!
    //! [MSC]: https://github.com/uhoreg/matrix-doc/blob/shrivelled_sessions/proposals/3814-dehydrated-devices-with-ssss.md

    use ruma_common::{
        api::{request, response, Metadata},
        events::AnyToDeviceEvent,
        metadata,
        serde::Raw,
        OwnedDeviceId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3814.v1/dehydrated_device/:device_id/events",
        }
    };

    /// Request type for the `dehydrated_device/{device_id}/events` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The unique ID of the device for which we would like to fetch events for.
        #[ruma_api(path)]
        pub device_id: OwnedDeviceId,
        /// A point in time to continue getting events from.
        ///
        /// Should be a token from the `next_batch` field of a previous `/events`
        /// request.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_batch: Option<String>,
    }

    /// Request type for the `dehydrated_device/{device_id}/events` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The batch token to supply in the `since` param of the next `/events` request.
        pub next_batch: String,

        /// Messages sent directly between devices.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub events: Vec<Raw<AnyToDeviceEvent>>,
    }

    impl Request {
        /// Create a new request.
        pub fn new(device_id: OwnedDeviceId) -> Self {
            Self { device_id, next_batch: None }
        }
    }
}
