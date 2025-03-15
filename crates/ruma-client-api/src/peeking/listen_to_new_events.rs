//! `GET /_matrix/client/*/events`
//!
//! Listen for new events related to a particular room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#peeking_get_matrixclientv3events

    use std::time::Duration;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        OwnedRoomId,
    };
    use ruma_events::AnyTimelineEvent;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/events",
            1.1 => "/_matrix/client/v3/events",
        }
    };

    /// Request type for the `listen_to_new_events` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {
        /// The token to stream from.
        ///
        /// This token is either from a previous request to this API or from the initial sync API.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub from: Option<String>,

        /// The room ID for which events should be returned.
        ///
        /// This field [should be required], but the spec marks it as optional so it is an
        /// `Option`.
        ///
        /// [should be required]: https://github.com/matrix-org/matrix-spec/issues/2098
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_id: Option<OwnedRoomId>,

        /// The maximum time to wait for an event.
        #[ruma_api(query)]
        #[serde(
            with = "ruma_common::serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        pub timeout: Option<Duration>,
    }

    impl Request {
        /// Creates a `Request` for the given room.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id: Some(room_id), ..Default::default() }
        }
    }

    /// Response type for the `listen_to_new_events` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// An array of new events.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub chunk: Vec<Raw<AnyTimelineEvent>>,

        /// A token which correlates to the last value in `chunk`.
        ///
        /// This token should be used in the next request to this endpoint.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub end: Option<String>,

        /// A token which correlates to the first value in `chunk`.
        ///
        /// This is usually the same token supplied to `from` in the request.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub start: Option<String>,
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self::default()
        }
    }
}
