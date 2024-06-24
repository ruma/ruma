use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{
    serde::{CanBeEmpty, Raw},
    MilliSecondsSinceUnixEpoch, VoipVersionId,
};
use ruma_events::{
    secret_storage::key::{SecretStorageEncryptionAlgorithm, SecretStorageV1AesHmacSha2Properties},
    AnyGlobalAccountDataEventContent, AnyMessageLikeEvent, AnyMessageLikeEventContent,
    MessageLikeEvent, RawExt as _,
};
use serde_json::{from_value as from_json_value, json, value::to_raw_value as to_raw_json_value};

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/it/ui/07-enum-sanity-check.rs");
    t.compile_fail("tests/it/ui/08-enum-invalid-path.rs");
    t.compile_fail("tests/it/ui/09-enum-invalid-kind.rs");
}

#[test]
fn deserialize_message_event() {
    let json_data = json!({
        "content": {
            "answer": {
                "type": "answer",
                "sdp": "Hello"
            },
            "call_id": "foofoo",
            "version": 0
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@carl:example.com",
        "type": "m.call.answer"
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::CallAnswer(MessageLikeEvent::Original(message_event))
    );

    assert_eq!(message_event.event_id, "$h29iv0s8:example.com");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(message_event.room_id, "!roomid:room.com");
    assert_eq!(message_event.sender, "@carl:example.com");
    assert!(message_event.unsigned.is_empty());

    let content = message_event.content;
    assert_eq!(content.answer.sdp, "Hello");
    assert_eq!(content.call_id, "foofoo");
    assert_eq!(content.version, VoipVersionId::V0);
}

#[test]
fn text_msgtype_plain_text_deserialization_as_any() {
    let serialized = json!({
        "body": "Hello world!",
        "msgtype": "m.text"
    });

    let raw_event: Raw<AnyMessageLikeEventContent> =
        Raw::from_json_string(serialized.to_string()).unwrap();

    let event = raw_event.deserialize_with_type("m.room.message".into()).unwrap();

    assert_matches!(event, AnyMessageLikeEventContent::RoomMessage(content));
    assert_eq!(content.body(), "Hello world!");
}

#[test]
fn secret_storage_key_deserialization_as_any() {
    let serialized = to_raw_json_value(&json!({
        "name": "my_key",
        "algorithm": "m.secret_storage.v1.aes-hmac-sha2",
        "iv": "YWJjZGVmZ2hpamtsbW5vcA",
        "mac": "aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U"
    }))
    .unwrap();

    let raw_event: Raw<AnyGlobalAccountDataEventContent> =
        Raw::from_json_string(serialized.to_string()).unwrap();

    let event = raw_event.deserialize_with_type("m.secret_storage.key.test".into()).unwrap();

    assert_matches!(event, AnyGlobalAccountDataEventContent::SecretStorageKey(content));

    assert_eq!(content.name.unwrap(), "my_key");
    assert_eq!(content.key_id, "test");
    assert_matches!(content.passphrase, None);

    assert_matches!(
        content.algorithm,
        SecretStorageEncryptionAlgorithm::V1AesHmacSha2(SecretStorageV1AesHmacSha2Properties {
            iv: Some(iv),
            mac: Some(mac),
            ..
        })
    );

    assert_eq!(iv.encode(), "YWJjZGVmZ2hpamtsbW5vcA");
    assert_eq!(mac.encode(), "aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U");
}
