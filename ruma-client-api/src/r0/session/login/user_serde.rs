//! Helper module for the Serialize / Deserialize impl's for the User struct
//! in the parent module.

use ruma_common::thirdparty::Medium;
use ruma_serde::Outgoing;
use serde::Serialize;

// The following three structs could just be used in place of the one in the parent module, but
// that one is arguably much easier to deal with.
#[derive(Clone, Debug, PartialEq, Eq, Outgoing, Serialize)]
pub(crate) struct UserInfo<'a> {
    pub identifier: UserIdentifier<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq, Outgoing, Serialize)]
#[serde(tag = "type")]
pub(crate) enum UserIdentifier<'a> {
    #[serde(rename = "m.id.user")]
    MatrixId { user: &'a str },
    #[serde(rename = "m.id.thirdparty")]
    ThirdPartyId { medium: Medium, address: &'a str },
    #[serde(rename = "m.id.phone")]
    PhoneNumber { country: &'a str, phone: &'a str },
}

impl<'a> From<super::UserInfo<'a>> for UserInfo<'a> {
    fn from(info: super::UserInfo<'a>) -> Self {
        use super::UserInfo as Info;
        use UserIdentifier as Id;

        match info {
            Info::MatrixId(user) => UserInfo { identifier: Id::MatrixId { user } },
            Info::ThirdPartyId { address, medium } => {
                UserInfo { identifier: Id::ThirdPartyId { address, medium } }
            }
            Info::PhoneNumber { country, phone } => {
                UserInfo { identifier: Id::PhoneNumber { country, phone } }
            }
        }
    }
}

impl From<IncomingUserInfo> for super::IncomingUserInfo {
    fn from(info: IncomingUserInfo) -> super::IncomingUserInfo {
        use super::IncomingUserInfo as Info;
        use IncomingUserIdentifier as Id;

        match info.identifier {
            Id::MatrixId { user } => Info::MatrixId(user),
            Id::ThirdPartyId { address, medium } => Info::ThirdPartyId { address, medium },
            Id::PhoneNumber { country, phone } => Info::PhoneNumber { country, phone },
        }
    }
}
