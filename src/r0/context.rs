//! Endpoints for event context.

/// [GET /_matrix/client/r0/rooms/{roomId}/context/{eventId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-context-eventid)
pub mod get_context {
    use ruma_identifiers::{EventId, RoomId};
    use ruma_events::collections::only;


    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub event_id: EventId,
        pub room_id: RoomId,
    }

    /// This API endpoint's query string parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct QueryParams {
        pub limit: u8,
    }

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub end: String,
        pub event: only::RoomEvent,
        pub events_after: Vec<only::RoomEvent>,
        pub events_before: Vec<only::RoomEvent>,
        pub start: String,
        pub state: Vec<only::StateEvent>,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = QueryParams;
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(params: Self::PathParams) -> String {
            format!("/_matrix/client/r0/rooms/{}/context/{}", params.room_id, params.event_id)
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/rooms/:room_id/context/:event_id"
        }

        fn name() -> &'static str {
            "get_context"
        }

        fn description() -> &'static str {
            "Get the events immediately preceding and following a given event."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            false
        }
    }
}
