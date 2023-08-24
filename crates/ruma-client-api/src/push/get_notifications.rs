//! `GET /_matrix/client/*/notifications`
//!
//! Paginate through the list of events that the user has been, or would have been notified about.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3notifications

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        push::Action,
        serde::Raw,
        MilliSecondsSinceUnixEpoch, OwnedRoomId,
    };
    use ruma_events::AnySyncTimelineEvent;
    use serde::{Deserialize, Serialize};

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/notifications",
            1.1 => "/_matrix/client/v3/notifications",
        }
    };

    /// Request type for the `get_notifications` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {
        /// Pagination token given to retrieve the next set of events.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub from: Option<String>,

        /// Limit on the number of events to return in this request.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,

        /// Allows basic filtering of events returned.
        ///
        /// Supply "highlight" to return only events where the notification had the 'highlight'
        /// tweak set.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub only: Option<String>,
    }

    /// Response type for the `get_notifications` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The token to supply in the from param of the next /notifications request in order to
        /// request more events.
        ///
        /// If this is absent, there are no more results.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_token: Option<String>,

        /// The list of events that triggered notifications.
        pub notifications: Vec<Notification>,
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    impl Response {
        /// Creates a new `Response` with the given notifications.
        pub fn new(notifications: Vec<Notification>) -> Self {
            Self { next_token: None, notifications }
        }
    }

    /// Represents a notification.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct Notification {
        /// The actions to perform when the conditions for this rule are met.
        pub actions: Vec<Action>,

        /// The event that triggered the notification.
        pub event: Raw<AnySyncTimelineEvent>,

        /// The profile tag of the rule that matched this event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub profile_tag: Option<String>,

        /// Indicates whether the user has sent a read receipt indicating that they have read this
        /// message.
        pub read: bool,

        /// The ID of the room in which the event was posted.
        pub room_id: OwnedRoomId,

        /// The time at which the event notification was sent.
        pub ts: MilliSecondsSinceUnixEpoch,
    }

    impl Notification {
        /// Creates a new `Notification` with the given actions, event, read flag, room ID and
        /// timestamp.
        pub fn new(
            actions: Vec<Action>,
            event: Raw<AnySyncTimelineEvent>,
            read: bool,
            room_id: OwnedRoomId,
            ts: MilliSecondsSinceUnixEpoch,
        ) -> Self {
            Self { actions, event, profile_tag: None, read, room_id, ts }
        }
    }
}
