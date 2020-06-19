use form_urlencoded::Serializer as Encoder;
use matches::assert_matches;
use ruma_serde::urlencoded::{self, ser::Error};
use serde::Serialize;

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

    assert_eq!(urlencoded::to_string(params), Ok("first=23&last=42".to_owned()));
}

#[test]
fn serialize_option_map_string() {
    let params = &[("first", Some("hello")), ("middle", None), ("last", Some("world"))];

    assert_eq!(urlencoded::to_string(params), Ok("first=hello&last=world".to_owned()));
}

#[test]
fn serialize_option_map_bool() {
    let params = &[("one", Some(true)), ("two", Some(false))];

    assert_eq!(urlencoded::to_string(params), Ok("one=true&two=false".to_owned()));
}

#[test]
fn serialize_map_bool() {
    let params = &[("one", true), ("two", false)];

    assert_eq!(urlencoded::to_string(params), Ok("one=true&two=false".to_owned()));
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
    assert_eq!(urlencoded::to_string(params), Ok("one=A&two=B&three=C".to_owned()));
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

#[test]
fn serialize_list_of_str() {
    let params = &[("list", vec!["hello", "world"])];

    assert_eq!(urlencoded::to_string(params), Ok("list=hello&list=world".to_owned()));
}

#[test]
fn serialize_multiple_lists() {
    #[derive(Serialize)]
    struct Lists {
        xs: Vec<bool>,
        ys: Vec<u32>,
    }

    let params = Lists { xs: vec![true, false], ys: vec![3, 2, 1] };

    assert_eq!(urlencoded::to_string(params), Ok("xs=true&xs=false&ys=3&ys=2&ys=1".to_owned()));
}

#[test]
fn serialize_nested_list() {
    let params = &[("list", vec![vec![0u8]])];
    assert_matches!(
        urlencoded::to_string(params),
        Err(Error::Custom(s)) if s.contains("unsupported")
    )
}

#[test]
fn serialize_list_of_option() {
    let params = &[("list", vec![Some(10), Some(100)])];
    assert_eq!(urlencoded::to_string(params), Ok("list=10&list=100".to_owned()));
}

#[test]
fn serialize_list_of_newtype() {
    let params = &[("list", vec![NewType("test".to_owned())])];
    assert_eq!(urlencoded::to_string(params), Ok("list=test".to_owned()));
}

#[test]
fn serialize_list_of_enum() {
    let params = &[("item", vec![X::A, X::B, X::C])];
    assert_eq!(urlencoded::to_string(params), Ok("item=A&item=B&item=C".to_owned()));
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
fn serialize_nested_struct() {
    let mut encoder = Encoder::new(String::new());

    let s = Nested { item: Inner { c: "hello".into(), a: 10, b: "bye".into() } };
    assert_eq!(
        encoder.append_pair("item", r#"{"c":"hello","a":10,"b":"bye"}"#).finish(),
        urlencoded::to_string(s).unwrap()
    );
}

#[test]
#[ignore]
fn serialize_nested_struct_with_list() {
    let mut encoder = Encoder::new(String::new());

    let s = Nested { item: InnerList { list: vec![1, 2, 3] } };
    assert_eq!(
        encoder.append_pair("item", r#"{"list":[1,2,3]}"#).finish(),
        urlencoded::to_string(s).unwrap()
    );
}
