//! `GET /_matrix/app/*/thirdparty/protocol/{protocol}`
//!
//! Fetches metadata about the various third party networks that an application service supports.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/application-service-api/#get_matrixappv1thirdpartyprotocolprotocol

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::{Protocol, ProtocolInstance, ProtocolInstanceInit},
    };
    use serde::{Deserialize, Serialize};

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/app/v1/thirdparty/protocol/:protocol",
        }
    };

    /// Request type for the `get_protocol` endpoint.
    #[request]
    pub struct Request {
        /// The name of the protocol.
        #[ruma_api(path)]
        pub protocol: String,
    }

    /// Response type for the `get_protocol` endpoint.
    #[response]
    pub struct Response {
        /// Metadata about the protocol.
        #[ruma_api(body)]
        pub protocol: AppserviceProtocol,
    }

    impl Request {
        /// Creates a new `Request` with the given protocol name.
        pub fn new(protocol: String) -> Self {
            Self { protocol }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given protocol.
        pub fn new(protocol: AppserviceProtocol) -> Self {
            Self { protocol }
        }
    }

    /// Metadata about a third party protocol, as returned by an appservice to a homeserver.
    ///
    /// To create an instance of this type, first create a [`ProtocolInit`] and convert it via
    /// `AppserviceProtocol::from` / `.into()`.
    ///
    /// [`ProtocolInit`]: ruma_common::thirdparty::ProtocolInit
    pub type AppserviceProtocol = Protocol<AppserviceProtocolInstance>;

    /// Metadata about an instance of a third party protocol, as returned by an appservice to a
    /// homeserver.
    ///
    /// To create an instance of this type, first create a [`ProtocolInstanceInit`] and convert it
    /// via `AppserviceProtocolInstance::from` / `.into()`.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct AppserviceProtocolInstance {
        /// A human-readable description for the protocol, such as the name.
        pub desc: String,

        /// An optional content URI representing the protocol.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub icon: Option<String>,

        /// Preset values for `fields` the client may use to search by.
        pub fields: BTreeMap<String, String>,

        /// A unique identifier across all instances.
        pub network_id: String,
    }

    impl From<ProtocolInstanceInit> for AppserviceProtocolInstance {
        fn from(init: ProtocolInstanceInit) -> Self {
            let ProtocolInstanceInit { desc, fields, network_id } = init;
            Self { desc, icon: None, fields, network_id }
        }
    }

    impl From<AppserviceProtocolInstance> for ProtocolInstance {
        fn from(value: AppserviceProtocolInstance) -> Self {
            let AppserviceProtocolInstance { desc, icon, fields, network_id } = value;
            let mut instance =
                ProtocolInstance::from(ProtocolInstanceInit { desc, fields, network_id });
            instance.icon = icon;
            instance
        }
    }
}
