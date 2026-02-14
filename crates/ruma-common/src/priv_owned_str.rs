/// Convenience macro to declare a struct named `PrivOwnedStr`.
///
/// The generated struct is a wrapper around `Box<str>` that cannot be used in a meaningful way
/// outside of the crate where it is defined. It is usually used for string enums because their
/// `_Custom` variant can't be truly private (only `#[doc(hidden)]`).
///
/// The struct implements `Clone`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash`.
///
/// ## Arguments
///
/// This macro can be called without any arguments, it will only generate the struct and its basic
/// implementations.
///
/// The following keywords can also be used as a comma-separated list to add more implementations to
/// the struct:
///
/// - `uniffi` - Expose the struct as an object named `PrivateString` to foreign languages via
///   [`uniffi`], behind an `unstable-uniffi` cargo feature. This is necessary to expose an enum or
///   record using `PrivOwnedStr` to foreign languages. Requires the crate calling the macro to have
///   an `unstable-uniffi` cargo feature and [`uniffi` must be set up][uniffi-setup].
///
/// ## Example
///
/// ```
/// ruma_common::priv_owned_str!(uniffi);
/// ```
///
/// [uniffi]: https://crates.io/crates/uniffi
/// [uniffi-setup]: https://mozilla.github.io/uniffi-rs/latest/tutorial/Rust_scaffolding.html
#[doc(hidden)]
#[macro_export]
macro_rules! priv_owned_str {
    () => {
        #[doc(hidden)]
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct PrivOwnedStr(std::boxed::Box<std::primitive::str>);

        impl std::fmt::Debug for PrivOwnedStr {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };

    ( $( $keyword:ident ),+ ) => {
        $crate::priv_owned_str!();
        $( $crate::priv_owned_str!(@keyword $keyword); )+
    };

    ( @keyword uniffi ) => {
        // Wrapper around `Box<str>` for transferring `PrivOwnedStr` over UniFFI.
        // We cannot derive `PrivOwnedStr` from `uniffi::Object` directly because
        // that would require wrapping it in an `Arc` inside the `_Custom` variants.
        #[cfg(feature = "unstable-uniffi")]
        #[derive(uniffi::Object)]
        #[doc(hidden)]
        pub struct PrivateString(std::boxed::Box<std::primitive::str>);

        #[cfg(feature = "unstable-uniffi")]
        uniffi::custom_type!(PrivOwnedStr, std::sync::Arc<PrivateString> , {
            lower: |value| std::sync::Arc::new(PrivateString(value.0)),
            try_lift: |value| Ok(PrivOwnedStr(value.0.clone())),
        });
    };
}
