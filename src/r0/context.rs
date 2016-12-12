//! Endpoints for event context.

/// GET /_matrix/client/r0/rooms/:room_id/context/:event_id
pub mod get_context {
    use ruma_identifiers::{EventId, RoomId};
    use ruma_events::{RoomEvent, StateEvent};

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub event_id: EventId,
        pub room_id: RoomId,
    }

    /// This API endpoint's query string parameters.
    #[derive(Clone, Debug)]
    pub struct QueryParams {
        pub limit: u8,
    }

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub end: String,
        pub event: Box<RoomEvent>,
        pub events_after: Vec<Box<RoomEvent>>,
        pub events_before: Vec<Box<RoomEvent>>,
        pub start: String,
        pub state: Vec<Box<StateEvent>>,
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

        fn router_path() -> String {
            "/_matrix/client/r0/rooms/:room_id/context/:event_id".to_string()
        }
    }
}
