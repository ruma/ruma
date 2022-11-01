use http::header::LOCATION;
use ruma_common::{
    api::{request, response, Metadata},
    metadata,
};

const METADATA: Metadata = metadata! {
    description: "Does something.",
    method: GET,
    name: "no_fields",
    rate_limited: false,
    authentication: None,
    history: {
        unstable => "/_matrix/my/endpoint",
    }
};

#[request]
pub struct Request {
    #[ruma_api(header = LOCATION)]
    pub location: Option<String>,
}

#[response]
pub struct Response {
    #[ruma_api(header = LOCATION)]
    pub stuff: Option<String>,
}
