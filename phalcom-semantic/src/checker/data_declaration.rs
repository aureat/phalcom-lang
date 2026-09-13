//! Semantic analysis and product construction for data declarations.

use crate::data_semantics::{
    DataComponentSemantic, DataConstructorParameter, DataConstructorSignature, DataInfo, DataShape,
};
use crate::db::product::DataDeclarationProduct;
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, DeclaredTypeState};
use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic, SemanticSourceSpan};
use crate::identity::{DataComponentId, DataConstructorId, DeclarationId, DispatchSide, ModuleId};
use crate::resolver::LinkedTypeResolver;
use crate::types::annotation::{ScopedTypeResolver, TypeFormationSite, resolve_type_annotation, type_level_binding_for_parameter};
use crate::types::evidence::TypeKnowledge;
use crate::types::id::{TypeId, TypeParameterId};
use crate::types::store::TypeStore;
use phalcom_ast::ast::{DataDef, DataShapeSyntax};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Builds complete semantic metadata and constructor product for a [`DataDef`].
pub fn build_data_semantics(
    owner: &DeclarationId,
    data_def: &DataDef,
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &LinkedTypeResolver,
    module_id: &ModuleId,
) -> Option<DataDeclarationProduct> {
    let mut diagnostics = Vec::new();

    let generic_sig = declarations.generic_signature(owner).cloned();
    let generic_params: Vec<TypeParameterId> = generic_sig.as_ref().map(|sig| sig.parameters.to_vec()).unwrap_or_default();

    let root_form = declarations.form(owner)?;

    let result_type_template = if generic_params.is_empty() {
        root_form
    } else {
        let param_forms: Vec<TypeId> = generic_params.iter().map(|&p| store.parameter_form(p)).collect();
        store.apply_type_form(root_form, &param_forms).unwrap_or(root_form)
    };

    let mut type_params_map = HashMap::new();
    if let Some(ref sig) = generic_sig {
        for &p in sig.parameters.iter() {
            let p_name = store.type_parameter(p).name.to_string();
            let binding = type_level_binding_for_parameter(store, p);
            type_params_map.insert(p_name, binding);
        }
    }

    let scoped_resolver = ScopedTypeResolver {
        parent: resolver,
        type_parameters: type_params_map,
    };
    let formation_site = TypeFormationSite::member(module_id.clone(), owner.clone(), DispatchSide::Instance);

    let (shape, components_syntax) = match &data_def.shape {
        DataShapeSyntax::Tuple { components, .. } => (DataShape::Tuple, components.as_slice()),
        DataShapeSyntax::Record { components, .. } => (DataShape::Record, components.as_slice()),
    };

    let mut seen_names: HashSet<String> = HashSet::new();
    let mut seen_labels: HashSet<String> = HashSet::new();
    let mut component_semantics = Vec::with_capacity(components_syntax.len());
    let mut constructor_params = Vec::with_capacity(components_syntax.len());

    for (idx, comp) in components_syntax.iter().enumerate() {
        let comp_id = DataComponentId::new(owner.clone(), idx as u32);
        let local_name: Box<str> = comp.local_name.clone().into_boxed_str();
        let external_label: Option<Box<str>> = comp.external_label.clone().map(String::into_boxed_str);

        match shape {
            DataShape::Record => {
                if !seen_names.insert(comp.local_name.clone()) {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        module_id.clone(),
                        DiagnosticCode::DataDuplicateComponent,
                        format!("duplicate component `{}` in data `{}`", comp.local_name, owner.name),
                        comp.range,
                    ));
                }
            }
            DataShape::Tuple => {
                if let Some(ref label) = comp.external_label {
                    if !seen_labels.insert(label.clone()) {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            module_id.clone(),
                            DiagnosticCode::DataDuplicateComponent,
                            format!("duplicate component label `{}` in data `{}`", label, owner.name),
                            comp.range,
                        ));
                    }
                }
            }
        }

        let mut ann_diags = Vec::new();
        let raw_knowledge = resolve_type_annotation(store, declarations, &scoped_resolver, &formation_site, &comp.annotation, &mut ann_diags);
        diagnostics.extend(ann_diags);

        let declared_type = match raw_knowledge {
            TypeKnowledge::Known(evidence) => DeclaredTypeFact {
                state: DeclaredTypeState::Known(crate::types::parameter::TypeTerm::Canonical(evidence.ty())),
                basis: DeclaredTypeBasis::SourceAnnotation,
            },
            _ => DeclaredTypeFact::from_knowledge(&raw_knowledge),
        };

        let source = Some(SemanticSourceSpan::new(module_id.clone(), comp.range));

        component_semantics.push(DataComponentSemantic {
            id: comp_id.clone(),
            local_name: local_name.clone(),
            external_label: external_label.clone(),
            declared_type: declared_type.clone(),
            source,
        });

        constructor_params.push(DataConstructorParameter {
            component: comp_id,
            external_label,
            local_name,
            declared_type,
        });
    }

    let constructor = DataConstructorSignature {
        constructor: DataConstructorId::new(owner.clone()),
        parameters: constructor_params.into_boxed_slice(),
        result_type_template,
        source: Some(SemanticSourceSpan::new(module_id.clone(), data_def.name_range)),
    };

    let data_info = DataInfo {
        owner: owner.clone(),
        root_form,
        generic_signature: generic_sig,
        shape,
        components: component_semantics.into_boxed_slice(),
        constructor,
        source: Some(SemanticSourceSpan::new(module_id.clone(), data_def.range)),
    };

    Some(DataDeclarationProduct {
        info: Arc::new(data_info),
        diagnostics: diagnostics.into(),
    })
}
