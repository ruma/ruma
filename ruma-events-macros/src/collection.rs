//! Implementation of the collection type macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

use parse::RumaCollectionInput;

fn marker_traits(ident: &Ident) -> TokenStream {
    match ident.to_string().as_str() {
        "AnyStateEventContent" => quote! {
            impl ::ruma_events::RoomEventContent for #ident {}
            impl ::ruma_events::StateEventContent for #ident {}
        },
        "AnyMessageEventContent" => quote! {
            impl ::ruma_events::RoomEventContent for #ident {}
            impl ::ruma_events::MessageEventContent for #ident {}
        },
        _ => TokenStream::new(),
    }
}

/// Create a collection from `RumaCollectionInput.
pub fn expand_collection(input: RumaCollectionInput) -> syn::Result<TokenStream> {
    let attrs = &input.attrs;
    let ident = &input.name;
    let event_type_str = &input.events;

    let variants = input.events.iter().map(to_camel_case).collect::<Vec<_>>();
    let content = input
        .events
        .iter()
        .map(to_event_content_path)
        .collect::<Vec<_>>();

    let collection = quote! {
        #( #attrs )*
        #[derive(Clone, Debug, ::serde::Serialize)]
        #[serde(untagged)]
        #[allow(clippy::large_enum_variant)]
        pub enum #ident {
            #(
                #[doc = #event_type_str]
                #variants(#content)
            ),*
        }
    };

    let event_content_impl = quote! {
        impl ::ruma_events::EventContent for #ident {
            fn event_type(&self) -> &str {
                match self {
                    #( Self::#variants(content) => content.event_type() ),*
                }
            }
        }
    };

    let try_from_raw_impl = quote! {
        impl ::ruma_events::TryFromRaw for #ident {
            type Raw = raw::#ident;
            type Err = String;

            fn try_from_raw(raw: Self::Raw) -> Result<Self, Self::Err> {
                use raw::#ident::*;

                match raw {
                    #( #variants(c) => {
                            let content = ::ruma_events::TryFromRaw::try_from_raw(c)
                                .map_err(|e: <#content as ::ruma_events::TryFromRaw>::Err| e.to_string())?;
                                // without this ^^^^^^^^^^^ the compiler fails to infer the type
                            Ok(Self::#variants(content))
                        }
                    ),*
                }
            }
        }
    };

    let marker_trait_impls = marker_traits(ident);

    let raw_mod = expand_raw_content_event(&input, &variants)?;

    Ok(quote! {
        #collection

        #try_from_raw_impl

        #event_content_impl

        #marker_trait_impls

        #raw_mod
    })
}

fn expand_raw_content_event(
    input: &RumaCollectionInput,
    variants: &[Ident],
) -> syn::Result<TokenStream> {
    let ident = &input.name;
    let event_type_str = &input.events;

    let raw_docs = format!("The raw version of {}, allows for deserialization.", ident);
    let raw_content = input
        .events
        .iter()
        .map(to_raw_event_content_path)
        .collect::<Vec<_>>();

    let raw_collection = quote! {
        #[doc = #raw_docs]
        #[derive(Clone, Debug)]
        #[allow(clippy::large_enum_variant)]
        pub enum #ident {
            #(
                #[doc = #event_type_str]
                #variants(#raw_content)
            ),*
        }
    };

    let raw_event_content_impl = quote! {
        impl ::ruma_events::RawEventContent for #ident {
            fn from_parts(event_type: &str, input: Box<::serde_json::value::RawValue>) -> Result<Self, String> {
                match event_type {
                    #(
                        #event_type_str => {
                            let content = #raw_content::from_parts(event_type, input)?;
                            Ok(#ident::#variants(content))
                        },
                    )*
                    ev => Err(format!("event not supported {}", ev)),
                }
            }
        }
    };

    Ok(quote! {
        mod raw {
            #raw_collection

            #raw_event_content_impl
        }
    })
}

fn to_event_content_path(
    name: &LitStr,
) -> syn::punctuated::Punctuated<syn::Token![::], syn::PathSegment> {
    let span = name.span();
    let name = name.value();

    assert_eq!(&name[..2], "m.");

    let path = name[2..].split('.').collect::<Vec<_>>();

    let event_str = path.last().unwrap();
    let event = event_str
        .split('_')
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect::<String>();

    let content_str = Ident::new(&format!("{}EventContent", event), span);
    let path = path.iter().map(|s| Ident::new(s, span));
    syn::parse_quote! {
        ::ruma_events::#( #path )::*::#content_str
    }
}

fn to_raw_event_content_path(
    name: &LitStr,
) -> syn::punctuated::Punctuated<syn::Token![::], syn::PathSegment> {
    let span = name.span();
    let name = name.value();

    assert_eq!(&name[..2], "m.");

    let path = name[2..].split('.').collect::<Vec<_>>();

    let event_str = path.last().unwrap();
    let event = event_str
        .split('_')
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect::<String>();

    let content_str = Ident::new(&format!("{}EventContent", event), span);
    let path = path.iter().map(|s| Ident::new(s, span));
    syn::parse_quote! {
        ::ruma_events::#( #path )::*::raw::#content_str
    }
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
