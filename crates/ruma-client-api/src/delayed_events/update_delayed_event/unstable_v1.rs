//! `msc4140` ([MSC])
//!
//! Old endpoint definition from MSC 4140. Does not correspond to the current state of the MSC.
//!
//! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/blob/3ee73abe5f81252b00877cfb5db941ee9aa6c18d/proposals/4140-delayed-events-futures.md

use ruma_common::{
    api::{auth_scheme::AccessToken, request, response},
    metadata,
};

use super::UpdateAction;

metadata! {
    method: POST,
    rate_limited: true,
    authentication: AccessToken,
    history: {
        unstable("org.matrix.msc4140") => "/_matrix/client/unstable/org.matrix.msc4140/delayed_events/{delay_id}",
    }
}

/// Request type for the [`update_delayed_event`](crate::delayed_events::update_delayed_event)
/// endpoint.
#[request]
pub struct Request {
    /// The delay id that we want to update.
    #[ruma_api(path)]
    pub delay_id: String,
    /// Which kind of update we want to request for the delayed event.
    pub action: UpdateAction,
}

impl Request {
    /// Creates a new `Request` to update a delayed event.
    pub fn new(delay_id: String, action: UpdateAction) -> Self {
        Self { delay_id, action }
    }
}

/// Response type for the [`update_delayed_event`](crate::delayed_events::update_delayed_event)
/// endpoint.
#[response]
pub struct Response {}
impl Response {
    /// Creates a new empty response for the
    /// [`update_delayed_event`](crate::delayed_events::update_delayed_event) endpoint.
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(all(test, feature = "client"))]
mod tests {
    use std::borrow::Cow;

    use ruma_common::api::{
        MatrixVersion, OutgoingRequest, SupportedVersions, auth_scheme::SendAccessToken,
    };
    use serde_json::{Value as JsonValue, json};

    use super::{Request, UpdateAction};
    #[test]
    fn serialize_update_delayed_event_request() {
        let supported = SupportedVersions {
            versions: [MatrixVersion::V1_1].into(),
            features: Default::default(),
        };
        let request: http::Request<Vec<u8>> = Request::new("1234".to_owned(), UpdateAction::Cancel)
            .try_into_http_request(
                "https://homeserver.tld",
                SendAccessToken::IfRequired("auth_tok"),
                Cow::Owned(supported),
            )
            .unwrap();

        let (parts, body) = request.into_parts();

        assert_eq!(
            "https://homeserver.tld/_matrix/client/unstable/org.matrix.msc4140/delayed_events/1234",
            parts.uri.to_string()
        );
        assert_eq!("POST", parts.method.to_string());
        assert_eq!(
            json!({"action": "cancel"}),
            serde_json::from_str::<JsonValue>(std::str::from_utf8(&body).unwrap()).unwrap()
        );
    }
}
