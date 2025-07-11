//! Parsing helpers specific to the `event_enum!` macro.

use std::{collections::BTreeMap, fmt};

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, IdentFragment, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Ident, LitStr, Path, Token,
};

use crate::{
    events::enums::{EventKind, EventKindVariation},
    util::m_prefix_name_to_type_name,
};

/// Custom keywords for the `event_enum!` macro
mod kw {
    syn::custom_keyword!(alias);
    syn::custom_keyword!(ident);
}

/// The entire `event_enum!` macro structure directly as it appears in the source code.
pub struct EventEnumInput {
    pub enums: Vec<EventEnumDecl>,
}

impl Parse for EventEnumInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut enums_map = BTreeMap::new();
        while !input.is_empty() {
            let attrs = input.call(Attribute::parse_outer)?;

            let _: Token![enum] = input.parse()?;
            let kind: EventKind = input.parse()?;

            let content;
            braced!(content in input);
            let events = content.parse_terminated(EventEnumEntry::parse, Token![,])?;
            let events = events.into_iter().collect();

            if enums_map.insert(kind, EventEnumDecl { attrs, kind, events }).is_some() {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("duplicate definition for kind `{kind:?}`"),
                ));
            }
        }

        // Mark event types which are declared for both account data kinds, because they use a
        // different name for the event struct.
        let mut room_account_data_enum = enums_map.remove(&EventKind::RoomAccountData);
        if let Some((global_account_data_enum, room_account_data_enum)) =
            enums_map.get_mut(&EventKind::GlobalAccountData).zip(room_account_data_enum.as_mut())
        {
            for global_event in global_account_data_enum.events.iter_mut() {
                if let Some(room_event) =
                    room_account_data_enum.events.iter_mut().find(|room_event| {
                        room_event.ev_type == global_event.ev_type
                            && room_event.ev_path == global_event.ev_path
                            && room_event.ident == global_event.ident
                    })
                {
                    global_event.both_account_data = true;
                    room_event.both_account_data = true;
                }
            }
        }
        if let Some(room_account_data_enum) = room_account_data_enum {
            enums_map.insert(EventKind::RoomAccountData, room_account_data_enum);
        }

        Ok(EventEnumInput { enums: enums_map.into_values().collect() })
    }
}

/// The declaration for a specific [`EventKind`] in the `event_enum!` macro.
pub struct EventEnumDecl {
    /// Outer attributes on the declaration, such as a docstring.
    pub attrs: Vec<Attribute>,

    /// The event kind.
    pub kind: EventKind,

    /// The event types for this kind.
    pub events: Vec<EventEnumEntry>,
}

/// An event entry in the `event_enum!` macro.
pub struct EventEnumEntry {
    pub attrs: Vec<Attribute>,
    pub aliases: Vec<LitStr>,
    pub ev_type: LitStr,
    pub ev_path: Path,
    pub ident: Option<Ident>,
    pub both_account_data: bool,
}

impl EventEnumEntry {
    /// Whether this entry has a type fragment.
    pub fn has_type_fragment(&self) -> bool {
        self.ev_type.value().ends_with(".*")
    }

    /// Convert this entry to an enum variant.
    pub fn to_variant(&self) -> syn::Result<EventEnumVariant> {
        let attrs = self.attrs.clone();
        let ident = self.ident()?;

        Ok(EventEnumVariant { attrs, ident })
    }

    /// Get the stable event type of this entry.
    pub fn stable_name(&self) -> syn::Result<&LitStr> {
        if self.ev_type.value().starts_with("m.") {
            Ok(&self.ev_type)
        } else {
            self.aliases.iter().find(|alias| alias.value().starts_with("m.")).ok_or_else(|| {
                syn::Error::new(
                    Span::call_site(),
                    format!(
                        "A matrix event must declare a well-known type that starts with `m.` \
                        either as the main type or as an alias, or must declare the ident that \
                        should be used if it is only an unstable type, found main type `{}`",
                        self.ev_type.value()
                    ),
                )
            })
        }
    }

    /// Get or generate the name of the event type for this entry.
    pub fn ident(&self) -> syn::Result<Ident> {
        if let Some(ident) = self.ident.clone() {
            Ok(ident)
        } else {
            m_prefix_name_to_type_name(self.stable_name()?)
        }
    }

    /// Get or generate the path of the event type for this entry.
    pub fn to_event_path(&self, kind: EventKind, var: EventEnumVariation) -> TokenStream {
        let path = &self.ev_path;
        let ident = self.ident().unwrap();
        let event_name = if kind == EventKind::ToDevice {
            assert_eq!(var, EventEnumVariation::None);
            format_ident!("ToDevice{ident}Event")
        } else {
            let type_prefix = match kind {
                EventKind::GlobalAccountData if self.both_account_data => "Global",
                EventKind::RoomAccountData if self.both_account_data => "Room",
                _ => "",
            };

            format_ident!("{}{type_prefix}{ident}Event", var)
        };
        quote! { #path::#event_name }
    }

    /// Get or generate the path of the event content type for this entry.
    pub fn to_event_content_path(&self, kind: EventKind, prefix: Option<&str>) -> TokenStream {
        let path = &self.ev_path;
        let ident = self.ident().unwrap();
        let content_str = match kind {
            EventKind::ToDevice => {
                format_ident!("ToDevice{}{ident}EventContent", prefix.unwrap_or(""))
            }
            _ => format_ident!("{}{ident}EventContent", prefix.unwrap_or("")),
        };

        quote! {
            #path::#content_str
        }
    }

    /// Generate the docs for this entry.
    pub fn docs(&self) -> syn::Result<TokenStream> {
        let main_name = self.stable_name().unwrap_or(&self.ev_type);

        let mut doc = quote! {
            #[doc = #main_name]
        };

        if self.ev_type != *main_name {
            let unstable_name =
                format!("This variant uses the unstable type `{}`.", self.ev_type.value());

            doc.extend(quote! {
                #[doc = ""]
                #[doc = #unstable_name]
            });
        }

        match self.aliases.len() {
            0 => {}
            1 => {
                let alias = format!(
                    "This variant can also be deserialized from the `{}` type.",
                    self.aliases[0].value()
                );
                doc.extend(quote! {
                    #[doc = ""]
                    #[doc = #alias]
                });
            }
            _ => {
                let aliases = format!(
                    "This variant can also be deserialized from the following types: {}.",
                    self.aliases
                        .iter()
                        .map(|alias| format!("`{}`", alias.value()))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                doc.extend(quote! {
                    #[doc = ""]
                    #[doc = #aliases]
                });
            }
        }

        Ok(doc)
    }
}

impl Parse for EventEnumEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let (ruma_enum_attrs, attrs) = input
            .call(Attribute::parse_outer)?
            .into_iter()
            .partition::<Vec<_>, _>(|attr| attr.path().is_ident("ruma_enum"));
        let ev_type: LitStr = input.parse()?;
        let _: Token![=>] = input.parse()?;
        let ev_path = input.call(Path::parse_mod_style)?;
        let has_suffix = ev_type.value().ends_with(".*");

        let mut aliases = Vec::with_capacity(ruma_enum_attrs.len());
        let mut ident = None;

        for attr_list in ruma_enum_attrs {
            for attr in attr_list
                .parse_args_with(Punctuated::<EventEnumAttr, Token![,]>::parse_terminated)?
            {
                match attr {
                    EventEnumAttr::Alias(alias) => {
                        if alias.value().ends_with(".*") == has_suffix {
                            aliases.push(alias);
                        } else {
                            return Err(syn::Error::new_spanned(
                                &attr_list,
                                "aliases should have the same `.*` suffix, or lack thereof, as the main event type",
                            ));
                        }
                    }
                    EventEnumAttr::Ident(i) => {
                        if ident.is_some() {
                            return Err(syn::Error::new_spanned(
                                &attr_list,
                                "multiple `ident` attributes found, there can be only one",
                            ));
                        }

                        ident = Some(i);
                    }
                }
            }
        }

        Ok(Self { attrs, aliases, ev_type, ev_path, ident, both_account_data: false })
    }
}

/// An attribute on an event entry in the `event_enum!` macro.
pub enum EventEnumAttr {
    Alias(LitStr),
    Ident(Ident),
}

impl Parse for EventEnumAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::alias) {
            let _: kw::alias = input.parse()?;
            let _: Token![=] = input.parse()?;
            let s: LitStr = input.parse()?;
            Ok(Self::Alias(s))
        } else if lookahead.peek(kw::ident) {
            let _: kw::ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            let i: Ident = input.parse()?;
            Ok(Self::Ident(i))
        } else {
            Err(lookahead.error())
        }
    }
}

/// A variant of an event enum.
pub(crate) struct EventEnumVariant {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
}

impl EventEnumVariant {
    /// Convert this variant to a `TokenStream`.
    pub fn to_tokens<T>(&self, prefix: Option<T>, with_attrs: bool) -> TokenStream
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
            tokens.extend(quote! { #p :: });
        }
        self.ident.to_tokens(&mut tokens);

        tokens
    }

    /// Generate the declaration for this variant.
    pub fn decl(&self) -> TokenStream {
        self.to_tokens::<TokenStream>(None, true)
    }

    /// Generate the match arm for this variant, with the given prefix.
    pub fn match_arm(&self, prefix: impl ToTokens) -> TokenStream {
        self.to_tokens(Some(prefix), true)
    }

    /// Generate the constructor for this variant, with the given prefix.
    pub fn ctor(&self, prefix: impl ToTokens) -> TokenStream {
        self.to_tokens(Some(prefix), false)
    }
}

/// The possible variations of an event enum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventEnumVariation {
    None,
    Sync,
    Stripped,
    Initial,
}

impl From<EventEnumVariation> for EventKindVariation {
    fn from(v: EventEnumVariation) -> Self {
        match v {
            EventEnumVariation::None => Self::None,
            EventEnumVariation::Sync => Self::Sync,
            EventEnumVariation::Stripped => Self::Stripped,
            EventEnumVariation::Initial => Self::Initial,
        }
    }
}

impl EventEnumVariation {
    pub fn to_sync(self) -> Self {
        match self {
            EventEnumVariation::None => EventEnumVariation::Sync,
            _ => panic!("No sync form of {self:?}"),
        }
    }

    pub fn to_full(self) -> Self {
        match self {
            EventEnumVariation::Sync => EventEnumVariation::None,
            _ => panic!("No full form of {self:?}"),
        }
    }
}

impl IdentFragment for EventEnumVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventEnumVariation::None => write!(f, ""),
            EventEnumVariation::Sync => write!(f, "Sync"),
            EventEnumVariation::Stripped => write!(f, "Stripped"),
            EventEnumVariation::Initial => write!(f, "Initial"),
        }
    }
}
