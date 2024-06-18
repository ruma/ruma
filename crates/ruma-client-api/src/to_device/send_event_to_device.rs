//! `PUT /_matrix/client/*/sendToDevice/{eventType}/{txnId}`
//!
//! Send an event to a device or devices.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3sendtodeviceeventtypetxnid

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        to_device::DeviceIdOrAllDevices,
        OwnedTransactionId, OwnedUserId,
    };
    use ruma_events::{AnyToDeviceEventContent, ToDeviceEventType};

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/sendToDevice/{event_type}/{txn_id}",
            1.1 => "/_matrix/client/v3/sendToDevice/{event_type}/{txn_id}",
        }
    };

    /// Request type for the `send_event_to_device` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// Type of event being sent to each device.
        #[ruma_api(path)]
        pub event_type: ToDeviceEventType,

        /// The transaction ID for this event.
        ///
        /// Clients should generate a unique ID across requests within the
        /// same session. A session is identified by an access token, and
        /// persists when the [access token is refreshed].
        ///
        /// It will be used by the server to ensure idempotency of requests.
        ///
        /// [access token is refreshed]: https://spec.matrix.org/latest/client-server-api/#refreshing-access-tokens
        #[ruma_api(path)]
        pub txn_id: OwnedTransactionId,

        /// Messages to send.
        ///
        /// Different message events can be sent to different devices in the same request, but all
        /// events within one request must be of the same type.
        pub messages: Messages,
    }

    /// Response type for the `send_event_to_device` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given event type, transaction ID and raw messages.
        pub fn new_raw(
            event_type: ToDeviceEventType,
            txn_id: OwnedTransactionId,
            messages: Messages,
        ) -> Self {
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
        BTreeMap<OwnedUserId, BTreeMap<DeviceIdOrAllDevices, Raw<AnyToDeviceEventContent>>>;
}
