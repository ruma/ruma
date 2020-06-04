//! Helpers for tests

use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};

pub fn serde_json_eq<T>(de: T, se: serde_json::Value)
where
    T: Clone + Debug + PartialEq + Serialize + DeserializeOwned,
{
    assert_eq!(se, serde_json::to_value(de.clone()).unwrap());
    assert_eq!(de, serde_json::from_value(se).unwrap());
}
