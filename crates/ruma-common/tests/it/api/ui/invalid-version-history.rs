use ruma_common::api::{Metadata, auth_scheme::NoAuthentication, metadata};

mod no_paths {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {},
    }

    pub struct Request;
}

mod variable_count_mismatch {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            unstable => "unstable/path/to/endpoint/{variable}/{other}",
            1.1 => "path/to/endpoint/{variable}",
        },
    }

    pub struct Request;
}

mod variable_name_mismatch {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            unstable => "unstable/path/to/endpoint/{var}",
            1.1 => "path/to/endpoint/{variable}",
        },
    }

    pub struct Request;
}

mod deprecated_without_added {
    use super::*;

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
}

mod deprecated_same_version {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.1 => "/a/path",
            1.1 => deprecated,
        }
    }

    pub struct Request;
}

mod removed_without_deprecated {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            unstable => "/a/path",
            1.1 => removed,
        }
    }

    pub struct Request;
}

mod removed_same_version {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.1 => "/a/path",
            1.2 => deprecated,
            1.2 => removed,
        }
    }

    pub struct Request;
}

mod duplicate_version {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.1 => "/a/path",
            1.1 => "/b/path",
        }
    }

    pub struct Request;
}

mod unsorted_versions {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.2 => "/a/path",
            1.1 => "/b/path",
        }
    }

    pub struct Request;
}

fn main() {
    let _ = no_paths::Request::PATH_BUILDER;
    let _ = variable_count_mismatch::Request::PATH_BUILDER;
    let _ = variable_name_mismatch::Request::PATH_BUILDER;
    let _ = deprecated_without_added::Request::PATH_BUILDER;
    let _ = deprecated_same_version::Request::PATH_BUILDER;
    let _ = removed_without_deprecated::Request::PATH_BUILDER;
    let _ = removed_same_version::Request::PATH_BUILDER;
    let _ = duplicate_version::Request::PATH_BUILDER;
    let _ = unsorted_versions::Request::PATH_BUILDER;
}
