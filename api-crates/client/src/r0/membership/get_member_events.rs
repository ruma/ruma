//! [GET /_matrix/client/r0/rooms/{roomId}/members](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-rooms-roomid-members)

use ruma_api::ruma_api;
use ruma_events::room::member::MemberEvent;
use ruma_identifiers::RoomId;
use ruma_serde::{Raw, StringEnum};

ruma_api! {
    metadata: {
        description: "Get membership events for a room.",
        method: GET,
        name: "get_member_events",
        path: "/_matrix/client/r0/rooms/:room_id/members",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The room to get the member events for.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The point in time (pagination token) to return members for in the room. This token can
        /// be obtained from a prev_batch token returned for each room by the sync API.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub at: Option<&'a str>,

        /// The kind of memberships to filter for. Defaults to no filtering if unspecified. When
        /// specified alongside not_membership, the two parameters create an 'or' condition: either
        /// the membership is the same as membership or is not the same as not_membership.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub membership: Option<MembershipEventFilter>,

        /// The kind of memberships to *exclude* from the results. Defaults to no filtering if
        /// unspecified.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub not_membership: Option<MembershipEventFilter>,
    }

    response: {
        /// A list of member events.
        pub chunk: Vec<Raw<MemberEvent>>
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
    pub fn new(chunk: Vec<Raw<MemberEvent>>) -> Self {
        Self { chunk }
    }
}

/// The kind of membership events to filter for.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
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
    _Custom(String),
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use matches::assert_matches;
    use ruma_api::IncomingRequest as _;

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
