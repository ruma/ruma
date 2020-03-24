//! Types for the *m.room.topic* event.

use ruma_events_macros::ruma_event;

ruma_event! {
    /// A topic is a short message detailing what is currently being discussed in the room.
    TopicEvent {
        kind: StateEvent,
        event_type: "m.room.topic",
        content: {
            /// The topic text.
            pub topic: String,
        },
    }
}
