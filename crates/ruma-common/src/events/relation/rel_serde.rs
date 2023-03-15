use serde::{de::DeserializeOwned, Deserialize, Deserializer};

use super::{BundledRelations, BundledThread, ReferenceChunk};
use crate::serde::Raw;

#[derive(Deserialize)]
struct BundledRelationsJsonRepr<E> {
    #[serde(rename = "m.replace")]
    replace: Option<Raw<Box<E>>>,
    #[serde(rename = "m.thread")]
    thread: Option<Box<BundledThread>>,
    #[serde(rename = "m.reference")]
    reference: Option<Box<ReferenceChunk>>,
}

impl<'de, E> Deserialize<'de> for BundledRelations<E>
where
    E: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let BundledRelationsJsonRepr { replace, thread, reference } =
            BundledRelationsJsonRepr::deserialize(deserializer)?;

        if let Ok(replace) = replace.as_ref().map(Raw::deserialize).transpose() {
            Ok(BundledRelations { replace, has_invalid_replacement: false, thread, reference })
        } else {
            Ok(BundledRelations { replace: None, has_invalid_replacement: true, thread, reference })
        }
    }
}
