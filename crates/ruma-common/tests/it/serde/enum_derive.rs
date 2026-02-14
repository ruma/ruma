use ruma_common::serde::StringEnum;
use serde_json::{from_value as from_json_value, json};

ruma_common::priv_owned_str!();

#[derive(StringEnum)]
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

#[derive(StringEnum)]
#[ruma_enum(rename_all(prefix = "unstable-", rule = "kebab-case"))]
enum MyUnstableEnum {
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

    assert_eq!(MyUnstableEnum::First.as_ref(), "unstable-first");
    assert_eq!(MyUnstableEnum::Second.as_ref(), "unstable-second");
    assert_eq!(MyUnstableEnum::Third.as_ref(), "m.third");
    assert_eq!(MyUnstableEnum::HelloWorld.as_ref(), "unstable-hello-world");
    assert_eq!(MyUnstableEnum::Stable.as_ref(), "io.ruma.unstable");
    assert_eq!(MyUnstableEnum::_Custom(PrivOwnedStr("HelloWorld".into())).as_ref(), "HelloWorld");
}

#[test]
fn display() {
    assert_eq!(MyEnum::First.to_string(), "first");
    assert_eq!(MyEnum::Second.to_string(), "second");
    assert_eq!(MyEnum::Third.to_string(), "m.third");
    assert_eq!(MyEnum::HelloWorld.to_string(), "hello_world");
    assert_eq!(MyEnum::Stable.to_string(), "io.ruma.unstable");
    assert_eq!(MyEnum::_Custom(PrivOwnedStr("HelloWorld".into())).to_string(), "HelloWorld");

    assert_eq!(MyUnstableEnum::First.to_string(), "unstable-first");
    assert_eq!(MyUnstableEnum::Second.to_string(), "unstable-second");
    assert_eq!(MyUnstableEnum::Third.to_string(), "m.third");
    assert_eq!(MyUnstableEnum::HelloWorld.to_string(), "unstable-hello-world");
    assert_eq!(MyUnstableEnum::Stable.to_string(), "io.ruma.unstable");
    assert_eq!(
        MyUnstableEnum::_Custom(PrivOwnedStr("HelloWorld".into())).to_string(),
        "HelloWorld"
    );
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

    assert_eq!(format!("{:?}", MyUnstableEnum::First), "\"unstable-first\"");
    assert_eq!(format!("{:?}", MyUnstableEnum::Second), "\"unstable-second\"");
    assert_eq!(format!("{:?}", MyUnstableEnum::Third), "\"m.third\"");
    assert_eq!(format!("{:?}", MyUnstableEnum::HelloWorld), "\"unstable-hello-world\"");
    assert_eq!(format!("{:?}", MyUnstableEnum::Stable), "\"io.ruma.unstable\"");
    assert_eq!(
        format!("{:?}", MyUnstableEnum::_Custom(PrivOwnedStr("HelloWorld".into()))),
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

    assert_eq!(MyUnstableEnum::from("unstable-first"), MyUnstableEnum::First);
    assert_eq!(MyUnstableEnum::from("unstable-second"), MyUnstableEnum::Second);
    assert_eq!(MyUnstableEnum::from("m.third"), MyUnstableEnum::Third);
    assert_eq!(MyUnstableEnum::from("unstable-hello-world"), MyUnstableEnum::HelloWorld);
    assert_eq!(MyUnstableEnum::from("io.ruma.unstable"), MyUnstableEnum::Stable);
    assert_eq!(MyUnstableEnum::from("m.stable"), MyUnstableEnum::Stable);
    assert_eq!(MyUnstableEnum::from("hs.notareal.unstable"), MyUnstableEnum::Stable);
    assert_eq!(
        MyUnstableEnum::from("HelloWorld"),
        MyUnstableEnum::_Custom(PrivOwnedStr("HelloWorld".into()))
    );
}

#[test]
fn serialize() {
    use ruma_common::canonical_json::assert_to_canonical_json_eq;

    assert_to_canonical_json_eq!(MyEnum::First, json!("first"));
    assert_to_canonical_json_eq!(MyEnum::HelloWorld, json!("hello_world"));
    assert_to_canonical_json_eq!(MyEnum::Stable, json!("io.ruma.unstable"));
    assert_to_canonical_json_eq!(MyEnum::_Custom(PrivOwnedStr("\\\n\\".into())), json!("\\\n\\"));

    assert_to_canonical_json_eq!(MyUnstableEnum::First, json!("unstable-first"));
    assert_to_canonical_json_eq!(MyUnstableEnum::HelloWorld, json!("unstable-hello-world"));
    assert_to_canonical_json_eq!(MyUnstableEnum::Stable, json!("io.ruma.unstable"));
    assert_to_canonical_json_eq!(
        MyUnstableEnum::_Custom(PrivOwnedStr("\\\n\\".into())),
        json!("\\\n\\"),
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

    assert_eq!(
        from_json_value::<MyUnstableEnum>(json!("unstable-first")).unwrap(),
        MyUnstableEnum::First
    );
    assert_eq!(
        from_json_value::<MyUnstableEnum>(json!("unstable-hello-world")).unwrap(),
        MyUnstableEnum::HelloWorld
    );
    assert_eq!(
        from_json_value::<MyUnstableEnum>(json!("io.ruma.unstable")).unwrap(),
        MyUnstableEnum::Stable
    );
    assert_eq!(
        from_json_value::<MyUnstableEnum>(json!("m.stable")).unwrap(),
        MyUnstableEnum::Stable
    );
    assert_eq!(
        from_json_value::<MyUnstableEnum>(json!("hs.notareal.unstable")).unwrap(),
        MyUnstableEnum::Stable
    );
    assert_eq!(
        from_json_value::<MyUnstableEnum>(json!("\\\n\\")).unwrap(),
        MyUnstableEnum::_Custom(PrivOwnedStr("\\\n\\".into()))
    );
}
