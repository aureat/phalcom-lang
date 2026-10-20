//! Semantic analysis and product construction for data declarations.

use crate::checker::algebraic_declaration::build_algebraic_semantics;
use crate::db::product::DataDeclarationProduct;
use crate::declarations::DeclarationTypeTable;
use crate::identity::{DeclarationId, ModuleId};
use crate::resolver::LinkedTypeResolver;
use crate::types::store::TypeStore;
use phalcom_ast::ast::DataDef;

/// Builds complete semantic metadata and constructor product for a [`DataDef`].
pub fn build_data_semantics(
    owner: &DeclarationId,
    data_def: &DataDef,
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &LinkedTypeResolver,
    module_id: &ModuleId,
) -> Option<DataDeclarationProduct> {
    let algebraic_prod = build_algebraic_semantics(owner, data_def, store, declarations, resolver, module_id)?;
    algebraic_prod.to_data_declaration_product()
}

pub fn build_data_semantics_with_static_products(
    owner: &DeclarationId,
    data_def: &DataDef,
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &LinkedTypeResolver,
    module_id: &ModuleId,
    catalog: Option<&crate::static_case::StaticDomainCaseCatalogProduct>,
    signatures: &crate::ordered_map::OrderedMap<crate::identity::StaticCaseId, std::sync::Arc<crate::static_case::StaticCaseSignatureProduct>>,
) -> Option<DataDeclarationProduct> {
    let algebraic_prod = crate::checker::algebraic_declaration::build_algebraic_semantics_with_static_products(
        owner,
        data_def,
        store,
        declarations,
        resolver,
        module_id,
        catalog,
        signatures,
    )?;
    algebraic_prod.to_data_declaration_product()
}
