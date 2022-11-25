use assert_matches::assert_matches;
use ruma_common::{
    device_id, event_id,
    events::{
        relation::{InReplyTo, Reference, Thread},
        room::encrypted::{
            EncryptedEventScheme, MegolmV1AesSha2ContentInit, Relation, Replacement,
            RoomEncryptedEventContent,
        },
    },
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

fn encrypted_scheme() -> EncryptedEventScheme {
    EncryptedEventScheme::MegolmV1AesSha2(
        MegolmV1AesSha2ContentInit {
            ciphertext: "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
                        FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
                        n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
                        MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
                        c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
                        lDl5mzVO3tPnJMKZ0hn+AF"
                .to_owned(),
            sender_key: "aV9BpqYFqJpKYmgERyGv/6QyKMcgLqxM05V0gvzg9Yk".to_owned(),
            device_id: device_id!("DEVICE").to_owned(),
            session_id: "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw".to_owned(),
        }
        .into(),
    )
}

#[test]
fn content_no_relation_serialization() {
    let content = RoomEncryptedEventContent::new(encrypted_scheme(), None);

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "sender_key": "aV9BpqYFqJpKYmgERyGv/6QyKMcgLqxM05V0gvzg9Yk",
            "ciphertext": "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
                          FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
                          n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
                          MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
                          c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
                          lDl5mzVO3tPnJMKZ0hn+AF",
            "session_id": "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw",
            "device_id": "DEVICE",
        })
    );
}

#[test]
fn content_no_relation_deserialization() {
    let json = json!({
        "algorithm": "m.megolm.v1.aes-sha2",
        "sender_key": "aV9BpqYFqJpKYmgERyGv/6QyKMcgLqxM05V0gvzg9Yk",
        "ciphertext": "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
                      FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
                      n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
                      MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
                      c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
                      lDl5mzVO3tPnJMKZ0hn+AF",
        "session_id": "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw",
        "device_id": "DEVICE",
    });

    let content = from_json_value::<RoomEncryptedEventContent>(json).unwrap();

    let encrypted_content = assert_matches!(
        content.scheme,
        EncryptedEventScheme::MegolmV1AesSha2(content) => content
    );
    assert_eq!(encrypted_content.session_id, "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw");
    assert_eq!(
        encrypted_content.ciphertext,
        "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
        FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
        n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
        MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
        c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
        lDl5mzVO3tPnJMKZ0hn+AF"
    );

    assert_matches!(content.relates_to, None);
}

#[test]
fn content_reply_serialization() {
    let content = RoomEncryptedEventContent::new(
        encrypted_scheme(),
        Some(Relation::Reply {
            in_reply_to: InReplyTo::new(event_id!("$replied_to_event").to_owned()),
        }),
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "sender_key": "aV9BpqYFqJpKYmgERyGv/6QyKMcgLqxM05V0gvzg9Yk",
            "ciphertext": "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
                          FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
                          n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
                          MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
                          c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
                          lDl5mzVO3tPnJMKZ0hn+AF",
            "session_id": "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw",
            "device_id": "DEVICE",
            "m.relates_to": {
                "m.in_reply_to": {
                    "event_id": "$replied_to_event",
                },
            },
        })
    );
}

#[test]
fn content_reply_deserialization() {
    let json = json!({
        "algorithm": "m.megolm.v1.aes-sha2",
        "sender_key": "aV9BpqYFqJpKYmgERyGv/6QyKMcgLqxM05V0gvzg9Yk",
        "ciphertext": "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
                      FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
                      n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
                      MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
                      c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
                      lDl5mzVO3tPnJMKZ0hn+AF",
        "session_id": "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw",
        "device_id": "DEVICE",
        "m.relates_to": {
            "m.in_reply_to": {
                "event_id": "$replied_to_event",
            },
        },
    });

    let content = from_json_value::<RoomEncryptedEventContent>(json).unwrap();

    let encrypted_content = assert_matches!(
        content.scheme,
        EncryptedEventScheme::MegolmV1AesSha2(content) => content
    );
    assert_eq!(encrypted_content.session_id, "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw");
    assert_eq!(
        encrypted_content.ciphertext,
        "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
        FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
        n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
        MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
        c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
        lDl5mzVO3tPnJMKZ0hn+AF"
    );

    let in_reply_to = assert_matches!(
        content.relates_to,
        Some(Relation::Reply { in_reply_to }) => in_reply_to
    );
    assert_eq!(in_reply_to.event_id, "$replied_to_event");
}

#[test]
fn content_replacement_serialization() {
    let content = RoomEncryptedEventContent::new(
        encrypted_scheme(),
        Some(Relation::Replacement(Replacement::new(event_id!("$replaced_event").to_owned()))),
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "sender_key": "aV9BpqYFqJpKYmgERyGv/6QyKMcgLqxM05V0gvzg9Yk",
            "ciphertext": "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
                          FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
                          n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
                          MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
                          c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
                          lDl5mzVO3tPnJMKZ0hn+AF",
            "session_id": "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw",
            "device_id": "DEVICE",
            "m.relates_to": {
                "rel_type": "m.replace",
                "event_id": "$replaced_event",
            },
        })
    );
}

#[test]
fn content_replacement_deserialization() {
    let json = json!({
        "algorithm": "m.megolm.v1.aes-sha2",
        "sender_key": "aV9BpqYFqJpKYmgERyGv/6QyKMcgLqxM05V0gvzg9Yk",
        "ciphertext": "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
                      FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
                      n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
                      MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
                      c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
                      lDl5mzVO3tPnJMKZ0hn+AF",
        "session_id": "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw",
        "device_id": "DEVICE",
        "m.relates_to": {
            "rel_type": "m.replace",
            "event_id": "$replaced_event",
        },
    });

    let content = from_json_value::<RoomEncryptedEventContent>(json).unwrap();

    let encrypted_content = assert_matches!(
        content.scheme,
        EncryptedEventScheme::MegolmV1AesSha2(content) => content
    );
    assert_eq!(encrypted_content.session_id, "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw");
    assert_eq!(
        encrypted_content.ciphertext,
        "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
        FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
        n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
        MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
        c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
        lDl5mzVO3tPnJMKZ0hn+AF"
    );

    let replacement = assert_matches!(
        content.relates_to,
        Some(Relation::Replacement(replacement)) => replacement
    );
    assert_eq!(replacement.event_id, "$replaced_event");
}

#[test]
fn content_reference_serialization() {
    let content = RoomEncryptedEventContent::new(
        encrypted_scheme(),
        Some(Relation::Reference(Reference::new(event_id!("$referenced_event").to_owned()))),
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "sender_key": "aV9BpqYFqJpKYmgERyGv/6QyKMcgLqxM05V0gvzg9Yk",
            "ciphertext": "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
                          FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
                          n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
                          MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
                          c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
                          lDl5mzVO3tPnJMKZ0hn+AF",
            "session_id": "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw",
            "device_id": "DEVICE",
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$referenced_event",
            },
        })
    );
}

#[test]
fn content_reference_deserialization() {
    let json = json!({
        "algorithm": "m.megolm.v1.aes-sha2",
        "sender_key": "aV9BpqYFqJpKYmgERyGv/6QyKMcgLqxM05V0gvzg9Yk",
        "ciphertext": "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
                      FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
                      n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
                      MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
                      c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
                      lDl5mzVO3tPnJMKZ0hn+AF",
        "session_id": "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw",
        "device_id": "DEVICE",
        "m.relates_to": {
            "rel_type": "m.reference",
            "event_id": "$referenced_event",
        },
    });

    let content = from_json_value::<RoomEncryptedEventContent>(json).unwrap();

    let encrypted_content = assert_matches!(
        content.scheme,
        EncryptedEventScheme::MegolmV1AesSha2(content) => content
    );
    assert_eq!(encrypted_content.session_id, "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw");
    assert_eq!(
        encrypted_content.ciphertext,
        "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
        FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
        n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
        MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
        c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
        lDl5mzVO3tPnJMKZ0hn+AF"
    );

    let reference = assert_matches!(
        content.relates_to,
        Some(Relation::Reference(reference)) => reference
    );
    assert_eq!(reference.event_id, "$referenced_event");
}

#[test]
fn content_thread_serialization() {
    let content = RoomEncryptedEventContent::new(
        encrypted_scheme(),
        Some(Relation::Thread(Thread::plain(
            event_id!("$thread_root").to_owned(),
            event_id!("$prev_event").to_owned(),
        ))),
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "sender_key": "aV9BpqYFqJpKYmgERyGv/6QyKMcgLqxM05V0gvzg9Yk",
            "ciphertext": "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
                          FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
                          n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
                          MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
                          c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
                          lDl5mzVO3tPnJMKZ0hn+AF",
            "session_id": "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw",
            "device_id": "DEVICE",
            "m.relates_to": {
                "rel_type": "m.thread",
                "event_id": "$thread_root",
                "is_falling_back": true,
                "m.in_reply_to": {
                    "event_id": "$prev_event",
                },
            },
        })
    );
}

#[test]
fn content_thread_deserialization() {
    let json = json!({
        "algorithm": "m.megolm.v1.aes-sha2",
        "sender_key": "aV9BpqYFqJpKYmgERyGv/6QyKMcgLqxM05V0gvzg9Yk",
        "ciphertext": "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
                      FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
                      n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
                      MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
                      c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
                      lDl5mzVO3tPnJMKZ0hn+AF",
        "session_id": "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw",
        "device_id": "DEVICE",
        "m.relates_to": {
            "rel_type": "m.thread",
            "event_id": "$thread_root",
            "m.in_reply_to": {
                "event_id": "$prev_event",
            },
        },
    });

    let content = from_json_value::<RoomEncryptedEventContent>(json).unwrap();

    let encrypted_content = assert_matches!(
        content.scheme,
        EncryptedEventScheme::MegolmV1AesSha2(content) => content
    );
    assert_eq!(encrypted_content.session_id, "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw");
    assert_eq!(
        encrypted_content.ciphertext,
        "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
        FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
        n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
        MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
        c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
        lDl5mzVO3tPnJMKZ0hn+AF"
    );

    let thread = assert_matches!(
        content.relates_to,
        Some(Relation::Thread(thread)) => thread
    );
    assert_eq!(thread.event_id, "$thread_root");
    assert_eq!(thread.in_reply_to.event_id, "$prev_event");
    assert!(!thread.is_falling_back);
}

#[test]
#[cfg(feature = "unstable-msc2677")]
fn content_annotation_serialization() {
    use ruma_common::events::relation::Annotation;

    let content = RoomEncryptedEventContent::new(
        encrypted_scheme(),
        Some(Relation::Annotation(Annotation::new(
            event_id!("$annotated_event").to_owned(),
            "some_key".to_owned(),
        ))),
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "sender_key": "aV9BpqYFqJpKYmgERyGv/6QyKMcgLqxM05V0gvzg9Yk",
            "ciphertext": "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
                          FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
                          n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
                          MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
                          c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
                          lDl5mzVO3tPnJMKZ0hn+AF",
            "session_id": "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw",
            "device_id": "DEVICE",
            "m.relates_to": {
                "rel_type": "m.annotation",
                "event_id": "$annotated_event",
                "key": "some_key",
            },
        })
    );
}

#[test]
#[cfg(feature = "unstable-msc2677")]
fn content_annotation_deserialization() {
    let json = json!({
        "algorithm": "m.megolm.v1.aes-sha2",
        "sender_key": "aV9BpqYFqJpKYmgERyGv/6QyKMcgLqxM05V0gvzg9Yk",
        "ciphertext": "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
                      FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
                      n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
                      MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
                      c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
                      lDl5mzVO3tPnJMKZ0hn+AF",
        "session_id": "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw",
        "device_id": "DEVICE",
        "m.relates_to": {
            "rel_type": "m.annotation",
            "event_id": "$annotated_event",
            "key": "some_key",
        },
    });

    let content = from_json_value::<RoomEncryptedEventContent>(json).unwrap();

    let encrypted_content = assert_matches!(
        content.scheme,
        EncryptedEventScheme::MegolmV1AesSha2(content) => content
    );
    assert_eq!(encrypted_content.session_id, "IkwqWxT2zy3DI1E/zM2Wq+CE8tr3eEpsxsVGjGrMPdw");
    assert_eq!(
        encrypted_content.ciphertext,
        "AwgAEpABjy6BHczo7UZE3alyej6y2YQ5v+L9eB+fBqL7yteCPv8Jig\
        FCXKWWuwpbZ4nQpvhUbqW0ZX2474FQf0l1dXGQWDMm0VP5p20elkzSf\
        n0uzmHVKGQe+NHUKIczRWsUJ6AbrLBbfFKoIPwfbZ7nQQndjA6F0+PW\
        MoMQHqcrtROrCV/TMux6kDKp7h7O77Y6wp6LD4rU1lwTmKnMYkQGnju\
        c3+FAMvkow26TuS0/fhJG5m+f0GLlP8FQ3fu0Kjw2YUOLl/BU6gPWdk\
        lDl5mzVO3tPnJMKZ0hn+AF"
    );

    let annotation = assert_matches!(
        content.relates_to,
        Some(Relation::Annotation(annotation)) => annotation
    );
    assert_eq!(annotation.event_id, "$annotated_event");
    assert_eq!(annotation.key, "some_key");
}
