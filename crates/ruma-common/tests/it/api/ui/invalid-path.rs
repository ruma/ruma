#![allow(unexpected_cfgs)]

use ruma_common::api::{Metadata, auth_scheme::NoAuthentication, metadata};

mod invalid_char_single_path {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        path: "µ/°/§/€",
    }

    pub struct Request;
}

mod invalid_char_version_history {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.1 => "µ/°/§/€",
        },
    }

    pub struct Request;
}

mod whitespace_single_path {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        path: "path/to/invalid space/endpoint",
    }

    pub struct Request;
}

mod whitespace_version_history {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.1 => "path/to/invalid space/endpoint",
        },
    }

    pub struct Request;
}

mod old_variable_syntax_single_path {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        path: "path/to/endpoint/:variable",
    }

    pub struct Request;
}

mod old_variable_syntax_version_history {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.1 => "path/to/endpoint/:variable",
        },
    }

    pub struct Request;
}

mod missing_variable_closing_single_path {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        path: "path/to/endpoint/{variable",
    }

    pub struct Request;
}

mod missing_variable_closing_version_history {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.1 => "path/to/endpoint/{variable",
        },
    }

    pub struct Request;
}

mod missing_variable_opening_single_path {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        path: "path/to/endpoint/variable}",
    }

    pub struct Request;
}

mod missing_variable_opening_version_history {
    use super::*;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.1 => "path/to/endpoint/variable}",
        },
    }

    pub struct Request;
}

fn main() {
    let _ = invalid_char_single_path::Request::PATH_BUILDER;
    let _ = invalid_char_version_history::Request::PATH_BUILDER;

    let _ = whitespace_single_path::Request::PATH_BUILDER;
    let _ = whitespace_version_history::Request::PATH_BUILDER;

    let _ = old_variable_syntax_single_path::Request::PATH_BUILDER;
    let _ = old_variable_syntax_version_history::Request::PATH_BUILDER;

    let _ = missing_variable_closing_single_path::Request::PATH_BUILDER;
    let _ = missing_variable_closing_version_history::Request::PATH_BUILDER;

    let _ = missing_variable_opening_single_path::Request::PATH_BUILDER;
    let _ = missing_variable_opening_version_history::Request::PATH_BUILDER;
}
