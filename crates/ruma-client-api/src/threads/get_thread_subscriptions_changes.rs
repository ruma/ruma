//! `GET /_matrix/client/*/thread_subscriptions`
//!
//! Retrieve a paginated range of thread subscriptions across all rooms.

pub mod unstable {
    //! `/unstable/` ([spec])
    //!
    //! [spec]: https://github.com/matrix-org/matrix-spec-proposals/pull/4308

    use std::collections::BTreeMap;

    use js_int::UInt;
    use ruma_common::{
        OwnedEventId, OwnedRoomId,
        api::{Direction, auth_scheme::AccessToken, request, response},
        metadata,
    };
    use serde::{Deserialize, Serialize};

    metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("org.matrix.msc4308") => "/_matrix/client/unstable/io.element.msc4308/thread_subscriptions",
        }
    }

    /// Request type for the `get_thread_subscriptions_changes` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The direction to use for pagination.
        ///
        /// Only `Direction::Backward` is meant to be supported, which is why this field is private
        /// for now (as of 2025-08-21).
        #[ruma_api(query)]
        // Because this field is private, it is never read.
        #[allow(dead_code)]
        dir: Direction,

        /// A token to continue pagination from.
        ///
        /// This token can be acquired from a previous `/thread_subscriptions` response, or the
        /// `prev_batch` in a sliding sync response's `thread_subscriptions` field.
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
        /// returned new thread subscriptions with a `prev_batch` token.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub to: Option<String>,

        /// A maximum number of thread subscriptions to fetch in one response.
        ///
        /// Defaults to 100, if not provided. Servers may impose a smaller limit than requested.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,
    }

    /// A thread has been subscribed to at some point.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ThreadSubscription {
        /// Whether the subscription was made automatically by a client, not by manual user choice.
        pub automatic: bool,

        /// The bump stamp of the thread subscription, to be used to compare with other changes
        /// related to the same thread.
        pub bump_stamp: UInt,
    }

    impl ThreadSubscription {
        /// Create a new [`ThreadSubscription`] with the given values.
        pub fn new(automatic: bool, bump_stamp: UInt) -> Self {
            Self { automatic, bump_stamp }
        }
    }

    /// A thread has been unsubscribed to at some point.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ThreadUnsubscription {
        /// The bump stamp of the thread subscription, to be used to compare with other changes
        /// related to the same thread.
        pub bump_stamp: UInt,
    }

    impl ThreadUnsubscription {
        /// Create a new [`ThreadUnsubscription`] with the given bump stamp.
        pub fn new(bump_stamp: UInt) -> Self {
            Self { bump_stamp }
        }
    }

    /// Response type for the `get_thread_subscriptions_changes` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// New thread subscriptions.
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        pub subscribed: BTreeMap<OwnedRoomId, BTreeMap<OwnedEventId, ThreadSubscription>>,

        /// New thread unsubscriptions.
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        pub unsubscribed: BTreeMap<OwnedRoomId, BTreeMap<OwnedEventId, ThreadUnsubscription>>,

        /// If there are still more results to fetch, this is the token to use as the next `from`
        /// value.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub end: Option<String>,
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
            Self { subscribed: BTreeMap::new(), unsubscribed: BTreeMap::new(), end: None }
        }
    }
}
