//! Types for the *m.room.join_rules* event.

use ruma_events_macros::ruma_event;
use serde::{Deserialize, Serialize};

ruma_event! {
    /// Describes how users are allowed to join the room.
    JoinRulesEvent {
        kind: StateEvent,
        event_type: "m.room.join_rules",
        content: {
            /// The type of rules used for users wishing to join this room.
            pub join_rule: JoinRule,
        },
    }
}

/// The rule used for users wishing to join this room.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum JoinRule {
    /// A user who wishes to join the room must first receive an invite to the room from someone
    /// already inside of the room.
    Invite,

    /// Reserved but not yet implemented by the Matrix specification.
    Knock,

    /// Reserved but not yet implemented by the Matrix specification.
    Private,

    /// Anyone can join the room without any prior action.
    Public,


}

impl_enum! {
    JoinRule {
        Invite => "invite",
        Knock => "knock",
        Private => "private",
        Public => "public",
    }
}
