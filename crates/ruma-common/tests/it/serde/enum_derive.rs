use ruma_common::serde::StringEnum;
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[derive(Debug, PartialEq)]
struct PrivOwnedStr(Box<str>);

#[derive(PartialEq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
enum MyEnum {
    First,
    Second,
    #[ruma_enum(rename = "m.third")]
    Third,
    HelloWorld,
    #[ruma_enum(rename = "io.ruma.unstable", alias = "m.stable", alias = "hs.notareal.unstable")]
    Stable,
    _Custom(PrivOwnedStr),
}

#[test]
fn as_ref_str() {
    assert_eq!(MyEnum::First.as_ref(), "first");
    assert_eq!(MyEnum::Second.as_ref(), "second");
    assert_eq!(MyEnum::Third.as_ref(), "m.third");
    assert_eq!(MyEnum::HelloWorld.as_ref(), "hello_world");
    assert_eq!(MyEnum::Stable.as_ref(), "io.ruma.unstable");
    assert_eq!(MyEnum::_Custom(PrivOwnedStr("HelloWorld".into())).as_ref(), "HelloWorld");
}

#[test]
fn display() {
    assert_eq!(MyEnum::First.to_string(), "first");
    assert_eq!(MyEnum::Second.to_string(), "second");
    assert_eq!(MyEnum::Third.to_string(), "m.third");
    assert_eq!(MyEnum::HelloWorld.to_string(), "hello_world");
    assert_eq!(MyEnum::Stable.to_string(), "io.ruma.unstable");
    assert_eq!(MyEnum::_Custom(PrivOwnedStr("HelloWorld".into())).to_string(), "HelloWorld");
}

#[test]
fn debug() {
    assert_eq!(format!("{:?}", MyEnum::First), "\"first\"");
    assert_eq!(format!("{:?}", MyEnum::Second), "\"second\"");
    assert_eq!(format!("{:?}", MyEnum::Third), "\"m.third\"");
    assert_eq!(format!("{:?}", MyEnum::HelloWorld), "\"hello_world\"");
    assert_eq!(format!("{:?}", MyEnum::Stable), "\"io.ruma.unstable\"");
    assert_eq!(
        format!("{:?}", MyEnum::_Custom(PrivOwnedStr("HelloWorld".into()))),
        "\"HelloWorld\""
    );
}

#[test]
fn from_string() {
    assert_eq!(MyEnum::from("first"), MyEnum::First);
    assert_eq!(MyEnum::from("second"), MyEnum::Second);
    assert_eq!(MyEnum::from("m.third"), MyEnum::Third);
    assert_eq!(MyEnum::from("hello_world"), MyEnum::HelloWorld);
    assert_eq!(MyEnum::from("io.ruma.unstable"), MyEnum::Stable);
    assert_eq!(MyEnum::from("m.stable"), MyEnum::Stable);
    assert_eq!(MyEnum::from("hs.notareal.unstable"), MyEnum::Stable);
    assert_eq!(MyEnum::from("HelloWorld"), MyEnum::_Custom(PrivOwnedStr("HelloWorld".into())));
}

#[test]
fn serialize() {
    assert_eq!(to_json_value(MyEnum::First).unwrap(), json!("first"));
    assert_eq!(to_json_value(MyEnum::HelloWorld).unwrap(), json!("hello_world"));
    assert_eq!(to_json_value(MyEnum::Stable).unwrap(), json!("io.ruma.unstable"));
    assert_eq!(
        to_json_value(MyEnum::_Custom(PrivOwnedStr("\\\n\\".into()))).unwrap(),
        json!("\\\n\\")
    );
}

#[test]
fn deserialize() {
    assert_eq!(from_json_value::<MyEnum>(json!("first")).unwrap(), MyEnum::First);
    assert_eq!(from_json_value::<MyEnum>(json!("hello_world")).unwrap(), MyEnum::HelloWorld);
    assert_eq!(from_json_value::<MyEnum>(json!("io.ruma.unstable")).unwrap(), MyEnum::Stable);
    assert_eq!(from_json_value::<MyEnum>(json!("m.stable")).unwrap(), MyEnum::Stable);
    assert_eq!(from_json_value::<MyEnum>(json!("hs.notareal.unstable")).unwrap(), MyEnum::Stable);
    assert_eq!(
        from_json_value::<MyEnum>(json!("\\\n\\")).unwrap(),
        MyEnum::_Custom(PrivOwnedStr("\\\n\\".into()))
    );
}
