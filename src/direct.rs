//! Types for the *m.direct* event.

use std::collections::HashMap;

use ruma_identifiers::{UserId, RoomId};

event! {
    /// Informs the client about the rooms that are considered direct by a user.
    pub struct DirectEvent(DirectEventContent) {}
}

/// The payload of a `DirectEvent`.
///
/// A mapping of `UserId`'s to a collection of `RoomId`'s which are considered
/// *direct* for that particular user.
pub type DirectEventContent = HashMap<UserId, Vec<RoomId>>;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ruma_identifiers::{UserId, RoomId};
    use serde_json::{from_str, to_string};

    use collections;
    use direct::{DirectEvent, DirectEventContent};
    use super::super::EventType;

    #[test]
    fn serialization() {
        let mut content: DirectEventContent = HashMap::new();
        let alice = UserId::new("ruma.io").unwrap();
        let room = vec![RoomId::new("ruma.io").unwrap()];

        content.insert(alice.clone(), room.clone());

        let event = DirectEvent {
            content: content,
            event_type: EventType::Direct,
        };

        assert_eq!(
            to_string(&event).unwrap(),
            format!(
                r#"{{"content":{{"{}":["{}"]}},"type":"m.direct"}}"#,
                alice.to_string(), room[0].to_string()
            )
        );
    }

    #[test]
    fn deserialization() {
        let alice = UserId::new("ruma.io").unwrap();
        let rooms = vec![
            RoomId::new("ruma.io").unwrap(),
            RoomId::new("ruma.io").unwrap()
        ];

        let json_data = format!(r#"{{
            "content": {{ "{}": ["{}", "{}"] }},
            "type": "m.direct"
        }}"#, alice.to_string(), rooms[0].to_string(), rooms[1].to_string());

        let event = from_str::<DirectEvent>(&json_data).unwrap();
        assert_eq!(event.event_type, EventType::Direct);

        let direct_rooms = event.content.get(&alice).unwrap();
        assert!(direct_rooms.contains(&rooms[0]));
        assert!(direct_rooms.contains(&rooms[1]));

        match from_str::<collections::all::Event>(&json_data).unwrap() {
            collections::all::Event::Direct(event) => {
                assert_eq!(event.event_type, EventType::Direct);

                let direct_rooms = event.content.get(&alice).unwrap();
                assert!(direct_rooms.contains(&rooms[0]));
                assert!(direct_rooms.contains(&rooms[1]));
            },
            _ => assert!(false)
        };

        match from_str::<collections::only::Event>(&json_data).unwrap() {
            collections::only::Event::Direct(event) => {
                assert_eq!(event.event_type, EventType::Direct);

                let direct_rooms = event.content.get(&alice).unwrap();
                assert!(direct_rooms.contains(&rooms[0]));
                assert!(direct_rooms.contains(&rooms[1]));
            },
            _ => assert!(false)
        };
    }
}
