//! [PUT /_matrix/client/r0/sendToDevice/{eventType}/{txnId}](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-sendtodevice-eventtype-txnid)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_common::to_device::DeviceIdOrAllDevices;
use ruma_events::EventType;
use ruma_identifiers::UserId;
use serde_json::value::RawValue as RawJsonValue;

ruma_api! {
    metadata: {
        description: "Send an event to a device or devices.",
        method: PUT,
        name: "send_event_to_device",
        path: "/_matrix/client/r0/sendToDevice/:event_type/:txn_id",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// Type of event being sent to each device.
        #[ruma_api(path)]
        pub event_type: EventType,

        /// A request identifier unique to the access token used to send the request.
        #[ruma_api(path)]
        pub txn_id: &'a str,

        /// A map of users to devices to a content for a message event to be
        /// sent to the user's device. Individual message events can be sent
        /// to devices, but all events must be of the same type.
        /// The content's type for this field will be updated in a future
        /// release, until then you can create a value using
        /// `serde_json::value::to_raw_value`.
        pub messages: BTreeMap<UserId, BTreeMap<DeviceIdOrAllDevices, Box<RawJsonValue>>>,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given event type, transaction ID and messages.
    pub fn new(
        event_type: EventType,
        txn_id: &'a str,
        messages: BTreeMap<UserId, BTreeMap<DeviceIdOrAllDevices, Box<RawJsonValue>>>,
    ) -> Self {
        Self { event_type, txn_id, messages }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
