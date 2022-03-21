//! Helper module for the Serialize / Deserialize impl's for the User struct
//! in the parent module.

use ruma_common::{serde::Incoming, thirdparty::Medium};
use serde::Serialize;

// The following structs could just be used in place of the one in the parent module, but
// that one is arguably much easier to deal with.
#[derive(Clone, Debug, PartialEq, Eq, Incoming, Serialize)]
#[serde(tag = "type")]
pub(crate) enum UserIdentifier<'a> {
    #[serde(rename = "m.id.user")]
    UserIdOrLocalpart { user: &'a str },
    #[serde(rename = "m.id.thirdparty")]
    ThirdPartyId { medium: Medium, address: &'a str },
    #[serde(rename = "m.id.phone")]
    PhoneNumber { country: &'a str, phone: &'a str },
}

impl<'a> From<super::UserIdentifier<'a>> for UserIdentifier<'a> {
    fn from(id: super::UserIdentifier<'a>) -> Self {
        use UserIdentifier as SerdeId;

        use super::UserIdentifier as SuperId;

        match id {
            SuperId::UserIdOrLocalpart(user) => SerdeId::UserIdOrLocalpart { user },
            SuperId::ThirdPartyId { address, medium } => SerdeId::ThirdPartyId { address, medium },
            SuperId::PhoneNumber { country, phone } => SerdeId::PhoneNumber { country, phone },
        }
    }
}

impl From<IncomingUserIdentifier> for super::IncomingUserIdentifier {
    fn from(id: IncomingUserIdentifier) -> super::IncomingUserIdentifier {
        use IncomingUserIdentifier as SerdeId;

        use super::IncomingUserIdentifier as SuperId;

        match id {
            SerdeId::UserIdOrLocalpart { user } => SuperId::UserIdOrLocalpart(user),
            SerdeId::ThirdPartyId { address, medium } => SuperId::ThirdPartyId { address, medium },
            SerdeId::PhoneNumber { country, phone } => SuperId::PhoneNumber { country, phone },
        }
    }
}
