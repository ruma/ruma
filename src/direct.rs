//! Types for the *m.direct* event.

use std::collections::BTreeMap;

use ruma_events_macros::ruma_event;
use ruma_identifiers::{RoomId, UserId};

ruma_event! {
    /// Informs the client about the rooms that are considered direct by a user.
    DirectEvent {
        kind: Event,
        event_type: "m.direct",
        content_type_alias: {
            /// The payload for `DirectEvent`.
            ///
            /// A mapping of `UserId`s to a list of `RoomId`s which are considered *direct* for that
            /// particular user.
            BTreeMap<UserId, Vec<RoomId>>
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ruma_identifiers::{RoomId, UserId};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{DirectEvent, DirectEventContent};
    use crate::EventResult;

    #[test]
    fn serialization() {
        let mut content: DirectEventContent = BTreeMap::new();
        let alice = UserId::new("ruma.io").unwrap();
        let room = vec![RoomId::new("ruma.io").unwrap()];

        content.insert(alice.clone(), room.clone());

        let event = DirectEvent { content };
        let json_data = json!({
            "content": {
                alice.to_string(): vec![room[0].to_string()],
            },
            "type": "m.direct"
        });

        assert_eq!(to_json_value(&event).unwrap(), json_data);
    }

    #[test]
    fn deserialization() {
        let alice = UserId::new("ruma.io").unwrap();
        let rooms = vec![
            RoomId::new("ruma.io").unwrap(),
            RoomId::new("ruma.io").unwrap(),
        ];

        let json_data = json!({
            "content": {
                alice.to_string(): vec![rooms[0].to_string(), rooms[1].to_string()],
            },
            "type": "m.direct"
        });

        let event: DirectEvent = from_json_value::<EventResult<_>>(json_data)
            .unwrap()
            .into_result()
            .unwrap();
        let direct_rooms = event.content.get(&alice).unwrap();

        assert!(direct_rooms.contains(&rooms[0]));
        assert!(direct_rooms.contains(&rooms[1]));
    }
}
