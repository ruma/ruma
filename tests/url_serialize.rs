use ruma_serde::urlencoded;
use serde::Serialize;
use url::form_urlencoded::Serializer as Encoder;

#[derive(Serialize)]
struct NewType<T>(T);

#[test]
fn serialize_newtype_i32() {
    let params = &[("field", Some(NewType(11)))];
    assert_eq!(urlencoded::to_string(params), Ok("field=11".to_owned()));
}

#[test]
fn serialize_option_map_int() {
    let params = &[("first", Some(23)), ("middle", None), ("last", Some(42))];

    assert_eq!(
        urlencoded::to_string(params),
        Ok("first=23&last=42".to_owned())
    );
}

#[test]
fn serialize_option_map_string() {
    let params = &[
        ("first", Some("hello")),
        ("middle", None),
        ("last", Some("world")),
    ];

    assert_eq!(
        urlencoded::to_string(params),
        Ok("first=hello&last=world".to_owned())
    );
}

#[test]
fn serialize_option_map_bool() {
    let params = &[("one", Some(true)), ("two", Some(false))];

    assert_eq!(
        urlencoded::to_string(params),
        Ok("one=true&two=false".to_owned())
    );
}

#[test]
fn serialize_map_bool() {
    let params = &[("one", true), ("two", false)];

    assert_eq!(
        urlencoded::to_string(params),
        Ok("one=true&two=false".to_owned())
    );
}

#[derive(Serialize)]
enum X {
    A,
    B,
    C,
}

#[test]
fn serialize_unit_enum() {
    let params = &[("one", X::A), ("two", X::B), ("three", X::C)];
    assert_eq!(
        urlencoded::to_string(params),
        Ok("one=A&two=B&three=C".to_owned())
    );
}

#[derive(Serialize)]
struct Unit;

#[test]
fn serialize_unit_struct() {
    assert_eq!(urlencoded::to_string(Unit), Ok("".to_owned()));
}

#[test]
fn serialize_unit_type() {
    assert_eq!(urlencoded::to_string(()), Ok("".to_owned()));
}

#[derive(Serialize)]
struct Wrapper<T> {
    item: T,
}

#[derive(Serialize)]
struct NewStruct {
    list: Vec<String>,
}

#[derive(Serialize)]
struct Struct {
    list: Vec<Option<String>>,
}

#[derive(Serialize)]
struct ListStruct {
    list: Vec<NewType<usize>>,
}

#[test]
#[ignore]
fn serialize_newstruct() {
    let s = NewStruct {
        list: vec!["hello".into(), "world".into()],
    };
    assert_eq!(
        "list=hello&list=world".to_string(),
        urlencoded::to_string(s).unwrap()
    );
}

#[test]
#[ignore]
fn serialize_vec_bool() {
    let params = Wrapper {
        item: vec![true, false, false],
    };
    assert_eq!(
        urlencoded::to_string(params).unwrap(),
        "item=true&item=false&item=false".to_owned()
    );
}

#[test]
#[ignore]
fn serialize_vec_num() {
    let params = Wrapper {
        item: vec![0, 1, 2],
    };
    assert_eq!(
        urlencoded::to_string(params).unwrap(),
        "item=0&item=1&item=2".to_owned()
    );
}

#[test]
#[ignore]
fn serialize_vec_str() {
    let params = Wrapper {
        item: vec!["hello", "world", "hello"],
    };
    assert_eq!(
        urlencoded::to_string(params).unwrap(),
        "item=hello&item=world&item=hello".to_owned()
    );
}

#[test]
#[ignore]
fn serialize_struct_opt() {
    let s = Struct {
        list: vec![Some("hello".into()), Some("world".into())],
    };
    assert_eq!(
        "list=hello&list=world".to_string(),
        urlencoded::to_string(s).unwrap()
    );
}

#[test]
#[ignore]
fn serialize_struct_newtype() {
    let s = ListStruct {
        list: vec![NewType(0), NewType(1)],
    };
    assert_eq!(
        "list=0&list=1".to_string(),
        urlencoded::to_string(s).unwrap()
    );
}

#[test]
#[ignore]
fn serialize_struct_unit_enum() {
    let params = Wrapper {
        item: vec![X::A, X::B, X::C],
    };
    assert_eq!(
        urlencoded::to_string(params),
        Ok("item=A&item=B&item=C".to_owned())
    );
}

#[test]
fn serialize_map() {
    let mut s = std::collections::BTreeMap::new();
    s.insert("hello", "world");
    s.insert("seri", "alize");
    s.insert("matrix", "ruma");

    let encoded = urlencoded::to_string(s).unwrap();
    assert_eq!("hello=world&matrix=ruma&seri=alize", encoded);
}

#[derive(Serialize)]
struct Nested<T> {
    item: T,
}

#[derive(Serialize)]
struct Inner {
    c: String,
    a: usize,
    b: String,
}

#[derive(Debug, Serialize, PartialEq)]
struct InnerList<T> {
    list: Vec<T>,
}

#[test]
#[ignore]
fn serialize_nested() {
    let mut encoder = Encoder::new(String::new());

    let s = Nested {
        item: Inner {
            c: "hello".into(),
            a: 10,
            b: "bye".into(),
        },
    };
    assert_eq!(
        encoder
            .append_pair("item", r#"{"c":"hello","a":10,"b":"bye"}"#)
            .finish(),
        urlencoded::to_string(s).unwrap()
    );
}

#[test]
#[ignore]
fn serialize_nested_list() {
    let mut encoder = Encoder::new(String::new());

    let s = Nested {
        item: InnerList {
            list: vec![1, 2, 3],
        },
    };
    assert_eq!(
        encoder.append_pair("item", r#"{"list":[1,2,3]}"#).finish(),
        urlencoded::to_string(s).unwrap()
    );
}

#[test]
#[ignore]
fn serialize_nested_list_option() {
    let mut encoder = Encoder::new(String::new());

    let s = Nested {
        item: InnerList {
            list: vec![Some(1), Some(2), None],
        },
    };
    assert_eq!(
        encoder
            .append_pair("item", r#"{"list":[1,2,null]}"#)
            .finish(),
        urlencoded::to_string(s).unwrap()
    );
}
