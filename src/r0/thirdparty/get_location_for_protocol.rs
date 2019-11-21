//! [GET /_matrix/client/r0/thirdparty/location/{protocol}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-thirdparty-location-protocol)

use std::collections::HashMap;

use ruma_api::ruma_api;

use super::Location;

ruma_api! {
    metadata {
        description: "Fetches third party locations for a protocol.",
        method: GET,
        name: "get_location_for_protocol",
        path: "/_matrix/client/r0/thirdparty/location/:protocol",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The protocol used to communicate to the third party network.
        #[ruma_api(path)]
        pub protocol: String,
        /// One or more custom fields to help identify the third party location.
        // The specification is incorrect for this parameter. See matrix-org/matrix-doc#2352.
        #[ruma_api(query_map)]
        pub fields: HashMap<String, String>,
    }

    response {
        /// List of matched third party locations.
        #[ruma_api(body)]
        pub locations: Vec<Location>,
    }
}
