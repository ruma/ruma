//! `POST /_matrix/client/*/delayed_events/{delayed_id}`
//!
//! Send a delayed event update. This can be a updateing/canceling/sending the associated delayed
//! event.

use ruma_common::serde::StringEnum;

use crate::PrivOwnedStr;

/// The possible update actions we can do for updating a delayed event.
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum UpdateAction {
    /// Restart the delayed event timeout. (heartbeat ping)
    Restart,
    /// Send the delayed event immediately independent of the timeout state. (deletes all
    /// timers)
    Send,
    /// Delete the delayed event and never send it. (deletes all timers)
    Cancel,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

pub mod unstable_v1;

pub mod unstable_v2 {
    //! `msc3814` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140

    use ruma_common::{
        api::{auth_scheme::NoAccessToken, request, response},
        metadata,
    };

    use super::UpdateAction;

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: NoAccessToken,
        history: {
            unstable("org.matrix.msc4140") => "/_matrix/client/unstable/org.matrix.msc4140/delayed_events/{delay_id}/{action}",
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
        #[ruma_api(path)]
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
    mod client_tests {
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
            let request: http::Request<Vec<u8>> =
                Request::new("1234".to_owned(), UpdateAction::Cancel)
                    .try_into_http_request(
                        "https://homeserver.tld",
                        SendAccessToken::None,
                        Cow::Owned(supported),
                    )
                    .unwrap();

            let (parts, body) = request.into_parts();

            assert_eq!(
                "https://homeserver.tld/_matrix/client/unstable/org.matrix.msc4140/delayed_events/1234/cancel",
                parts.uri.to_string()
            );
            assert_eq!("POST", parts.method.to_string());
            assert_eq!(
                json!({}),
                serde_json::from_str::<JsonValue>(std::str::from_utf8(&body).unwrap()).unwrap()
            );
        }
    }

    #[cfg(all(test, feature = "server"))]
    mod server_tests {

        use ruma_common::api::IncomingRequest;

        use super::{Request, UpdateAction};

        #[test]
        fn deserialize_update_delayed_events_request() {
            let uri = http::Uri::builder()
                .scheme("https")
                .authority("matrix.org")
                .path_and_query(
                    "/_matrix/client/unstable/org.matrix.msc4140/delayed_events/a_delay_id/send",
                )
                .build()
                .unwrap();

            let req = Request::try_from_http_request(
                http::Request::builder().method("POST").uri(uri).body("").unwrap(),
                &["a_delay_id", "send"],
            )
            .unwrap();

            assert_eq!(req.delay_id, "a_delay_id".to_owned());
            assert_eq!(req.action, UpdateAction::Send);
        }
    }
}
