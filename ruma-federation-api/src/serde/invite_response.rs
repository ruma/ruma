//! Deserialization for `InviteResponse` from incorrectly specified `create_invite` endpoint.
//!
//! See [this GitHub issue][issue] for more information.
//!
//! [issue]: https://github.com/matrix-org/matrix-doc/issues/2541

use std::fmt;

use serde::{
    de::{Deserializer, Error, IgnoredAny, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
};

use crate::membership::create_invite::v1::InviteResponse;

pub fn serialize<S>(invite_response: &InviteResponse, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(2))?;
    seq.serialize_element(&200)?;
    seq.serialize_element(invite_response)?;
    seq.end()
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<InviteResponse, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(InviteResponseVisitor)
}

struct InviteResponseVisitor;

impl<'de> Visitor<'de> for InviteResponseVisitor {
    type Value = InviteResponse;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Invite response wrapped in an array.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let expected = "a two-element list in the response";
        // Ignore first list element (200 http status code).
        if seq.next_element::<IgnoredAny>()?.is_none() {
            return Err(A::Error::invalid_length(0, &expected));
        }

        let invite_response =
            seq.next_element()?.ok_or_else(|| A::Error::invalid_length(1, &expected))?;

        // Ignore extra elements.
        while let Some(IgnoredAny) = seq.next_element()? {}

        Ok(invite_response)
    }
}
