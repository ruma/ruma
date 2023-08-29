//! `GET /_matrix/client/*/rooms/{roomId}/members`
//!
//! Get membership events for a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3roomsroomidmembers

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::{Raw, StringEnum},
        OwnedRoomId,
    };
    use ruma_events::room::member::RoomMemberEvent;

    use crate::PrivOwnedStr;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/:room_id/members",
            1.1 => "/_matrix/client/v3/rooms/:room_id/members",
        }
    };

    /// Request type for the `get_member_events` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room to get the member events for.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The point in time (pagination token) to return members for in the room.
        ///
        /// This token can be obtained from a prev_batch token returned for each room by the sync
        /// API.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub at: Option<String>,

        /// The kind of memberships to filter for.
        ///
        /// Defaults to no filtering if unspecified. When specified alongside not_membership, the
        /// two parameters create an 'or' condition: either the membership is the same as
        /// membership or is not the same as not_membership.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub membership: Option<MembershipEventFilter>,

        /// The kind of memberships to *exclude* from the results.
        ///
        /// Defaults to no filtering if unspecified.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub not_membership: Option<MembershipEventFilter>,
    }

    /// Response type for the `get_member_events` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// A list of member events.
        pub chunk: Vec<Raw<RoomMemberEvent>>,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id, at: None, membership: None, not_membership: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given member event chunk.
        pub fn new(chunk: Vec<Raw<RoomMemberEvent>>) -> Self {
            Self { chunk }
        }
    }

    /// The kind of membership events to filter for.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, PartialEq, Eq, StringEnum)]
    #[ruma_enum(rename_all = "lowercase")]
    #[non_exhaustive]
    pub enum MembershipEventFilter {
        /// The user has joined.
        Join,

        /// The user has been invited.
        Invite,

        /// The user has left.
        Leave,

        /// The user has been banned.
        Ban,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }

    #[cfg(all(test, feature = "server"))]
    mod tests {
        use ruma_common::api::IncomingRequest as _;

        use super::{MembershipEventFilter, Request};

        #[test]
        fn deserialization() {
            let uri = http::Uri::builder()
                .scheme("https")
                .authority("example.org")
                .path_and_query(
                    "/_matrix/client/r0/rooms/!dummy%3Aexample.org/members\
                 ?not_membership=leave\
                 &at=1026",
                )
                .build()
                .unwrap();

            let req = Request::try_from_http_request(
                http::Request::builder().uri(uri).body(&[] as &[u8]).unwrap(),
                &["!dummy:example.org"],
            )
            .unwrap();

            assert_eq!(req.room_id, "!dummy:example.org");
            assert_eq!(req.at.as_deref(), Some("1026"));
            assert_eq!(req.membership, None);
            assert_eq!(req.not_membership, Some(MembershipEventFilter::Leave));
        }
    }
}
