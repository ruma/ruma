use ruma_common::{api::Metadata, metadata};

const _: Metadata = metadata! {
    method: GET,
    rate_limited: false,
    authentication: None,
    history: {
        unstable => "/a/path",
        1.1 => deprecated,
    }
};

fn main() {}
