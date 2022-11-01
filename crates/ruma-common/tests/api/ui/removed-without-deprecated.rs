use ruma_common::{api::Metadata, metadata};

const METADATA: Metadata = metadata! {
    description: "This will fail.",
    method: GET,
    name: "invalid_versions",
    rate_limited: false,
    authentication: None,
    history: {
        unstable => "/a/path",
        1.1 => removed,
    }
};

fn main() {}
