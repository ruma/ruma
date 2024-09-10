#[allow(deprecated)]
#[cfg(all(feature = "server", not(feature = "unstable-unspecified")))]
mod v1 {
    use ruma_common::api::OutgoingResponse;
    use ruma_federation_api::membership::create_join_event::v1::{Response, RoomState};
    use serde_json::{from_slice as from_json_slice, json, Value as JsonValue};

    #[test]
    fn response_body() {
        let res = Response::new(RoomState::new("ORIGIN".to_owned()))
            .try_into_http_response::<Vec<u8>>()
            .unwrap();

        assert_eq!(
            from_json_slice::<JsonValue>(res.body()).unwrap(),
            json!([200, { "auth_chain": [], "origin": "ORIGIN", "state": [] }])
        );
    }
}
