//! A module to deserialize a response from incorrectly specified endpoint:
//!
//! - [PUT /_matrix/federation/v1/send_join/{roomId}/{eventId}](https://spec.matrix.org/v1.2/server-server-api/#put_matrixfederationv1send_joinroomideventid)
//! - [PUT /_matrix/federation/v1/invite/{roomId}/{eventId}](https://spec.matrix.org/v1.2/server-server-api/#put_matrixfederationv1inviteroomideventid)
//! - [PUT /_matrix/federation/v1/send_leave/{roomId}/{eventId}](https://spec.matrix.org/v1.2/server-server-api/#put_matrixfederationv1send_leaveroomideventid)
//!
//! For more information, see this [GitHub issue][issue].
//!
//! [issue]: https://github.com/matrix-org/matrix-spec-proposals/issues/2541

use std::{fmt, marker::PhantomData};

use serde::{
    de::{Deserialize, Deserializer, Error, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};

pub fn serialize<T, S>(val: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    let mut seq = serializer.serialize_seq(Some(2))?;
    seq.serialize_element(&200)?;
    seq.serialize_element(val)?;
    seq.end()
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    deserializer.deserialize_seq(PduVisitor { phantom: PhantomData })
}

struct PduVisitor<T> {
    phantom: PhantomData<T>,
}

impl<'de, T> Visitor<'de> for PduVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a PDU wrapped in an array.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let expected = "a two-element list in the response";
        if seq.next_element::<IgnoredAny>()?.is_none() {
            return Err(A::Error::invalid_length(0, &expected));
        }

        let val = seq.next_element()?.ok_or_else(|| A::Error::invalid_length(1, &expected))?;

        while let Some(IgnoredAny) = seq.next_element()? {
            // ignore extra elements
        }

        Ok(val)
    }
}

#[cfg(not(feature = "unstable-pre-spec"))]
#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use serde_json::{json, to_value as to_json_value};

    use super::{deserialize, serialize};
    use crate::membership::create_join_event::RoomState;

    #[test]
    fn deserialize_response() {
        let response = json!([
            200,
            {
                "origin": "example.com",
                "auth_chain": [],
                "state": []
            }
        ]);

        let RoomState { origin, auth_chain, state } = deserialize(response).unwrap();
        assert_eq!(origin, "example.com");
        assert_matches!(auth_chain.as_slice(), []);
        assert_matches!(state.as_slice(), []);
    }

    #[test]
    fn serialize_response() {
        let room_state =
            RoomState { origin: "matrix.org".into(), auth_chain: Vec::new(), state: Vec::new() };

        let serialized = serialize(&room_state, serde_json::value::Serializer).unwrap();
        let expected = to_json_value(&json!(
            [
                200,
                {
                    "origin": "matrix.org",
                    "auth_chain": [],
                    "state": []
                }
            ]
        ))
        .unwrap();

        assert_eq!(serialized, expected);
    }

    #[test]
    fn too_short_array() {
        let json = json!([200]);
        let failed_room_state = deserialize::<RoomState, _>(json);
        assert_eq!(
            failed_room_state.unwrap_err().to_string(),
            "invalid length 1, expected a two-element list in the response"
        );
    }

    #[test]
    fn not_an_array() {
        let json = json!({
            "origin": "matrix.org",
            "auth_chain": [],
            "state": []
        });
        let failed_room_state = deserialize::<RoomState, _>(json);

        assert_eq!(
            failed_room_state.unwrap_err().to_string(),
            "invalid type: map, expected a PDU wrapped in an array.",
        )
    }

    #[test]
    fn too_long_array() {
        let json = json!([200, { "origin": "", "auth_chain": [], "state": [] }, 200]);
        let RoomState { origin, auth_chain, state } = deserialize(json).unwrap();
        assert_eq!(origin, "");
        assert_matches!(auth_chain.as_slice(), []);
        assert_matches!(state.as_slice(), []);
    }
}
