use assert_matches2::assert_matches;
use http::header;
use ruma_common::api::auth_scheme::{
    AccessToken, AccessTokenOptional, AppserviceToken, AppserviceTokenOptional, AuthScheme,
    NoAuthentication, SendAccessToken,
};

const TOKEN: &str = "token";
const HEADER_VALUE: http::HeaderValue = http::HeaderValue::from_static("Bearer token");

fn http_request() -> http::Request<Vec<u8>> {
    http::Request::builder()
        .method(http::Method::GET)
        .uri("http://localhost/_matrix/client/versions")
        .body(vec![])
        .unwrap()
}

#[test]
fn send_access_token_none() {
    let input = SendAccessToken::None;
    let mut request = http_request();

    NoAuthentication::add_authentication(&mut request, input).unwrap();
    assert_eq!(request.headers_mut().remove(header::AUTHORIZATION), None);

    AccessToken::add_authentication(&mut request, input).unwrap_err();

    AccessTokenOptional::add_authentication(&mut request, input).unwrap();
    assert_eq!(request.headers_mut().remove(header::AUTHORIZATION), None);

    AppserviceToken::add_authentication(&mut request, input).unwrap_err();

    AppserviceTokenOptional::add_authentication(&mut request, input).unwrap();
    assert_eq!(request.headers_mut().remove(header::AUTHORIZATION), None);
}

#[test]
fn send_access_token_if_required() {
    let input = SendAccessToken::IfRequired(TOKEN);
    let mut request = http_request();

    NoAuthentication::add_authentication(&mut request, input).unwrap();
    assert_eq!(request.headers_mut().remove(header::AUTHORIZATION), None);

    AccessToken::add_authentication(&mut request, input).unwrap();
    assert_matches!(request.headers_mut().remove(header::AUTHORIZATION), Some(value));
    assert_eq!(value, HEADER_VALUE);

    AccessTokenOptional::add_authentication(&mut request, input).unwrap();
    assert_matches!(request.headers_mut().remove(header::AUTHORIZATION), Some(value));
    assert_eq!(value, HEADER_VALUE);

    AppserviceToken::add_authentication(&mut request, input).unwrap_err();

    AppserviceTokenOptional::add_authentication(&mut request, input).unwrap();
    assert_eq!(request.headers_mut().remove(header::AUTHORIZATION), None);
}

#[test]
fn send_access_token_always() {
    let input = SendAccessToken::Always(TOKEN);
    let mut request = http_request();

    NoAuthentication::add_authentication(&mut request, input).unwrap();
    assert_matches!(request.headers_mut().remove(header::AUTHORIZATION), Some(value));
    assert_eq!(value, HEADER_VALUE);

    AccessToken::add_authentication(&mut request, input).unwrap();
    assert_matches!(request.headers_mut().remove(header::AUTHORIZATION), Some(value));
    assert_eq!(value, HEADER_VALUE);

    AccessTokenOptional::add_authentication(&mut request, input).unwrap();
    assert_matches!(request.headers_mut().remove(header::AUTHORIZATION), Some(value));
    assert_eq!(value, HEADER_VALUE);

    AppserviceToken::add_authentication(&mut request, input).unwrap();
    assert_matches!(request.headers_mut().remove(header::AUTHORIZATION), Some(value));
    assert_eq!(value, HEADER_VALUE);

    AppserviceTokenOptional::add_authentication(&mut request, input).unwrap();
    assert_matches!(request.headers_mut().remove(header::AUTHORIZATION), Some(value));
    assert_eq!(value, HEADER_VALUE);
}

#[test]
fn send_access_token_appservice() {
    let input = SendAccessToken::Appservice(TOKEN);
    let mut request = http_request();

    NoAuthentication::add_authentication(&mut request, input).unwrap();
    assert_eq!(request.headers_mut().remove(header::AUTHORIZATION), None);

    AccessToken::add_authentication(&mut request, input).unwrap();
    assert_matches!(request.headers_mut().remove(header::AUTHORIZATION), Some(value));
    assert_eq!(value, HEADER_VALUE);

    AccessTokenOptional::add_authentication(&mut request, input).unwrap();
    assert_matches!(request.headers_mut().remove(header::AUTHORIZATION), Some(value));
    assert_eq!(value, HEADER_VALUE);

    AppserviceToken::add_authentication(&mut request, input).unwrap();
    assert_matches!(request.headers_mut().remove(header::AUTHORIZATION), Some(value));
    assert_eq!(value, HEADER_VALUE);

    AppserviceTokenOptional::add_authentication(&mut request, input).unwrap();
    assert_matches!(request.headers_mut().remove(header::AUTHORIZATION), Some(value));
    assert_eq!(value, HEADER_VALUE);
}

#[test]
fn extract_authentication_bearer() {
    let request_without_token = http_request();
    let mut request_with_valid_header = http_request();
    request_with_valid_header.headers_mut().insert(header::AUTHORIZATION, HEADER_VALUE);
    let mut request_with_invalid_scheme = http_request();
    request_with_invalid_scheme
        .headers_mut()
        .insert(header::AUTHORIZATION, http::HeaderValue::from_static("Basic dGVzdDoxMjPCow=="));
    let mut request_with_query = http_request();
    *request_with_query.uri_mut() = http::Uri::from_static(
        "http://localhost/_matrix/client/versions?foo=bar&access_token=token",
    );

    NoAuthentication::extract_authentication(&request_without_token).unwrap();
    NoAuthentication::extract_authentication(&request_with_valid_header).unwrap();
    NoAuthentication::extract_authentication(&request_with_invalid_scheme).unwrap();
    NoAuthentication::extract_authentication(&request_with_query).unwrap();

    AccessToken::extract_authentication(&request_without_token).unwrap_err();
    let token = AccessToken::extract_authentication(&request_with_valid_header).unwrap();
    assert_eq!(token, TOKEN);
    AccessToken::extract_authentication(&request_with_invalid_scheme).unwrap_err();
    let token = AccessToken::extract_authentication(&request_with_query).unwrap();
    assert_eq!(token, TOKEN);

    let token = AccessTokenOptional::extract_authentication(&request_without_token).unwrap();
    assert_eq!(token, None);
    let token = AccessTokenOptional::extract_authentication(&request_with_valid_header).unwrap();
    assert_eq!(token.as_deref(), Some(TOKEN));
    AccessTokenOptional::extract_authentication(&request_with_invalid_scheme).unwrap_err();
    let token = AccessTokenOptional::extract_authentication(&request_with_query).unwrap();
    assert_eq!(token.as_deref(), Some(TOKEN));

    AppserviceToken::extract_authentication(&request_without_token).unwrap_err();
    let token = AppserviceToken::extract_authentication(&request_with_valid_header).unwrap();
    assert_eq!(token, TOKEN);
    AppserviceToken::extract_authentication(&request_with_invalid_scheme).unwrap_err();
    let token = AppserviceToken::extract_authentication(&request_with_query).unwrap();
    assert_eq!(token, TOKEN);

    let token = AppserviceTokenOptional::extract_authentication(&request_without_token).unwrap();
    assert_eq!(token, None);
    let token =
        AppserviceTokenOptional::extract_authentication(&request_with_valid_header).unwrap();
    assert_eq!(token.as_deref(), Some(TOKEN));
    AppserviceTokenOptional::extract_authentication(&request_with_invalid_scheme).unwrap_err();
    let token = AppserviceTokenOptional::extract_authentication(&request_with_query).unwrap();
    assert_eq!(token.as_deref(), Some(TOKEN));
}
