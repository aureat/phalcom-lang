use crate::DeclError;
use crate::normalized::NormalizedPrimitiveDecl;
use phalcom_common::selector::Selector;

pub fn validate_decl(decl: &NormalizedPrimitiveDecl) -> Result<(), DeclError> {
    let selector = Selector::try_decode_exact(&decl.key.selector).map_err(|error| DeclError::InvalidSelector(error.to_string()))?;
    if selector.encode() != decl.key.selector {
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
