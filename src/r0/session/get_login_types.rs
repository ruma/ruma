//! [GET /_matrix/client/r0/login](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-login)

use ruma_api::ruma_api;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Gets the homeserver's supported login types to authenticate users. Clients should pick one of these and supply it as the type when logging in.",
        method: GET,
        name: "get_login_types",
        path: "/_matrix/client/r0/login",
        rate_limited: true,
        requires_authentication: false,
    }

    request {}

    response {
        /// The homeserver's supported login types.
        pub flows: Vec<LoginType>
    }

    error: crate::Error
}

/// An authentication mechanism.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum LoginType {
    /// A password is supplied to authenticate.
    #[serde(rename = "m.login.password")]
    Password,
    /// Token-based login.
    #[serde(rename = "m.login.token")]
    Token,
}

#[cfg(test)]
mod tests {
    use super::LoginType;

    #[test]
    fn deserialize_login_type() {
        assert_eq!(
            serde_json::from_str::<LoginType>(r#" {"type": "m.login.password"} "#).unwrap(),
            LoginType::Password,
        );
    }
}
