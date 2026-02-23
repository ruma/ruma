mod string {
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde::{Deserialize, Serialize};
    use serde_json::{from_value as from_json_value, json};

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
        assert_to_canonical_json_eq!(decoded, json!({ "x": "" }));
    }

    #[test]
    fn some_se() {
        let decoded = StringStruct { x: Some("foo".into()) };
        assert_to_canonical_json_eq!(decoded, json!({ "x": "foo" }));
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
    use ruma_common::{UserId, canonical_json::assert_to_canonical_json_eq, owned_user_id};
    use serde::{Deserialize, Serialize};
    use serde_json::{from_value as from_json_value, json};

    const CARL: &str = "@carl:example.com";

    fn carl() -> UserId {
        owned_user_id!("@carl:example.com")
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct User {
        #[serde(
            default,
            deserialize_with = "ruma_common::serde::empty_string_as_none",
            serialize_with = "ruma_common::serde::none_as_empty_string"
        )]
        x: Option<UserId>,
    }

    #[test]
    fn none_se() {
        let decoded = User { x: None };
        assert_to_canonical_json_eq!(decoded, json!({ "x": "" }));
    }

    #[test]
    fn some_se() {
        let decoded = User { x: Some(carl()) };
        assert_to_canonical_json_eq!(decoded, json!({ "x": CARL }));
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
