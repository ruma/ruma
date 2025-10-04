use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Field, Ident, LitStr};

pub(crate) fn import_ruma_common() -> TokenStream {
    if let Ok(FoundCrate::Name(name)) = crate_name("ruma-common") {
        let import = format_ident!("{name}");
        quote! { ::#import }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("ruma") {
        let import = format_ident!("{name}");
        quote! { ::#import }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("matrix-sdk") {
        let import = format_ident!("{name}");
        quote! { ::#import::ruma }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("matrix-sdk-appservice") {
        let import = format_ident!("{name}");
        quote! { ::#import::ruma }
    } else {
        quote! { ::ruma_common }
    }
}

pub(crate) fn import_ruma_events() -> TokenStream {
    if let Ok(FoundCrate::Name(name)) = crate_name("ruma-events") {
        let import = format_ident!("{name}");
        quote! { ::#import }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("ruma") {
        let import = format_ident!("{name}");
        quote! { ::#import::events }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("matrix-sdk") {
        let import = format_ident!("{name}");
        quote! { ::#import::ruma::events }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("matrix-sdk-appservice") {
        let import = format_ident!("{name}");
        quote! { ::#import::ruma::events }
    } else {
        quote! { ::ruma_events }
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
    use syn::{visit_mut::VisitMut, Fields, LitBool, Meta};

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

/// Whether the given field has a `#[serde(flatten)]` attribute.
pub fn field_has_serde_flatten_attribute(field: &Field) -> bool {
    field.attrs.iter().any(is_serde_flatten_attribute)
}

/// Whether the given attribute is a `#[serde(flatten)]` attribute.
fn is_serde_flatten_attribute(attr: &Attribute) -> bool {
    if !attr.path().is_ident("serde") {
        return false;
    }

    let mut contains_flatten = false;
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("flatten") {
            contains_flatten = true;
            // Return an error to stop the parsing early.
            return Err(meta.error("found"));
        }

        Ok(())
    });

    contains_flatten
}
