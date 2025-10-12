use ruma_common::metadata;

metadata! {
    method: GET,
    rate_limited: false,
    authentication: None,
    history: {
        unstable => "/a/path",
        1.1 => deprecated,
    }
}

pub struct Request;

fn main() {}
