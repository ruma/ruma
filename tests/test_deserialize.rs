extern crate serde_urlencoded;

#[test]
fn deserialize_bytes() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(serde_urlencoded::from_bytes(b"first=23&last=42"),
               Ok(result));
}

#[test]
fn deserialize_str() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(serde_urlencoded::from_str("first=23&last=42"),
               Ok(result));
}

#[test]
fn deserialize_reader() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(serde_urlencoded::from_reader(b"first=23&last=42" as &[_]),
               Ok(result));
}
