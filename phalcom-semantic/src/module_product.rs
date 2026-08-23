//! Module query product models.

use crate::identity::ModuleId;
use phalcom_common::range::SourceRange;
use std::collections::BTreeMap;

/// Stored product for resolved module imports and import diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedImportsProduct {
    pub module: ModuleId,
    pub imports: BTreeMap<String, ModuleId>,
    pub unresolved_diagnostics: Vec<(String, SourceRange)>,
}

impl ResolvedImportsProduct {
    pub fn new(module: ModuleId, imports: BTreeMap<String, ModuleId>, unresolved_diagnostics: Vec<(String, SourceRange)>) -> Self {
        Self {
            module,
            imports,
            unresolved_diagnostics,
        }
    }
}
