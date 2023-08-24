use ruma_common::serde::StringEnum;
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

/// The payload for a server notice message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.server_notice")]
pub struct ServerNoticeMessageEventContent {
    /// A human-readable description of the notice.
    pub body: String,

    /// The type of notice being represented.
    pub server_notice_type: ServerNoticeType,

    /// A URI giving a contact method for the server administrator.
    ///
    /// Required if the notice type is `m.server_notice.usage_limit_reached`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_contact: Option<String>,

    /// The kind of usage limit the server has exceeded.
    ///
    /// Required if the notice type is `m.server_notice.usage_limit_reached`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_type: Option<LimitType>,
}

impl ServerNoticeMessageEventContent {
    /// Creates a new `ServerNoticeMessageEventContent` with the given body and notice type.
    pub fn new(body: String, server_notice_type: ServerNoticeType) -> Self {
        Self { body, server_notice_type, admin_contact: None, limit_type: None }
    }
}

/// Types of server notices.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
pub enum ServerNoticeType {
    /// The server has exceeded some limit which requires the server administrator to intervene.
    #[ruma_enum(rename = "m.server_notice.usage_limit_reached")]
    UsageLimitReached,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// Types of usage limits.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum LimitType {
    /// The server's number of active users in the last 30 days has exceeded the maximum.
    ///
    /// New connections are being refused by the server. What defines "active" is left as an
    /// implementation detail, however servers are encouraged to treat syncing users as "active".
    MonthlyActiveUser,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
