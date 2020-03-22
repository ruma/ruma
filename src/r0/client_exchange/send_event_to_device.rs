//! [PUT /_matrix/client/r0/sendToDevice/{eventType}/{txnId}](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-sendtodevice-eventtype-txnid)

use std::collections::HashMap;

use ruma_api::ruma_api;
use ruma_events::{collections::all, EventResult};
use ruma_identifiers::UserId;

use super::DeviceIdOrAllDevices;

ruma_api! {
    metadata {
        description: "Send an event to a device or devices.",
        method: PUT,
        name: "send_event_to_device",
        path: "/_matrix/client/r0/sendToDevice/:event_type/:txn_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// Type of event being sent to each device.
        #[ruma_api(path)]
        pub event_type: String,
        /// A request identifier unique to the access token used to send the request.
        #[ruma_api(path)]
        pub txn_id: String,
        /// A map of users to devices to a message event to be sent to the user's
        /// device. Individual message events can be sent to devices, but all
        /// events must be of the same type.
        #[wrap_incoming(all::Event with EventResult)]
        pub messages: HashMap<UserId, HashMap<DeviceIdOrAllDevices, all::Event>>
    }

    response {}

    error: crate::Error
}
