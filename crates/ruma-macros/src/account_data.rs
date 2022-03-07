use std::str::FromStr;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    Data, DataStruct, DeriveInput, Fields, FieldsNamed, Ident, LitStr, Token,
};

use crate::util::import_ruma_common;

mod enums;

pub use enums::{expand_account_data_enums, AccountDataEnumDecl};

/// Create an `AccountDataContent` implementation for a struct.
pub fn expand_account_data_content(
    input: &DeriveInput,
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let content_attr = input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("account_data"))
        .map(|attr| attr.parse_args::<MetaAttrs>())
        .collect::<syn::Result<Vec<_>>>()?;

    let mut data_types: Vec<_> =
        content_attr.iter().filter_map(|attrs| attrs.find_type()).collect();
    let data_type = match data_types.as_slice() {
        [] => {
            return Err(syn::Error::new(
                Span::call_site(),
                "no type attribute found, please add
                 `#[account_data(type = \"m.someting\")]`",
            ));
        }
        [_] => data_types.pop().unwrap(),
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "multiple type attribute found, there can only be one",
            ));
        }
    };

    let mut kinds: Vec<_> = content_attr.iter().filter_map(|attrs| attrs.find_kind()).collect();
    let kind = match kinds.as_slice() {
        [] => {
            return Err(syn::Error::new(
                Span::call_site(),
                "no kind attribute found, please add
                 `#[account_data(kind = Kind)]`",
            ));
        }
        [_] => Some(kinds.pop().unwrap()),
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "multiple kind attribute found, there can only be one",
            ));
        }
    };

    let main_impl = generate_account_data_content_impl(&input.ident, data_type, ruma_common);
    let marker_trait_impl = kind.map(|k| expand_marker_trait_impl(k, &input.ident, ruma_common));
    let type_aliases = kind
        .map(|k| {
            generate_account_data_type_aliases(k, &input.ident, &data_type.value(), ruma_common)
        })
        .transpose()?;

    Ok(quote! {
        #main_impl
        #marker_trait_impl
        #type_aliases
    })
}

mod kw {
    syn::custom_keyword!(kind);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountDataKind {
    Global,
    Room,
}

impl AccountDataKind {
    fn to_struct_name(self) -> Ident {
        match self {
            Self::Global => format_ident!("GlobalAccountData"),
            Self::Room => format_ident!("RoomAccountData"),
        }
    }

    fn to_enum_name(self) -> Ident {
        format_ident!("Any{}", self.to_struct_name())
    }

    fn to_content_enum_name(self) -> Ident {
        format_ident!("{}Content", self.to_enum_name())
    }
}

impl FromStr for AccountDataKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Global" => Ok(AccountDataKind::Global),
            "Room" => Ok(AccountDataKind::Room),
            _ => Err(()),
        }
    }
}

impl Parse for AccountDataKind {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let id = ident.to_string();

        id.parse().map_err(|_| {
            syn::Error::new_spanned(
                ident,
                format!("valid account data kinds are Global and Room, found `{}`", id),
            )
        })
    }
}

/// Parses attributes for `AccountDataContent` derives.
///
/// `#[account_data(type = "m.room.alias", kind = Kind)]`
enum AccountDataMeta {
    Type(LitStr),
    Kind(AccountDataKind),
}

impl AccountDataMeta {
    fn data_type(&self) -> Option<&LitStr> {
        match self {
            Self::Type(t) => Some(t),
            _ => None,
        }
    }

    fn data_kind(&self) -> Option<AccountDataKind> {
        match self {
            Self::Kind(k) => Some(*k),
            _ => None,
        }
    }
}

impl Parse for AccountDataMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![type]) {
            let _: Token![type] = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(AccountDataMeta::Type)
        } else if lookahead.peek(kw::kind) {
            let _: kw::kind = input.parse()?;
            let _: Token![=] = input.parse()?;
            AccountDataKind::parse(input).map(AccountDataMeta::Kind)
        } else {
            Err(lookahead.error())
        }
    }
}

struct MetaAttrs(Vec<AccountDataMeta>);

impl MetaAttrs {
    fn find_type(&self) -> Option<&LitStr> {
        self.0.iter().find_map(|a| a.data_type())
    }

    fn find_kind(&self) -> Option<AccountDataKind> {
        self.0.iter().find_map(|a| a.data_kind())
    }
}

impl Parse for MetaAttrs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs =
            syn::punctuated::Punctuated::<AccountDataMeta, Token![,]>::parse_terminated(input)?;
        Ok(Self(attrs.into_iter().collect()))
    }
}

fn generate_account_data_content_impl(
    ident: &Ident,
    data_type: &LitStr,
    ruma_common: &TokenStream,
) -> TokenStream {
    let serde = quote! { #ruma_common::exports::serde };
    let serde_json = quote! { #ruma_common::exports::serde_json };

    quote! {
        #[automatically_derived]
        impl #ruma_common::account_data::AccountDataContent for #ident {
            fn data_type(&self) -> &str {
                #data_type
            }

            fn from_parts(
                ev_type: &str,
                content: &#serde_json::value::RawValue,
            ) -> #serde_json::Result<Self> {
                if ev_type != #data_type {
                    return Err(#serde::de::Error::custom(
                        format!("expected type `{}`, found `{}`", #data_type, ev_type)
                    ));
                }

                #serde_json::from_str(content.get())
            }
        }
    }
}

fn generate_account_data_type_aliases(
    data_kind: AccountDataKind,
    ident: &Ident,
    data_type: &str,
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let alias = Ident::new(
        ident.to_string().strip_suffix("Content").ok_or_else(|| {
            syn::Error::new_spanned(ident, "Expected content struct name ending in `Content`")
        })?,
        Span::call_site(),
    );

    let generic_struct = data_kind.to_struct_name();
    let alias_docs = format!("An `{}` object.", data_type);

    Ok(quote! {
        #[doc = #alias_docs]
        pub type #alias = #ruma_common::account_data::#generic_struct<#ident>;
    })
}

fn expand_marker_trait_impl(
    data_kind: AccountDataKind,
    ident: &Ident,
    ruma_common: &TokenStream,
) -> TokenStream {
    let marker_trait = match data_kind {
        AccountDataKind::Global => quote! { GlobalAccountDataContent },
        AccountDataKind::Room => quote! { RoomAccountDataContent },
    };

    quote! {
        #[automatically_derived]
        impl #ruma_common::account_data::#marker_trait for #ident {}
    }
}

/// Derive `AccountData` macro code generation.
pub fn expand_account_data(input: DeriveInput) -> syn::Result<TokenStream> {
    let ruma_common = import_ruma_common();

    let ident = &input.ident;
    let kind: AccountDataKind = match ident.to_string().strip_suffix("AccountData") {
        Some(kind) => kind.parse().map_err(|_| {
            syn::Error::new_spanned(ident, "expected one of `GlobalAccountData`, `RoomAccountData`")
        })?,
        None => return Err(syn::Error::new_spanned(ident, "expected an `AccountData` suffix")),
    };

    if let Data::Struct(DataStruct { fields: Fields::Named(FieldsNamed { named, .. }), .. }) =
        &input.data
    {
        if named.len() != 1 {
            return Err(syn::Error::new(Span::call_site(), "expected a single `content` field"));
        }

        let ident = &named[0].ident.as_ref().unwrap();
        if *ident != "content" {
            return Err(syn::Error::new_spanned(ident, "expected `content` field"));
        }
    } else {
        return Err(syn::Error::new_spanned(
            input.ident,
            "the `AccountData` derive only supports structs with named fields",
        ));
    };

    let serialize = expand_serialize_account_data(&input, &ruma_common);
    let deserialize = expand_deserialize_account_data(&input, kind, &ruma_common)?;

    Ok(quote! {
        #serialize
        #deserialize
    })
}

fn expand_serialize_account_data(input: &DeriveInput, ruma_common: &TokenStream) -> TokenStream {
    let serde = quote! { #ruma_common::exports::serde };

    let ident = &input.ident;
    let (impl_gen, ty_gen, where_clause) = input.generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_gen #serde::ser::Serialize for #ident #ty_gen #where_clause {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: #serde::ser::Serializer,
            {
                use #serde::ser::{SerializeStruct as _, Error as _};

                let data_type =
                    #ruma_common::account_data::AccountDataContent::data_type(&self.content);

                let mut state = serializer.serialize_struct(stringify!(#ident), 2)?;
                state.serialize_field("type", data_type)?;
                state.serialize_field("content", &self.content)?;
                state.end()
            }
        }
    }
}

fn expand_deserialize_account_data(
    input: &DeriveInput,
    _kind: AccountDataKind,
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let serde = quote! { #ruma_common::exports::serde };
    let serde_json = quote! { #ruma_common::exports::serde_json };

    let ident = &input.ident;

    let (impl_generics, ty_gen, where_clause) = input.generics.split_for_impl();
    let is_generic = !input.generics.params.is_empty();

    let deserialize_impl_gen = if is_generic {
        let gen = &input.generics.params;
        quote! { <'de, #gen> }
    } else {
        quote! { <'de> }
    };
    let deserialize_phantom_type = if is_generic {
        quote! { ::std::marker::PhantomData }
    } else {
        quote! {}
    };

    Ok(quote! {
        #[automatically_derived]
        impl #deserialize_impl_gen #serde::de::Deserialize<'de> for #ident #ty_gen #where_clause {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: #serde::de::Deserializer<'de>,
            {
                #[derive(#serde::Deserialize)]
                #[serde(field_identifier, rename_all = "snake_case")]
                enum Field {
                    Type,
                    Content,
                    #[serde(other)]
                    Unknown,
                }

                /// Visits the fields of an account data struct to handle deserialization of
                /// the `content` and `prev_content` fields.
                struct AccountDataVisitor #impl_generics (#deserialize_phantom_type #ty_gen);

                #[automatically_derived]
                impl #deserialize_impl_gen #serde::de::Visitor<'de>
                    for AccountDataVisitor #ty_gen #where_clause
                {
                    type Value = #ident #ty_gen;

                    fn expecting(
                        &self,
                        formatter: &mut ::std::fmt::Formatter<'_>,
                    ) -> ::std::fmt::Result {
                        write!(formatter, "a struct with type and content fields")
                    }

                    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                    where
                        A: #serde::de::MapAccess<'de>,
                    {
                        use #serde::de::Error as _;

                        let mut data_type: ::std::option::Option<::std::string::String> =
                            ::std::option::Option::None;
                        let mut content: ::std::option::Option<
                            ::std::boxed::Box<#serde_json::value::RawValue>,
                        > = ::std::option::Option::None;

                        while let Some(key) = map.next_key()? {
                            match key {
                                Field::Unknown => {
                                    let _: #serde::de::IgnoredAny = map.next_value()?;
                                },
                                Field::Type => {
                                    if data_type.is_some() {
                                        return Err(#serde::de::Error::duplicate_field("type"));
                                    }
                                    data_type = Some(map.next_value()?);
                                }
                                Field::Content => {
                                    if content.is_some() {
                                        return Err(#serde::de::Error::duplicate_field("content"));
                                    }
                                    content = Some(map.next_value()?);
                                }
                            }
                        }

                        let data_type =
                            data_type.ok_or_else(|| #serde::de::Error::missing_field("type"))?;
                        let content = {
                            let json = content
                                .ok_or_else(|| #serde::de::Error::missing_field("content"))?;
                            C::from_parts(&data_type, &json).map_err(A::Error::custom)?
                        };

                        Ok(#ident { content })
                    }
                }

                deserializer.deserialize_map(AccountDataVisitor(#deserialize_phantom_type))
            }
        }
    })
}
