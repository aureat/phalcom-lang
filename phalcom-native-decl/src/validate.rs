use crate::DeclError;
use crate::normalized::NormalizedPrimitiveDecl;
use phalcom_common::selector::Selector;

pub fn validate_decl(decl: &NormalizedPrimitiveDecl) -> Result<(), DeclError> {
    let selector = Selector::try_decode_exact(&decl.key.selector).map_err(|error| DeclError::InvalidSelector(error.to_string()))?;
    let canonical_rest_spelling = decl.key.selector.replace("***", "_");
    if selector.encode() != decl.key.selector && selector.encode() != canonical_rest_spelling {
        return Err(DeclError::InvalidSelector(format!("noncanonical spelling `{}`", decl.key.selector)));
    }
    if decl.replacement.is_some() && decl.deprecated_since.is_none() {
        return Err(DeclError::InvalidMetadata("replacement requires deprecated_since".into()));
    }
    Ok(())
}
