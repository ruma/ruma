//! `POST /_matrix/client/*/safety/report`
//!
//! Report content.

pub mod unstable {
    //! MSC4457 ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4457

    use ruma_common::{
        OwnedEventId, OwnedMxcUri, OwnedRoomOrAliasId, OwnedServerName, OwnedTransactionId,
        OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        harm::Harm,
        metadata,
    };
    use serde::{Deserialize, Serialize};

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc4457/safety/report/{transaction_id}",
        }
    }

    /// Request type for the `report` endpoint.
    #[request]
    pub struct Request {
        /// The transaction ID supplied by the client.
        #[ruma_api(path)]
        pub transaction_id: OwnedTransactionId,

        /// The report data.
        #[ruma_api(body)]
        pub report: Report,
    }

    /// Response type for the `report` endpoint.
    #[response]
    pub struct Response {
        /// The opaque ID of the report in the homeserver's system.
        pub report_id: String,
    }

    /// A report.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    #[non_exhaustive]
    pub enum Report {
        /// A complaint about inappropriate behavior.
        Complaint(ComplaintReport),
    }

    /// A complaint about inappropriate behavior.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[non_exhaustive]
    pub struct ComplaintReport {
        /// The target of this report.
        pub regarding: ReportTarget,
        /// The harm which the reporter claims is being caused by the target.
        pub harm: Harm,
        /// The reporter's description of the report.
        pub description: String,
    }

    /// The target of a report.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    #[non_exhaustive]
    pub enum ReportTarget {
        /// A user.
        User(OwnedUserId),
        /// An event.
        Event(OwnedEventId),
        /// A room.
        Room(OwnedRoomOrAliasId),
        /// A homeserver.
        #[serde(with = "serde_server_target")]
        Server(OwnedServerName),
        /// A media file.
        Media(OwnedMxcUri),
        /// The reporting system itself.
        #[serde(rename = "m.system")]
        ReportSystem,

        #[doc(hidden)]
        _Custom(String),
    }

    mod serde_server_target {
        use ruma_common::{OwnedServerName, ServerName};
        use serde::{Deserialize, Deserializer, Serializer, de::Error};

        pub(super) fn deserialize<'de, D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<OwnedServerName, D::Error> {
            let string = <&str>::deserialize(deserializer)?;

            if let Some(server_name) = string.strip_prefix("server:") {
                ServerName::parse(server_name)
                    .map_err(|_| Error::custom("failed to parse server name"))
            } else {
                Err(Error::custom("not a server"))
            }
        }

        pub(super) fn serialize<S: Serializer>(
            server_name: &OwnedServerName,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&format!("server:{server_name}"))
        }
    }
}
