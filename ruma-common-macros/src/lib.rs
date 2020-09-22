use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use outgoing::expand_derive_outgoing;

mod outgoing;
mod util;

/// Derive the `Outgoing` trait, possibly generating an 'Incoming' version of the struct this
/// derive macro is used on. Specifically, if no lifetime variables are used on any of the fields
/// of the struct, this simple implementation will be generated:
///
/// ```ignore
/// impl Outgoing for MyType {
///     type Incoming = Self;
/// }
/// ```
/*

TODO: Extend docs. Previously:

If, however, `#[wrap_incoming]` is used (which is the only reason you should ever use this
derive macro manually), a new struct `IncomingT` (where `T` is the type this derive is used on)
is generated, with all of the fields with `#[wrap_incoming]` replaced:

```ignore
#[derive(Outgoing)]
struct MyType {
    pub foo: Foo,
    #[wrap_incoming]
    pub bar: Bar,
    #[wrap_incoming(Baz)]
    pub baz: Option<Baz>,
    #[wrap_incoming(with EventResult)]
    pub x: XEvent,
    #[wrap_incoming(YEvent with EventResult)]
    pub ys: Vec<YEvent>,
}

// generated
struct IncomingMyType {
    pub foo: Foo,
    pub bar: IncomingBar,
    pub baz: Option<IncomingBaz>,
    pub x: EventResult<XEvent>,
    pub ys: Vec<EventResult<YEvent>>,
}
```

*/
#[proc_macro_derive(Outgoing, attributes(incoming_derive))]
pub fn derive_outgoing(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_derive_outgoing(input).unwrap_or_else(|err| err.to_compile_error()).into()
}
