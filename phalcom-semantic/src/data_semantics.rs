//! Canonical data declaration semantic products.

use crate::declaration_type::DeclaredTypeFact;
use crate::identity::{DataComponentId, DataConstructorId, DeclarationId, SemanticSourceSpan};
use crate::types::id::{KindId, TypeId};
use crate::types::parameter::GenericSignature;
use crate::types::row::RecordRowTail;
use crate::types::store::{StaticArgument, TypeData, TypeStore};
use phalcom_common::modifier::Visibility;
use std::sync::Arc;
use thiserror::Error;

/// Structural shape of a data declaration: Tuple vs Record.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DataShape {
    Tuple,
    Record,
}

/// Semantic identity and declared type of one data component.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataComponentSemantic {
    pub id: DataComponentId,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub declared_type: DeclaredTypeFact,
    pub source: Option<SemanticSourceSpan>,
}

/// Formal constructor parameter corresponding to a data component.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataConstructorParameter {
    pub component: DataComponentId,
    pub external_label: Option<Box<str>>,
    pub local_name: Box<str>,
    pub declared_type: DeclaredTypeFact,
}

/// Formal signature for a data constructor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataConstructorSignature {
    pub constructor: DataConstructorId,
    pub visibility: Visibility,
    pub parameters: Box<[DataConstructorParameter]>,
    pub result_type_template: TypeId,
    pub source: Option<SemanticSourceSpan>,
}

/// Semantic model of an expansion contribution within a data declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataProductExpansionSemantic {
    pub mode: phalcom_ast::ast::ExpansionMode,
    pub operand: DeclaredTypeFact,
    pub source: Option<SemanticSourceSpan>,
}

/// Semantic contribution to a data product shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataProductContribution {
    Component(DataComponentSemantic),
    Expansion(DataProductExpansionSemantic),
}

/// Structural template for a product shape, preserving fixed components and expansions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataProductShapeTemplate {
    pub shape: DataShape,
    pub contributions: Box<[DataProductContribution]>,
}

impl DataProductShapeTemplate {
    pub fn has_expansions(&self) -> bool {
        self.contributions.iter().any(|c| matches!(c, DataProductContribution::Expansion(_)))
    }
}

/// Semantic contribution to a data constructor's parameter list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataConstructorParameterContribution {
    Parameter(DataConstructorParameter),
    Expansion(DataProductExpansionSemantic),
}

/// Structural template for a data constructor signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataConstructorTemplate {
    pub constructor: DataConstructorId,
    pub visibility: Visibility,
    pub parameters: Box<[DataConstructorParameterContribution]>,
    pub result_type_template: TypeId,
    pub source: Option<SemanticSourceSpan>,
}

/// Complete structural semantic metadata for one declared data root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataInfo {
    pub owner: DeclarationId,
    pub root_form: TypeId,
    pub generic_signature: Option<GenericSignature>,
    /// Canonical static-domain declaration product when this algebraic
    /// declaration is type-only. The exporter must consume this product
    /// rather than enumerate interned TypeStore history.
    pub static_data: Option<crate::algebraic_semantics::StaticAlgebraicDeclarationInfo>,
    pub shape: DataShape,
    pub components: Box<[DataComponentSemantic]>,
    pub constructor: DataConstructorSignature,
    pub shape_template: DataProductShapeTemplate,
    pub constructor_template: Option<DataConstructorTemplate>,
    pub source: Option<SemanticSourceSpan>,
}

impl DataInfo {
    pub fn find_component(&self, name: &str) -> Option<&DataComponentSemantic> {
        self.components
            .iter()
            .find(|c| c.local_name.as_ref() == name || c.external_label.as_deref() == Some(name))
    }

    pub fn constructor_operation(&self) -> crate::types::family::FamilyOperationShape {
        let slots = match self.shape {
            DataShape::Tuple => self
                .constructor
                .parameters
                .iter()
                .map(|p| match &p.external_label {
                    Some(label) => phalcom_common::selector::SelectorSlot::Label(label.to_string()),
                    None => phalcom_common::selector::SelectorSlot::Positional,
                })
                .collect::<Box<[_]>>(),
            DataShape::Record => self
                .constructor
                .parameters
                .iter()
                .map(|p| match &p.external_label {
                    Some(label) => phalcom_common::selector::SelectorSlot::Label(label.to_string()),
                    None => phalcom_common::selector::SelectorSlot::Label(p.local_name.to_string()),
                })
                .collect::<Box<[_]>>(),
        };
        crate::types::family::FamilyOperationShape::new(phalcom_common::selector::SelectorKind::Method, slots)
    }

    pub fn constructor_selector(&self) -> Result<phalcom_common::selector::Selector, phalcom_common::selector::SelectorError> {
        let op = self.constructor_operation();
        phalcom_common::selector::Selector::new(
            phalcom_common::selector::SelectorBase::Named("Construct".into()),
            phalcom_common::selector::SelectorKind::Method,
            op.slots,
        )
    }
}

/// One concrete or partially known logical component of a specialized product.
/// Components introduced by an expansion deliberately have no declaration
/// `DataComponentId`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataProductMaterializedComponent {
    pub component: Option<DataComponentId>,
    pub local_name: Option<Box<str>>,
    pub label: Option<Box<str>>,
    pub ty: Option<TypeId>,
    pub source: Option<SemanticSourceSpan>,
    pub expansion_source: Option<SemanticSourceSpan>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DataProductShapeState {
    Concrete,
    Symbolic,
}

/// Product shape after substituting the exact generic application. A symbolic
/// state means the declaration is valid but one or more pack/row tails remain
/// unresolved, so it cannot be used to allocate an exact runtime layout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataProductShapeMaterialization {
    pub state: DataProductShapeState,
    pub components: Box<[DataProductMaterializedComponent]>,
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum DataProductShapeMaterializationError {
    #[error("specialized data type does not belong to the product declaration")]
    WrongOwner,
    #[error("specialized data type has the wrong generic argument arity")]
    GenericArity,
    #[error("specialized generic argument has the wrong domain")]
    GenericArgumentDomain,
    #[error("expanded product contribution has kind {actual:?}, expected {expected:?}")]
    ExpansionKind { expected: KindId, actual: KindId },
    #[error("RecordFields contribution did not materialize to a Record row")]
    InvalidRecordExpansion,
    #[error("duplicate Record product field `{field}` after expansion materialization")]
    DuplicateRecordField { field: Box<str> },
    #[error("type materialization failed: {0}")]
    Type(#[from] crate::types::instantiation::TypeMaterializationError),
    #[error("record row could not be materialized: {0}")]
    RecordRow(#[from] crate::types::row::RecordRowFormationError),
}

/// Materializes a declared product template against one canonical specialized
/// type. Pack values and RecordFields rows are read from the exact type's
/// heterogeneous generic arguments; no inference or identity allocation is
/// performed here.
pub fn materialize_data_product_shape(
    store: &mut TypeStore,
    info: &DataInfo,
    specialized_type: TypeId,
) -> Result<DataProductShapeMaterialization, DataProductShapeMaterializationError> {
    let instantiation = data_specialization_instantiation(store, info, specialized_type)?;
    let mut components = Vec::new();
    let mut state = DataProductShapeState::Concrete;

    for contribution in info.shape_template.contributions.iter() {
        match contribution {
            DataProductContribution::Component(component) => {
                let ty = match component.declared_type.canonical_type() {
                    Some(ty) => Some(crate::types::instantiation::materialize_type(
                        store,
                        ty,
                        &instantiation,
                        crate::types::instantiation::RowMaterializationMode::PreserveUnboundStableTail,
                    )?),
                    None => {
                        state = DataProductShapeState::Symbolic;
                        None
                    }
                };
                let materialized = DataProductMaterializedComponent {
                    component: Some(component.id.clone()),
                    local_name: Some(component.local_name.clone()),
                    label: match info.shape {
                        DataShape::Tuple => component.external_label.clone(),
                        DataShape::Record => Some(component.external_label.as_ref().unwrap_or(&component.local_name).clone()),
                    },
                    ty,
                    source: component.source.clone(),
                    expansion_source: None,
                };
                if info.shape == DataShape::Record {
                    append_record_component(&mut components, materialized)?;
                } else {
                    components.push(materialized);
                }
            }
            DataProductContribution::Expansion(expansion) => {
                let Some(operand) = expansion.operand.canonical_type() else {
                    state = DataProductShapeState::Symbolic;
                    continue;
                };
                let operand = crate::types::instantiation::materialize_type(
                    store,
                    operand,
                    &instantiation,
                    crate::types::instantiation::RowMaterializationMode::PreserveUnboundStableTail,
                )?;
                match info.shape {
                    DataShape::Tuple => {
                        materialize_tuple_expansion(store, expansion, operand, &mut components, &mut state)?;
                    }
                    DataShape::Record => {
                        materialize_record_expansion(store, expansion, operand, &mut components, &mut state)?;
                    }
                }
            }
        }
    }

    Ok(DataProductShapeMaterialization {
        state,
        components: components.into_boxed_slice(),
    })
}

fn data_specialization_instantiation(
    store: &mut TypeStore,
    info: &DataInfo,
    specialized_type: TypeId,
) -> Result<crate::types::instantiation::GenericInstantiation, DataProductShapeMaterializationError> {
    let mut instantiation = crate::types::instantiation::GenericInstantiation::default();
    let Some(signature) = &info.generic_signature else {
        return Ok(instantiation);
    };

    if signature.parameters.is_empty() {
        return Ok(instantiation);
    }

    if let Some((owner, arguments)) = store.applied_nominal_static_parts(specialized_type) {
        if owner != info.owner {
            return Err(DataProductShapeMaterializationError::WrongOwner);
        }
        if arguments.len() != signature.parameters.len() {
            return Err(DataProductShapeMaterializationError::GenericArity);
        }
        for (&parameter, argument) in signature.parameters.iter().zip(arguments) {
            match argument {
                StaticArgument::RecordRow(row) if store.type_parameter(parameter).kind == KindId::RECORD_ROW => {
                    let data = store.record_row(row);
                    if data.tail != RecordRowTail::Parameter(parameter) || !data.fields.is_empty() {
                        instantiation.bind_row(parameter, row);
                    }
                }
                StaticArgument::Term(ty) if store.type_parameter(parameter).kind == store.kind_of(ty) => {
                    instantiation.bind_type(parameter, ty);
                }
                _ => return Err(DataProductShapeMaterializationError::GenericArgumentDomain),
            }
        }
        return Ok(instantiation);
    }

    // Intrinsic Tuple and Record applications reduce directly to structural
    // types. Recover their already-canonical pack/row argument without
    // nominalizing the semantic result.
    match store.get(specialized_type).clone() {
        TypeData::TupleFromPacks { .. } | TypeData::PackFromPacks { .. } | TypeData::Pack(_)
            if info.shape == DataShape::Tuple && signature.parameters.len() == 1 =>
        {
            let complete = store
                .project_complete(specialized_type)
                .map_err(|_| DataProductShapeMaterializationError::GenericArgumentDomain)?;
            let parameter = signature.parameters[0];
            if store.type_parameter(parameter).kind != KindId::COMPLETE_TYPE_PACK {
                return Err(DataProductShapeMaterializationError::GenericArgumentDomain);
            }
            instantiation.bind_type(parameter, complete);
            Ok(instantiation)
        }
        TypeData::Record(row) if info.shape == DataShape::Record && signature.parameters.len() == 1 => {
            let parameter = signature.parameters[0];
            if store.type_parameter(parameter).kind != KindId::RECORD_ROW {
                return Err(DataProductShapeMaterializationError::GenericArgumentDomain);
            }
            let data = store.record_row(row);
            if data.tail != RecordRowTail::Parameter(parameter) || !data.fields.is_empty() {
                instantiation.bind_row(parameter, row);
            }
            Ok(instantiation)
        }
        TypeData::Nominal { ref declaration } if declaration == &info.owner => Ok(instantiation),
        _ => Err(DataProductShapeMaterializationError::WrongOwner),
    }
}

fn materialize_tuple_expansion(
    store: &mut TypeStore,
    expansion: &DataProductExpansionSemantic,
    operand: TypeId,
    components: &mut Vec<DataProductMaterializedComponent>,
    state: &mut DataProductShapeState,
) -> Result<(), DataProductShapeMaterializationError> {
    let expected_kind = match expansion.mode {
        phalcom_ast::ast::ExpansionMode::Positional => KindId::TYPE_PACK,
        phalcom_ast::ast::ExpansionMode::Labeled => KindId::LABELED_TYPE_PACK,
        phalcom_ast::ast::ExpansionMode::Complete => KindId::COMPLETE_TYPE_PACK,
    };
    let actual_kind = store.kind_of(operand);
    if actual_kind != expected_kind {
        return Err(DataProductShapeMaterializationError::ExpansionKind {
            expected: expected_kind,
            actual: actual_kind,
        });
    }

    let (positionals, labeled) = match expansion.mode {
        phalcom_ast::ast::ExpansionMode::Positional => (Some(operand), None),
        phalcom_ast::ast::ExpansionMode::Labeled => (None, Some(operand)),
        phalcom_ast::ast::ExpansionMode::Complete => (
            Some(
                store
                    .project_positionals(operand)
                    .map_err(|_| DataProductShapeMaterializationError::GenericArgumentDomain)?,
            ),
            Some(
                store
                    .project_labeled(operand)
                    .map_err(|_| DataProductShapeMaterializationError::GenericArgumentDomain)?,
            ),
        ),
    };
    if let Some(positionals) = positionals {
        let concrete = append_positional_pack(store, positionals, expansion, components)?;
        if !concrete {
            *state = DataProductShapeState::Symbolic;
        }
    }
    if let Some(labeled) = labeled {
        let concrete = append_labeled_pack(store, labeled, expansion, components)?;
        if !concrete {
            *state = DataProductShapeState::Symbolic;
        }
    }
    Ok(())
}

fn append_positional_pack(
    store: &TypeStore,
    pack: TypeId,
    expansion: &DataProductExpansionSemantic,
    components: &mut Vec<DataProductMaterializedComponent>,
) -> Result<bool, DataProductShapeMaterializationError> {
    match store.get(pack).clone() {
        TypeData::TypePack(elements) => {
            components.extend(elements.iter().map(|&ty| DataProductMaterializedComponent {
                component: None,
                local_name: None,
                label: None,
                ty: Some(ty),
                source: None,
                expansion_source: expansion.source.clone(),
            }));
            Ok(true)
        }
        TypeData::TypePackConcat(segments) => {
            let mut concrete = true;
            for segment in segments.iter().copied() {
                concrete &= append_positional_pack(store, segment, expansion, components)?;
            }
            Ok(concrete)
        }
        TypeData::TypeMemberProjection(_) | TypeData::Parameter(_) => Ok(false),
        _ => Err(DataProductShapeMaterializationError::ExpansionKind {
            expected: KindId::TYPE_PACK,
            actual: store.kind_of(pack),
        }),
    }
}

fn append_labeled_pack(
    store: &TypeStore,
    pack: TypeId,
    expansion: &DataProductExpansionSemantic,
    components: &mut Vec<DataProductMaterializedComponent>,
) -> Result<bool, DataProductShapeMaterializationError> {
    match store.get(pack).clone() {
        TypeData::LabeledTypePack(elements) => {
            components.extend(elements.iter().map(|element| DataProductMaterializedComponent {
                component: None,
                local_name: Some(element.label.clone()),
                label: Some(element.label.clone()),
                ty: Some(element.element),
                source: None,
                expansion_source: expansion.source.clone(),
            }));
            Ok(true)
        }
        TypeData::LabeledTypePackConcat(segments) => {
            let mut concrete = true;
            for segment in segments.iter().copied() {
                concrete &= append_labeled_pack(store, segment, expansion, components)?;
            }
            Ok(concrete)
        }
        TypeData::TypeMemberProjection(_) | TypeData::Parameter(_) => Ok(false),
        _ => Err(DataProductShapeMaterializationError::ExpansionKind {
            expected: KindId::LABELED_TYPE_PACK,
            actual: store.kind_of(pack),
        }),
    }
}

fn materialize_record_expansion(
    store: &mut TypeStore,
    expansion: &DataProductExpansionSemantic,
    operand: TypeId,
    components: &mut Vec<DataProductMaterializedComponent>,
    state: &mut DataProductShapeState,
) -> Result<(), DataProductShapeMaterializationError> {
    if expansion.mode != phalcom_ast::ast::ExpansionMode::Labeled {
        return Err(DataProductShapeMaterializationError::ExpansionKind {
            expected: KindId::RECORD_ROW,
            actual: store.kind_of(operand),
        });
    }
    let row = match store.get(operand) {
        TypeData::Record(row) => *row,
        TypeData::TypeMemberProjection(_) if store.kind_of(operand) == KindId::RECORD_ROW => {
            store.record_row_checked(Vec::new(), RecordRowTail::Projection(operand))?
        }
        _ => return Err(DataProductShapeMaterializationError::InvalidRecordExpansion),
    };
    let row_data = store.record_row(row);
    for field in row_data.fields.iter() {
        let Some(name) = field.key.as_symbol() else {
            return Err(DataProductShapeMaterializationError::InvalidRecordExpansion);
        };
        append_record_component(
            components,
            DataProductMaterializedComponent {
                component: None,
                local_name: Some(name.into()),
                label: Some(name.into()),
                ty: Some(field.ty),
                source: None,
                expansion_source: expansion.source.clone(),
            },
        )?;
    }
    if row_data.tail != RecordRowTail::Closed {
        *state = DataProductShapeState::Symbolic;
    }
    Ok(())
}

fn append_record_component(
    components: &mut Vec<DataProductMaterializedComponent>,
    component: DataProductMaterializedComponent,
) -> Result<(), DataProductShapeMaterializationError> {
    if let Some(label) = component.label.as_ref() {
        if components.iter().any(|existing| existing.label.as_ref() == Some(label)) {
            return Err(DataProductShapeMaterializationError::DuplicateRecordField { field: label.clone() });
        }
    }
    components.push(component);
    Ok(())
}

/// Table of all published data declarations in a snapshot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DataSemanticTable {
    pub data_decls: Arc<crate::parent_map::ParentMap<DeclarationId, Arc<DataInfo>>>,
    module_data: std::collections::HashMap<phalcom_modules::identity::ModuleId, std::collections::BTreeSet<DeclarationId>>,
    parent: Option<Arc<Self>>,
}

impl DataSemanticTable {
    /// Shares canonical entries and starts with empty local contributions.
    pub fn with_parent(parent: Arc<Self>) -> Self {
        Self {
            data_decls: Arc::new(crate::parent_map::ParentMap::with_parent(parent.data_decls.clone())),
            module_data: std::collections::HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn from_frozen_entries_with_allocation_budget(
        entries: Vec<(DeclarationId, Arc<DataInfo>)>,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, crate::parent_map::FrozenParentMapError> {
        let data_decls = crate::parent_map::ParentMap::from_frozen_entries_with_allocation_budget(entries, allocations)?;
        let data_decls = allocations
            .arc(data_decls)
            .map_err(crate::parent_map::FrozenParentMapError::Allocation)?;
        Ok(Self {
            data_decls,
            module_data: std::collections::HashMap::new(),
            parent: None,
        })
    }

    pub fn insert_data(&mut self, info: Arc<DataInfo>) {
        self.module_data.entry(info.owner.module.clone()).or_default().insert(info.owner.clone());
        Arc::make_mut(&mut self.data_decls).insert(info.owner.clone(), info);
    }

    pub fn get(&self, owner: &DeclarationId) -> Option<&Arc<DataInfo>> {
        self.data_decls.get(owner)
    }

    pub fn data_info(&self, owner: &DeclarationId) -> Option<&Arc<DataInfo>> {
        self.get(owner)
    }

    pub fn remove_module(&mut self, module: &phalcom_modules::identity::ModuleId) {
        self.module_data.remove(module);
        Arc::make_mut(&mut self.data_decls).retain(|owner, _| &owner.module != module);
    }

    /// Iterates data declarations belonging to the specified module without full-table scanning.
    pub fn iter_module<'a>(&'a self, module: &'a phalcom_modules::identity::ModuleId) -> Box<dyn Iterator<Item = (&'a DeclarationId, &'a Arc<DataInfo>)> + 'a> {
        let local_iter = self
            .module_data
            .get(module)
            .into_iter()
            .flat_map(move |ids| ids.iter().filter_map(move |id| self.data_decls.get(id).map(|info| (id, info))));
        let frozen_iter = self
            .data_decls
            .iter_frozen()
            .filter(move |(owner, _)| &owner.module == module);
        match &self.parent {
            Some(parent) => Box::new(parent.iter_module(module).chain(frozen_iter).chain(local_iter)),
            None => Box::new(frozen_iter.chain(local_iter)),
        }
    }
}
