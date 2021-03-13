//! [GET /_matrix/client/r0/login](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-login)

use ruma_api::ruma_api;
use ruma_serde::StringEnum;

ruma_api! {
    metadata: {
        description: "Gets the homeserver's supported login types to authenticate users. Clients should pick one of these and supply it as the type when logging in.",
        method: GET,
        name: "get_login_types",
        path: "/_matrix/client/r0/login",
        rate_limited: true,
        authentication: None,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// The homeserver's supported login types.
        #[serde(with = "login_type_list_serde")]
        pub flows: Vec<LoginType>,
    }

    error: crate::Error
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self
    }
}

impl Response {
    /// Creates a new `Response` with the given login types.
    pub fn new(flows: Vec<LoginType>) -> Self {
        Self { flows }
    }
}

/// An authentication mechanism.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
pub enum LoginType {
    /// A password is supplied to authenticate.
    #[ruma_enum(rename = "m.login.password")]
    Password,

    /// Token-based login.
    #[ruma_enum(rename = "m.login.token")]
    Token,

    /// SSO-based login.
    #[ruma_enum(rename = "m.login.sso")]
    Sso,

    #[doc(hidden)]
    _Custom(String),
}

mod login_type_list_serde;

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use serde::Deserialize;
    use serde_json::{from_value as from_json_value, json};

    use super::{login_type_list_serde, LoginType};

    #[derive(Debug, Deserialize)]
    struct Foo {
        #[serde(with = "login_type_list_serde")]
        pub flows: Vec<LoginType>,
    }

    #[test]
    fn deserialize_login_type() {
        assert_matches!(
            from_json_value::<Foo>(json!({
                "flows": [
                    { "type": "m.login.password" }
                ],
            })),
            Ok(Foo { flows })
            if flows.len() == 1
                && flows[0] == LoginType::Password
        );
    }
}
