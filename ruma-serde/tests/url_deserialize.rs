use form_urlencoded::Serializer as Encoder;
use matches::assert_matches;
use ruma_serde::urlencoded;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
struct NewType<T>(T);

#[test]
fn deserialize_newtype_i32() {
    let result = vec![("field".to_owned(), NewType(11))];

    assert_eq!(urlencoded::from_str("field=11"), Ok(result));
}

#[test]
fn deserialize_bytes() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(urlencoded::from_bytes(b"first=23&last=42"), Ok(result));
}

#[test]
fn deserialize_str() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(urlencoded::from_str("first=23&last=42"), Ok(result));
}

#[test]
fn deserialize_borrowed_str() {
    let result = vec![("first", 23), ("last", 42)];

    assert_eq!(urlencoded::from_str("first=23&last=42"), Ok(result));
}

#[test]
fn deserialize_reader() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(urlencoded::from_reader(b"first=23&last=42" as &[_]), Ok(result));
}

#[test]
fn deserialize_option() {
    let result = vec![("first".to_owned(), Some(23)), ("last".to_owned(), Some(42))];
    assert_eq!(urlencoded::from_str("first=23&last=42"), Ok(result));
}

#[test]
fn deserialize_unit() {
    assert_eq!(urlencoded::from_str(""), Ok(()));
    assert_eq!(urlencoded::from_str("&"), Ok(()));
    assert_eq!(urlencoded::from_str("&&"), Ok(()));
    assert!(urlencoded::from_str::<()>("first=23").is_err());
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
enum X {
    A,
    B,
    C,
}

#[test]
fn deserialize_unit_enum() {
    let result: Vec<(String, X)> = urlencoded::from_str("one=A&two=B&three=C").unwrap();

    assert_eq!(result.len(), 3);
    assert!(result.contains(&("one".to_owned(), X::A)));
    assert!(result.contains(&("two".to_owned(), X::B)));
    assert!(result.contains(&("three".to_owned(), X::C)));
}

#[test]
fn deserialize_unit_type() {
    assert_eq!(urlencoded::from_str(""), Ok(()));
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
struct Params<'a> {
    a: usize,
    b: &'a str,
    c: Option<u8>,
}

#[test]
fn deserialize_struct() {
    let de = Params { a: 10, b: "Hello", c: None };
    assert_eq!(urlencoded::from_str("a=10&b=Hello"), Ok(de));
    assert_eq!(urlencoded::from_str("b=Hello&a=10"), Ok(de));
}

#[test]
fn deserialize_list_of_str() {
    // TODO: It would make sense to support this.
    assert_matches!(
        urlencoded::from_str::<Vec<(&str, &str)>>("a=a&a=b"),
        Err(error) if error.to_string().contains("unsupported")
    );

    assert_eq!(urlencoded::from_str("a=a&a=b"), Ok(vec![("a", vec!["a", "b"])]))
}

#[test]
fn deserialize_multiple_lists() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Lists {
        xs: Vec<bool>,
        ys: Vec<u32>,
    }

    assert_eq!(
        urlencoded::from_str("xs=true&xs=false&ys=3&ys=2&ys=1"),
        Ok(Lists { xs: vec![true, false], ys: vec![3, 2, 1] })
    );

    assert_eq!(
        urlencoded::from_str("ys=3&xs=true&ys=2&xs=false&ys=1"),
        Ok(Lists { xs: vec![true, false], ys: vec![3, 2, 1] })
    );
}

#[test]
fn deserialize_with_serde_attributes() {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[derive(Debug, PartialEq, Deserialize)]
    struct FieldsWithAttributes {
        #[serde(default)]
        xs: Vec<bool>,
        #[serde(default)]
        def: Option<u8>,
        #[serde(
            default,
            deserialize_with = "ruma_serde::time::opt_ms_since_unix_epoch::deserialize"
        )]
        time: Option<SystemTime>,
        #[serde(default)]
        flag: bool,
    }

    assert_eq!(
        urlencoded::from_str("xs=true&xs=false&def=3&time=1&flag=true"),
        Ok(FieldsWithAttributes {
            xs: vec![true, false],
            def: Some(3),
            time: Some(UNIX_EPOCH + Duration::from_millis(1)),
            flag: true,
        })
    );

    assert_eq!(
        urlencoded::from_str(""),
        Ok(FieldsWithAttributes { xs: vec![], def: None, time: None, flag: false })
    );
}

#[test]
fn deserialize_nested_list() {
    assert!(urlencoded::from_str::<Vec<(&str, Vec<Vec<bool>>)>>("a=b").is_err());
}

#[test]
fn deserialize_list_of_option() {
    assert_eq!(
        urlencoded::from_str("list=10&list=100"),
        Ok(vec![("list", vec![Some(10), Some(100)])])
    );
}

#[test]
fn deserialize_list_of_newtype() {
    assert_eq!(urlencoded::from_str("list=test"), Ok(vec![("list", vec![NewType("test")])]));
}

#[test]
fn deserialize_list_of_enum() {
    assert_eq!(
        urlencoded::from_str("item=A&item=B&item=C"),
        Ok(vec![("item", vec![X::A, X::B, X::C])])
    );
}

#[derive(Debug, Deserialize, PartialEq)]
struct Wrapper<T> {
    item: T,
}

#[derive(Debug, PartialEq, Deserialize)]
struct NewStruct<'a> {
    #[serde(borrow)]
    list: Vec<&'a str>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Struct<'a> {
    #[serde(borrow)]
    list: Vec<Option<&'a str>>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct NumList {
    list: Vec<u8>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct ListStruct {
    list: Vec<NewType<usize>>,
}

#[test]
fn deserialize_newstruct() {
    let de = NewStruct { list: vec!["hello", "world"] };
    assert_eq!(urlencoded::from_str("list=hello&list=world"), Ok(de));
}

#[test]
fn deserialize_numlist() {
    let de = NumList { list: vec![1, 2, 3, 4] };
    assert_eq!(urlencoded::from_str("list=1&list=2&list=3&list=4"), Ok(de));
}

#[derive(Debug, Deserialize, PartialEq)]
struct Nested<T> {
    item: T,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Inner<'a> {
    c: &'a str,
    a: usize,
    b: &'a str,
}

#[derive(Debug, Deserialize, PartialEq)]
struct InnerList<T> {
    list: Vec<T>,
}

#[test]
#[ignore]
fn deserialize_nested_struct() {
    let mut encoder = Encoder::new(String::new());

    let nested = Nested { item: Inner { c: "hello", a: 10, b: "bye" } };
    assert_eq!(
        urlencoded::from_str(
            &encoder.append_pair("item", r#"{"c":"hello","a":10,"b":"bye"}"#).finish(),
        ),
        Ok(nested)
    );
}

#[test]
#[ignore]
fn deserialize_nested_struct_with_list() {
    let mut encoder = Encoder::new(String::new());

    let nested = Nested { item: InnerList { list: vec![1, 2, 3] } };

    assert_eq!(
        urlencoded::from_str(&encoder.append_pair("item", r#"{"list":[1,2,3]}"#).finish()),
        Ok(nested)
    );
}

#[test]
#[ignore]
fn deserialize_nested_list_option() {
    let mut encoder = Encoder::new(String::new());

    let nested = Nested { item: InnerList { list: vec![Some(1), Some(2), None] } };
    assert_eq!(
        urlencoded::from_str(&encoder.append_pair("item", r#"{"list":[1,2,null]}"#).finish()),
        Ok(nested)
    );
}
