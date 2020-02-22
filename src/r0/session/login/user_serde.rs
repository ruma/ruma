//! Helper module for the Serialize / Deserialize impl's for the User struct
//! in the parent module.

use serde::{Deserialize, Serialize};

use super::Medium;

// The following three structs could just be used in place of the one in the parent module, but
// that one is arguably much easier to deal with.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct UserInfo<'a> {
    #[serde(borrow)]
    pub identifier: UserIdentifier<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub(crate) enum UserIdentifier<'a> {
    #[serde(rename = "m.id.user")]
    MatrixId { user: &'a str },
    #[serde(rename = "m.id.thirdparty")]
    ThirdPartyId { medium: Medium, address: &'a str },
    #[serde(rename = "m.id.phone")]
    PhoneNumber { country: &'a str, phone: &'a str },
}

impl<'a> From<&'a super::UserInfo> for UserInfo<'a> {
    fn from(su: &'a super::UserInfo) -> Self {
        use super::UserInfo::*;

        match su {
            MatrixId(user) => UserInfo {
                identifier: UserIdentifier::MatrixId { user },
            },
            ThirdPartyId { address, medium } => UserInfo {
                identifier: UserIdentifier::ThirdPartyId {
                    address,
                    medium: *medium,
                },
            },
            PhoneNumber { country, phone } => UserInfo {
                identifier: UserIdentifier::PhoneNumber { country, phone },
            },
        }
    }
}

impl Into<super::UserInfo> for UserInfo<'_> {
    fn into(self) -> super::UserInfo {
        use super::UserInfo::*;

        match self.identifier {
            UserIdentifier::MatrixId { user } => MatrixId(user.to_owned()),
            UserIdentifier::ThirdPartyId { address, medium } => ThirdPartyId {
                address: address.to_owned(),
                medium: medium.to_owned(),
            },
            UserIdentifier::PhoneNumber { country, phone } => PhoneNumber {
                country: country.to_owned(),
                phone: phone.to_owned(),
            },
        }
    }
}
