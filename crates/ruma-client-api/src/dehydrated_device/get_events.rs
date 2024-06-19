//! `POST /_matrix/client/*/dehydrated_device/{device_id}/events`
//!
//! Get to-device events for a dehydrated device.

pub mod unstable {
    //! `msc3814` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3814

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        OwnedDeviceId,
    };
    use ruma_events::AnyToDeviceEvent;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3814.v1/dehydrated_device/{device_id}/events",
        }
    };

    /// Request type for the `dehydrated_device/{device_id}/events` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The unique ID of the device for which we would like to fetch events.
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
        /// The batch token to supply in the `since` param of the next `/events` request. Will be
        /// none if no further events can be found.
        pub next_batch: Option<String>,

        /// Messages sent directly between devices.
        pub events: Vec<Raw<AnyToDeviceEvent>>,
    }

    impl Request {
        /// Create a new request.
        pub fn new(device_id: OwnedDeviceId) -> Self {
            Self { device_id, next_batch: None }
        }
    }

    impl Response {
        /// Create a new response with the given events.
        pub fn new(events: Vec<Raw<AnyToDeviceEvent>>) -> Self {
            Self { next_batch: None, events }
        }
    }
}
