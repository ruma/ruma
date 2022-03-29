use syn::{
    parse::{Parse, ParseStream},
    LitStr, Token,
};

use super::case::RenameRule;

mod kw {
    syn::custom_keyword!(alias);
    syn::custom_keyword!(rename);
    syn::custom_keyword!(rename_all);
}

#[derive(Default)]
pub struct EnumAttrs {
    pub rename: Option<LitStr>,
    pub aliases: Vec<LitStr>,
}

pub enum Attr {
    Alias(LitStr),
    Rename(LitStr),
}

impl Parse for Attr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::alias) {
            let _: kw::alias = input.parse()?;
            let _: Token![=] = input.parse()?;
            Ok(Self::Alias(input.parse()?))
        } else if lookahead.peek(kw::rename) {
            let _: kw::rename = input.parse()?;
            let _: Token![=] = input.parse()?;
            Ok(Self::Rename(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct RenameAllAttr(RenameRule);

impl RenameAllAttr {
    pub fn into_inner(self) -> RenameRule {
        self.0
    }
}

impl Parse for RenameAllAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: kw::rename_all = input.parse()?;
        let _: Token![=] = input.parse()?;
        let s: LitStr = input.parse()?;
        Ok(Self(
            s.value()
                .parse()
                .map_err(|_| syn::Error::new_spanned(s, "invalid value for rename_all"))?,
        ))
    }
}
