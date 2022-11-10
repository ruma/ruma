use http::header::LOCATION;
use ruma_common::{
    api::{request, response, Metadata},
    metadata,
};

const METADATA: Metadata = metadata! {
    method: GET,
    rate_limited: false,
    authentication: None,
    history: {
        unstable => "/_matrix/my/endpoint",
    }
};

/// Request type for the `no_fields` endpoint.
#[request]
pub struct Request {
    #[ruma_api(header = LOCATION)]
    pub location: Option<String>,
}

/// Response type for the `no_fields` endpoint.
#[response]
pub struct Response {
    #[ruma_api(header = LOCATION)]
    pub stuff: Option<String>,
}
