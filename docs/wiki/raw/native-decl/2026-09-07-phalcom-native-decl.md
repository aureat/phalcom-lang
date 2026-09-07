# Phalcom Native Declarations source snapshot

> Source: Local repository snapshot at `phalcom-native-decl/`
> Collected: 2026-09-07
> Published: Unknown

## `phalcom-native-decl/Cargo.toml`

~~~~toml
[package]
name = "phalcom-native-decl"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
phalcom-common = { path = "../phalcom-common" }
phalcom-native-meta = { path = "../phalcom-native-meta" }
proc-macro2 = "1.0"
quote = "1.0"
syn = { workspace = true, features = ["full", "extra-traits", "parsing", "printing"] }
thiserror.workspace = true
~~~~

## `phalcom-native-decl/src/lib.rs`

~~~~rust
//! Shared parser and validator for native primitive declarations.

mod normalized;
mod parse;
mod validate;

pub use normalized::{NormalizedPrimitiveDecl, PrimitiveDeclField, PrimitiveDeclKey};
pub use parse::{docs_from_attributes, parse_primitive_attribute};
pub use validate::validate_decl;

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DeclError {
    #[error("invalid primitive declaration: {0}")]
    Parse(String),
    #[error("invalid primitive declaration field `{0}`")]
    UnknownField(String),
    #[error("duplicate primitive declaration field `{0}`")]
    DuplicateField(String),
    #[error("invalid primitive selector: {0}")]
    InvalidSelector(String),
    #[error("invalid primitive metadata: {0}")]
    InvalidMetadata(String),
}
~~~~

## `phalcom-native-decl/src/normalized.rs`

~~~~rust
use phalcom_native_meta::{NativeAnchorPolicy, NativeDispatch, NativeStability, NativeTrust, NativeVisibility, PrimitiveAbi, UniverseKey};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrimitiveDeclKey {
    pub owner: UniverseKey,
    pub selector: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrimitiveDeclField {
    pub name: String,
    pub value: String,
}

/// Owned, VM-free normalized form shared by proc-macro and source tooling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedPrimitiveDecl {
    pub key: PrimitiveDeclKey,
    pub fields: Vec<PrimitiveDeclField>,
    pub params: Option<String>,
    pub returns: Option<String>,
    pub types: Option<String>,
    pub raises: Option<String>,
    pub effects: Option<String>,
    pub side: NativeDispatch,
    pub visibility: Option<NativeVisibility>,
    pub stability: NativeStability,
    pub anchor: NativeAnchorPolicy,
    pub since: Option<String>,
    pub deprecated_since: Option<String>,
    pub replacement: Option<String>,
    pub abi: PrimitiveAbi,
    pub flow: Option<String>,
    pub intrinsic: Option<String>,
    pub trust: NativeTrust,
    pub conceptual: Option<String>,
    pub docs: Vec<String>,
}
~~~~

## `phalcom-native-decl/src/parse.rs`

~~~~rust
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Expr, Ident, LitStr, Token};

use crate::normalized::{NormalizedPrimitiveDecl, PrimitiveDeclField, PrimitiveDeclKey};
use crate::{DeclError, validate};
use phalcom_native_meta::{NativeAnchorPolicy, NativeDispatch, NativeStability, NativeTrust, NativeVisibility, PrimitiveAbi, UniverseKey};

struct RawPrimitiveDecl {
    owner: Ident,
    selector: LitStr,
    fields: Vec<(Ident, TokenStream)>,
}

impl Parse for RawPrimitiveDecl {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let owner = input.parse()?;
        input.parse::<Token![,]>()?;
        let selector = input.parse()?;
        let mut fields = Vec::new();
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let name: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value = if input.peek(syn::token::Bracket) {
                let content;
                syn::bracketed!(content in input);
                let inner: TokenStream = content.parse()?;
                quote::quote!([#inner])
            } else {
                let value: Expr = input.parse()?;
                value.into_token_stream()
            };
            fields.push((name, value));
        }
        Ok(Self { owner, selector, fields })
    }
}

/// Parses one `#[phalcom_native_macros::primitive(...)]` attribute.
pub fn parse_primitive_attribute(attribute: &Attribute) -> Result<NormalizedPrimitiveDecl, DeclError> {
    let raw = attribute
        .parse_args::<RawPrimitiveDecl>()
        .map_err(|error| DeclError::Parse(error.to_string()))?;
    let owner_name = raw.owner.to_string();
    let owner = UniverseKey::from_name(&owner_name).ok_or_else(|| DeclError::InvalidMetadata(format!("unknown universe owner `{owner_name}`")))?;
    let selector = raw.selector.value();

    let mut fields = Vec::new();
    let mut normalized = NormalizedPrimitiveDecl {
        key: PrimitiveDeclKey { owner, selector },
        fields: Vec::new(),
        params: None,
        returns: None,
        types: None,
        raises: None,
        effects: None,
        side: NativeDispatch::Instance,
        visibility: None,
        stability: NativeStability::Unspecified,
        anchor: NativeAnchorPolicy::Required,
        since: None,
        deprecated_since: None,
        replacement: None,
        abi: PrimitiveAbi::Value,
        flow: None,
        intrinsic: None,
        trust: NativeTrust::Ordinary,
        conceptual: None,
        docs: Vec::new(),
    };

    for (name, tokens) in raw.fields {
        let name_string = name.to_string();
        if fields.iter().any(|field: &PrimitiveDeclField| field.name == name_string) {
            return Err(DeclError::DuplicateField(name_string));
        }
        let value = tokens.to_string();
        fields.push(PrimitiveDeclField {
            name: name_string.clone(),
            value: value.clone(),
        });
        match name_string.as_str() {
            "params" => normalized.params = Some(value),
            "returns" => normalized.returns = Some(value),
            "types" => normalized.types = Some(parse_string_literal(&tokens).unwrap_or(value)),
            "raises" => normalized.raises = Some(value),
            "effects" => normalized.effects = Some(value),
            "side" => {
                normalized.side = match parse_ident_string(&tokens)?.as_str() {
                    "instance" => NativeDispatch::Instance,
                    "class" => NativeDispatch::Class,
                    other => return Err(DeclError::InvalidMetadata(format!("unknown dispatch side `{other}`"))),
                }
            }
            "visibility" => normalized.visibility = Some(parse_visibility(&tokens)?),
            "stability" => normalized.stability = parse_stability(&tokens)?,
            "anchor" => normalized.anchor = parse_anchor(&tokens)?,
            "since" => normalized.since = Some(parse_string_literal(&tokens)?),
            "deprecated_since" => normalized.deprecated_since = Some(parse_string_literal(&tokens)?),
            "replacement" => normalized.replacement = Some(parse_string_literal(&tokens)?),
            "abi" => normalized.abi = parse_abi(&tokens)?,
            "flow" => normalized.flow = Some(value),
            "intrinsic" => normalized.intrinsic = Some(parse_ident_string(&tokens)?),
            "trust" => normalized.trust = parse_trust(&tokens)?,
            "conceptual" => normalized.conceptual = Some(parse_string_literal(&tokens).unwrap_or(value)),
            other => return Err(DeclError::UnknownField(other.to_string())),
        }
    }
    normalized.fields = fields;
    validate::validate_decl(&normalized)?;
    Ok(normalized)
}

/// Captures attached Rust `///` documentation without interpreting its
/// content. Phaldoc normalization remains a downstream presentation concern.
pub fn docs_from_attributes(attributes: &[Attribute]) -> Vec<String> {
    attributes
        .iter()
        .filter_map(|attribute| match &attribute.meta {
            syn::Meta::NameValue(meta) => match &meta.value {
                syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(value), .. }) if attribute.path().is_ident("doc") => Some(value.value()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn parse_string_literal(tokens: &TokenStream) -> Result<String, DeclError> {
    syn::parse2::<LitStr>(tokens.clone())
        .map(|lit| lit.value())
        .map_err(|error| DeclError::Parse(error.to_string()))
}

fn parse_ident_string(tokens: &TokenStream) -> Result<String, DeclError> {
    syn::parse2::<Ident>(tokens.clone())
        .map(|ident| ident.to_string())
        .map_err(|error| DeclError::Parse(error.to_string()))
}

fn parse_visibility(tokens: &TokenStream) -> Result<NativeVisibility, DeclError> {
    match parse_ident_string(tokens)?.as_str() {
        "public" => Ok(NativeVisibility::Public),
        "internal" => Ok(NativeVisibility::Internal),
        other => Err(DeclError::InvalidMetadata(format!("unknown visibility `{other}`"))),
    }
}

fn parse_anchor(tokens: &TokenStream) -> Result<NativeAnchorPolicy, DeclError> {
    match parse_ident_string(tokens)?.as_str() {
        "required" => Ok(NativeAnchorPolicy::Required),
        "hidden" => Ok(NativeAnchorPolicy::Hidden),
        other => Err(DeclError::InvalidMetadata(format!("unknown anchor policy `{other}`"))),
    }
}

fn parse_stability(tokens: &TokenStream) -> Result<NativeStability, DeclError> {
    match parse_ident_string(tokens)?.as_str() {
        "unspecified" => Ok(NativeStability::Unspecified),
        "experimental" => Ok(NativeStability::Experimental),
        "stable" => Ok(NativeStability::Stable),
        other => Err(DeclError::InvalidMetadata(format!("unknown stability `{other}`"))),
    }
}

fn parse_abi(tokens: &TokenStream) -> Result<PrimitiveAbi, DeclError> {
    match parse_ident_string(tokens)?.as_str() {
        "value" => Ok(PrimitiveAbi::Value),
        "shape" => Ok(PrimitiveAbi::Shape),
        other => Err(DeclError::InvalidMetadata(format!("unknown ABI `{other}`"))),
    }
}

fn parse_trust(tokens: &TokenStream) -> Result<NativeTrust, DeclError> {
    match parse_ident_string(tokens)?.as_str() {
        "ordinary" => Ok(NativeTrust::Ordinary),
        "privileged" => Ok(NativeTrust::Privileged),
        other => Err(DeclError::InvalidMetadata(format!("unknown trust `{other}`"))),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_primitive_attribute;

    #[test]
    fn internal_selector_requires_internal_visibility() {
        let attribute: syn::Attribute = syn::parse_quote!(
            #[primitive(
                Object,
                "_$attach(_)",
                params = [Object],
                returns = Option,
                types = "(Object) -> Option",
                visibility = internal
            )]
        );

        assert!(parse_primitive_attribute(&attribute).is_ok());

        let invalid: syn::Attribute = syn::parse_quote!(
            #[primitive(
                Object,
                "_$attach(_)",
                params = [Object],
                returns = Option,
                types = "(Object) -> Option",
                visibility = public
            )]
        );
        assert!(parse_primitive_attribute(&invalid).is_err());
    }
}
~~~~

## `phalcom-native-decl/src/validate.rs`

~~~~rust
use crate::DeclError;
use crate::normalized::NormalizedPrimitiveDecl;
use phalcom_common::selector::Selector;

pub fn validate_decl(decl: &NormalizedPrimitiveDecl) -> Result<(), DeclError> {
    let selector = Selector::try_decode_exact(&decl.key.selector).map_err(|error| DeclError::InvalidSelector(error.to_string()))?;
    let canonical_rest_spelling = decl.key.selector.replace("***", "_");
    if selector.encode() != decl.key.selector && selector.encode() != canonical_rest_spelling {
        return Err(DeclError::InvalidSelector(format!("noncanonical spelling `{}`", decl.key.selector)));
    }
    let internal_selector = decl.key.selector.starts_with("_$");
    if internal_selector {
        if decl.visibility != Some(phalcom_native_meta::NativeVisibility::Internal) {
            return Err(DeclError::InvalidMetadata("internal selector requires visibility = internal".into()));
        }
    } else if decl.visibility == Some(phalcom_native_meta::NativeVisibility::Internal) {
        return Err(DeclError::InvalidMetadata("internal visibility requires an _$ selector".into()));
    }
    if decl.replacement.is_some() && decl.deprecated_since.is_none() {
        return Err(DeclError::InvalidMetadata("replacement requires deprecated_since".into()));
    }
    Ok(())
}
~~~~


