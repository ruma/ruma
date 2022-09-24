//! Details of the `metadata` section of the procedural macro.

use std::collections::BTreeSet;

use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, LitBool, LitStr, Token,
};

use super::{auth_scheme::AuthScheme, util, version::MatrixVersionLiteral};

mod kw {
    syn::custom_keyword!(metadata);
    syn::custom_keyword!(description);
    syn::custom_keyword!(method);
    syn::custom_keyword!(name);
    syn::custom_keyword!(unstable_path);
    syn::custom_keyword!(r0_path);
    syn::custom_keyword!(stable_path);
    syn::custom_keyword!(rate_limited);
    syn::custom_keyword!(authentication);
    syn::custom_keyword!(added);
    syn::custom_keyword!(deprecated);
    syn::custom_keyword!(removed);
    syn::custom_keyword!(unstable);
    syn::custom_keyword!(msc);
    syn::custom_keyword!(history);
}

/// The result of processing the `metadata` section of the macro.
pub struct Metadata {
    /// The description field.
    pub description: LitStr,

    /// The method field.
    pub method: Ident,

    /// The name field.
    pub name: LitStr,

    // /// The unstable path field.
    // pub unstable_path: Option<EndpointPath>,

    // /// The pre-v1.1 path field.
    // pub r0_path: Option<EndpointPath>,

    // /// The stable path field.
    // pub stable_path: Option<EndpointPath>,
    /// The rate_limited field.
    pub rate_limited: LitBool,

    /// The authentication field.
    pub authentication: AuthScheme,

    // /// The added field.
    // pub added: Option<MatrixVersionLiteral>,

    // /// The deprecated field.
    // pub deprecated: Option<MatrixVersionLiteral>,

    // /// The removed field.
    // pub removed: Option<MatrixVersionLiteral>,
    /// The version history field.
    pub history: History,
}

fn set_field<T: ToTokens>(field: &mut Option<T>, value: T) -> syn::Result<()> {
    match field {
        Some(existing_value) => {
            let mut error = syn::Error::new_spanned(value, "duplicate field assignment");
            error.combine(syn::Error::new_spanned(existing_value, "first one here"));
            Err(error)
        }
        None => {
            *field = Some(value);
            Ok(())
        }
    }
}

impl Parse for Metadata {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let metadata_kw: kw::metadata = input.parse()?;
        let _: Token![:] = input.parse()?;

        let field_values;
        braced!(field_values in input);

        let field_values =
            field_values.parse_terminated::<FieldValue, Token![,]>(FieldValue::parse)?;

        let mut description = None;
        let mut method = None;
        let mut name = None;
        let mut unstable_path = None;
        let mut r0_path = None;
        let mut stable_path = None;
        let mut rate_limited = None;
        let mut authentication = None;
        let mut added = None;
        let mut deprecated = None;
        let mut removed = None;
        let mut history = None;

        for field_value in field_values {
            match field_value {
                FieldValue::Description(d) => set_field(&mut description, d)?,
                FieldValue::Method(m) => set_field(&mut method, m)?,
                FieldValue::Name(n) => set_field(&mut name, n)?,
                FieldValue::RateLimited(rl) => set_field(&mut rate_limited, rl)?,
                FieldValue::Authentication(a) => set_field(&mut authentication, a)?,

                FieldValue::History(h) => set_field(&mut history, h)?,

                FieldValue::Added(v) => set_field(&mut added, v)?,
                FieldValue::Deprecated(v) => set_field(&mut deprecated, v)?,
                FieldValue::Removed(v) => set_field(&mut removed, v)?,

                FieldValue::UnstablePath(p) => set_field(&mut unstable_path, p)?,
                FieldValue::R0Path(p) => set_field(&mut r0_path, p)?,
                FieldValue::StablePath(p) => set_field(&mut stable_path, p)?,
            }
        }

        let missing_field =
            |name| syn::Error::new_spanned(metadata_kw, format!("missing field `{}`", name));

        // Todo: we could construct the history anyways from the scrapped-together legacy fields,
        //  and compare for an extra sanity check
        let history = match history {
            None => {
                let stable_or_r0 = stable_path.as_ref().or(r0_path.as_ref());

                if let Some(path) = stable_or_r0 {
                    if added.is_none() {
                        return Err(syn::Error::new_spanned(
                            path,
                            "stable path was defined, while `added` version was not defined",
                        ));
                    }
                }

                if let Some(deprecated) = &deprecated {
                    if added.is_none() {
                        return Err(syn::Error::new_spanned(
                            deprecated,
                            "deprecated version is defined while added version is not defined",
                        ));
                    }
                }

                // note: It is possible that matrix will remove endpoints in a single version, while
                // not having a deprecation version inbetween, but that would not be
                // allowed by their own deprecation policy, so lets just assume
                // there's always a deprecation version before a removal one.
                //
                // If matrix does so anyways, we can just alter this.
                if let Some(removed) = &removed {
                    if deprecated.is_none() {
                        return Err(syn::Error::new_spanned(
                            removed,
                            "removed version is defined while deprecated version is not defined",
                        ));
                    }
                }

                if let Some(added) = &added {
                    if stable_or_r0.is_none() {
                        return Err(syn::Error::new_spanned(
                            added,
                            "added version is defined, but no stable or r0 path exists",
                        ));
                    }
                }

                if let Some(r0) = &r0_path {
                    let added =
                        added.as_ref().expect("we error if r0 or stable is defined without added");

                    if added.major.get() == 1 && added.minor > 0 {
                        return Err(syn::Error::new_spanned(
                            r0,
                            "r0 defined while added version is newer than v1.0",
                        ));
                    }

                    if stable_path.is_none() {
                        return Err(syn::Error::new_spanned(r0, "r0 defined without stable path"));
                    }

                    if !r0.value().contains("/r0/") {
                        return Err(syn::Error::new_spanned(
                            r0,
                            "r0 endpoint does not contain /r0/",
                        ));
                    }
                }

                if let Some(stable) = &stable_path {
                    if stable.value().contains("/r0/") {
                        return Err(syn::Error::new_spanned(
                            stable,
                            "stable endpoint contains /r0/ (did you make a copy-paste error?)",
                        ));
                    }
                }

                if unstable_path.is_none() && r0_path.is_none() && stable_path.is_none() {
                    return Err(syn::Error::new_spanned(
                        metadata_kw,
                        "need to define one of [r0_path, stable_path, unstable_path]",
                    ));
                }

                History::construct(
                    deprecated,
                    removed,
                    unstable_path,
                    r0_path,
                    stable_path.zip(added),
                )
            }

            Some(h) => h,
        };

        Ok(Self {
            description: description.ok_or_else(|| missing_field("description"))?,
            method: method.ok_or_else(|| missing_field("method"))?,
            name: name.ok_or_else(|| missing_field("name"))?,
            rate_limited: rate_limited.ok_or_else(|| missing_field("rate_limited"))?,
            authentication: authentication.ok_or_else(|| missing_field("authentication"))?,
            history,
        })
    }
}

enum Field {
    Description,
    Method,
    Name,
    UnstablePath,
    R0Path,
    StablePath,
    RateLimited,
    Authentication,
    Added,
    Deprecated,
    Removed,
    History,
}

impl Parse for Field {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::description) {
            let _: kw::description = input.parse()?;
            Ok(Self::Description)
        } else if lookahead.peek(kw::method) {
            let _: kw::method = input.parse()?;
            Ok(Self::Method)
        } else if lookahead.peek(kw::name) {
            let _: kw::name = input.parse()?;
            Ok(Self::Name)
        } else if lookahead.peek(kw::unstable_path) {
            let _: kw::unstable_path = input.parse()?;
            Ok(Self::UnstablePath)
        } else if lookahead.peek(kw::r0_path) {
            let _: kw::r0_path = input.parse()?;
            Ok(Self::R0Path)
        } else if lookahead.peek(kw::stable_path) {
            let _: kw::stable_path = input.parse()?;
            Ok(Self::StablePath)
        } else if lookahead.peek(kw::rate_limited) {
            let _: kw::rate_limited = input.parse()?;
            Ok(Self::RateLimited)
        } else if lookahead.peek(kw::authentication) {
            let _: kw::authentication = input.parse()?;
            Ok(Self::Authentication)
        } else if lookahead.peek(kw::added) {
            let _: kw::added = input.parse()?;
            Ok(Self::Added)
        } else if lookahead.peek(kw::deprecated) {
            let _: kw::deprecated = input.parse()?;
            Ok(Self::Deprecated)
        } else if lookahead.peek(kw::removed) {
            let _: kw::removed = input.parse()?;
            Ok(Self::Removed)
        } else if lookahead.peek(kw::history) {
            let _: kw::history = input.parse()?;
            Ok(Self::History)
        } else {
            Err(lookahead.error())
        }
    }
}

enum FieldValue {
    Description(LitStr),
    Method(Ident),
    Name(LitStr),
    UnstablePath(EndpointPath),
    R0Path(EndpointPath),
    StablePath(EndpointPath),
    RateLimited(LitBool),
    Authentication(AuthScheme),
    Added(MatrixVersionLiteral),
    Deprecated(MatrixVersionLiteral),
    Removed(MatrixVersionLiteral),
    History(History),
}

impl Parse for FieldValue {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let field: Field = input.parse()?;
        let _: Token![:] = input.parse()?;

        Ok(match field {
            Field::Description => Self::Description(input.parse()?),
            Field::Method => Self::Method(input.parse()?),
            Field::Name => Self::Name(input.parse()?),
            Field::UnstablePath => Self::UnstablePath(input.parse()?),
            Field::R0Path => Self::R0Path(input.parse()?),
            Field::StablePath => Self::StablePath(input.parse()?),
            Field::RateLimited => Self::RateLimited(input.parse()?),
            Field::Authentication => Self::Authentication(input.parse()?),
            Field::Added => Self::Added(input.parse()?),
            Field::Deprecated => Self::Deprecated(input.parse()?),
            Field::Removed => Self::Removed(input.parse()?),
            Field::History => Self::History(input.parse()?),
        })
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct History {
    entries: Vec<HistoryEntry>,

    misc: MiscVersioning,
}

impl History {
    // TODO(j0j0): remove after codebase conversion is complete
    /// Construct a History object from legacy parts.
    pub fn construct(
        deprecated: Option<MatrixVersionLiteral>,
        removed: Option<MatrixVersionLiteral>,
        unstable_path: Option<EndpointPath>,
        r0_path: Option<EndpointPath>,
        stable_path_and_version: Option<(EndpointPath, MatrixVersionLiteral)>,
    ) -> Self {
        let unstable = unstable_path.map(|path| HistoryEntry::Unstable { path });
        let r0 = r0_path.map(|path| HistoryEntry::Stable {
            path,
            version: (1, 0).try_into().expect("statically defined and checked"),
        });
        let stable =
            stable_path_and_version.map(|(path, version)| HistoryEntry::Stable { path, version });

        let misc = match (deprecated, removed) {
            (None, None) => MiscVersioning::None,
            (Some(deprecated), None) => MiscVersioning::Deprecated(deprecated),
            (Some(deprecated), Some(removed)) => MiscVersioning::Removed { deprecated, removed },

            (None, Some(_)) => unreachable!("removed implies deprecated"),
        };

        let entries = [unstable, r0, stable].into_iter().flatten().collect();

        History { entries, misc }
    }

    /// Convenience method for getting path arguments.
    pub fn path_args(&self) -> Vec<String> {
        self.entries
            .first()
            .expect("history has at least one entry")
            .path()
            .expect("non-path entries have been purged")
            .args()
    }
}

#[derive(Debug)]
pub enum MiscVersioning {
    None,
    Deprecated(MatrixVersionLiteral),
    Removed { deprecated: MatrixVersionLiteral, removed: MatrixVersionLiteral },
}

fn set_ref_version(set: &mut BTreeSet<(u8, u8)>, value: &MatrixVersionLiteral) -> syn::Result<()> {
    if set.contains(&value.to_parts()) {
        Err(syn::Error::new_spanned(value, "duplicate version reference"))
    } else {
        set.insert(value.to_parts());
        Ok(())
    }
}

impl Parse for History {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let values;
        braced!(values in input);

        let mut deprecated = None;
        let mut removed = None;

        let mut entries: Vec<_> = values
            .parse_terminated::<HistoryEntry, Token![,]>(HistoryEntry::parse)?
            .into_iter()
            .collect();

        let mut versions: BTreeSet<(u8, u8)> = BTreeSet::new();

        for entry in &entries {
            match entry {
                HistoryEntry::Deprecated { version } => {
                    set_ref_version(&mut versions, version)?;
                    set_field(&mut deprecated, version.clone())?;
                }
                HistoryEntry::Removed { version } => {
                    set_ref_version(&mut versions, version)?;
                    set_field(&mut removed, version.clone())?;
                }
                HistoryEntry::Stable { version, path: _ } => {
                    set_ref_version(&mut versions, version)?;
                }
                _ => {}
            }
        }

        let misc = match (deprecated, removed) {
            (Some(d), Some(r)) => MiscVersioning::Removed { deprecated: d, removed: r },
            (Some(d), None) => MiscVersioning::Deprecated(d),
            (None, Some(v)) => {
                // note: It is possible that matrix will remove endpoints in a single version, while
                // not having a deprecation version inbetween, but that would not be
                // allowed by their own deprecation policy, so lets just assume
                // there's always a deprecation version before a removal one.
                //
                // If matrix does so anyways, we can just alter this.
                return Err(syn::Error::new_spanned(
                    v,
                    "cannot define removed version without deprecated version",
                ));
            }

            (None, None) => MiscVersioning::None,
        };

        entries.retain(|h| match h {
            HistoryEntry::Unstable { path: _ } => true,
            HistoryEntry::Stable { version: _, path: _ } => true,

            HistoryEntry::Deprecated { version: _ } => false,
            HistoryEntry::Removed { version: _ } => false,
        });

        // Sort so that the order is [Unstable, Unstable, 1.0, 1.1, 1.2, 2.0, 2.1]
        // for optimized method purposes.
        entries.sort_by_key(|a| a.version().map(|v| v.to_parts()).unwrap_or((0, 0)));

        // Check if all path arguments are equal, and in order.
        let mut last_args = None;
        for entry in &entries {
            let path = entry.path().expect("all non-path entries have been removed");
            if let Some(args) = &last_args {
                if &path.args() != args {
                    return Err(syn::Error::new_spanned(&path.0, "path arguments are not equal"));
                }
            } else {
                last_args = Some(path.args());
            }
        }

        Ok(History { entries, misc })
    }
}

impl ToTokens for History {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        fn endpointpath_to_pathdata_ts(endpoint: &EndpointPath) -> proc_macro2::TokenStream {
            let path = endpoint.value();

            let format_parts = util::path_into_format_parts(&path);

            quote! {
                ::ruma_common::api::PathData {
                    canon: #path,
                    parts: &[ #(#format_parts),* ]
                }
            }
        }

        let unstable = self.entries.iter().filter_map(|e| {
            if let HistoryEntry::Unstable { path } = e {
                Some(endpointpath_to_pathdata_ts(path))
            } else {
                None
            }
        });
        let versioned = self.entries.iter().filter_map(|e| {
            if let HistoryEntry::Stable { path, version } = e {
                let path = endpointpath_to_pathdata_ts(path);
                Some(quote! {( #version, #path )})
            } else {
                None
            }
        });

        let (deprecated, removed) = match &self.misc {
            MiscVersioning::None => (None, None),
            MiscVersioning::Deprecated(deprecated) => (Some(deprecated), None),
            MiscVersioning::Removed { deprecated, removed } => (Some(deprecated), Some(removed)),
        };

        let deprecated = util::map_option_literal(&deprecated);
        let removed = util::map_option_literal(&removed);

        tokens.extend(quote! {
            ::ruma_common::api::VersionHistory {
                unstable_paths: &[ #(#unstable),* ],
                path_versions: &[ #(#versioned),* ],
                deprecated: #deprecated,
                removed: #removed,
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum HistoryEntry {
    Unstable { path: EndpointPath },
    Stable { version: MatrixVersionLiteral, path: EndpointPath },
    Deprecated { version: MatrixVersionLiteral },
    Removed { version: MatrixVersionLiteral },
}

impl HistoryEntry {
    fn version(&self) -> Option<&MatrixVersionLiteral> {
        Some(match self {
            HistoryEntry::Deprecated { version } => version,
            HistoryEntry::Removed { version } => version,
            HistoryEntry::Stable { version, path: _ } => version,

            _ => return None,
        })
    }

    fn path(&self) -> Option<&EndpointPath> {
        Some(match self {
            HistoryEntry::Stable { version: _, path } => path,
            HistoryEntry::Unstable { path } => path,

            _ => return None,
        })
    }
}

impl Parse for HistoryEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let key: HistoryEntryKey = input.parse()?;
        let _: syn::token::FatArrow = input.parse()?;

        match key {
            HistoryEntryKey::Unstable => Ok(HistoryEntry::Unstable { path: input.parse()? }),
            HistoryEntryKey::Version(version) => {
                let lookahead = input.lookahead1();

                if lookahead.peek(kw::deprecated) {
                    let _: kw::deprecated = input.parse()?;

                    Ok(Self::Deprecated { version })
                } else if lookahead.peek(kw::removed) {
                    let _: kw::removed = input.parse()?;

                    Ok(Self::Removed { version })
                } else {
                    Ok(HistoryEntry::Stable { version, path: input.parse()? })
                }
            }
        }
    }
}

pub enum HistoryEntryKey {
    Unstable,
    Version(MatrixVersionLiteral),
}

impl Parse for HistoryEntryKey {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::unstable) {
            let _: kw::unstable = input.parse()?;

            Ok(Self::Unstable)
        } else {
            let version: MatrixVersionLiteral = input.parse()?;

            Ok(Self::Version(version))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EndpointPath(LitStr);

impl EndpointPath {
    pub fn value(&self) -> String {
        self.0.value()
    }

    pub fn args(&self) -> Vec<String> {
        self.value().split('/').filter_map(|s| s.strip_prefix(':')).map(String::from).collect()
    }
}

impl Parse for EndpointPath {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let path: LitStr = input.parse()?;

        if util::is_valid_endpoint_path(&path.value()) {
            Ok(Self(path))
        } else {
            Err(syn::Error::new_spanned(
                &path,
                "path may only contain printable ASCII characters with no spaces",
            ))
        }
    }
}

impl ToTokens for EndpointPath {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens);
    }
}
