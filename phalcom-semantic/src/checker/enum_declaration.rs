//! Semantic analysis and product construction for enum declarations.

use crate::checker::algebraic_declaration::build_algebraic_semantics;
use crate::db::product::EnumDeclarationProduct;
use crate::declarations::DeclarationTypeTable;
use crate::identity::{DeclarationId, ModuleId};
use crate::resolver::LinkedTypeResolver;
use crate::types::store::TypeStore;
use phalcom_ast::ast::EnumDef;

/// Builds complete semantic metadata and variant products for an [`EnumDef`].
pub fn build_enum_semantics(
    owner: &DeclarationId,
    enum_def: &EnumDef,
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &LinkedTypeResolver,
    module_id: &ModuleId,
) -> Option<EnumDeclarationProduct> {
    let algebraic_prod = build_algebraic_semantics(owner, enum_def, store, declarations, resolver, module_id)?;
    algebraic_prod.to_enum_declaration_product()
}
