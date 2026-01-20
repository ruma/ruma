/// Asserts that the canonical JSON serialization of the two expressions are equal to each other.
///
/// This is a convenience macro that serializes both expressions to [`CanonicalJsonValue`] and
/// asserts the equality of those values.
///
/// Because it uses [`to_canonical_value()`] internally, it has stricter serializing rules than
/// using [`serde_json::Value`], which allows to avoid some pitfalls like serializing the same key
/// twice in an object.
///
/// Both expressions must implement [`serde::Serialize`].
///
/// Panics if any of the expressions fails to serialize to [`CanonicalJsonValue`], or if the two
/// values are not equal.
///
/// This macro will print the error if one of the serializations fails, or both debug
/// representations of the [`CanonicalJsonValue`]s if they are not equal.
///
/// ## Example
///
/// ```
/// use ruma_common::canonical_json::assert_to_canonical_json_eq;
/// use serde::Serialize;
/// use serde_json::json;
///
/// #[derive(Serialize)]
/// struct Data {
///     id: String,
///     flag: bool,
/// }
///
/// let data = Data { id: "abcdef".to_owned(), flag: true };
/// assert_to_canonical_json_eq!(data, json!({ "id": "abcdef", "flag": true }));
/// ```
///
/// [`CanonicalJsonValue`]: super::CanonicalJsonValue
/// [`to_canonical_value()`]: super::to_canonical_value
#[doc(hidden)]
#[macro_export]
macro_rules! assert_to_canonical_json_eq {
    ($left:expr, $right:expr $(,)?) => {
        let left_val =
            $crate::canonical_json::to_canonical_value(&$left).expect("left serialization failed");
        let right_val = $crate::canonical_json::to_canonical_value(&$right)
            .expect("right serialization failed");
        assert_eq!(left_val, right_val);
    };
}
