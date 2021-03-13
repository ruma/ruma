use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::LoginType;

pub fn serialize<S>(login_types: &[LoginType], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(Serialize)]
    struct Wrap<'a> {
        #[serde(rename = "type")]
        inner: &'a LoginType,
    }

    serializer.collect_seq(login_types.iter().map(|ty| Wrap { inner: ty }))
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<LoginType>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrap {
        #[serde(rename = "type")]
        inner: LoginType,
    }

    // Could be optimized by using a visitor, but that's a bunch of extra code
    let vec = Vec::<Wrap>::deserialize(deserializer)?;
    Ok(vec.into_iter().map(|w| w.inner).collect())
}
