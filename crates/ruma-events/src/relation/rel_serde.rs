use ruma_common::serde::Raw;
use serde::{de::DeserializeOwned, Deserialize, Deserializer};

use super::{BundledMessageLikeRelations, BundledThread, ReferenceChunk};

#[derive(Deserialize)]
struct BundledMessageLikeRelationsJsonRepr<E> {
    #[serde(rename = "m.replace")]
    replace: Option<Raw<Box<E>>>,
    #[serde(rename = "m.thread")]
    thread: Option<Box<BundledThread>>,
    #[serde(rename = "m.reference")]
    reference: Option<Box<ReferenceChunk>>,
}

impl<'de, E> Deserialize<'de> for BundledMessageLikeRelations<E>
where
    E: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let BundledMessageLikeRelationsJsonRepr { replace, thread, reference } =
            BundledMessageLikeRelationsJsonRepr::deserialize(deserializer)?;

        let (replace, has_invalid_replacement) =
            match replace.as_ref().map(Raw::deserialize).transpose() {
                Ok(replace) => (replace, false),
                Err(_) => (None, true),
            };

        Ok(BundledMessageLikeRelations { replace, has_invalid_replacement, thread, reference })
    }
}
