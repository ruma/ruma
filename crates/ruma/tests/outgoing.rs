// This test should really be part of ruma_serde, but some tooling doesn't like
// cyclic dev-dependencies, which are required for this test to be moved there.

#![allow(clippy::exhaustive_structs, clippy::redundant_allocation)]

use ruma::{Outgoing, UserId};

#[allow(unused)]
pub struct Thing<'t, T> {
    some: &'t str,
    t: &'t T,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct IncomingThing<T> {
    some: String,
    t: T,
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, Outgoing, serde::Serialize)]
#[serde(crate = "serde")]
pub struct OtherThing<'t> {
    pub some: &'t str,
    pub t: &'t [u8],
}

#[derive(Outgoing)]
#[incoming_derive(!Deserialize)]
#[non_exhaustive]
pub struct FakeRequest<'a, T> {
    pub abc: &'a str,
    pub thing: Thing<'a, T>,
    pub device_id: &'a ::ruma_common::DeviceId,
    pub user_id: &'a UserId,
    pub bytes: &'a [u8],
    pub recursive: &'a [Thing<'a, T>],
    pub option: Option<&'a [u8]>,
    pub depth: Option<&'a [(&'a str, &'a str)]>,
    pub arc_type: std::sync::Arc<&'a ::ruma_common::ServerName>,
    pub thing_ref: &'a Thing<'a, T>,
    pub double_ref: &'a &'a u8,
    pub triple_ref: &'a &'a &'a str,
}

#[derive(Outgoing)]
#[incoming_derive(!Deserialize)]
#[non_exhaustive]
pub enum EnumThing<'a, T> {
    Abc(&'a str),
    Stuff(Thing<'a, T>),
    Boxy(&'a ::ruma_common::DeviceId),
    Other(Option<&'a str>),
    StructVar { stuff: &'a str, more: &'a ::ruma_common::ServerName },
}
