//! Types for the *m.call.answer* event.

use js_int::UInt;
use ruma_events_macros::ruma_event;

use super::SessionDescription;

ruma_event! {
    /// This event is sent by the callee when they wish to answer the call.
    AnswerEvent {
        kind: RoomEvent,
        event_type: CallAnswer,
        content: {
            /// The VoIP session description object. The session description type must be *answer*.
            pub answer: SessionDescription,

            /// The ID of the call this event relates to.
            pub call_id: String,

            /// The version of the VoIP specification this messages adheres to.
            pub version: UInt,
        },
    }
}
