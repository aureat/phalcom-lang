//! Module query product models.

use crate::identity::ModuleId;
use phalcom_common::range::SourceRange;
use phalcom_modules::authority::ModuleAuthorityMap;

/// Stored product for resolved module imports and import diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedImportsProduct {
    pub module: ModuleId,
    pub imports: ModuleAuthorityMap<String, ModuleId>,
    pub unresolved_diagnostics: Vec<(String, SourceRange)>,
}

impl ResolvedImportsProduct {
    pub fn new(module: ModuleId, imports: impl Into<ModuleAuthorityMap<String, ModuleId>>, unresolved_diagnostics: Vec<(String, SourceRange)>) -> Self {
        Self {
            module,
            imports: imports.into(),
            unresolved_diagnostics,
        }
    }
}
