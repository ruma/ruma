//! [PUT /_matrix/client/r0/sendToDevice/{eventType}/{txnId}](https://matrix.org/docs/spec/client_server/r0.6.1#put-matrix-client-r0-sendtodevice-eventtype-txnid)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_common::to_device::DeviceIdOrAllDevices;
use ruma_events::AnyToDeviceEventContent;
use ruma_identifiers::{TransactionId, UserId};
use ruma_serde::Raw;

ruma_api! {
    metadata: {
        description: "Send an event to a device or devices.",
        method: PUT,
        name: "send_event_to_device",
        r0: "/_matrix/client/r0/sendToDevice/:event_type/:txn_id",
        stable: "/_matrix/client/v3/sendToDevice/:event_type/:txn_id",
        rate_limited: false,
        authentication: AccessToken,
        added: 1.0,
    }

    request: {
        /// Type of event being sent to each device.
        #[ruma_api(path)]
        pub event_type: &'a str,

        /// A request identifier unique to the access token used to send the request.
        #[ruma_api(path)]
        pub txn_id: &'a TransactionId,

        /// Messages to send.
        ///
        /// Different message events can be sent to different devices in the same request, but all
        /// events within one request must be of the same type.
        pub messages: Messages,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given event type, transaction ID and raw messages.
    pub fn new_raw(event_type: &'a str, txn_id: &'a TransactionId, messages: Messages) -> Self {
        Self { event_type, txn_id, messages }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self {}
    }
}

/// Messages to send in a send-to-device request.
///
/// Represented as a map of `{ user-ids => { device-ids => message-content } }`.
pub type Messages =
    BTreeMap<Box<UserId>, BTreeMap<DeviceIdOrAllDevices, Raw<AnyToDeviceEventContent>>>;
