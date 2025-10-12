use ruma_common::{api::Metadata, metadata};

metadata! {
    method: GET,
    rate_limited: false,
    authentication: None,
    history: {
        unstable => "/a/path",
        1.1 => removed,
    }
}

pub struct Request;

fn main() {}
