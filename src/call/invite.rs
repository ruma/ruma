//! Types for the *m.call.invite* event.

use js_int::UInt;
use ruma_events_macros::ruma_event;

use super::SessionDescription;

ruma_event! {
    /// This event is sent by the caller when they wish to establish a call.
    InviteEvent {
        kind: RoomEvent,
        event_type: CallInvite,
        content: {
            /// A unique identifer for the call.
            pub call_id: String,

            /// The time in milliseconds that the invite is valid for. Once the invite age exceeds this
            /// value, clients should discard it. They should also no longer show the call as awaiting an
            /// answer in the UI.
            pub lifetime: UInt,

            /// The session description object. The session description type must be *offer*.
            pub offer: SessionDescription,

            /// The version of the VoIP specification this messages adheres to.
            pub version: UInt,
        },
    }
}
