//! `PUT /_matrix/client/*/sendToDevice/{eventType}/{txnId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#put_matrixclientv3sendtodeviceeventtypetxnid

    use std::collections::BTreeMap;

    use ruma_common::{
        api::ruma_api, events::AnyToDeviceEventContent, serde::Raw,
        to_device::DeviceIdOrAllDevices, OwnedUserId, TransactionId,
    };

    ruma_api! {
        metadata: {
            description: "Send an event to a device or devices.",
            method: PUT,
            name: "send_event_to_device",
            r0_path: "/_matrix/client/r0/sendToDevice/:event_type/:txn_id",
            stable_path: "/_matrix/client/v3/sendToDevice/:event_type/:txn_id",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// Type of event being sent to each device.
            #[ruma_api(path)]
            pub event_type: &'a str,

            /// The transaction ID for this event.
            ///
            /// Clients should generate a unique ID across requests within the
            /// same session. A session is identified by an access token, and
            /// persists when the [access token is refreshed].
            ///
            /// It will be used by the server to ensure idempotency of requests.
            ///
            /// [access token is refreshed]: https://spec.matrix.org/v1.4/client-server-api/#refreshing-access-tokens
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
        BTreeMap<OwnedUserId, BTreeMap<DeviceIdOrAllDevices, Raw<AnyToDeviceEventContent>>>;
}
