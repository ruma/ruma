use ruma_serde::urlencoded;
use serde::Deserialize;
use url::form_urlencoded::Serializer as Encoder;

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

    assert_eq!(
        urlencoded::from_reader(b"first=23&last=42" as &[_]),
        Ok(result)
    );
}

#[test]
fn deserialize_option() {
    let result = vec![
        ("first".to_owned(), Some(23)),
        ("last".to_owned(), Some(42)),
    ];
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
    let result = vec![
        ("one".to_owned(), X::A),
        ("two".to_owned(), X::B),
        ("three".to_owned(), X::C),
    ];

    assert_eq!(urlencoded::from_str("one=A&two=B&three=C"), Ok(result));
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
    let de = Params {
        a: 10,
        b: "Hello",
        c: None,
    };
    assert_eq!(Ok(de), urlencoded::from_str("a=10&b=Hello"));
    assert_eq!(Ok(de), urlencoded::from_str("b=Hello&a=10"));
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
#[ignore]
fn deserialize_newstruct() {
    let de = NewStruct {
        list: vec!["hello", "world"],
    };
    assert_eq!(urlencoded::from_str("list=hello&list=world"), Ok(de));
}

#[test]
#[ignore]
fn deserialize_numlist() {
    let de = NumList {
        list: vec![1, 2, 3, 4],
    };
    assert_eq!(urlencoded::from_str("list=1&list=2&list=3&list=4"), Ok(de));
}

#[test]
#[ignore]
fn deserialize_vec_bool() {
    assert_eq!(
        urlencoded::from_str("item=true&item=false&item=false"),
        Ok(Wrapper {
            item: vec![true, false, false]
        })
    );
}

#[test]
#[ignore]
fn deserialize_vec_string() {
    assert_eq!(
        urlencoded::from_str("item=hello&item=matrix&item=hello"),
        Ok(Wrapper {
            item: vec![
                "hello".to_string(),
                "matrix".to_string(),
                "hello".to_string()
            ],
        })
    );
}

#[test]
#[ignore]
fn deserialize_struct_unit_enum() {
    let result = Wrapper {
        item: vec![X::A, X::B, X::C],
    };

    assert_eq!(urlencoded::from_str("item=A&item=B&item=C"), Ok(result));
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
fn deserialize_nested() {
    let mut encoder = Encoder::new(String::new());

    let nested = Nested {
        item: Inner {
            c: "hello",
            a: 10,
            b: "bye",
        },
    };
    assert_eq!(
        urlencoded::from_str(
            &encoder
                .append_pair("item", r#"{"c":"hello","a":10,"b":"bye"}"#)
                .finish(),
        ),
        Ok(nested)
    );
}

#[test]
#[ignore]
fn deserialize_nested_list() {
    let mut encoder = Encoder::new(String::new());

    let nested = Nested {
        item: InnerList {
            list: vec![1, 2, 3],
        },
    };

    assert_eq!(
        urlencoded::from_str(
            &encoder.append_pair("item", r#"{"list":[1,2,3]}"#).finish(),
        ),
        Ok(nested)
    );
}

#[test]
#[ignore]
fn deserialize_nested_list_option() {
    let mut encoder = Encoder::new(String::new());

    let nested = Nested {
        item: InnerList {
            list: vec![Some(1), Some(2), None],
        },
    };
    assert_eq!(
        urlencoded::from_str(
            &encoder
                .append_pair("item", r#"{"list":[1,2,null]}"#)
                .finish(),
        ),
        Ok(nested)
    );
}
