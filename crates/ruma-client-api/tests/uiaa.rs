use assert_matches::assert_matches;
use assign::assign;
use ruma_client_api::{
    error::{ErrorBody, ErrorKind},
    uiaa::{
        self, AuthData, AuthFlow, AuthType, IncomingAuthData, IncomingUserIdentifier, UiaaInfo,
        UiaaResponse,
    },
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
        IncomingUserIdentifier::UserIdOrLocalpart(id)
        if id == "cheeky_monkey"
    );
}

#[test]
fn serialize_auth_data_registration_token() {
    let auth_data = AuthData::RegistrationToken(
        assign!(uiaa::RegistrationToken::new("mytoken"), { session: Some("session") }),
    );

    assert_matches!(
        to_json_value(auth_data),
        Ok(val) if val == json!({
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

    assert_matches!(
        from_json_value(json),
        Ok(IncomingAuthData::RegistrationToken(
            uiaa::IncomingRegistrationToken { token, session: Some(session), .. },
        ))
        if token == "mytoken" && session == "session"
    );
}

#[test]
fn serialize_auth_data_fallback() {
    let auth_data = AuthData::FallbackAcknowledgement(uiaa::FallbackAcknowledgement::new("ZXY000"));

    assert_eq!(json!({ "session": "ZXY000" }), to_json_value(auth_data).unwrap());
}

#[test]
fn deserialize_auth_data_fallback() {
    let json = json!({ "session": "opaque_session_id" });

    assert_matches!(
        from_json_value(json).unwrap(),
        IncomingAuthData::FallbackAcknowledgement(
            uiaa::IncomingFallbackAcknowledgement { session, .. },
        )
        if session == "opaque_session_id"
    );
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

    assert_matches!(
        from_json_value::<UiaaInfo>(json).unwrap(),
        UiaaInfo {
            auth_error: Some(ErrorBody {
                kind: ErrorKind::Forbidden,
                message: error_message,
            }),
            completed,
            flows,
            params,
            session: Some(session),
            ..
        } if error_message == "Invalid password"
            && completed == vec![AuthType::ReCaptcha]
            && matches!(
                flows.as_slice(),
                [f1, f2]
                if f1.stages == vec![AuthType::Password]
                    && f2.stages == vec![AuthType::EmailIdentity, AuthType::Msisdn]
            )
            && from_json_str::<JsonValue>(params.get()).unwrap() == json!({
                "example.type.baz": {
                    "example_key": "foobar"
                }
            })
            && session == "xxxxxx"
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

    assert_matches!(
        from_json_slice::<UiaaInfo>(uiaa_response.body()).unwrap(),
        UiaaInfo {
            flows,
            completed,
            params,
            session: None,
            auth_error: None,
            ..
        } if matches!(
            flows.as_slice(),
            [flow] if flow.stages == vec![AuthType::Password, AuthType::Dummy]
        ) && completed == vec![AuthType::ReCaptcha]
            && from_json_str::<JsonValue>(params.get()).unwrap() == json!({
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

    let parsed_uiaa_info = match UiaaResponse::try_from_http_response(http_response).unwrap() {
        UiaaResponse::AuthResponse(uiaa_info) => uiaa_info,
        _ => panic!("Expected UiaaResponse::AuthResponse"),
    };

    assert_matches!(
        parsed_uiaa_info,
        UiaaInfo {
            auth_error: Some(ErrorBody {
                kind: ErrorKind::Forbidden,
                message: error_message,
            }),
            completed,
            flows,
            params,
            session: Some(session),
            ..
        } if error_message == "Invalid password"
            && completed == vec![AuthType::ReCaptcha]
            && matches!(
                flows.as_slice(),
                [f1, f2]
                if f1.stages == vec![AuthType::Password]
                    && f2.stages == vec![AuthType::EmailIdentity, AuthType::Msisdn]
            )
            && from_json_str::<JsonValue>(params.get()).unwrap() == json!({
                "example.type.baz": {
                    "example_key": "foobar"
                }
            })
            && session == "xxxxxx"
    );
}
