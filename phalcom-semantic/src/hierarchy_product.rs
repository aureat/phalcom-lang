//! Hierarchy edge query product.

use crate::identity::DeclarationId;

/// Stored product for a class hierarchy edge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyEdgeProduct {
    pub class_decl: DeclarationId,
    pub super_decl: Option<DeclarationId>,
}

impl HierarchyEdgeProduct {
    pub fn new(class_decl: DeclarationId, super_decl: Option<DeclarationId>) -> Self {
        Self { class_decl, super_decl }
    }
}
