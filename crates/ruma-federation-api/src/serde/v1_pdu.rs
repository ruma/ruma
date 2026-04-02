//! A module to deserialize a response from incorrectly specified endpoint:
//!
//! - [PUT /_matrix/federation/v1/send_join/{roomId}/{eventId}](https://spec.matrix.org/v1.17/server-server-api/#put_matrixfederationv1send_joinroomideventid)
//! - [PUT /_matrix/federation/v1/invite/{roomId}/{eventId}](https://spec.matrix.org/v1.18/server-server-api/#put_matrixfederationv1inviteroomideventid)
//! - [PUT /_matrix/federation/v1/send_leave/{roomId}/{eventId}](https://spec.matrix.org/v1.17/server-server-api/#put_matrixfederationv1send_leaveroomideventid)
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
    use serde_json::{Value as JsonValue, json};

    use super::deserialize;

    #[test]
    fn deserialize_response() {
        let content = json!({
            "auth_chain": [],
            "state": []
        });
        let json = json!([200, content]);

        let deserialized = deserialize::<JsonValue, _>(json).unwrap();
        assert_eq!(deserialized, content);
    }

    #[test]
    fn too_short_array() {
        let json = json!([200]);
        #[allow(deprecated)]
        let failed_room_state = deserialize::<JsonValue, _>(json);
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
        let failed_room_state = deserialize::<JsonValue, _>(json);

        assert_eq!(
            failed_room_state.unwrap_err().to_string(),
            "invalid type: map, expected a PDU wrapped in an array.",
        );
    }

    #[test]
    fn too_long_array() {
        let content = json!({
            "auth_chain": [],
            "state": []
        });
        let json = json!([200, content, 200]);

        let deserialized = deserialize::<JsonValue, _>(json).unwrap();
        assert_eq!(deserialized, content);
    }
}

#[cfg(all(test, feature = "server"))]
mod tests_server {
    use serde_json::json;

    use super::serialize;

    #[test]
    fn serialize_response() {
        let content = json!({
            "auth_chain": [],
            "state": []
        });

        let serialized = serialize(&content, serde_json::value::Serializer).unwrap();
        let expected = json!([200, content]);

        assert_eq!(serialized, expected);
    }
}
