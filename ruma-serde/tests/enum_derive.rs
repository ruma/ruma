use ruma_serde::StringEnum;
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[derive(Debug, PartialEq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
enum MyEnum {
    First,
    Second,
    #[ruma_enum(rename = "m.third")]
    Third,
    HelloWorld,
    _Custom(String),
}

#[test]
fn as_ref_str() {
    assert_eq!(MyEnum::First.as_ref(), "first");
    assert_eq!(MyEnum::Second.as_ref(), "second");
    assert_eq!(MyEnum::Third.as_ref(), "m.third");
    assert_eq!(MyEnum::HelloWorld.as_ref(), "hello_world");
    assert_eq!(MyEnum::_Custom("HelloWorld".into()).as_ref(), "HelloWorld");
}

#[test]
fn display() {
    assert_eq!(MyEnum::First.to_string(), "first");
    assert_eq!(MyEnum::Second.to_string(), "second");
    assert_eq!(MyEnum::Third.to_string(), "m.third");
    assert_eq!(MyEnum::HelloWorld.to_string(), "hello_world");
    assert_eq!(MyEnum::_Custom("HelloWorld".into()).to_string(), "HelloWorld");
}

#[test]
fn from_string() {
    assert_eq!(MyEnum::from("first"), MyEnum::First);
    assert_eq!(MyEnum::from("second"), MyEnum::Second);
    assert_eq!(MyEnum::from("m.third"), MyEnum::Third);
    assert_eq!(MyEnum::from("hello_world"), MyEnum::HelloWorld);
    assert_eq!(MyEnum::from("HelloWorld"), MyEnum::_Custom("HelloWorld".into()));
}

#[test]
fn serialize() {
    assert_eq!(to_json_value(MyEnum::First).unwrap(), json!("first"));
    assert_eq!(to_json_value(MyEnum::HelloWorld).unwrap(), json!("hello_world"));
    assert_eq!(to_json_value(MyEnum::_Custom("\\\n\\".into())).unwrap(), json!("\\\n\\"));
}

#[test]
fn deserialize() {
    assert_eq!(from_json_value::<MyEnum>(json!("first")).unwrap(), MyEnum::First);
    assert_eq!(from_json_value::<MyEnum>(json!("hello_world")).unwrap(), MyEnum::HelloWorld);
    assert_eq!(
        from_json_value::<MyEnum>(json!("\\\n\\")).unwrap(),
        MyEnum::_Custom("\\\n\\".into())
    );
}
