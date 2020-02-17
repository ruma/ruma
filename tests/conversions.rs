use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata {
        description: "Does something.",
        method: POST,
        name: "my_endpoint",
        path: "/_matrix/foo/:bar/:baz",
        rate_limited: false,
        requires_authentication: false,
    }

    request {
        pub hello: String,
        #[ruma_api(header = CONTENT_TYPE)]
        pub world: String,
        #[ruma_api(query)]
        pub q1: String,
        #[ruma_api(query)]
        pub q2: u32,
        #[ruma_api(path)]
        pub bar: String,
        #[ruma_api(path)]
        pub baz: UserId,
    }

    response {
        pub hello: String,
        #[ruma_api(header = CONTENT_TYPE)]
        pub world: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub optional_flag: Option<bool>,
    }
}

#[test]
fn request_serde() -> Result<(), Box<dyn std::error::Error + 'static>> {
    use std::convert::TryFrom;

    let req = Request {
        hello: "hi".to_owned(),
        world: "test".to_owned(),
        q1: "query_param_special_chars %/&@!".to_owned(),
        q2: 55,
        bar: "bar".to_owned(),
        baz: UserId::try_from("@bazme:ruma.io")?,
    };

    let http_req = http::Request::<Vec<u8>>::try_from(req.clone())?;
    let req2 = Request::try_from(http_req)?;

    assert_eq!(req.hello, req2.hello);
    assert_eq!(req.world, req2.world);
    assert_eq!(req.q1, req2.q1);
    assert_eq!(req.q2, req2.q2);
    assert_eq!(req.bar, req2.bar);
    assert_eq!(req.baz, req2.baz);

    Ok(())
}
