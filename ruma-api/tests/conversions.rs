use ruma_api::{
    ruma_api, IncomingRequest as _, OutgoingRequest as _, OutgoingRequestAppserviceExt as _,
};
use ruma_identifiers::{user_id, UserId};

ruma_api! {
    metadata: {
        description: "Does something.",
        method: POST,
        name: "my_endpoint",
        path: "/_matrix/foo/:bar/:user",
        rate_limited: false,
        authentication: None,
    }

    request: {
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
        pub user: UserId,
    }

    response: {
        pub hello: String,
        #[ruma_api(header = CONTENT_TYPE)]
        pub world: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub optional_flag: Option<bool>,
    }
}

#[test]
fn request_serde() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let req = Request {
        hello: "hi".to_owned(),
        world: "test".to_owned(),
        q1: "query_param_special_chars %/&@!".to_owned(),
        q2: 55,
        bar: "barVal".to_owned(),
        user: user_id!("@bazme:ruma.io"),
    };

    let http_req = req.clone().try_into_http_request("https://homeserver.tld", None)?;
    let req2 = Request::try_from_http_request(http_req.map(std::io::Cursor::new))?;

    assert_eq!(req.hello, req2.hello);
    assert_eq!(req.world, req2.world);
    assert_eq!(req.q1, req2.q1);
    assert_eq!(req.q2, req2.q2);
    assert_eq!(req.bar, req2.bar);
    assert_eq!(req.user, req2.user);

    Ok(())
}

#[test]
fn request_with_user_id_serde() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let req = Request {
        hello: "hi".to_owned(),
        world: "test".to_owned(),
        q1: "query_param_special_chars %/&@!".to_owned(),
        q2: 55,
        bar: "barVal".to_owned(),
        user: user_id!("@bazme:ruma.io"),
    };

    let user_id = user_id!("@_virtual_:ruma.io");
    let http_req =
        req.try_into_http_request_with_user_id("https://homeserver.tld", None, user_id)?;

    let query = http_req.uri().query().unwrap();

    assert_eq!(
        query,
        "q1=query_param_special_chars+%25%2F%26%40%21&q2=55&user_id=%40_virtual_%3Aruma.io"
    );

    Ok(())
}

mod without_query {
    use super::*;

    ruma_api! {
        metadata: {
            description: "Does something without query.",
            method: POST,
            name: "my_endpoint",
            path: "/_matrix/foo/:bar/:user",
            rate_limited: false,
            authentication: None,
        }

        request: {
            pub hello: String,
            #[ruma_api(header = CONTENT_TYPE)]
            pub world: String,
            #[ruma_api(path)]
            pub bar: String,
            #[ruma_api(path)]
            pub user: UserId,
        }

        response: {
            pub hello: String,
            #[ruma_api(header = CONTENT_TYPE)]
            pub world: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub optional_flag: Option<bool>,
        }
    }

    #[test]
    fn request_without_query_with_user_id_serde() -> Result<(), Box<dyn std::error::Error + 'static>>
    {
        let req = Request {
            hello: "hi".to_owned(),
            world: "test".to_owned(),
            bar: "barVal".to_owned(),
            user: user_id!("@bazme:ruma.io"),
        };

        let user_id = user_id!("@_virtual_:ruma.io");
        let http_req =
            req.try_into_http_request_with_user_id("https://homeserver.tld", None, user_id)?;

        let query = http_req.uri().query().unwrap();

        assert_eq!(query, "user_id=%40_virtual_%3Aruma.io");

        Ok(())
    }
}
