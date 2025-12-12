use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use syn::{
    Attribute, Field, Ident, LitStr, meta::ParseNestedMeta, punctuated::Punctuated, visit::Visit,
};

/// The path to use for imports from the ruma-common crate.
///
/// To access a reexported crate, prefer to use the [`reexported()`](Self::reexported) method.
pub(crate) struct RumaCommon(TokenStream);

impl RumaCommon {
    /// Construct a new `RumaCommon`.
    pub(crate) fn new() -> Self {
        let inner = if let Ok(FoundCrate::Name(name)) = crate_name("ruma-common") {
            let import = format_ident!("{name}");
            quote! { ::#import }
        } else if let Ok(FoundCrate::Name(name)) = crate_name("ruma") {
            let import = format_ident!("{name}");
            quote! { ::#import }
        } else if let Ok(FoundCrate::Name(name)) = crate_name("matrix-sdk") {
            let import = format_ident!("{name}");
            quote! { ::#import::ruma }
        } else {
            quote! { ::ruma_common }
        };

        Self(inner)
    }

    /// The path to use for imports from the given reexported crate.
    pub(crate) fn reexported(&self, reexport: RumaCommonReexport) -> TokenStream {
        quote! { #self::exports::#reexport }
    }
}

impl ToTokens for RumaCommon {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

/// The crates reexported by ruma-common.
pub(crate) enum RumaCommonReexport {
    /// The ruma-macros crate.
    RumaMacros,

    /// The serde crate.
    Serde,

    /// The serde_html_form crate.
    SerdeHtmlForm,

    /// The serde_json crate.
    SerdeJson,

    /// The http crate.
    Http,

    /// The bytes crate.
    Bytes,
}

impl ToTokens for RumaCommonReexport {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_name = match self {
            Self::RumaMacros => "ruma_macros",
            Self::Serde => "serde",
            Self::SerdeHtmlForm => "serde_html_form",
            Self::SerdeJson => "serde_json",
            Self::Http => "http",
            Self::Bytes => "bytes",
        };

        tokens.append(Ident::new(crate_name, Span::call_site()));
    }
}

/// The path to use for imports from the ruma-events crate.
///
/// To access a reexported crate, prefer to use [`reexported()`](Self::reexported) or one of the
/// other methods.
pub(crate) struct RumaEvents(TokenStream);

impl RumaEvents {
    /// Construct a new `RumaEvents`.
    pub(crate) fn new() -> Self {
        let inner = if let Ok(FoundCrate::Name(name)) = crate_name("ruma-events") {
            let import = format_ident!("{name}");
            quote! { ::#import }
        } else if let Ok(FoundCrate::Name(name)) = crate_name("ruma") {
            let import = format_ident!("{name}");
            quote! { ::#import::events }
        } else if let Ok(FoundCrate::Name(name)) = crate_name("matrix-sdk") {
            let import = format_ident!("{name}");
            quote! { ::#import::ruma::events }
        } else {
            quote! { ::ruma_events }
        };

        Self(inner)
    }

    /// The path to use for imports from the given reexported crate.
    pub(crate) fn reexported(&self, reexport: RumaEventsReexport) -> TokenStream {
        quote! { #self::exports::#reexport }
    }

    /// The path to use for imports from the ruma-common crate.
    pub(crate) fn ruma_common(&self) -> RumaCommon {
        RumaCommon(quote! { #self::exports::ruma_common })
    }
}

impl ToTokens for RumaEvents {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

/// The crates reexported by ruma-events.
pub(crate) enum RumaEventsReexport {
    /// The serde crate.
    Serde,

    /// The serde_json crate.
    SerdeJson,
}

impl ToTokens for RumaEventsReexport {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_name = match self {
            Self::Serde => "serde",
            Self::SerdeJson => "serde_json",
        };

        tokens.append(Ident::new(crate_name, Span::call_site()));
    }
}

/// CamelCase's a field ident like "foo_bar" to "FooBar".
pub(crate) fn to_camel_case(name: &Ident) -> Ident {
    let span = name.span();
    let name = name.to_string();

    let s: String = name
        .split('_')
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect();
    Ident::new(&s, span)
}

/// Splits the given string on `.` and `_` removing the `m.` then camel casing to give a Rust type
/// name.
pub(crate) fn m_prefix_name_to_type_name(name: &LitStr) -> syn::Result<Ident> {
    let span = name.span();
    let name = name.value();

    let name = name.strip_prefix("m.").ok_or_else(|| {
        syn::Error::new(
            span,
            format!("well-known matrix events have to start with `m.` found `{name}`"),
        )
    })?;

    let s: String = name
        .strip_suffix(".*")
        .unwrap_or(name)
        .split(&['.', '_'] as &[char])
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect();

    Ok(Ident::new(&s, span))
}

/// Wrapper around [`syn::Field`] that emits the field without its visibility,
/// thus making it private.
pub struct PrivateField<'a>(pub &'a Field);

impl ToTokens for PrivateField<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Field { attrs, vis: _, mutability, ident, colon_token, ty } = self.0;
        assert_eq!(*mutability, syn::FieldMutability::None);

        for attr in attrs {
            attr.to_tokens(tokens);
        }
        ident.to_tokens(tokens);
        colon_token.to_tokens(tokens);
        ty.to_tokens(tokens);
    }
}

#[cfg(feature = "__internal_macro_expand")]
pub fn cfg_expand_struct(item: &mut syn::ItemStruct) {
    use std::mem;

    use proc_macro2::TokenTree;
    use syn::{Fields, LitBool, Meta, visit_mut::VisitMut};

    fn eval_cfg(cfg_expr: TokenStream) -> Option<bool> {
        let cfg_macro_call = quote! { ::core::cfg!(#cfg_expr) };
        let expanded = match proc_macro::TokenStream::from(cfg_macro_call).expand_expr() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to expand cfg! {e}");
                return None;
            }
        };

        let lit: LitBool = syn::parse(expanded).expect("cfg! must expand to a boolean literal");
        Some(lit.value())
    }

    fn tokentree_not_comma(tree: &TokenTree) -> bool {
        match tree {
            TokenTree::Punct(p) => p.as_char() != ',',
            _ => true,
        }
    }

    struct CfgAttrExpand;

    impl VisitMut for CfgAttrExpand {
        fn visit_attribute_mut(&mut self, attr: &mut Attribute) {
            if attr.meta.path().is_ident("cfg_attr") {
                // Ignore invalid cfg attributes
                let Meta::List(list) = &attr.meta else { return };
                let mut token_iter = list.tokens.clone().into_iter();

                // Take all the tokens until the first toplevel comma.
                // That's the cfg-expression part of cfg_attr.
                let cfg_expr: TokenStream =
                    token_iter.by_ref().take_while(tokentree_not_comma).collect();

                let Some(cfg_value) = eval_cfg(cfg_expr) else { return };
                if cfg_value {
                    // If we had the whole attribute list and could emit more
                    // than one attribute, we'd split the remaining arguments to
                    // cfg_attr by commas and turn them into regular attributes
                    //
                    // Because we can emit only one, do the first and error if
                    // there's any more after it.
                    let attr_tokens: TokenStream =
                        token_iter.by_ref().take_while(tokentree_not_comma).collect();

                    if attr_tokens.is_empty() {
                        // no-op cfg_attr??
                        return;
                    }

                    attr.meta = syn::parse2(attr_tokens)
                        .expect("syn must be able to parse cfg-attr arguments as syn::Meta");

                    let rest: TokenStream = token_iter.collect();
                    assert!(
                        rest.is_empty(),
                        "cfg_attr's with multiple arguments after the cfg expression are not \
                         currently supported by __internal_macro_expand."
                    );
                }
            }
        }
    }

    CfgAttrExpand.visit_item_struct_mut(item);

    let Fields::Named(fields) = &mut item.fields else {
        panic!("only named fields are currently supported by __internal_macro_expand");
    };

    // Take out all the fields
    'fields: for mut field in mem::take(&mut fields.named) {
        // Take out all the attributes
        for attr in mem::take(&mut field.attrs) {
            // For non-cfg attrs, put them back
            if !attr.meta.path().is_ident("cfg") {
                field.attrs.push(attr);
                continue;
            }

            // Also put back / ignore invalid cfg attributes
            let Meta::List(list) = &attr.meta else {
                field.attrs.push(attr);
                continue;
            };
            // Also put back / ignore cfg attributes we can't eval
            let Some(cfg_value) = eval_cfg(list.tokens.clone()) else {
                field.attrs.push(attr);
                continue;
            };

            // Finally, if the cfg is `false`, skip the part where it's put back
            if !cfg_value {
                continue 'fields;
            }
        }

        // If `continue 'fields` above wasn't hit, we didn't find a cfg that
        // evals to false, so put the field back
        fields.named.push(field);
    }
}

/// Helper trait for a [`syn::Field`] belonging to a `struct`.
pub(crate) trait StructFieldExt {
    /// Get a reference to the `ident` of this field.
    ///
    /// Panics if this is not a named field.
    fn ident(&self) -> &Ident;

    /// Get the `#[cfg]` attributes on this field.
    fn cfg_attrs(&self) -> impl Iterator<Item = &'_ Attribute>;

    /// Get the serde meta items on this field, if it has `#[serde(…)]` attributes.
    fn serde_meta_items(&self) -> impl Iterator<Item = syn::Meta>;

    /// Whether this field has a `#[serde(…)]` containing the given meta item.
    fn has_serde_meta_item(&self, meta: SerdeMetaItem) -> bool;
}

impl StructFieldExt for Field {
    fn ident(&self) -> &Ident {
        self.ident.as_ref().expect("struct field should be named")
    }

    fn cfg_attrs(&self) -> impl Iterator<Item = &'_ Attribute> {
        self.attrs.iter().filter(|a| a.path().is_ident("cfg"))
    }

    fn serde_meta_items(&self) -> impl Iterator<Item = syn::Meta> {
        self.attrs.iter().flat_map(AttributeExt::serde_meta_items)
    }

    fn has_serde_meta_item(&self, meta: SerdeMetaItem) -> bool {
        self.serde_meta_items().any(|serde_meta| serde_meta == meta)
    }
}

/// Possible meta items for `#[serde(…)]` attributes.
#[derive(Clone, Copy)]
pub(crate) enum SerdeMetaItem {
    /// `flatten`.
    Flatten,

    /// `default`.
    Default,

    /// `rename`.
    Rename,

    /// `alias`.
    Alias,
}

impl SerdeMetaItem {
    /// The string representation of this meta item.
    fn as_str(self) -> &'static str {
        match self {
            Self::Flatten => "flatten",
            Self::Default => "default",
            Self::Rename => "rename",
            Self::Alias => "alias",
        }
    }
}

impl PartialEq<SerdeMetaItem> for syn::Meta {
    fn eq(&self, other: &SerdeMetaItem) -> bool {
        self.path().is_ident(other.as_str())
    }
}

/// Helper trait for a [`syn::Attribute`].
pub(crate) trait AttributeExt {
    /// Get the list of meta items if this is a `#[serde(…)]` attribute.
    fn serde_meta_items(&self) -> impl Iterator<Item = syn::Meta>;
}

impl AttributeExt for Attribute {
    fn serde_meta_items(&self) -> impl Iterator<Item = syn::Meta> {
        if self.path().is_ident("serde")
            && let syn::Meta::List(list) = &self.meta
        {
            list.parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated).ok()
        } else {
            None
        }
        .into_iter()
        .flatten()
    }
}

/// Helper trait for a [`syn::Type`].
pub(crate) trait TypeExt {
    /// Get the inner type if this is wrapped in an `Option`.
    fn option_inner_type(&self) -> Option<&syn::Type>;

    /// Whether this type has a lifetime.
    fn has_lifetime(&self) -> bool;
}

impl TypeExt for syn::Type {
    fn option_inner_type(&self) -> Option<&syn::Type> {
        let syn::Type::Path(syn::TypePath { path: syn::Path { segments, .. }, .. }) = self else {
            return None;
        };

        if segments.last().unwrap().ident != "Option" {
            return None;
        }

        let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            args: option_args,
            ..
        }) = &segments.last().unwrap().arguments
        else {
            panic!("Option should use angle brackets");
        };
        let syn::GenericArgument::Type(inner_type) = option_args.first().unwrap() else {
            panic!("Option brackets should contain type");
        };

        Some(inner_type)
    }

    fn has_lifetime(&self) -> bool {
        struct Visitor {
            found_lifetime: bool,
        }

        impl<'ast> Visit<'ast> for Visitor {
            fn visit_lifetime(&mut self, _lt: &'ast syn::Lifetime) {
                self.found_lifetime = true;
            }
        }

        let mut vis = Visitor { found_lifetime: false };
        vis.visit_type(self);

        vis.found_lifetime
    }
}

/// Generate code for a list of struct fields.
///
/// If the fields have `cfg` attributes, they are also used.
///
/// This generates code looking like this for each field:
///
/// ```ignore
/// #[cfg(feature = "my-feature")]
/// ident,
/// ```
pub(crate) fn expand_fields_as_list<'a>(
    fields: impl IntoIterator<Item = &'a Field>,
) -> TokenStream {
    fields
        .into_iter()
        .map(|field| {
            let ident = field.ident();
            let cfg_attrs = field.cfg_attrs();

            quote! {
                #( #cfg_attrs )*
                #ident,
            }
        })
        .collect()
}

/// Extension trait for [`syn::meta::ParseNestedMeta`].
pub(crate) trait ParseNestedMetaExt {
    /// Whether this meta item has a value.
    fn has_value(&self) -> bool;
}

impl ParseNestedMetaExt for ParseNestedMeta<'_> {
    fn has_value(&self) -> bool {
        !self.input.is_empty() && !self.input.peek(syn::Token![,])
    }
}
