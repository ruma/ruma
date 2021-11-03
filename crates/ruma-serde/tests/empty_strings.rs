mod string {
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

    #[test]
    fn none_se() {
        let decoded = StringStruct { x: None };
        let encoded = json!({ "x": "" });
        assert_eq!(to_json_value(decoded).unwrap(), encoded);
    }

    #[test]
    fn some_se() {
        let decoded = StringStruct { x: Some("foo".into()) };
        let encoded = json!({ "x": "foo" });
        assert_eq!(to_json_value(decoded).unwrap(), encoded);
    }

    #[test]
    fn absent_de() {
        let encoded = json!({});
        let decoded = StringStruct { x: None };
        assert_eq!(from_json_value::<StringStruct>(encoded).unwrap(), decoded);
    }

    #[test]
    fn empty_de() {
        let encoded = json!({ "x": "" });
        let decoded = StringStruct { x: None };
        assert_eq!(from_json_value::<StringStruct>(encoded).unwrap(), decoded);
    }

    #[test]
    fn some_de() {
        let encoded = json!({ "x": "foo" });
        let decoded = StringStruct { x: Some("foo".into()) };
        assert_eq!(from_json_value::<StringStruct>(encoded).unwrap(), decoded);
    }
}

mod user {
    use ruma_identifiers::{user_id, UserId};
    use serde::{Deserialize, Serialize};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    const CARL: &str = "@carl:example.com";

    fn carl() -> UserId {
        user_id!("@carl:example.com")
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct User {
        #[serde(
            default,
            deserialize_with = "ruma_serde::empty_string_as_none",
            serialize_with = "ruma_serde::none_as_empty_string"
        )]
        x: Option<UserId>,
    }

    #[test]
    fn none_se() {
        let decoded = User { x: None };
        let encoded = json!({ "x": "" });
        assert_eq!(to_json_value(decoded).unwrap(), encoded);
    }

    #[test]
    fn some_se() {
        let decoded = User { x: Some(carl()) };
        let encoded = json!({ "x": CARL });
        assert_eq!(to_json_value(decoded).unwrap(), encoded);
    }

    #[test]
    fn absent_de() {
        let encoded = json!({});
        let decoded = User { x: None };
        assert_eq!(from_json_value::<User>(encoded).unwrap(), decoded);
    }

    #[test]
    fn empty_de() {
        let encoded = json!({ "x": "" });
        let decoded = User { x: None };
        assert_eq!(from_json_value::<User>(encoded).unwrap(), decoded);
    }

    #[test]
    fn some_de() {
        let encoded = json!({ "x": CARL });
        let decoded = User { x: Some(carl()) };
        assert_eq!(from_json_value::<User>(encoded).unwrap(), decoded);
    }
}

mod int {
    use serde::{Deserialize, Serialize};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Int {
        #[serde(
            default,
            deserialize_with = "ruma_serde::empty_string_as_none",
            serialize_with = "ruma_serde::none_as_empty_string"
        )]
        x: Option<i32>,
    }

    #[test]
    fn none_se() {
        let decoded = Int { x: None };
        let encoded = json!({ "x": "" });
        assert_eq!(to_json_value(decoded).unwrap(), encoded);
    }

    #[test]
    fn some_se() {
        let decoded = Int { x: Some(1) };
        let encoded = json!({ "x": 1 });
        assert_eq!(to_json_value(decoded).unwrap(), encoded);
    }

    #[test]
    fn absent_de() {
        let encoded = json!({});
        let decoded = Int { x: None };
        assert_eq!(from_json_value::<Int>(encoded).unwrap(), decoded);
    }

    #[test]
    fn empty_de() {
        let encoded = json!({ "x": "" });
        let decoded = Int { x: None };
        assert_eq!(from_json_value::<Int>(encoded).unwrap(), decoded);
    }

    #[test]
    fn some_de() {
        // TODO doesn't work with `json!({ "x": 1 })` - unquoted `1`
        let encoded = json!({ "x": "1" });
        let decoded = Int { x: Some(1) };
        assert_eq!(from_json_value::<Int>(encoded).unwrap(), decoded);
    }
}
