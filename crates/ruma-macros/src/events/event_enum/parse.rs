//! Parsing helpers specific to the `event_enum!` macro.

use std::collections::BTreeMap;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Ident, Path, Token,
};

use crate::events::enums::{EventKind, EventType, EventTypes, EventVariation};

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
                        room_event.types.ev_type == global_event.types.ev_type
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
    pub types: EventTypes,
    pub ev_path: Path,
    pub ident: Ident,
    pub both_account_data: bool,
}

impl EventEnumEntry {
    /// Whether this entry has a type fragment.
    pub fn has_type_fragment(&self) -> bool {
        self.types.ev_type.is_prefix()
    }

    /// Convert this entry to an enum variant.
    pub fn to_variant(&self) -> EventEnumVariant {
        let attrs = self.attrs.clone();
        let ident = self.ident.clone();

        EventEnumVariant { attrs, ident }
    }

    /// Get or generate the path of the event type for this entry.
    pub fn to_event_path(&self, kind: EventKind, var: EventVariation) -> TokenStream {
        let path = &self.ev_path;
        let ident = &self.ident;

        let type_prefix = match kind {
            EventKind::ToDevice => "ToDevice",
            EventKind::GlobalAccountData if self.both_account_data => "Global",
            EventKind::RoomAccountData if self.both_account_data => "Room",
            EventKind::State
                if self
                    .types
                    .stable_type()
                    .is_some_and(|ev_type| ev_type.as_str() == "m.room.encrypted") =>
            {
                "State"
            }
            _ => "",
        };
        let event_name = format_ident!("{var}{type_prefix}{ident}Event");

        quote! { #path::#event_name }
    }

    /// Get or generate the path of the event content type for this entry.
    pub fn to_event_content_path(&self, kind: EventKind) -> TokenStream {
        let path = &self.ev_path;
        let ident = &self.ident;

        let type_prefix = match kind {
            EventKind::ToDevice => "ToDevice",
            EventKind::State
                if self
                    .types
                    .stable_type()
                    .is_some_and(|ev_type| ev_type.as_str() == "m.room.encrypted") =>
            {
                "State"
            }
            _ => "",
        };
        let content_name = format_ident!("{type_prefix}{ident}EventContent");

        quote! {
            #path::#content_name
        }
    }

    /// Generate the docs for this entry.
    pub fn docs(&self) -> TokenStream {
        let main_type = self.types.main_type();

        let mut doc = quote! {
            #[doc = #main_type]
        };

        if self.types.ev_type != *main_type {
            let unstable_name =
                format!("This variant uses the unstable type `{}`.", self.types.ev_type);

            doc.extend(quote! {
                #[doc = ""]
                #[doc = #unstable_name]
            });
        }

        let aliases = &self.types.aliases;
        match aliases.len() {
            0 => {}
            1 => {
                let alias = format!(
                    "This variant can also be deserialized from the `{}` type.",
                    aliases[0]
                );
                doc.extend(quote! {
                    #[doc = ""]
                    #[doc = #alias]
                });
            }
            _ => {
                let aliases = format!(
                    "This variant can also be deserialized from the following types: {}.",
                    aliases.iter().map(|alias| format!("`{alias}`")).collect::<Vec<_>>().join(", ")
                );
                doc.extend(quote! {
                    #[doc = ""]
                    #[doc = #aliases]
                });
            }
        }

        doc
    }
}

impl Parse for EventEnumEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let (ruma_enum_attrs, attrs) = input
            .call(Attribute::parse_outer)?
            .into_iter()
            .partition::<Vec<_>, _>(|attr| attr.path().is_ident("ruma_enum"));
        let ev_type: EventType = input.parse()?;
        let _: Token![=>] = input.parse()?;
        let ev_path = input.call(Path::parse_mod_style)?;

        let mut aliases = Vec::with_capacity(ruma_enum_attrs.len());
        let mut ident = None;

        for attr_list in ruma_enum_attrs {
            for attr in attr_list
                .parse_args_with(Punctuated::<EventEnumAttr, Token![,]>::parse_terminated)?
            {
                match attr {
                    EventEnumAttr::Alias(alias) => {
                        aliases.push(alias);
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

        let types = EventTypes::try_from_parts(ev_type, aliases)?;

        // We will need the name of the event type so compute it right now to make sure that we have
        // enough data for it.
        let ident = if let Some(ident) = ident { ident } else { types.as_event_ident()? };

        Ok(Self { attrs, types, ev_path, ident, both_account_data: false })
    }
}

/// An attribute on an event entry in the `event_enum!` macro.
pub enum EventEnumAttr {
    Alias(EventType),
    Ident(Ident),
}

impl Parse for EventEnumAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::alias) {
            let _: kw::alias = input.parse()?;
            let _: Token![=] = input.parse()?;
            let s: EventType = input.parse()?;
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
