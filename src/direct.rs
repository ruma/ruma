//! Types for the *m.direct* event.

use std::collections::HashMap;

use ruma_events_macros::ruma_event;
use ruma_identifiers::{RoomId, UserId};

ruma_event! {
    /// Informs the client about the rooms that are considered direct by a user.
    DirectEvent {
        kind: Event,
        event_type: Direct,
        content_type_alias: {
            /// The payload for `DirectEvent`.
            ///
            /// A mapping of `UserId`s to a list of `RoomId`s which are considered *direct* for that
            /// particular user.
            HashMap<UserId, Vec<RoomId>>
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ruma_identifiers::{RoomId, UserId};
    use serde_json::to_string;

    use super::{DirectEvent, DirectEventContent};

    #[test]
    fn serialization() {
        let mut content: DirectEventContent = HashMap::new();
        let alice = UserId::new("ruma.io").unwrap();
        let room = vec![RoomId::new("ruma.io").unwrap()];

        content.insert(alice.clone(), room.clone());

        let event = DirectEvent { content };

        assert_eq!(
            to_string(&event).unwrap(),
            format!(
                r#"{{"content":{{"{}":["{}"]}},"type":"m.direct"}}"#,
                alice.to_string(),
                room[0].to_string()
            )
        );
    }

    #[test]
    fn deserialization() {
        let alice = UserId::new("ruma.io").unwrap();
        let rooms = vec![
            RoomId::new("ruma.io").unwrap(),
            RoomId::new("ruma.io").unwrap(),
        ];

        let json_data = format!(
            r#"{{
            "content": {{ "{}": ["{}", "{}"] }},
            "type": "m.direct"
        }}"#,
            alice.to_string(),
            rooms[0].to_string(),
            rooms[1].to_string()
        );

        let event = DirectEvent::from_str(&json_data).unwrap();
        let direct_rooms = event.content.get(&alice).unwrap();

        assert!(direct_rooms.contains(&rooms[0]));
        assert!(direct_rooms.contains(&rooms[1]));
    }
}
