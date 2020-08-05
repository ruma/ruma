use ruma_api::Outgoing;
use ruma_identifiers::UserId;

#[allow(unused)]
pub struct Thing<'t, T> {
    some: &'t str,
    t: &'t T,
}

#[derive(Debug)]
pub struct IncomingThing<T> {
    some: String,
    t: T,
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct OtherThing<'t> {
    some: &'t str,
    t: &'t [u8],
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct IncomingOtherThing {
    some: String,
    t: Vec<u8>,
}

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Does something.",
        method: GET,
        name: "no_fields",
        path: "/_matrix/my/endpoint/:thing",
        rate_limited: false,
        requires_authentication: false,
    }

    request: {
        #[ruma_api(query)]
        pub abc: &'a str,
        #[ruma_api(path)]
        pub thing: &'a str,
        #[ruma_api(header = CONTENT_TYPE)]
        pub stuff: &'a str,
    }

    response: {
        pub body: &'a str,
        pub thing: OtherThing<'a>,
        pub stuff: &'a [u8],
    }
}

#[derive(Outgoing)]
#[incoming_no_deserialize]
pub struct FakeRequest<'a, T> {
    pub abc: &'a str,
    pub thing: Thing<'a, T>,
    pub device_id: &'a ::ruma_identifiers::DeviceId,
    pub user_id: &'a UserId,
    pub bytes: &'a [u8],
    pub recursive: &'a [Thing<'a, T>],
    pub option: Option<&'a [u8]>,
}

#[derive(Outgoing)]
#[incoming_no_deserialize]
pub enum EnumThing<'a, T> {
    Abc(&'a str),
    Stuff(Thing<'a, T>),
    Boxy(&'a ::ruma_identifiers::DeviceId),
    Other(Option<&'a str>),
    StructVar { stuff: &'a str, more: &'a ::ruma_identifiers::ServerName },
}
