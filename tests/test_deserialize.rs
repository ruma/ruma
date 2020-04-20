use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
struct NewType<T>(T);

#[test]
fn deserialize_newtype_i32() {
    let result = vec![("field".to_owned(), NewType(11))];

    assert_eq!(serde_urlencoded::from_str("field=11"), Ok(result));
}

#[test]
fn deserialize_bytes() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(
        serde_urlencoded::from_bytes(b"first=23&last=42"),
        Ok(result)
    );
}

#[test]
fn deserialize_str() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(serde_urlencoded::from_str("first=23&last=42"), Ok(result));
}

#[test]
fn deserialize_borrowed_str() {
    let result = vec![("first", 23), ("last", 42)];

    assert_eq!(serde_urlencoded::from_str("first=23&last=42"), Ok(result));
}

#[test]
fn deserialize_reader() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(
        serde_urlencoded::from_reader(b"first=23&last=42" as &[_]),
        Ok(result)
    );
}

#[test]
fn deserialize_option() {
    let result = vec![
        ("first".to_owned(), Some(23)),
        ("last".to_owned(), Some(42)),
    ];
    assert_eq!(serde_urlencoded::from_str("first=23&last=42"), Ok(result));
}

#[test]
fn deserialize_unit() {
    assert_eq!(serde_urlencoded::from_str(""), Ok(()));
    assert_eq!(serde_urlencoded::from_str("&"), Ok(()));
    assert_eq!(serde_urlencoded::from_str("&&"), Ok(()));
    assert!(serde_urlencoded::from_str::<()>("first=23").is_err());
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

    assert_eq!(
        serde_urlencoded::from_str("one=A&two=B&three=C"),
        Ok(result)
    );
}

#[test]
fn deserialize_unit_type() {
    assert_eq!(serde_urlencoded::from_str(""), Ok(()));
}

#[derive(Debug, Deserialize, PartialEq)]
struct Wrapper<T> {
    item: T,
}

#[derive(Debug, PartialEq, Deserialize)]
struct NewStruct {
    list: Vec<String>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Struct {
    list: Vec<Option<String>>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct NumList {
    list: Vec<u8>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct ListStruct {
    list: Vec<NewType<usize>>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct MapStruct {
    a: usize,
    b: String,
    c: Option<u8>,
}

#[test]
fn deserialize_mapstruct() {
    let de = MapStruct {
        a: 10,
        b: "Hello".into(),
        c: None,
    };
    assert_eq!(
        de,
        serde_urlencoded::from_str::<MapStruct>("a=10&b=Hello").unwrap()
    );
}

#[test]
fn deserialize_newstruct() {
    let de = NewStruct {
        list: vec!["hello".into(), "world".into()],
    };
    assert_eq!(
        de,
        serde_urlencoded::from_str::<NewStruct>("list=hello&list=world")
            .unwrap()
    );
}

#[test]
fn deserialize_numlist() {
    let de = NumList {
        list: vec![1, 2, 3, 4],
    };
    assert_eq!(
        de,
        serde_urlencoded::from_str::<NumList>("list=1&list=2&list=3&list=4")
            .unwrap()
    );
}

#[test]
fn deserialize_vec_bool() {
    assert_eq!(
        Wrapper {
            item: vec![true, false, false]
        },
        serde_urlencoded::from_str::<Wrapper<_>>(
            "item=true&item=false&item=false"
        )
        .unwrap()
    );
}

#[test]
fn deserialize_vec_string() {
    assert_eq!(
        Wrapper {
            item: vec![
                "hello".to_string(),
                "matrix".to_string(),
                "hello".to_string()
            ],
        },
        serde_urlencoded::from_str::<Wrapper<_>>(
            "item=hello&item=matrix&item=hello"
        )
        .unwrap()
    );
}

#[test]
fn deserialize_struct_unit_enum() {
    let result = Wrapper {
        item: vec![X::A, X::B, X::C],
    };

    assert_eq!(
        serde_urlencoded::from_str("item=A&item=B&item=C"),
        Ok(result)
    );
}
