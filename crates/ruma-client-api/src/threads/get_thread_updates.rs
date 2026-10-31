//! `GET /_matrix/client/*/thread_updates`
//!
//! Retrieve a paginated range of thread updates across all joined rooms.

pub mod unstable {
    //! `/unstable/` ([spec])
    //!
    //! [spec]: https://github.com/matrix-org/matrix-spec-proposals/pull/4360

    use std::collections::BTreeMap;

    use js_int::UInt;
    use ruma_common::{
        OwnedEventId, OwnedRoomId,
        api::{Direction, auth_scheme::AccessToken, request, response},
        metadata,
        serde::Raw,
    };
    use ruma_events::AnySyncTimelineEvent;
    use serde::{Deserialize, Serialize};

    metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/io.element.msc4360/thread_updates",
        }
    }

    /// Request type for the `get_thread_updates` endpoint.
    #[request]
    pub struct Request {
        /// The direction to use for pagination.
        ///
        /// Only `Direction::Backward` is meant to be supported, which is why this field is
        /// private.
        #[ruma_api(query)]
        // Because this field is private, it is never read.
        #[allow(dead_code)]
        dir: Direction,

        /// A token to continue pagination from.
        ///
        /// This token can be acquired from a previous `/thread_updates` response, or the
        /// `prev_batch` in a sliding sync response's `threads` field.
        ///
        /// The token is opaque and has no client-discernible meaning.
        ///
        /// If not provided, then the pagination starts from the "end".
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub from: Option<String>,

        /// A token used to limit the pagination.
        ///
        /// The token can be set to the value of a sliding sync `pos` field used in a request that
        /// returned thread updates with a `prev_batch` token.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub to: Option<String>,

        /// A maximum number of thread updates to fetch in one response.
        ///
        /// Defaults to 100, if not provided. Servers may impose a smaller limit than requested.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,
    }

    /// Response type for the `get_thread_updates` endpoint.
    #[response]
    pub struct Response {
        /// The thread updates, by room ID and thread root event ID.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub chunk: BTreeMap<OwnedRoomId, BTreeMap<OwnedEventId, ThreadUpdate>>,

        /// If there are still more results to fetch, this is the token to use as the next `from`
        /// value.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_batch: Option<String>,
    }

    impl Request {
        /// Creates a new empty `Request`.
        pub fn new() -> Self {
            Self { dir: Direction::Backward, from: None, to: None, limit: None }
        }
    }

    impl Response {
        /// Creates a new empty `Response`.
        pub fn new() -> Self {
            Self { chunk: BTreeMap::new(), next_batch: None }
        }
    }

    /// An update to a thread.
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ThreadUpdate {
        /// The thread root event, with the bundled `m.thread` aggregation.
        ///
        /// In the sliding sync extension, it is only present if `include_roots` is set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub thread_root: Option<Raw<AnySyncTimelineEvent>>,

        /// A token to paginate the other updates in this thread with the `/relations` endpoint.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prev_batch: Option<String>,
    }

    impl ThreadUpdate {
        /// Creates an empty `ThreadUpdate`.
        pub fn new() -> Self {
            Default::default()
        }
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use ruma_common::api::IncomingRequestExt as _;

    use super::unstable::Request;

    #[test]
    fn deserialize_request() {
        let http_req = http::Request::builder()
            .method(http::Method::GET)
            .uri("/_matrix/client/unstable/io.element.msc4360/thread_updates?dir=b&from=s1&to=s2&limit=10")
            .body(&[] as &[u8])
            .unwrap();

        let req = Request::try_from_http_request(http_req, &[] as &[&str]).unwrap();

        assert_eq!(req.from.as_deref(), Some("s1"));
        assert_eq!(req.to.as_deref(), Some("s2"));
        assert_eq!(req.limit, Some(js_int::uint!(10)));
    }
}
