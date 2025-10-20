use ruma_common::{api::auth_scheme::NoAuthentication, metadata};

metadata! {
    method: GET,
    rate_limited: false,
    authentication: NoAuthentication,
    history: {
        unstable => "/a/path",
        1.1 => deprecated,
    }
}

pub struct Request;

fn main() {}
