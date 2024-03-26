#![cfg(any(feature = "client", feature = "server"))]

use assert_matches2::assert_matches;
use assign::assign;
use ruma_client_api::{
    error::ErrorKind,
    uiaa::{self, AuthData, AuthFlow, AuthType, UiaaInfo, UiaaResponse, UserIdentifier},
};
use ruma_common::api::{EndpointError, OutgoingResponse};
use serde_json::{
    from_slice as from_json_slice, from_str as from_json_str, from_value as from_json_value, json,
    to_value as to_json_value, value::to_raw_value as to_raw_json_value, Value as JsonValue,
};

#[test]
fn deserialize_user_identifier() {
    assert_matches!(
        from_json_value(json!({
            "type": "m.id.user",
            "user": "cheeky_monkey"
        }))
        .unwrap(),
        UserIdentifier::UserIdOrLocalpart(id)
    );
    assert_eq!(id, "cheeky_monkey");
}

#[test]
fn serialize_auth_data_registration_token() {
    let auth_data =
        AuthData::RegistrationToken(assign!(uiaa::RegistrationToken::new("mytoken".to_owned()), {
            session: Some("session".to_owned()),
        }));

    assert_eq!(
        to_json_value(auth_data).unwrap(),
        json!({
            "type": "m.login.registration_token",
            "token": "mytoken",
            "session": "session",
        })
    );
}

#[test]
fn deserialize_auth_data_registration_token() {
    let json = json!({
        "type": "m.login.registration_token",
        "token": "mytoken",
        "session": "session",
    });

    assert_matches!(from_json_value(json), Ok(AuthData::RegistrationToken(data)));
    assert_eq!(data.token, "mytoken");
    assert_eq!(data.session.as_deref(), Some("session"));
}

#[test]
fn serialize_auth_data_fallback() {
    let auth_data =
        AuthData::FallbackAcknowledgement(uiaa::FallbackAcknowledgement::new("ZXY000".to_owned()));

    assert_eq!(json!({ "session": "ZXY000" }), to_json_value(auth_data).unwrap());
}

#[test]
fn deserialize_auth_data_fallback() {
    let json = json!({ "session": "opaque_session_id" });

    assert_matches!(from_json_value(json).unwrap(), AuthData::FallbackAcknowledgement(data));
    assert_eq!(data.session, "opaque_session_id");
}

#[test]
fn serialize_uiaa_info() {
    let flows = vec![AuthFlow::new(vec!["m.login.password".into(), "m.login.dummy".into()])];
    let params = to_raw_json_value(&json!({
        "example.type.baz": {
            "example_key": "foobar"
        }
    }))
    .unwrap();
    let uiaa_info = assign!(UiaaInfo::new(flows, params), {
        completed: vec!["m.login.password".into()],
    });

    let json = json!({
        "flows": [{ "stages": ["m.login.password", "m.login.dummy"] }],
        "completed": ["m.login.password"],
        "params": {
            "example.type.baz": {
                "example_key": "foobar"
            }
        }
    });
    assert_eq!(to_json_value(uiaa_info).unwrap(), json);
}

#[test]
fn deserialize_uiaa_info() {
    let json = json!({
        "errcode": "M_FORBIDDEN",
        "error": "Invalid password",
        "completed": ["m.login.recaptcha"],
        "flows": [
            {
                "stages": ["m.login.password"]
            },
            {
                "stages": ["m.login.email.identity", "m.login.msisdn"]
            }
        ],
        "params": {
            "example.type.baz": {
                "example_key": "foobar"
            }
        },
        "session": "xxxxxx"
    });

    let info = from_json_value::<UiaaInfo>(json).unwrap();
    assert_eq!(info.completed, vec![AuthType::ReCaptcha]);
    assert_eq!(info.flows.len(), 2);
    assert_eq!(info.flows[0].stages, vec![AuthType::Password]);
    assert_eq!(info.flows[1].stages, vec![AuthType::EmailIdentity, AuthType::Msisdn]);
    assert_eq!(info.session.as_deref(), Some("xxxxxx"));
    let auth_error = info.auth_error.unwrap();
    assert_matches!(auth_error.kind, ErrorKind::Forbidden { .. });
    assert_eq!(auth_error.message, "Invalid password");
    assert_eq!(
        from_json_str::<JsonValue>(info.params.get()).unwrap(),
        json!({
            "example.type.baz": {
                "example_key": "foobar"
            }
        })
    );
}

#[test]
fn try_uiaa_response_into_http_response() {
    let flows = vec![AuthFlow::new(vec![AuthType::Password, AuthType::Dummy])];
    let params = to_raw_json_value(&json!({
        "example.type.baz": {
            "example_key": "foobar"
        }
    }))
    .unwrap();
    let uiaa_info = assign!(UiaaInfo::new(flows, params), {
        completed: vec![AuthType::ReCaptcha],
    });
    let uiaa_response =
        UiaaResponse::AuthResponse(uiaa_info).try_into_http_response::<Vec<u8>>().unwrap();

    let info = from_json_slice::<UiaaInfo>(uiaa_response.body()).unwrap();
    assert_eq!(info.flows.len(), 1);
    assert_eq!(info.flows[0].stages, vec![AuthType::Password, AuthType::Dummy]);
    assert_eq!(info.completed, vec![AuthType::ReCaptcha]);
    assert_eq!(info.session, None);
    assert_matches!(info.auth_error, None);
    assert_eq!(
        from_json_str::<JsonValue>(info.params.get()).unwrap(),
        json!({
            "example.type.baz": {
                "example_key": "foobar"
            }
        })
    );
    assert_eq!(uiaa_response.status(), http::status::StatusCode::UNAUTHORIZED);
}

#[test]
fn try_uiaa_response_from_http_response() {
    let json = serde_json::to_string(&json!({
        "errcode": "M_FORBIDDEN",
        "error": "Invalid password",
        "completed": ["m.login.recaptcha"],
        "flows": [
            {
                "stages": ["m.login.password"]
            },
            {
                "stages": ["m.login.email.identity", "m.login.msisdn"]
            }
        ],
        "params": {
            "example.type.baz": {
                "example_key": "foobar"
            }
        },
        "session": "xxxxxx"
    }))
    .unwrap();

    let http_response = http::Response::builder()
        .status(http::StatusCode::UNAUTHORIZED)
        .body(json.as_bytes())
        .unwrap();

    assert_matches!(
        UiaaResponse::from_http_response(http_response),
        UiaaResponse::AuthResponse(info)
    );
    assert_eq!(info.completed, vec![AuthType::ReCaptcha]);
    assert_eq!(info.flows.len(), 2);
    assert_eq!(info.flows[0].stages, vec![AuthType::Password]);
    assert_eq!(info.flows[1].stages, vec![AuthType::EmailIdentity, AuthType::Msisdn]);
    assert_eq!(info.session.as_deref(), Some("xxxxxx"));
    let auth_error = info.auth_error.unwrap();
    assert_matches!(auth_error.kind, ErrorKind::Forbidden { .. });
    assert_eq!(auth_error.message, "Invalid password");
    assert_eq!(
        from_json_str::<JsonValue>(info.params.get()).unwrap(),
        json!({
            "example.type.baz": {
                "example_key": "foobar"
            }
        })
    );
}
