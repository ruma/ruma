//! A module to deserialize a response from incorrectly specified endpoint:
//!
//! - [PUT /_matrix/federation/v1/send_join/{roomId}/{eventId}](https://spec.matrix.org/latest/server-server-api/#put_matrixfederationv1send_joinroomideventid)
//! - [PUT /_matrix/federation/v1/invite/{roomId}/{eventId}](https://spec.matrix.org/latest/server-server-api/#put_matrixfederationv1inviteroomideventid)
//! - [PUT /_matrix/federation/v1/send_leave/{roomId}/{eventId}](https://spec.matrix.org/latest/server-server-api/#put_matrixfederationv1send_leaveroomideventid)
//!
//! For more information, see this [GitHub issue][issue].
//!
//! [issue]: https://github.com/matrix-org/matrix-spec-proposals/issues/2541

#[cfg(feature = "client")]
use std::{fmt, marker::PhantomData};

#[cfg(feature = "client")]
use serde::de::{Deserialize, Deserializer, Error, IgnoredAny, SeqAccess, Visitor};
#[cfg(feature = "server")]
use serde::ser::{Serialize, SerializeSeq, Serializer};

#[cfg(feature = "server")]
pub(crate) fn serialize<T, S>(val: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    let mut seq = serializer.serialize_seq(Some(2))?;
    seq.serialize_element(&200)?;
    seq.serialize_element(val)?;
    seq.end()
}

#[cfg(feature = "client")]
pub(crate) fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    deserializer.deserialize_seq(PduVisitor { phantom: PhantomData })
}

#[cfg(feature = "client")]
struct PduVisitor<T> {
    phantom: PhantomData<T>,
}

#[cfg(feature = "client")]
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

#[cfg(all(test, feature = "client"))]
mod tests_client {
    use assert_matches2::assert_matches;
    use serde_json::json;

    use super::deserialize;
    #[allow(deprecated)]
    use crate::membership::create_join_event::v1::RoomState;

    #[test]
    fn deserialize_response() {
        let response = json!([
            200,
            {
                "auth_chain": [],
                "state": []
            }
        ]);

        #[allow(deprecated)]
        let RoomState { auth_chain, state, event } = deserialize(response).unwrap();
        assert_matches!(auth_chain.as_slice(), []);
        assert_matches!(state.as_slice(), []);
        assert_matches!(event, None);
    }

    #[test]
    fn too_short_array() {
        let json = json!([200]);
        #[allow(deprecated)]
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
        #[allow(deprecated)]
        let failed_room_state = deserialize::<RoomState, _>(json);

        assert_eq!(
            failed_room_state.unwrap_err().to_string(),
            "invalid type: map, expected a PDU wrapped in an array.",
        );
    }

    #[test]
    fn too_long_array() {
        let json = json!([200, { "auth_chain": [], "state": [] }, 200]);
        #[allow(deprecated)]
        let RoomState { auth_chain, state, event } = deserialize(json).unwrap();
        assert_matches!(auth_chain.as_slice(), []);
        assert_matches!(state.as_slice(), []);
        assert_matches!(event, None);
    }
}

#[cfg(all(test, feature = "server"))]
mod tests_server {
    use serde_json::json;

    use super::serialize;
    #[allow(deprecated)]
    use crate::membership::create_join_event::v1::RoomState;

    #[test]
    fn serialize_response() {
        #[allow(deprecated)]
        let room_state = RoomState { auth_chain: Vec::new(), state: Vec::new(), event: None };

        let serialized = serialize(&room_state, serde_json::value::Serializer).unwrap();
        let expected = json!(
            [
                200,
                {
                    "auth_chain": [],
                    "state": []
                }
            ]
        );

        assert_eq!(serialized, expected);
    }
}
