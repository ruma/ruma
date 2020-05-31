//! Implementation of the collection type macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

use parse::RumaCollectionInput;

/// Create a collection from `RumaCollectionInput.
pub fn expand_collection(input: RumaCollectionInput) -> syn::Result<TokenStream> {
    let attrs = &input.attrs;
    let ident = &input.name;

    let variants = input
        .events
        .iter()
        .map(|lit| {
            let content_docstring = lit;
            let var = to_camel_case(lit);
            let content = to_event_content(lit);

            quote! {
                #[doc = #content_docstring]
                #var(#content)
            }
        })
        .collect::<Vec<_>>();

    let collection = quote! {
        #( #attrs )*
        #[derive(Clone, Debug, /*Serialize*/)]
        //#[serde(untagged)]
        #[allow(clippy::large_enum_variant)]
        pub enum #ident {
            #( #variants ),*
        }
    };

    Ok(collection)
}

/// Splits the given `event_type` string on `.` and `_` removing the `m.` then
/// using only the event name append "EventContent".
fn to_event_content(name: &LitStr) -> Ident {
    let span = name.span();
    let name = name.value();

    assert_eq!(&name[..2], "m.");

    let event = name[2..].split('.').last().unwrap();

    let event = event
        .split('_')
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect::<String>();

    let content_str = format!("{}EventContent", event);
    Ident::new(&content_str, span)
}

/// Splits the given `event_type` string on `.` and `_` removing the `m.room.` then
/// camel casing to give the `EventContent` struct name.
pub(crate) fn to_camel_case(name: &LitStr) -> Ident {
    let span = name.span();
    let name = name.value();
    assert_eq!(&name[..2], "m.");
    let s = name[2..]
        .split(&['.', '_'] as &[char])
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect::<String>();
    Ident::new(&s, span)
}

/// Details of parsing input for the `event_content_collection` procedural macro.
pub mod parse {
    use syn::{
        parse::{self, Parse, ParseStream},
        Attribute, Expr, ExprLit, Ident, Lit, LitStr, Token,
    };

    /// Custom keywords for the `event_content_collection!` macro
    mod kw {
        syn::custom_keyword!(name);
        syn::custom_keyword!(events);
    }

    /// The entire `event_content_collection!` macro structure directly as it appears in the source code..
    pub struct RumaCollectionInput {
        /// Outer attributes on the field, such as a docstring.
        pub attrs: Vec<Attribute>,

        /// The name of the event.
        pub name: Ident,

        /// An array of valid matrix event types. This will generate the variants of the event content type "name".
        /// There needs to be a corresponding variant in `ruma_events::EventType` for
        /// this event (converted to a valid Rust-style type name by stripping `m.`, replacing the
        /// remaining dots by underscores and then converting from snake_case to CamelCase).
        pub events: Vec<LitStr>,
    }

    impl Parse for RumaCollectionInput {
        fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
            let attrs = input.call(Attribute::parse_outer)?;
            // name field
            input.parse::<kw::name>()?;
            input.parse::<Token![:]>()?;
            // the name of our collection enum
            let name: Ident = input.parse()?;
            input.parse::<Token![,]>()?;

            // events field
            input.parse::<kw::events>()?;
            input.parse::<Token![:]>()?;

            // an array of event names `["m.room.whatever"]`
            let ev_array = input.parse::<syn::ExprArray>()?;
            let events = ev_array
                .elems
                .into_iter()
                .map(|item| {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = item
                    {
                        Ok(lit_str)
                    } else {
                        let msg = "values of field `events` are required to be a string literal";
                        Err(syn::Error::new_spanned(item, msg))
                    }
                })
                .collect::<syn::Result<_>>()?;

            Ok(Self {
                attrs,
                name,
                events,
            })
        }
    }
}
