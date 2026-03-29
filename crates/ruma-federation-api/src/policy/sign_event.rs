//! `POST /_matrix/policy/*/sign`
//!
//! Ask the [Policy Server] to sign an event.
//!
//! This endpoint MUST NOT be called for events which have a type of `m.room.policy` and an empty
//! string `state_key`. All other events, including state events, non-state `m.room.policy` events,
//! and `m.room.policy` state events with non-empty string `state_key`s are processed by this
//! endpoint.
//!
//! Whether a signature is required by a Policy Server further depends on whether the room has
//! enabled a Policy Server.
//!
//! [Policy Server]: https://spec.matrix.org/v1.18/server-server-api/#policy-servers

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.18/server-server-api/#post_matrixpolicyv1sign

    use ruma_common::{
        OwnedServerName, ServerName, ServerSignatures as ServerSignaturesMap,
        ServerSigningKeyVersion, SigningKeyId,
        api::{request, response},
        metadata,
    };
    use serde_json::value::RawValue as RawJsonValue;

    use crate::authentication::ServerSignatures as ServerSignaturesAuth;

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: ServerSignaturesAuth,
        path: "/_matrix/policy/v1/sign",
    }

    /// Request type for the `sign_event` endpoint.
    #[request]
    pub struct Request {
        /// The PDU to sign.
        #[ruma_api(body)]
        pub pdu: Box<RawJsonValue>,
    }

    /// Response type for the `sign_event` endpoint.
    #[response]
    pub struct Response {
        /// A map containing the Policy Server's signature of the event.
        ///
        /// This signature is to be added to the event before sending or processing the event
        /// further.
        ///
        /// `ed25519:policy_server` is always used for Ed25519 signatures.
        #[ruma_api(body)]
        pub signatures: ServerSignaturesMap,
    }

    impl Request {
        /// Creates a new `Request` with the given PDU.
        pub fn new(pdu: Box<RawJsonValue>) -> Self {
            Self { pdu }
        }
    }

    impl Response {
        /// The signing key ID that must be used by the Policy Server for the ed25519 signature.
        pub const POLICY_SERVER_ED25519_SIGNING_KEY_ID: &str = "ed25519:policy_server";

        /// Creates a new `Response` with the given Policy Server name and event signature.
        pub fn new(server_name: OwnedServerName, ed25519_signature: String) -> Self {
            Self {
                signatures: ServerSignaturesMap::from_iter(std::iter::once((
                    server_name,
                    SigningKeyId::parse(Self::POLICY_SERVER_ED25519_SIGNING_KEY_ID)
                        .expect("Policy Server default ed25519 signing key ID should be valid"),
                    ed25519_signature,
                ))),
            }
        }

        /// Get the signature of the event for the given Policy Server name, if any.
        pub fn ed25519_signature(&self, server_name: &ServerName) -> Option<&str> {
            self.signatures
                .get(server_name)?
                .get(
                    <&SigningKeyId<ServerSigningKeyVersion>>::try_from(
                        Self::POLICY_SERVER_ED25519_SIGNING_KEY_ID,
                    )
                    .expect("Policy Server default ed25519 signing key ID should be valid"),
                )
                .map(String::as_str)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::v1::Response;

    #[cfg(feature = "server")]
    #[test]
    fn construct_and_serialize_response() {
        use ruma_common::{api::OutgoingResponse, owned_server_name};
        use serde_json::{Value as JsonValue, from_slice as from_json_slice, json};

        let response = Response::new(owned_server_name!("policy.example.org"), "zLFxllD0pbBuBpfHh8NuHNaICpReF/PAOpUQTsw+bFGKiGfDNAsnhcP7pbrmhhpfbOAxIdLraQLeeiXBryLmBw".to_owned());

        let http_response = response.try_into_http_response::<Vec<u8>>().unwrap();

        assert_eq!(
            from_json_slice::<JsonValue>(http_response.body()).unwrap(),
            json!({
                "policy.example.org": {
                    "ed25519:policy_server": "zLFxllD0pbBuBpfHh8NuHNaICpReF/PAOpUQTsw+bFGKiGfDNAsnhcP7pbrmhhpfbOAxIdLraQLeeiXBryLmBw",
                },
            })
        );
    }

    #[cfg(feature = "client")]
    #[test]
    fn deserialize_response() {
        use ruma_common::{api::IncomingResponse, server_name};
        use serde_json::{json, to_vec as to_json_vec};

        let http_response = http::Response::new(to_json_vec(&json!({
            "policy.example.org": {
                "ed25519:policy_server": "zLFxllD0pbBuBpfHh8NuHNaICpReF/PAOpUQTsw+bFGKiGfDNAsnhcP7pbrmhhpfbOAxIdLraQLeeiXBryLmBw",
            },
        })).unwrap());

        let response = Response::try_from_http_response(http_response).unwrap();

        assert_eq!(
            response.ed25519_signature(server_name!("policy.example.org")),
            Some(
                "zLFxllD0pbBuBpfHh8NuHNaICpReF/PAOpUQTsw+bFGKiGfDNAsnhcP7pbrmhhpfbOAxIdLraQLeeiXBryLmBw"
            )
        );
    }
}
