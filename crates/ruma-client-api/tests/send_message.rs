#![cfg(all(test, feature = "client"))]

use ruma_client_api::{
    message::send_message_event::v3::Request as SendMessageRequest,
    state::send_state_event::v3::Request as SendStateRequest,
};
use ruma_common::{
    api::{MatrixVersion, OutgoingRequest, SendAccessToken, SupportedVersions},
    owned_room_id,
    serde::Raw,
};
#[cfg(feature = "unstable-msc4354")]
use ruma_events::StickyDurationMs;
use ruma_events::MessageLikeEventType;
use serde_json::json;

#[cfg(feature = "unstable-msc4354")]
#[test]
fn test_sticky_send_message_request() {
    let supported =
        SupportedVersions { versions: [MatrixVersion::V1_1].into(), features: Default::default() };

    let mut request = SendMessageRequest::new_raw(
        owned_room_id!("!roomid:example.org"),
        "0000".into(),
        MessageLikeEventType::RoomMessage,
        Raw::new(&json!({ "body": "Hello" })).unwrap().cast_unchecked(),
    );
    request.sticky_duration_ms = Some(StickyDurationMs::new_clamped(123_456_u32));
    let http_request: http::Request<Vec<u8>> = request
        .try_into_http_request(
            "https://homeserver.tld",
            SendAccessToken::IfRequired("auth_tok"),
            &supported,
        )
        .unwrap();

    assert_eq!(http_request.uri().query().unwrap(), "org.matrix.msc4354.sticky_duration_ms=123456");
}

#[test]
fn test_send_message_serialize() {
    use ruma_common::{
        api::{MatrixVersion, OutgoingRequest as _, SendAccessToken, SupportedVersions},
        owned_room_id,
    };
    use ruma_events::{room::name::RoomNameEventContent, EmptyStateKey};

    let supported =
        SupportedVersions { versions: [MatrixVersion::V1_1].into(), features: Default::default() };

    // This used to panic in make_endpoint_url because of a mismatch in the path parameter count
    let req = SendStateRequest::new(
        owned_room_id!("!room:server.tld"),
        &EmptyStateKey,
        &RoomNameEventContent::new("Test room".to_owned()),
    )
    .unwrap()
    .try_into_http_request::<Vec<u8>>(
        "https://server.tld",
        SendAccessToken::IfRequired("access_token"),
        &supported,
    )
    .unwrap();

    assert_eq!(
        req.uri(),
        "https://server.tld/_matrix/client/v3/rooms/!room:server.tld/state/m.room.name/"
    );
}

#[cfg(feature = "unstable-msc4354")]
#[test]
fn serialize_sticky_state_event() {
    use ruma_common::{
        api::{MatrixVersion, OutgoingRequest as _, SendAccessToken, SupportedVersions},
        owned_room_id,
    };
    use ruma_events::{room::name::RoomNameEventContent, EmptyStateKey};

    let supported =
        SupportedVersions { versions: [MatrixVersion::V1_1].into(), features: Default::default() };

    // This used to panic in make_endpoint_url because of a mismatch in the path parameter count
    let mut req = SendStateRequest::new(
        owned_room_id!("!room:server.tld"),
        &EmptyStateKey,
        &RoomNameEventContent::new("Test room".to_owned()),
    )
    .unwrap();

    req.sticky_duration_ms = Some(StickyDurationMs::new_clamped(1_000_u32));

    let http_req = req
        .try_into_http_request::<Vec<u8>>(
            "https://server.tld",
            SendAccessToken::IfRequired("access_token"),
            &supported,
        )
        .unwrap();

    assert_eq!(http_req.uri().query().unwrap(), "org.matrix.msc4354.sticky_duration_ms=1000");
}
