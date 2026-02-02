/// Asserts that the canonical JSON serialization of a type is equal to an expected JSON value.
///
/// By calling [`to_canonical_value()`] internally this macro enforces strict serialization rules
/// which allows to avoid some pitfalls like having a [`serde::Serialize`] implementation that
/// generates the same key twice in an object, which is ignored by [`serde_json::to_value()`].
///
/// The expression on the left is the one whose serialization needs to be checked. It must
/// implement [`serde::Serialize`], and is serialized using [`to_canonical_value()`].
///
/// The expression on the right is the expected JSON serialization as a [`serde_json::Value`]. It is
/// usually a declaration using the [`serde_json::json!`] macro. It is then converted to a
/// [`CanonicalJsonValue`] and compared with the result of the serialization of the expression on
/// the left.
///
/// Panics if the expression on the left fails to serialize, if the expected JSON fails to be
/// converted to a [`CanonicalJsonValue`], or if the two values are not equal.
///
/// This macro will print the error if the serialization or the conversion fails, or both debug
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
        let right_val = <serde_json::Value as TryInto<
            $crate::canonical_json::CanonicalJsonValue,
        >>::try_into($right)
        .expect("right conversion failed");
        assert_eq!(left_val, right_val);
    };
}
