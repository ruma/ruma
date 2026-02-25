//! `GET /_matrix/client/v1/summary/{roomIdOrAlias}`
//!
//! Returns a short description of the state of a room.

pub mod v1 {
    //! `v1` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv1room_summaryroomidoralias

    use ruma_common::{
        RoomOrAliasId, ServerName,
        api::{auth_scheme::AccessTokenOptional, request},
        metadata,
        room::RoomSummary,
    };
    use ruma_events::room::member::MembershipState;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessTokenOptional,
        history: {
            unstable => "/_matrix/client/unstable/im.nheko.summary/rooms/{room_id_or_alias}/summary",
            1.15 => "/_matrix/client/v1/room_summary/{room_id_or_alias}",
        }
    }

    /// Request type for the `get_summary` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// Alias or ID of the room to be summarized.
        #[ruma_api(path)]
        pub room_id_or_alias: RoomOrAliasId,

        /// A list of servers the homeserver should attempt to use to peek at the room.
        ///
        /// Defaults to an empty `Vec`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ruma_api(query)]
        pub via: Vec<ServerName>,
    }

    impl Request {
        /// Creates a new `Request` with the given room or alias ID and via server names.
        pub fn new(room_id_or_alias: RoomOrAliasId, via: Vec<ServerName>) -> Self {
            Self { room_id_or_alias, via }
        }
    }

    /// Response type for the `get_summary` endpoint.
    #[derive(Debug, Clone)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Response {
        /// The summary of the room.
        pub summary: RoomSummary,

        /// The current membership of this user in the room.
        ///
        /// This field will not be present when called unauthenticated, but is required when called
        /// authenticated. It should be `leave` if the server doesn't know about the room, since
        /// for all other membership states the server would know about the room already.
        pub membership: Option<MembershipState>,
    }

    impl Response {
        /// Creates a new [`Response`] with the given summary.
        pub fn new(summary: RoomSummary) -> Self {
            Self { summary, membership: None }
        }
    }

    impl From<RoomSummary> for Response {
        fn from(value: RoomSummary) -> Self {
            Self::new(value)
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::OutgoingResponse for Response {
        fn try_into_http_response<T: Default + bytes::BufMut>(
            self,
        ) -> Result<http::Response<T>, ruma_common::api::error::IntoHttpError> {
            #[derive(serde::Serialize)]
            struct ResponseSerHelper {
                #[serde(flatten)]
                summary: RoomSummary,
                #[serde(skip_serializing_if = "Option::is_none")]
                membership: Option<MembershipState>,
            }

            let body = ResponseSerHelper { summary: self.summary, membership: self.membership };

            http::Response::builder()
                .header(http::header::CONTENT_TYPE, ruma_common::http_headers::APPLICATION_JSON)
                .body(ruma_common::serde::json_to_buf(&body)?)
                .map_err(Into::into)
        }
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::IncomingResponse for Response {
        type EndpointError = crate::Error;

        fn try_from_http_response<T: AsRef<[u8]>>(
            response: http::Response<T>,
        ) -> Result<Self, ruma_common::api::error::FromHttpResponseError<Self::EndpointError>>
        {
            use ruma_common::{api::EndpointError, serde::from_raw_json_value};

            #[derive(serde::Deserialize)]
            struct ResponseDeHelper {
                membership: Option<MembershipState>,
            }

            if response.status().as_u16() >= 400 {
                return Err(ruma_common::api::error::FromHttpResponseError::Server(
                    Self::EndpointError::from_http_response(response),
                ));
            }

            let raw_json = serde_json::from_slice::<Box<serde_json::value::RawValue>>(
                response.body().as_ref(),
            )?;
            let summary = from_raw_json_value::<RoomSummary, serde_json::Error>(&raw_json)?;
            let membership =
                from_raw_json_value::<ResponseDeHelper, serde_json::Error>(&raw_json)?.membership;

            Ok(Self { summary, membership })
        }
    }
}

#[cfg(all(test, feature = "client"))]
mod tests {
    use ruma_common::api::IncomingResponse;
    use ruma_events::room::member::MembershipState;
    use serde_json::{json, to_vec as to_json_vec};

    use super::v1::Response;

    #[test]
    fn deserialize_response() {
        let body = json!({
            "room_id": "!room:localhost",
            "num_joined_members": 5,
            "world_readable": false,
            "guest_can_join": false,
            "join_rule": "restricted",
            "allowed_room_ids": ["!otherroom:localhost"],
            "membership": "invite",
        });
        let response = http::Response::new(to_json_vec(&body).unwrap());

        let response = Response::try_from_http_response(response).unwrap();
        assert_eq!(response.summary.room_id, "!room:localhost");
        assert_eq!(response.membership, Some(MembershipState::Invite));
    }
}
