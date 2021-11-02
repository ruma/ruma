use serde::{Deserialize, Serialize};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct StringStruct {
    #[serde(
        default,
        deserialize_with = "ruma_serde::empty_string_as_none",
        serialize_with = "ruma_serde::none_as_empty_string"
    )]
    x: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct IntStruct {
    #[serde(
        default,
        deserialize_with = "ruma_serde::empty_string_as_none",
        serialize_with = "ruma_serde::none_as_empty_string"
    )]
    x: Option<i32>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct NoneStruct {
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<String>,
}

#[test]
fn empty_se() {
    let string = StringStruct { x: None };
    let none = NoneStruct { x: None };
    assert_eq!(to_json_value(string).unwrap(), json!({"x": ""}));
    assert_eq!(to_json_value(none).unwrap(), json!({}));
}

#[test]
fn empty_de_string_none() {
    let string = StringStruct { x: None };
    let none = NoneStruct { x: None };
    assert_eq!(from_json_value::<StringStruct>(json!({"x": ""})).unwrap(), string);
    assert_eq!(from_json_value::<StringStruct>(json!({})).unwrap(), string);
    assert_eq!(from_json_value::<NoneStruct>(json!({})).unwrap(), none);
}

#[test]
fn empty_de_string_some() {
    let string = StringStruct { x: Some("foo".into()) };
    assert_eq!(from_json_value::<StringStruct>(json!({"x": "foo"})).unwrap(), string);
}

#[test]
fn empty_de_int_none() {
    let expected = IntStruct { x: None };
    assert_eq!(from_json_value::<IntStruct>(json!({"x": ""})).unwrap(), expected);
    assert_eq!(from_json_value::<IntStruct>(json!({})).unwrap(), expected);
}

#[test]
fn empty_de_int_some() {
    let expected = IntStruct { x: Some(1) };
    assert_eq!(from_json_value::<IntStruct>(json!({"x": 1})).unwrap(), expected);
}
