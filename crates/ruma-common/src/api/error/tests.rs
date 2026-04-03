use assert_matches2::assert_let;
use ruma_common::api::{EndpointError, OutgoingResponse};
use serde_json::{
    Value as JsonValue, from_slice as from_json_slice, from_value as from_json_value, json,
};
use web_time::{Duration, UNIX_EPOCH};

use super::{
    Error, ErrorBody, ErrorKind, LimitExceededErrorData, RetryAfter, StandardErrorBody,
    WrongRoomKeysVersionErrorData,
};

#[test]
fn deserialize_forbidden() {
    let deserialized: StandardErrorBody = from_json_value(json!({
        "errcode": "M_FORBIDDEN",
        "error": "You are not authorized to ban users in this room.",
    }))
    .unwrap();

    assert_eq!(deserialized.kind, ErrorKind::Forbidden);
    assert_eq!(deserialized.message, "You are not authorized to ban users in this room.");
}

#[test]
fn deserialize_wrong_room_key_version() {
    let deserialized: StandardErrorBody = from_json_value(json!({
        "current_version": "42",
        "errcode": "M_WRONG_ROOM_KEYS_VERSION",
        "error": "Wrong backup version."
    }))
    .expect("We should be able to deserialize a wrong room keys version error");

    assert_let!(
        ErrorKind::WrongRoomKeysVersion(WrongRoomKeysVersionErrorData { current_version }) =
            deserialized.kind
    );
    assert_eq!(current_version, "42");
    assert_eq!(deserialized.message, "Wrong backup version.");
}

#[test]
fn deserialize_limit_exceeded_no_retry_after() {
    let response = http::Response::builder()
        .status(http::StatusCode::TOO_MANY_REQUESTS)
        .body(
            serde_json::to_string(&json!({
                "errcode": "M_LIMIT_EXCEEDED",
                "error": "Too many requests",
            }))
            .unwrap(),
        )
        .unwrap();
    let error = Error::from_http_response(response);

    assert_eq!(error.status_code, http::StatusCode::TOO_MANY_REQUESTS);
    assert_let!(
        ErrorBody::Standard(StandardErrorBody {
            kind: ErrorKind::LimitExceeded(LimitExceededErrorData { retry_after: None }),
            message
        }) = error.body
    );
    assert_eq!(message, "Too many requests");
}

#[test]
fn deserialize_limit_exceeded_retry_after_body() {
    let response = http::Response::builder()
        .status(http::StatusCode::TOO_MANY_REQUESTS)
        .body(
            serde_json::to_string(&json!({
                "errcode": "M_LIMIT_EXCEEDED",
                "error": "Too many requests",
                "retry_after_ms": 2000,
            }))
            .unwrap(),
        )
        .unwrap();
    let error = Error::from_http_response(response);

    assert_eq!(error.status_code, http::StatusCode::TOO_MANY_REQUESTS);
    assert_let!(
        ErrorBody::Standard(StandardErrorBody {
            kind: ErrorKind::LimitExceeded(LimitExceededErrorData {
                retry_after: Some(retry_after)
            }),
            message
        }) = error.body
    );
    assert_let!(RetryAfter::Delay(delay) = retry_after);
    assert_eq!(delay.as_millis(), 2000);
    assert_eq!(message, "Too many requests");
}

#[test]
fn deserialize_limit_exceeded_retry_after_header_delay() {
    let response = http::Response::builder()
        .status(http::StatusCode::TOO_MANY_REQUESTS)
        .header(http::header::RETRY_AFTER, "2")
        .body(
            serde_json::to_string(&json!({
                "errcode": "M_LIMIT_EXCEEDED",
                "error": "Too many requests",
            }))
            .unwrap(),
        )
        .unwrap();
    let error = Error::from_http_response(response);

    assert_eq!(error.status_code, http::StatusCode::TOO_MANY_REQUESTS);
    assert_let!(
        ErrorBody::Standard(StandardErrorBody {
            kind: ErrorKind::LimitExceeded(LimitExceededErrorData {
                retry_after: Some(retry_after)
            }),
            message
        }) = error.body
    );
    assert_let!(RetryAfter::Delay(delay) = retry_after);
    assert_eq!(delay.as_millis(), 2000);
    assert_eq!(message, "Too many requests");
}

#[test]
fn deserialize_limit_exceeded_retry_after_header_datetime() {
    let response = http::Response::builder()
        .status(http::StatusCode::TOO_MANY_REQUESTS)
        .header(http::header::RETRY_AFTER, "Fri, 15 May 2015 15:34:21 GMT")
        .body(
            serde_json::to_string(&json!({
                "errcode": "M_LIMIT_EXCEEDED",
                "error": "Too many requests",
            }))
            .unwrap(),
        )
        .unwrap();
    let error = Error::from_http_response(response);

    assert_eq!(error.status_code, http::StatusCode::TOO_MANY_REQUESTS);
    assert_let!(
        ErrorBody::Standard(StandardErrorBody {
            kind: ErrorKind::LimitExceeded(LimitExceededErrorData {
                retry_after: Some(retry_after)
            }),
            message
        }) = error.body
    );
    assert_let!(RetryAfter::DateTime(time) = retry_after);
    assert_eq!(time.duration_since(UNIX_EPOCH).unwrap().as_secs(), 1_431_704_061);
    assert_eq!(message, "Too many requests");
}

#[test]
fn deserialize_limit_exceeded_retry_after_header_over_body() {
    let response = http::Response::builder()
        .status(http::StatusCode::TOO_MANY_REQUESTS)
        .header(http::header::RETRY_AFTER, "2")
        .body(
            serde_json::to_string(&json!({
                "errcode": "M_LIMIT_EXCEEDED",
                "error": "Too many requests",
                "retry_after_ms": 3000,
            }))
            .unwrap(),
        )
        .unwrap();
    let error = Error::from_http_response(response);

    assert_eq!(error.status_code, http::StatusCode::TOO_MANY_REQUESTS);
    assert_let!(
        ErrorBody::Standard(StandardErrorBody {
            kind: ErrorKind::LimitExceeded(LimitExceededErrorData {
                retry_after: Some(retry_after)
            }),
            message
        }) = error.body
    );
    assert_let!(RetryAfter::Delay(delay) = retry_after);
    assert_eq!(delay.as_millis(), 2000);
    assert_eq!(message, "Too many requests");
}

#[test]
fn serialize_limit_exceeded_retry_after_none() {
    let error = Error::new(
        http::StatusCode::TOO_MANY_REQUESTS,
        ErrorBody::Standard(StandardErrorBody {
            kind: ErrorKind::LimitExceeded(LimitExceededErrorData { retry_after: None }),
            message: "Too many requests".to_owned(),
        }),
    );

    let response = error.try_into_http_response::<Vec<u8>>().unwrap();

    assert_eq!(response.status(), http::StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(response.headers().get(http::header::RETRY_AFTER), None);

    let json_body: JsonValue = from_json_slice(response.body()).unwrap();
    assert_eq!(
        json_body,
        json!({
            "errcode": "M_LIMIT_EXCEEDED",
            "error": "Too many requests",
        })
    );
}

#[test]
fn serialize_limit_exceeded_retry_after_delay() {
    let error = Error::new(
        http::StatusCode::TOO_MANY_REQUESTS,
        ErrorBody::Standard(StandardErrorBody {
            kind: ErrorKind::LimitExceeded(LimitExceededErrorData {
                retry_after: Some(RetryAfter::Delay(Duration::from_secs(3))),
            }),
            message: "Too many requests".to_owned(),
        }),
    );

    let response = error.try_into_http_response::<Vec<u8>>().unwrap();

    assert_eq!(response.status(), http::StatusCode::TOO_MANY_REQUESTS);
    let retry_after_header = response.headers().get(http::header::RETRY_AFTER).unwrap();
    assert_eq!(retry_after_header.to_str().unwrap(), "3");

    let json_body: JsonValue = from_json_slice(response.body()).unwrap();
    assert_eq!(
        json_body,
        json!({
            "errcode": "M_LIMIT_EXCEEDED",
            "error": "Too many requests",
            "retry_after_ms": 3000,
        })
    );
}

#[test]
fn serialize_limit_exceeded_retry_after_datetime() {
    let error = Error::new(
        http::StatusCode::TOO_MANY_REQUESTS,
        ErrorBody::Standard(StandardErrorBody {
            kind: ErrorKind::LimitExceeded(LimitExceededErrorData {
                retry_after: Some(RetryAfter::DateTime(
                    UNIX_EPOCH + Duration::from_secs(1_431_704_061),
                )),
            }),
            message: "Too many requests".to_owned(),
        }),
    );

    let response = error.try_into_http_response::<Vec<u8>>().unwrap();

    assert_eq!(response.status(), http::StatusCode::TOO_MANY_REQUESTS);
    let retry_after_header = response.headers().get(http::header::RETRY_AFTER).unwrap();
    assert_eq!(retry_after_header.to_str().unwrap(), "Fri, 15 May 2015 15:34:21 GMT");

    let json_body: JsonValue = from_json_slice(response.body()).unwrap();
    assert_eq!(
        json_body,
        json!({
            "errcode": "M_LIMIT_EXCEEDED",
            "error": "Too many requests",
        })
    );
}

#[test]
fn serialize_user_locked() {
    let error = Error::new(
        http::StatusCode::UNAUTHORIZED,
        ErrorBody::Standard(StandardErrorBody {
            kind: ErrorKind::UserLocked,
            message: "This account has been locked".to_owned(),
        }),
    );

    let response = error.try_into_http_response::<Vec<u8>>().unwrap();

    assert_eq!(response.status(), http::StatusCode::UNAUTHORIZED);
    let json_body: JsonValue = from_json_slice(response.body()).unwrap();
    assert_eq!(
        json_body,
        json!({
            "errcode": "M_USER_LOCKED",
            "error": "This account has been locked",
            "soft_logout": true,
        })
    );
}

#[test]
fn deserialize_custom_error_kind() {
    let deserialized: StandardErrorBody = from_json_value(json!({
        "errcode": "LOCAL_DEV_ERROR",
        "error": "You are using the homeserver in local dev mode.",
        "foo": "bar",
    }))
    .unwrap();

    assert_eq!(deserialized.kind.errcode().as_str(), "LOCAL_DEV_ERROR");
    let json_data = deserialized.kind.custom_json_data().unwrap();
    assert_let!(Some(JsonValue::String(foo)) = json_data.get("foo"));
    assert_eq!(foo, "bar");
    assert_eq!(deserialized.message, "You are using the homeserver in local dev mode.");
}
