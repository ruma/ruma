use syn::{
    parse::{Parse, ParseStream},
    LitStr, Token,
};

use super::case::RenameRule;

mod kw {
    syn::custom_keyword!(rename);
    syn::custom_keyword!(rename_all);
}

pub struct RenameAttr(LitStr);

impl RenameAttr {
    pub fn into_inner(self) -> LitStr {
        self.0
    }
}

impl Parse for RenameAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: kw::rename = input.parse()?;
        let _: Token![=] = input.parse()?;
        Ok(Self(input.parse()?))
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
