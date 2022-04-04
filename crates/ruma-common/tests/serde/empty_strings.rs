mod string {
    use serde::{Deserialize, Serialize};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct StringStruct {
        #[serde(
            default,
            deserialize_with = "ruma_common::serde::empty_string_as_none",
            serialize_with = "ruma_common::serde::none_as_empty_string"
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
    use ruma_common::{user_id, OwnedUserId, UserId};
    use serde::{Deserialize, Serialize};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    const CARL: &str = "@carl:example.com";

    fn carl() -> &'static UserId {
        user_id!("@carl:example.com")
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct User {
        #[serde(
            default,
            deserialize_with = "ruma_common::serde::empty_string_as_none",
            serialize_with = "ruma_common::serde::none_as_empty_string"
        )]
        x: Option<OwnedUserId>,
    }

    #[test]
    fn none_se() {
        let decoded = User { x: None };
        let encoded = json!({ "x": "" });
        assert_eq!(to_json_value(decoded).unwrap(), encoded);
    }

    #[test]
    fn some_se() {
        let decoded = User { x: Some(carl().to_owned()) };
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
        let decoded = User { x: Some(carl().to_owned()) };
        assert_eq!(from_json_value::<User>(encoded).unwrap(), decoded);
    }
}
