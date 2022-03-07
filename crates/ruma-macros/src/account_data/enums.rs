use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    braced,
    parse::{self, Parse, ParseStream},
    Attribute, Ident, LitStr, Token,
};

use super::{expand_marker_trait_impl, AccountDataKind};
use crate::util::{import_ruma_common, m_prefix_name_to_type_name};

pub struct AccountDataEnumEntry {
    pub attrs: Vec<Attribute>,
    pub data_type: LitStr,
}

impl AccountDataEnumEntry {
    fn to_variant(&self) -> syn::Result<AccountDataEnumVariant> {
        let attrs = self.attrs.clone();
        let ident = m_prefix_name_to_type_name(&self.data_type)?;
        Ok(AccountDataEnumVariant { attrs, ident })
    }
}

impl Parse for AccountDataEnumEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self { attrs: input.call(Attribute::parse_outer)?, data_type: input.parse()? })
    }
}

struct AccountDataEnumVariant {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
}

impl AccountDataEnumVariant {
    pub(crate) fn to_tokens<T>(&self, prefix: Option<T>, with_attrs: bool) -> TokenStream
    where
        T: ToTokens,
    {
        let mut tokens = TokenStream::new();
        if with_attrs {
            for attr in &self.attrs {
                attr.to_tokens(&mut tokens);
            }
        }
        if let Some(p) = prefix {
            tokens.extend(quote! { #p :: })
        }
        self.ident.to_tokens(&mut tokens);

        tokens
    }

    pub(crate) fn decl(&self) -> TokenStream {
        self.to_tokens::<TokenStream>(None, true)
    }

    pub(crate) fn match_arm(&self, prefix: impl ToTokens) -> TokenStream {
        self.to_tokens(Some(prefix), true)
    }

    pub(crate) fn ctor(&self, prefix: impl ToTokens) -> TokenStream {
        self.to_tokens(Some(prefix), false)
    }
}

/// The entire `event_enum!` macro structure directly as it appears in the source code.
pub struct AccountDataEnumDecl {
    /// Outer attributes on the field, such as a docstring.
    pub attrs: Vec<Attribute>,

    /// The event kind.
    pub kind: AccountDataKind,

    /// An array of valid matrix event types.
    ///
    /// This will generate the variants of the event type "kind".
    pub events: Vec<AccountDataEnumEntry>,
}

impl Parse for AccountDataEnumDecl {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let _: Token![enum] = input.parse()?;
        let ident: Ident = input.parse()?;
        let kind = match ident.to_string().strip_suffix("AccountData") {
            Some(kind) => kind.parse().map_err(|_| {
                syn::Error::new_spanned(
                    ident,
                    "expected one of `GlobalAccountData`, `RoomAccountData`",
                )
            })?,
            None => return Err(syn::Error::new_spanned(ident, "expected an `AccountData` suffix")),
        };

        let content;
        braced!(content in input);
        let events = content.parse_terminated::<_, Token![,]>(AccountDataEnumEntry::parse)?;
        let events = events.into_iter().collect();
        Ok(AccountDataEnumDecl { attrs, kind, events })
    }
}

/// Create a content enum.
pub fn expand_account_data_enums(input: &AccountDataEnumDecl) -> syn::Result<TokenStream> {
    let ruma_common = import_ruma_common();

    let kind = input.kind;
    let attrs = &input.attrs;
    let events: Vec<_> = input.events.iter().map(|entry| entry.data_type.clone()).collect();
    let variants: Vec<_> =
        input.events.iter().map(AccountDataEnumEntry::to_variant).collect::<syn::Result<_>>()?;

    let events = &events;
    let variants = &variants;
    let ruma_common = &ruma_common;

    let main_enum = expand_account_data_enum(kind, events, attrs, variants, ruma_common);
    let content_enum = expand_content_enum(kind, events, attrs, variants, ruma_common);
    Ok(quote! {
        #main_enum
        #content_enum
    })
}

fn expand_account_data_enum(
    kind: AccountDataKind,
    events: &[LitStr],
    attrs: &[Attribute],
    variants: &[AccountDataEnumVariant],
    ruma_common: &TokenStream,
) -> TokenStream {
    let event_struct = kind.to_struct_name();
    let ident = kind.to_enum_name();

    let variant_decls = variants.iter().map(|v| v.decl());
    let content: Vec<_> =
        events.iter().map(|event| to_account_data_path(event, ruma_common)).collect();

    let deserialize_impl = expand_deserialize_impl(kind, events, variants, ruma_common);
    let field_accessor_impl = expand_accessor_methods(kind, variants, ruma_common);
    let from_impl = expand_from_impl(&ident, &content, variants);

    quote! {
        #( #attrs )*
        #[derive(Clone, Debug)]
        #[allow(clippy::large_enum_variant)]
        #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
        pub enum #ident {
            #(
                #[doc = #events]
                #variant_decls(#content),
            )*
            /// An event not defined by the Matrix specification
            #[doc(hidden)]
            _Custom(
                #ruma_common::account_data::#event_struct<
                    #ruma_common::account_data::_custom::CustomAccountDataContent,
                >,
            ),
        }

        #deserialize_impl
        #field_accessor_impl
        #from_impl
    }
}

/// Create an account data content enum.
fn expand_content_enum(
    kind: AccountDataKind,
    events: &[LitStr],
    attrs: &[Attribute],
    variants: &[AccountDataEnumVariant],
    ruma_common: &TokenStream,
) -> TokenStream {
    let serde = quote! { #ruma_common::exports::serde };
    let serde_json = quote! { #ruma_common::exports::serde_json };

    let ident = kind.to_content_enum_name();

    let data_type_str = events;

    let content: Vec<_> =
        events.iter().map(|ev| to_account_data_content_path(ev, None, ruma_common)).collect();

    let variant_decls = variants.iter().map(|v| v.decl()).collect::<Vec<_>>();

    let variant_attrs = variants.iter().map(|v| {
        let attrs = &v.attrs;
        quote! { #(#attrs)* }
    });
    let variant_arms = variants.iter().map(|v| v.match_arm(quote! { Self })).collect::<Vec<_>>();
    let variant_ctors = variants.iter().map(|v| v.ctor(quote! { Self }));

    let marker_trait_impl = expand_marker_trait_impl(kind, &ident, ruma_common);
    let from_impl = expand_from_impl(&ident, &content, variants);

    let serialize_custom_event_error_path =
        quote! { #ruma_common::events::serialize_custom_event_error }.to_string();

    quote! {
        #( #attrs )*
        #[derive(Clone, Debug, #serde::Serialize)]
        #[serde(untagged)]
        #[allow(clippy::large_enum_variant)]
        #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
        pub enum #ident {
            #(
                #[doc = #data_type_str]
                #variant_decls(#content),
            )*
            #[doc(hidden)]
            #[serde(serialize_with = #serialize_custom_event_error_path)]
            _Custom {
                data_type: crate::PrivOwnedStr,
            },
        }

        #[automatically_derived]
        impl #ruma_common::account_data::AccountDataContent for #ident {
            fn data_type(&self) -> &::std::primitive::str {
                match self {
                    #( #variant_arms(content) => content.data_type(), )*
                    Self::_Custom { data_type } => &data_type.0,
                }
            }

            fn from_parts(
                data_type: &::std::primitive::str,
                input: &#serde_json::value::RawValue,
            ) -> #serde_json::Result<Self> {
                match data_type {
                    #(
                        #variant_attrs #data_type_str => {
                            let content = #content::from_parts(data_type, input)?;
                            ::std::result::Result::Ok(#variant_ctors(content))
                        }
                    )*
                    ty => {
                        ::std::result::Result::Ok(Self::_Custom {
                            data_type: crate::PrivOwnedStr(ty.into()),
                        })
                    }
                }
            }
        }

        #marker_trait_impl
        #from_impl
    }
}

fn expand_accessor_methods(
    kind: AccountDataKind,
    variants: &[AccountDataEnumVariant],
    ruma_common: &TokenStream,
) -> TokenStream {
    let ident = kind.to_enum_name();
    let self_variants: Vec<_> = variants.iter().map(|v| v.match_arm(quote! { Self })).collect();

    let content_accessors = {
        let content_enum = kind.to_content_enum_name();
        let content_variants: Vec<_> = variants.iter().map(|v| v.ctor(&content_enum)).collect();

        quote! {
            /// Returns the content for this event.
            pub fn content(&self) -> #content_enum {
                match self {
                    #( #self_variants(event) => #content_variants(event.content.clone()), )*
                    Self::_Custom(event) => #content_enum::_Custom {
                        data_type: crate::PrivOwnedStr(
                            #ruma_common::account_data::AccountDataContent::data_type(
                                &event.content,
                            ).into(),
                        ),
                    },
                }
            }
        }
    };

    quote! {
        #[automatically_derived]
        impl #ident {
            /// Returns the `type` of this event.
            pub fn data_type(&self) -> &::std::primitive::str {
                match self {
                    #( #self_variants(event) =>
                        #ruma_common::account_data::AccountDataContent::data_type(&event.content), )*
                    Self::_Custom(event) =>
                        #ruma_common::account_data::AccountDataContent::data_type(&event.content),
                }
            }

            #content_accessors
        }
    }
}

fn expand_deserialize_impl(
    kind: AccountDataKind,
    events: &[LitStr],
    variants: &[AccountDataEnumVariant],
    ruma_common: &TokenStream,
) -> TokenStream {
    let ruma_serde = quote! { #ruma_common::exports::ruma_serde };
    let serde = quote! { #ruma_common::exports::serde };
    let serde_json = quote! { #ruma_common::exports::serde_json };

    let ident = kind.to_enum_name();

    let variant_attrs = variants.iter().map(|v| {
        let attrs = &v.attrs;
        quote! { #(#attrs)* }
    });
    let self_variants = variants.iter().map(|v| v.ctor(quote! { Self }));
    let content = events.iter().map(|event| to_account_data_path(event, ruma_common));

    quote! {
        impl<'de> #serde::de::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: #serde::de::Deserializer<'de>,
            {
                use #serde::de::Error as _;

                let json = Box::<#serde_json::value::RawValue>::deserialize(deserializer)?;
                let #ruma_common::events::EventTypeDeHelper { ev_type, .. } =
                    #ruma_serde::from_raw_json_value(&json)?;

                match &*ev_type {
                    #(
                        #variant_attrs #events => {
                            let event = #serde_json::from_str::<#content>(json.get())
                                .map_err(D::Error::custom)?;
                            Ok(#self_variants(event))
                        },
                    )*
                    _ => {
                        let event = #serde_json::from_str(json.get()).map_err(D::Error::custom)?;
                        Ok(Self::_Custom(event))
                    },
                }
            }
        }
    }
}

fn expand_from_impl(
    ty: &Ident,
    content: &[TokenStream],
    variants: &[AccountDataEnumVariant],
) -> TokenStream {
    let from_impls = content.iter().zip(variants).map(|(content, variant)| {
        let ident = &variant.ident;
        let attrs = &variant.attrs;

        quote! {
            #[automatically_derived]
            #(#attrs)*
            impl ::std::convert::From<#content> for #ty {
                fn from(c: #content) -> Self {
                    Self::#ident(c)
                }
            }
        }
    });

    quote! { #( #from_impls )* }
}

fn to_account_data_path(name: &LitStr, ruma_common: &TokenStream) -> TokenStream {
    let span = name.span();
    let name = name.value();

    // There is no need to give a good compiler error as `to_camel_case` is called first.
    assert_eq!(&name[..2], "m.");

    let path: Vec<_> = name[2..].split('.').collect();

    let name: String = name[2..]
        .split(&['.', '_'] as &[char])
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect();

    let path = path.iter().map(|s| Ident::new(s, span));
    let ident = Ident::new(&name, Span::call_site());

    quote! { #ruma_common::account_data::#( #path )::*::#ident }
}

fn to_account_data_content_path(
    name: &LitStr,
    prefix: Option<&str>,
    ruma_common: &TokenStream,
) -> TokenStream {
    let span = name.span();
    let name = name.value();

    // There is no need to give a good compiler error as `to_camel_case` is called first.
    assert_eq!(&name[..2], "m.");

    let path: Vec<_> = name[2..].split('.').collect();

    let event: String = name[2..]
        .split(&['.', '_'] as &[char])
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect();

    let content_str = format_ident!("{}{}Content", prefix.unwrap_or(""), event);

    let path = path.iter().map(|s| Ident::new(s, span));

    quote! {
        #ruma_common::account_data::#( #path )::*::#content_str
    }
}
