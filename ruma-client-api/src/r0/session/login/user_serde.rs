//! Helper module for the Serialize / Deserialize impl's for the User struct
//! in the parent module.

use serde::{Deserialize, Serialize};

use super::Medium;

// The following three structs could just be used in place of the one in the parent module, but
// that one is arguably much easier to deal with.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct UserInfo {
    pub identifier: UserIdentifier,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub(crate) enum UserIdentifier {
    #[serde(rename = "m.id.user")]
    MatrixId { user: String },
    #[serde(rename = "m.id.thirdparty")]
    ThirdPartyId { medium: Medium, address: String },
    #[serde(rename = "m.id.phone")]
    PhoneNumber { country: String, phone: String },
}

impl From<super::UserInfo> for UserInfo {
    fn from(info: super::UserInfo) -> Self {
        use super::UserInfo::*;

        match info {
            MatrixId(user) => UserInfo {
                identifier: UserIdentifier::MatrixId { user },
            },
            ThirdPartyId { address, medium } => UserInfo {
                identifier: UserIdentifier::ThirdPartyId { address, medium },
            },
            PhoneNumber { country, phone } => UserInfo {
                identifier: UserIdentifier::PhoneNumber { country, phone },
            },
        }
    }
}

impl From<UserInfo> for super::UserInfo {
    fn from(info: UserInfo) -> super::UserInfo {
        use super::UserInfo::*;

        match info.identifier {
            UserIdentifier::MatrixId { user } => MatrixId(user),
            UserIdentifier::ThirdPartyId { address, medium } => ThirdPartyId { address, medium },
            UserIdentifier::PhoneNumber { country, phone } => PhoneNumber { country, phone },
        }
    }
}
