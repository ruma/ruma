#![cfg(feature = "server")]

use assert_matches2::assert_matches;
use ruma_client_api::{message::send_message_event, state::send_state_event};
use ruma_common::api::{
    IncomingRequest as _,
    error::{DeserializationError, FromHttpRequestError},
};

const ROOM_ID: &str = "!room:server.tld";

fn http_put(uri: &str, body: &[u8]) -> http::Request<Vec<u8>> {
    http::Request::builder().method(http::Method::PUT).uri(uri).body(body.to_vec()).unwrap()
}

fn assert_non_object_rejected(err: FromHttpRequestError) {
    assert_matches!(err, FromHttpRequestError::Deserialization(DeserializationError::Json(_)));
}

#[test]
fn send_message_event_rejects_non_object_body() {
    let path_args = [ROOM_ID, "m.room.message", "txn1"];
    let bodies: &[&[u8]] = &[br#""string""#, br#"["array"]"#, b"42", b"null", b"true"];

    for body in bodies {
        let req =
            http_put("/_matrix/client/v3/rooms/!room:server.tld/send/m.room.message/txn1", body);
        let err = send_message_event::v3::Request::try_from_http_request(req, &path_args)
            .expect_err(&format!("body {body:?} should be rejected"));
        assert_non_object_rejected(err);
    }
}

#[test]
fn send_message_event_accepts_object_body() {
    let path_args = [ROOM_ID, "m.room.message", "txn1"];
    let req = http_put(
        "/_matrix/client/v3/rooms/!room:server.tld/send/m.room.message/txn1",
        br#"{"msgtype":"m.text","body":"hi"}"#,
    );
    send_message_event::v3::Request::try_from_http_request(req, &path_args).unwrap();
}

#[test]
fn send_state_event_rejects_non_object_body() {
    let path_args = [ROOM_ID, "m.room.topic", ""];
    let bodies: &[&[u8]] = &[br#""string""#, br#"["array"]"#, b"42", b"null", b"true"];

    for body in bodies {
        let req = http_put("/_matrix/client/v3/rooms/!room:server.tld/state/m.room.topic/", body);
        let err = send_state_event::v3::Request::try_from_http_request(req, &path_args)
            .expect_err(&format!("body {body:?} should be rejected"));
        assert_non_object_rejected(err);
    }
}

#[test]
fn send_state_event_accepts_object_body() {
    let path_args = [ROOM_ID, "m.room.topic", ""];
    let req = http_put(
        "/_matrix/client/v3/rooms/!room:server.tld/state/m.room.topic/",
        br#"{"topic":"hi"}"#,
    );
    send_state_event::v3::Request::try_from_http_request(req, &path_args).unwrap();
}
