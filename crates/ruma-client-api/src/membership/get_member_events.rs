//! `GET /_matrix/client/*/rooms/{roomId}/members`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3roomsroomidmembers

    use ruma_common::{
        api::ruma_api,
        events::room::member::RoomMemberEvent,
        serde::{Raw, StringEnum},
        RoomId,
    };

    use crate::PrivOwnedStr;

    ruma_api! {
        metadata: {
            description: "Get membership events for a room.",
            method: GET,
            name: "get_member_events",
            r0_path: "/_matrix/client/r0/rooms/:room_id/members",
            stable_path: "/_matrix/client/v3/rooms/:room_id/members",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The room to get the member events for.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// The point in time (pagination token) to return members for in the room.
            ///
            /// This token can be obtained from a prev_batch token returned for each room by the sync
            /// API.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub at: Option<&'a str>,

            /// The kind of memberships to filter for.
            ///
            /// Defaults to no filtering if unspecified. When specified alongside not_membership, the
            /// two parameters create an 'or' condition: either the membership is the same as membership
            /// or is not the same as not_membership.
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

        response: {
            /// A list of member events.
            pub chunk: Vec<Raw<RoomMemberEvent>>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID.
        pub fn new(room_id: &'a RoomId) -> Self {
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
    #[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
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

    impl MembershipEventFilter {
        /// Creates a string slice from this `MembershipEventFilter`.
        pub fn as_str(&self) -> &str {
            self.as_ref()
        }
    }

    #[cfg(all(test, feature = "server"))]
    mod tests {
        use assert_matches::assert_matches;
        use ruma_common::api::IncomingRequest as _;

        use super::{IncomingRequest, MembershipEventFilter};

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

            let req = IncomingRequest::try_from_http_request(
                http::Request::builder().uri(uri).body(&[] as &[u8]).unwrap(),
                &["!dummy:example.org"],
            );

            assert_matches!(
                req,
                Ok(IncomingRequest {
                    room_id,
                    at: Some(at),
                    membership: None,
                    not_membership: Some(MembershipEventFilter::Leave),
                }) if room_id == "!dummy:example.org" && at == "1026"
            );
        }
    }
}
