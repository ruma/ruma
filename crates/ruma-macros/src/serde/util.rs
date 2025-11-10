use proc_macro2::Span;
use syn::{ItemEnum, Token, Variant, punctuated::Punctuated};

use super::attr::{Attr, EnumAttrs, RenameAllAttr};

pub fn get_rename_all(input: &ItemEnum) -> syn::Result<RenameAllAttr> {
    let mut rename_all = RenameAllAttr::default();

    for attr in input.attrs.iter().filter(|attr| attr.path().is_ident("ruma_enum")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                rename_all.try_merge(meta)?;
                return Ok(());
            }

            Err(meta.error("unsupported `ruma_enum` attribute"))
        })?;
    }

    Ok(rename_all)
}

pub fn get_enum_attributes(input: &Variant) -> syn::Result<EnumAttrs> {
    let mut attributes = EnumAttrs::default();

    for attr in &input.attrs {
        if !attr.path().is_ident("ruma_enum") {
            continue;
        }

        let enum_attrs = attr.parse_args_with(Punctuated::<_, Token![,]>::parse_terminated)?;
        for enum_attr in enum_attrs {
            match enum_attr {
                Attr::Rename(s) => {
                    if attributes.rename.is_some() {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            "found multiple ruma_enum(rename) attributes",
                        ));
                    }
                    attributes.rename = Some(s);
                }
                Attr::Alias(s) => {
                    attributes.aliases.push(s);
                }
            }
        }
    }

    Ok(attributes)
}
