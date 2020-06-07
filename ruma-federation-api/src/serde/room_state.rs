//! A module to deserialize a RoomState struct from incorrectly specified v1
//! send_join endpoint.
//!
//! For more information, see this [GitHub issue](https://github.com/matrix-org/matrix-doc/issues/2541).

use std::fmt;

use serde::{
    de::{Deserializer, Error, IgnoredAny, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
};

use crate::membership::create_join_event::RoomState;

pub fn serialize<S>(room_state: &RoomState, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(2))?;
    seq.serialize_element(&200)?;
    seq.serialize_element(room_state)?;
    seq.end()
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<RoomState, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(RoomStateVisitor)
}

struct RoomStateVisitor;

impl<'de> Visitor<'de> for RoomStateVisitor {
    type Value = RoomState;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Room State response wrapped in an array.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let expected = "a two-element list in the response";
        if seq.next_element::<IgnoredAny>()?.is_none() {
            return Err(A::Error::invalid_length(0, &expected));
        }

        let room_state = seq
            .next_element()?
            .ok_or_else(|| A::Error::invalid_length(1, &expected))?;

        while let Some(IgnoredAny) = seq.next_element()? {
            // ignore extra elements
        }

        Ok(room_state)
    }
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use serde_json::{json, to_value as to_json_value};

    use super::{deserialize, serialize, RoomState};

    #[test]
    fn test_deserialize_response() {
        let response = json!([
            200,
            {
                "origin": "example.com",
                "auth_chain": [],
                "state": []
            }
        ]);

        let parsed = deserialize(response).unwrap();

        assert_matches!(
            parsed,
            RoomState { origin, auth_chain, state }
            if origin == "example.com"
                && auth_chain.is_empty()
                && state.is_empty()
        );
    }

    #[test]
    fn test_serialize_response() {
        let room_state = RoomState {
            origin: "matrix.org".to_string(),
            auth_chain: Vec::new(),
            state: Vec::new(),
        };

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
    fn test_too_short_array() {
        let json = json!([200]);
        let failed_room_state = deserialize(json);
        assert_eq!(
            failed_room_state.unwrap_err().to_string(),
            "invalid length 1, expected a two-element list in the response"
        );
    }

    #[test]
    fn test_not_an_array() {
        let json = json!({
            "origin": "matrix.org",
            "auth_chain": [],
            "state": []
        });
        let failed_room_state = deserialize(json);

        assert_eq!(
            failed_room_state.unwrap_err().to_string(),
            "invalid type: map, expected Room State response wrapped in an array.",
        )
    }

    #[test]
    fn test_too_long_array() {
        let json = json!([200, {"origin": "", "auth_chain": [], "state": []}, 200]);
        assert_matches!(
            deserialize(json).unwrap(),
            RoomState { origin, auth_chain, state }
            if origin == ""
              && auth_chain.is_empty()
              && state.is_empty()
        );
    }
}
