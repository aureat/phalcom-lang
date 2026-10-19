//! Relocates portable `SEMANTIC_WORLD` type-graph rows into the current `TypeStore`.

use crate::field_key::FieldKey;
use crate::identity::{CallableId, CallableOwnerId, DeclarationId, DispatchSide, ImplId, ImplLocalId, VariantId};
use crate::metadata::import::{resolve_stable_declaration, resolve_stable_module};
use crate::scalar::StaticInt;
use crate::traits::{AssociatedTypeRequirementId, TraitRef};
use crate::types::family::{FamilyMemberType, FamilyMemberTypeKind, FamilyOperationShape};
use crate::types::id::{KindId, RecordRowId, StaticDomainId, TypeId, TypeLambdaId, TypeParameterId, VariantTypeId};
use crate::types::kind::KindData;
use crate::types::parameter::{GenericParameterMode, SelfRole, SelfTypeTerm, TypeParameterData, TypeParameterOwner};
use crate::types::row::{AnonymousRecordRemainder, RecordRowField, RecordRowTail, StructuralMemberPresence};
use crate::types::store::{
    AlgebraicStaticTerm, AnonymousTupleRemainder, CallableParameterType, CallableType, LabeledTypePackElement, StaticArgument, StaticDomainData, TupleType,
    TupleTypeElement, TypeData, TypeStore,
};
use crate::types::type_lambda::{
    ScopedAnonymousRecordRemainder, ScopedCallableParameter, ScopedCallableType, ScopedLabeledTypePackElement, ScopedOpenRecord, ScopedPackProjectionLane,
    ScopedPackRemainder, ScopedPackType, ScopedRecordField, ScopedRecordRow, ScopedRecordRowTail, ScopedRecordTail, ScopedTupleElement, ScopedTypeData,
};
use crate::types::type_member::{ConstTerm, ProjectionSubject, TypeLevelProjection, TypeMemberArgument, TypeMemberDeclarationId, TypeMemberSelector};
use crate::types::variance::Variance;
use num_bigint::BigInt;
use phalcom_ast::ast::RestMode;
use phalcom_common::selector::{Selector, SelectorKind, SelectorSlot};
use phalcom_compiler_artifact::semantic::type_store as wire;
use phalcom_compiler_artifact::semantic::type_store::{
    ArtifactKindId, ArtifactRecordRowId, ArtifactScopedTypeId, ArtifactTypeId, ArtifactTypeParameterId, FamilyMemberKindV1, FieldKeyV1, KindV1, PackLaneV1,
    ParameterModeV1, PresenceV1, ProjectionSubjectV1, RowTailV1, ScopedRemainderV1, ScopedRowTailV1, ScopedTypeV1, SelectorKindV1, SelectorSlotV1,
    StaticArgumentV1, StaticDomainV1, TypeMemberArgumentV1, TypeMemberSelectorV1, TypeV1, VarianceV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_modules::ProjectUniverse;
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::identity::{StableCallableOwnerRef, StableDispatchSide};
use std::collections::HashSet;
use std::str::FromStr;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct ReconstructedTypeGraph {
    pub store: TypeStore,
    pub types: Box<[TypeId]>,
    pub kinds: Box<[KindId]>,
    pub parameters: Box<[TypeParameterId]>,
    pub rows: Box<[RecordRowId]>,
    pub families: Box<[crate::types::family::FamilyTypeId]>,
    pub variants: Box<[VariantTypeId]>,
    pub static_domains: Box<[StaticDomainId]>,
    pub static_constructors: Box<[crate::types::id::AlgebraicStaticConstructorId]>,
    pub scoped: Box<[crate::types::id::ScopedTypeId]>,
    pub lambdas: Box<[TypeLambdaId]>,
}

/// Current-main semantic type authority reconstructed from an artifact's portable graph.
///
/// The row maps are private because their numeric indices are transport references, not
/// semantic identities. Downstream semantic-product reconstruction uses them only while
/// importing the same validated image.
#[derive(Clone, Debug)]
pub struct ImportedTypeGraph {
    pub(crate) inner: ReconstructedTypeGraph,
}

impl ImportedTypeGraph {
    pub fn store(&self) -> &TypeStore {
        &self.inner.store
    }

    #[allow(dead_code)]
    pub(crate) fn type_id(&self, artifact_id: ArtifactTypeId) -> Option<TypeId> {
        self.inner.types.get(artifact_id.0 as usize).copied()
    }
}

pub(crate) fn reconstruct_type_graph(
    graph: &wire::TypeGraphV1,
    limits: &phalcom_compiler_artifact::limits::ValidationLimits,
    projects: &ProjectUniverse,
    allocations: &mut AllocationBudget,
) -> Result<ReconstructedTypeGraph, ArtifactError> {
    graph.validate(limits)?;
    let mut importer = TypeGraphImporter::new(graph, projects, allocations)?;

    // Resolving each table through the same memoized relocation path accepts
    // arbitrary acyclic row order while rejecting cycles the live interners
    // cannot represent.
    for index in 0..graph.types.len() {
        importer.resolve_type(index)?;
    }
    for index in 0..graph.kinds.len() {
        importer.resolve_kind(index)?;
    }
    for index in 0..graph.parameters.len() {
        importer.resolve_parameter(index)?;
    }
    for index in 0..graph.rows.len() {
        importer.resolve_row(index)?;
    }
    for index in 0..graph.families.len() {
        importer.resolve_family(index)?;
    }
    for index in 0..graph.variants.len() {
        importer.resolve_variant(index)?;
    }
    for index in 0..graph.static_domains.len() {
        importer.resolve_static_domain(index)?;
    }
    for index in 0..graph.static_constructors.len() {
        importer.resolve_static_constructor(index)?;
    }
    for index in 0..graph.scoped.len() {
        importer.resolve_scoped(index)?;
    }
    for index in 0..graph.lambdas.len() {
        importer.resolve_lambda(index)?;
    }
    for parameter in &graph.current_parameters {
        importer.resolve_parameter(parameter.0 as usize)?;
    }
    for override_row in &graph.variance_overrides {
        let owner = importer.declaration(&override_row.owner)?;
        importer
            .store
            .set_parameter_variance(owner, override_row.index, map_variance(override_row.variance.clone()));
    }

    importer.finish()
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum GraphNode {
    Type(usize),
    Kind(usize),
    Parameter(usize),
    Row(usize),
    Family(usize),
    Variant(usize),
    StaticDomain(usize),
    StaticConstructor(usize),
    Scoped(usize),
    Lambda(usize),
}

struct TypeGraphImporter<'a> {
    graph: &'a wire::TypeGraphV1,
    projects: &'a ProjectUniverse,
    allocations: &'a mut AllocationBudget,
    store: TypeStore,
    active: HashSet<GraphNode>,
    types: Vec<Option<TypeId>>,
    kinds: Vec<Option<KindId>>,
    parameters: Vec<Option<TypeParameterId>>,
    rows: Vec<Option<RecordRowId>>,
    families: Vec<Option<crate::types::family::FamilyTypeId>>,
    variants: Vec<Option<VariantTypeId>>,
    static_domains: Vec<Option<StaticDomainId>>,
    static_constructors: Vec<Option<crate::types::id::AlgebraicStaticConstructorId>>,
    scoped: Vec<Option<crate::types::id::ScopedTypeId>>,
    lambdas: Vec<Option<TypeLambdaId>>,
    parameter_slots: HashSet<(TypeParameterOwner, u32)>,
}

impl<'a> TypeGraphImporter<'a> {
    fn new(graph: &'a wire::TypeGraphV1, projects: &'a ProjectUniverse, allocations: &'a mut AllocationBudget) -> Result<Self, ArtifactError> {
        fn slots<T>(allocations: &mut AllocationBudget, len: usize) -> Result<Vec<Option<T>>, ArtifactError> {
            let mut result = allocations.vector::<Option<T>>(len).map_err(ArtifactError::Allocation)?;
            result.resize_with(len, || None);
            Ok(result)
        }

        allocations
            .reserve_layout(std::alloc::Layout::new::<TypeStore>())
            .map_err(ArtifactError::Allocation)?;
        let store = TypeStore::new();

        let types = slots(allocations, graph.types.len())?;
        let kinds = slots(allocations, graph.kinds.len())?;
        let parameters = slots(allocations, graph.parameters.len())?;
        let rows = slots(allocations, graph.rows.len())?;
        let families = slots(allocations, graph.families.len())?;
        let variants = slots(allocations, graph.variants.len())?;
        let static_domains = slots(allocations, graph.static_domains.len())?;
        let static_constructors = slots(allocations, graph.static_constructors.len())?;
        let scoped = slots(allocations, graph.scoped.len())?;
        let lambdas = slots(allocations, graph.lambdas.len())?;

        Ok(Self {
            graph,
            projects,
            allocations,
            store,
            active: HashSet::new(),
            types,
            kinds,
            parameters,
            rows,
            families,
            variants,
            static_domains,
            static_constructors,
            scoped,
            lambdas,
            parameter_slots: HashSet::new(),
        })
    }

    fn cycle(node: GraphNode) -> ArtifactError {
        referential(format!("semantic type graph contains an unrepresentable dependency cycle at {node:?}"))
    }

    fn begin(&mut self, node: GraphNode) -> Result<(), ArtifactError> {
        if !self.active.insert(node) {
            return Err(Self::cycle(node));
        }
        Ok(())
    }

    fn end(&mut self, node: GraphNode) {
        self.active.remove(&node);
    }

    fn declaration(&self, reference: &phalcom_type_meta::identity::StableDeclarationRef) -> Result<DeclarationId, ArtifactError> {
        resolve_stable_declaration(reference, self.projects)
            .ok_or_else(|| referential(format!("unresolved or noncanonical stable declaration owner: {reference:?}")))
    }

    fn type_id(&mut self, id: ArtifactTypeId) -> Result<TypeId, ArtifactError> {
        self.resolve_type(id.0 as usize)
    }

    fn kind_id(&mut self, id: ArtifactKindId) -> Result<KindId, ArtifactError> {
        self.resolve_kind(id.0 as usize)
    }

    fn parameter_id(&mut self, id: ArtifactTypeParameterId) -> Result<TypeParameterId, ArtifactError> {
        self.resolve_parameter(id.0 as usize)
    }

    fn row_id(&mut self, id: ArtifactRecordRowId) -> Result<RecordRowId, ArtifactError> {
        self.resolve_row(id.0 as usize)
    }

    fn scoped_id(&mut self, id: ArtifactScopedTypeId) -> Result<crate::types::id::ScopedTypeId, ArtifactError> {
        self.resolve_scoped(id.0 as usize)
    }

    fn static_argument(&mut self, argument: &StaticArgumentV1) -> Result<StaticArgument, ArtifactError> {
        Ok(match argument {
            StaticArgumentV1::Term { term } => StaticArgument::Term(self.type_id(*term)?),
            StaticArgumentV1::RecordRow { row } => StaticArgument::RecordRow(self.row_id(*row)?),
            StaticArgumentV1::DynamicType => StaticArgument::DynamicType,
        })
    }

    fn type_remainder(&mut self, remainder: &wire::TupleRemainderV1) -> Result<AnonymousTupleRemainder, ArtifactError> {
        Ok(match remainder {
            wire::TupleRemainderV1::None => AnonymousTupleRemainder::None,
            wire::TupleRemainderV1::Unconstrained => AnonymousTupleRemainder::Unconstrained,
            wire::TupleRemainderV1::Typed { ty } => AnonymousTupleRemainder::Typed(self.type_id(*ty)?),
        })
    }

    fn tuple_type(&mut self, tuple: &wire::TupleV1) -> Result<TupleType, ArtifactError> {
        let mut elements = self
            .allocations
            .vector::<TupleTypeElement>(tuple.elements.len())
            .map_err(ArtifactError::Allocation)?;
        for element in &tuple.elements {
            elements.push(TupleTypeElement {
                label: element.label.as_deref().map(str::to_owned).map(String::into_boxed_str),
                ty: self.type_id(element.ty)?,
            });
        }
        Ok(TupleType {
            elements: elements.into_boxed_slice(),
            positional_remainder: self.type_remainder(&tuple.positionals)?,
            labeled_remainder: self.type_remainder(&tuple.labeled)?,
        })
    }

    fn resolve_type(&mut self, index: usize) -> Result<TypeId, ArtifactError> {
        if let Some(id) = self.types.get(index).ok_or_else(|| referential(format!("type row {index} is out of bounds")))? {
            return Ok(*id);
        }
        let node = GraphNode::Type(index);
        self.begin(node)?;
        let graph = self.graph;
        let entry = graph
            .types
            .get(index)
            .ok_or_else(|| referential(format!("type row {index} is out of bounds")))?;
        let result = (|| {
            let kind = self.kind_id(entry.kind)?;
            let data = match &entry.form {
                TypeV1::Never => TypeData::Never,
                TypeV1::Unit => TypeData::Unit,
                TypeV1::ClassObject { declaration } => TypeData::ClassObject {
                    declaration: self.declaration(declaration)?,
                },
                TypeV1::Nominal { declaration } => TypeData::Nominal {
                    declaration: self.declaration(declaration)?,
                },
                TypeV1::StaticDomainForm { domain } => TypeData::StaticDomainForm {
                    domain: self.resolve_static_domain(domain.0 as usize)?,
                },
                TypeV1::Applied { origin, arguments } => {
                    let origin = self.type_id(*origin)?;
                    let mut converted = self.allocations.vector::<StaticArgument>(arguments.len()).map_err(ArtifactError::Allocation)?;
                    for argument in arguments {
                        converted.push(self.static_argument(argument)?);
                    }
                    TypeData::Applied {
                        origin,
                        arguments: converted.into_boxed_slice(),
                    }
                }
                TypeV1::ExactCase { variant, enum_type } => TypeData::ExactCase {
                    variant: self.resolve_variant(variant.0 as usize)?,
                    enum_type: self.type_id(*enum_type)?,
                },
                TypeV1::Union { members } => {
                    let mut converted = self.allocations.vector::<TypeId>(members.len()).map_err(ArtifactError::Allocation)?;
                    for member in members {
                        converted.push(self.type_id(*member)?);
                    }
                    TypeData::Union(converted.into_boxed_slice())
                }
                TypeV1::Tuple { tuple } => TypeData::Tuple(self.tuple_type(tuple)?),
                TypeV1::Record { row } => TypeData::Record(self.row_id(*row)?),
                TypeV1::TypePack { members } => {
                    let mut converted = self.allocations.vector::<TypeId>(members.len()).map_err(ArtifactError::Allocation)?;
                    for member in members {
                        converted.push(self.type_id(*member)?);
                    }
                    TypeData::TypePack(converted.into_boxed_slice())
                }
                TypeV1::LabeledTypePack { members } => {
                    let mut converted = self
                        .allocations
                        .vector::<LabeledTypePackElement>(members.len())
                        .map_err(ArtifactError::Allocation)?;
                    for member in members {
                        converted.push(LabeledTypePackElement {
                            label: member.label.as_str().into(),
                            element: self.type_id(member.ty)?,
                        });
                    }
                    TypeData::LabeledTypePack(converted.into_boxed_slice())
                }
                TypeV1::CompleteTypePack { positionals, labeled } => TypeData::CompleteTypePack {
                    positionals: self.type_id(*positionals)?,
                    labeled: self.type_id(*labeled)?,
                },
                TypeV1::TypePackConcat { members } => {
                    let mut converted = self.allocations.vector::<TypeId>(members.len()).map_err(ArtifactError::Allocation)?;
                    for member in members {
                        converted.push(self.type_id(*member)?);
                    }
                    TypeData::TypePackConcat(converted.into_boxed_slice())
                }
                TypeV1::LabeledTypePackConcat { members } => {
                    let mut converted = self.allocations.vector::<TypeId>(members.len()).map_err(ArtifactError::Allocation)?;
                    for member in members {
                        converted.push(self.type_id(*member)?);
                    }
                    TypeData::LabeledTypePackConcat(converted.into_boxed_slice())
                }
                TypeV1::TupleFromPacks { positionals, labeled } => TypeData::TupleFromPacks {
                    positionals: self.type_id(*positionals)?,
                    labeled: self.type_id(*labeled)?,
                },
                TypeV1::Pack { pack } => TypeData::Pack(crate::types::store::PackType {
                    elements: self.tuple_type(pack)?.elements,
                    positional_remainder: self.type_remainder(&pack.positionals)?,
                    labeled_remainder: self.type_remainder(&pack.labeled)?,
                }),
                TypeV1::PackFromPacks { positionals, labeled } => TypeData::PackFromPacks {
                    positionals: self.type_id(*positionals)?,
                    labeled: self.type_id(*labeled)?,
                },
                TypeV1::CallableFromPacks { positionals, labeled, result } => TypeData::CallableFromPacks {
                    positionals: self.type_id(*positionals)?,
                    labeled: self.type_id(*labeled)?,
                    return_type: self.type_id(*result)?,
                },
                TypeV1::Callable { callable } => {
                    let mut parameters = self
                        .allocations
                        .vector::<CallableParameterType>(callable.parameters.len())
                        .map_err(ArtifactError::Allocation)?;
                    for parameter in &callable.parameters {
                        parameters.push(CallableParameterType {
                            label: parameter.label.as_deref().map(str::to_owned).map(String::into_boxed_str),
                            ty: self.type_id(parameter.ty)?,
                            rest: map_rest_mode(parameter.rest.clone()),
                        });
                    }
                    TypeData::Callable(CallableType {
                        parameters: parameters.into_boxed_slice(),
                        return_type: self.type_id(callable.result)?,
                    })
                }
                TypeV1::Family { family } => {
                    let family_type = self.resolve_family(family.0 as usize)?;
                    TypeData::Family(family_type)
                }
                TypeV1::Parameter { parameter } => TypeData::Parameter(self.parameter_id(*parameter)?),
                TypeV1::Lambda { lambda } => TypeData::Lambda(self.resolve_lambda(lambda.0 as usize)?),
                TypeV1::SelfType { self_type } => TypeData::SelfType(SelfTypeTerm {
                    owner: self.declaration(&self_type.owner)?,
                    side: map_dispatch_side(self_type.side),
                    role: match self_type.role.clone() {
                        wire::SelfRoleV1::InstanceType => SelfRole::InstanceType,
                        wire::SelfRoleV1::ReceiverValue => SelfRole::ReceiverValue,
                    },
                }),
                TypeV1::AssociatedProjection { projection } => TypeData::AssociatedProjection(crate::types::store::AssociatedTypeProjection {
                    subject: self.type_id(projection.subject)?,
                    trait_ref: TraitRef::new(self.declaration(&projection.trait_owner)?, self.resolve_types(&projection.trait_arguments)?),
                    requirement: AssociatedTypeRequirementId {
                        owner: self.declaration(&projection.requirement_owner)?,
                        index: projection.requirement_index,
                    },
                }),
                TypeV1::TypeMemberProjection { projection } => {
                    let subject = match projection.subject.clone() {
                        ProjectionSubjectV1::Type { ty } => ProjectionSubject::Type(self.type_id(ty)?),
                        ProjectionSubjectV1::RecordRow { row } => ProjectionSubject::RecordRow(self.row_id(row)?),
                    };
                    let selector = map_member_selector(projection.selector.clone());
                    let mut arguments = self
                        .allocations
                        .vector::<TypeMemberArgument>(projection.arguments.len())
                        .map_err(ArtifactError::Allocation)?;
                    for argument in &projection.arguments {
                        arguments.push(match argument {
                            TypeMemberArgumentV1::Type { ty } => TypeMemberArgument::Type(self.type_id(*ty)?),
                            TypeMemberArgumentV1::ConstInt { decimal } => TypeMemberArgument::Const(ConstTerm::Int(parse_static_int(decimal)?)),
                            TypeMemberArgumentV1::RecordRow { row } => TypeMemberArgument::RecordRow(self.row_id(*row)?),
                        });
                    }
                    let source_declaration = projection
                        .source
                        .as_ref()
                        .map(|source| {
                            Ok::<TypeMemberDeclarationId, ArtifactError>(TypeMemberDeclarationId::new(
                                self.declaration(&source.owner)?,
                                if source.selector.as_ref() == "[]" {
                                    TypeMemberSelector::Subscript
                                } else {
                                    TypeMemberSelector::Named(source.selector.clone())
                                },
                            ))
                        })
                        .transpose()?;
                    if source_declaration.as_ref().is_some_and(|source| source.selector != selector) {
                        return Err(referential("type-member source identity disagrees with its projection selector"));
                    }
                    TypeData::TypeMemberProjection(TypeLevelProjection {
                        subject,
                        selector,
                        arguments: arguments.into_boxed_slice(),
                        source_declaration,
                    })
                }
                TypeV1::ConstInt { decimal } => TypeData::ConstInt(parse_static_int(decimal)?),
                TypeV1::StaticData {
                    domain,
                    constructor,
                    arguments,
                } => {
                    let mut converted = self.allocations.vector::<StaticArgument>(arguments.len()).map_err(ArtifactError::Allocation)?;
                    for argument in arguments {
                        converted.push(self.static_argument(argument)?);
                    }
                    TypeData::StaticData(AlgebraicStaticTerm {
                        domain: self.resolve_static_domain(domain.0 as usize)?,
                        constructor: self.resolve_static_constructor(constructor.0 as usize)?,
                        arguments: converted.into_boxed_slice(),
                    })
                }
            };
            self.validate_and_intern_type(data, kind)
        })();
        self.end(node);
        let id = result?;
        self.types[index] = Some(id);
        Ok(id)
    }

    fn resolve_types(&mut self, ids: &[ArtifactTypeId]) -> Result<Vec<TypeId>, ArtifactError> {
        let mut result = self.allocations.vector::<TypeId>(ids.len()).map_err(ArtifactError::Allocation)?;
        for id in ids {
            result.push(self.type_id(*id)?);
        }
        Ok(result)
    }

    fn validate_and_intern_type(&mut self, data: TypeData, declared_kind: KindId) -> Result<TypeId, ArtifactError> {
        let id = match data {
            TypeData::Never => self.store.never(),
            TypeData::Unit => self.store.unit(),
            TypeData::ClassObject { declaration } => {
                if declared_kind != KindId::TYPE {
                    return Err(referential("class-object form has a non-Type kind"));
                }
                self.store.class_object_type(declaration)
            }
            TypeData::Nominal { declaration } => self.store.nominal_form(declaration, declared_kind),
            TypeData::StaticDomainForm { domain } => {
                let expected = self.store.static_domain_kind(domain);
                if expected != declared_kind {
                    return Err(referential("static-domain form has an inconsistent declared kind"));
                }
                self.store.intern_with_kind(TypeData::StaticDomainForm { domain }, expected)
            }
            TypeData::Applied { origin, arguments } => {
                let mut argument_kinds = self.allocations.vector::<KindId>(arguments.len()).map_err(ArtifactError::Allocation)?;
                for argument in &arguments {
                    argument_kinds.push(match argument {
                        StaticArgument::Term(ty) => self.store.kind_of(*ty),
                        StaticArgument::RecordRow(_) => KindId::RECORD_ROW,
                        StaticArgument::DynamicType => KindId::TYPE,
                    });
                }
                let actual_kind = self
                    .store
                    .apply_kind(self.store.kind_of(origin), &argument_kinds)
                    .map_err(|error| referential(format!("invalid applied type form: {error}")))?;
                if actual_kind != declared_kind {
                    return Err(referential("applied type form has an inconsistent declared kind"));
                }
                self.store
                    .apply_static_form(origin, &arguments)
                    .map_err(|error| referential(format!("invalid applied type form: {error}")))?
            }
            TypeData::ExactCase { variant, enum_type } => {
                if declared_kind != KindId::TYPE {
                    return Err(referential("exact case type has a non-Type kind"));
                }
                self.store.intern_with_kind(TypeData::ExactCase { variant, enum_type }, declared_kind)
            }
            TypeData::Union(members) => {
                if declared_kind != KindId::TYPE {
                    return Err(referential("union type has a non-Type kind"));
                }
                if members.iter().any(|member| !self.store.is_proper_type(*member)) {
                    return Err(referential("union member is not a proper type"));
                }
                self.store.union(&members)
            }
            TypeData::Tuple(tuple) => {
                if declared_kind != KindId::TYPE {
                    return Err(referential("tuple type has a non-Type kind"));
                }
                self.validate_tuple(&tuple)?;
                self.store.open_tuple(tuple.elements, tuple.positional_remainder, tuple.labeled_remainder)
            }
            TypeData::Record(row) => {
                if declared_kind != KindId::TYPE {
                    return Err(referential("record type has a non-Type kind"));
                }
                self.store.intern_with_kind(TypeData::Record(row), declared_kind)
            }
            TypeData::TypePack(elements) => self.store.type_pack(elements).map_err(|error| referential(error.to_string()))?,
            TypeData::LabeledTypePack(elements) => self.store.labeled_type_pack(elements).map_err(|error| referential(error.to_string()))?,
            TypeData::CompleteTypePack { positionals, labeled } => self
                .store
                .complete_type_pack(positionals, labeled)
                .map_err(|error| referential(error.to_string()))?,
            TypeData::TypePackConcat(elements) => self.store.type_pack_concat(&elements).map_err(|error| referential(error.to_string()))?,
            TypeData::LabeledTypePackConcat(elements) => self.store.labeled_type_pack_concat(&elements).map_err(|error| referential(error.to_string()))?,
            TypeData::TupleFromPacks { positionals, labeled } => self
                .store
                .tuple_from_packs(positionals, labeled)
                .map_err(|error| referential(error.to_string()))?,
            TypeData::Pack(pack) => self
                .store
                .open_pack(pack.elements, pack.positional_remainder, pack.labeled_remainder)
                .map_err(|error| referential(error.to_string()))?,
            TypeData::PackFromPacks { positionals, labeled } => self
                .store
                .pack_from_packs(positionals, labeled)
                .map_err(|error| referential(error.to_string()))?,
            TypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => self
                .store
                .callable_from_packs(positionals, labeled, return_type)
                .map_err(|error| referential(error.to_string()))?,
            TypeData::Callable(callable) => {
                if declared_kind != KindId::TYPE {
                    return Err(referential("callable type has a non-Type kind"));
                }
                for parameter in callable.parameters.iter() {
                    if !self.store.is_proper_type(parameter.ty) {
                        return Err(referential("callable parameter is not a proper type"));
                    }
                }
                if !self.store.is_proper_type(callable.return_type) {
                    return Err(referential("callable result is not a proper type"));
                }
                self.store.callable(callable)
            }
            TypeData::Family(family) => {
                if declared_kind != KindId::TYPE {
                    return Err(referential("family type has a non-Type kind"));
                }
                self.store.intern_with_kind(TypeData::Family(family), declared_kind)
            }
            TypeData::Parameter(parameter) => {
                if declared_kind == KindId::RECORD_ROW || self.store.type_parameter(parameter).kind != declared_kind {
                    return Err(referential("parameter form has an inconsistent kind"));
                }
                self.store.parameter_form(parameter)
            }
            TypeData::Lambda(lambda) => {
                let result = self.store.type_lambda(lambda);
                if self.store.kind_of(result) != declared_kind {
                    return Err(referential("lambda form has an inconsistent kind"));
                }
                result
            }
            TypeData::SelfType(term) => {
                if declared_kind != KindId::TYPE {
                    return Err(referential("Self form has a non-Type kind"));
                }
                self.store.self_type(term)
            }
            TypeData::AssociatedProjection(projection) => {
                if declared_kind != KindId::TYPE {
                    return Err(referential("associated projection has a non-Type kind"));
                }
                self.store.intern_with_kind(TypeData::AssociatedProjection(projection), declared_kind)
            }
            TypeData::TypeMemberProjection(projection) => self.store.type_member_projection(projection, declared_kind),
            TypeData::ConstInt(value) => {
                let result = self.store.const_int(value);
                if self.store.kind_of(result) != declared_kind {
                    return Err(referential("integer constant has an inconsistent static-domain kind"));
                }
                result
            }
            TypeData::StaticData(term) => {
                let result = self
                    .store
                    .static_data(term.domain, term.constructor, term.arguments)
                    .map_err(|error| referential(error.to_string()))?;
                if self.store.kind_of(result) != declared_kind {
                    return Err(referential("static data term has an inconsistent static-domain kind"));
                }
                result
            }
        };
        if self.store.kind_of(id) != declared_kind {
            return Err(referential("reconstructed type form does not match its declared kind"));
        }
        Ok(id)
    }

    fn resolve_kind(&mut self, index: usize) -> Result<KindId, ArtifactError> {
        if let Some(id) = self.kinds.get(index).ok_or_else(|| referential(format!("kind row {index} is out of bounds")))? {
            return Ok(*id);
        }
        let node = GraphNode::Kind(index);
        self.begin(node)?;
        let graph = self.graph;
        let form = graph
            .kinds
            .get(index)
            .ok_or_else(|| referential(format!("kind row {index} is out of bounds")))?;
        let result: Result<KindId, ArtifactError> = (|| {
            let data = match form {
                KindV1::Type => KindData::Type,
                KindV1::RecordRow => KindData::RecordRow,
                KindV1::TypePack => KindData::TypePack,
                KindV1::LabeledTypePack => KindData::LabeledTypePack,
                KindV1::CompleteTypePack => KindData::CompleteTypePack,
                KindV1::Arrow { parameters, result } => KindData::Arrow {
                    parameters: self.resolve_kinds(parameters)?.into_boxed_slice(),
                    result: self.kind_id(*result)?,
                },
                KindV1::StaticDomain { domain } => KindData::StaticDomain(self.resolve_static_domain(domain.0 as usize)?),
            };
            Ok(self.store.intern_kind(data))
        })();
        self.end(node);
        let id = result?;
        self.kinds[index] = Some(id);
        Ok(id)
    }

    fn validate_tuple(&self, tuple: &TupleType) -> Result<(), ArtifactError> {
        let mut labels = HashSet::new();
        let mut saw_labeled = false;
        for element in tuple.elements.iter() {
            if !self.store.is_proper_type(element.ty) {
                return Err(referential("tuple element is not a proper type"));
            }
            if let Some(label) = &element.label {
                saw_labeled = true;
                if !labels.insert(label) {
                    return Err(referential("tuple contains a duplicate label"));
                }
            } else if saw_labeled {
                return Err(referential("unlabeled tuple element follows a labeled element"));
            }
        }
        for remainder in [tuple.positional_remainder, tuple.labeled_remainder] {
            if let AnonymousTupleRemainder::Typed(ty) = remainder
                && !self.store.is_proper_type(ty)
            {
                return Err(referential("tuple remainder is not a proper type"));
            }
        }
        Ok(())
    }

    fn resolve_kinds(&mut self, ids: &[ArtifactKindId]) -> Result<Vec<KindId>, ArtifactError> {
        let mut result = self.allocations.vector::<KindId>(ids.len()).map_err(ArtifactError::Allocation)?;
        for id in ids {
            result.push(self.kind_id(*id)?);
        }
        Ok(result)
    }

    fn resolve_parameter(&mut self, index: usize) -> Result<TypeParameterId, ArtifactError> {
        if let Some(id) = self
            .parameters
            .get(index)
            .ok_or_else(|| referential(format!("parameter row {index} is out of bounds")))?
        {
            return Ok(*id);
        }
        let node = GraphNode::Parameter(index);
        self.begin(node)?;
        let graph = self.graph;
        let row = graph
            .parameters
            .get(index)
            .ok_or_else(|| referential(format!("parameter row {index} is out of bounds")))?;
        let result = (|| {
            let owner = match &row.owner {
                wire::ParameterOwnerV1::Declaration { owner } => TypeParameterOwner::Declaration(self.declaration(owner)?),
                wire::ParameterOwnerV1::Callable { owner } => TypeParameterOwner::Callable(self.callable(owner)?),
                wire::ParameterOwnerV1::Impl { owner } => TypeParameterOwner::Impl(self.implementation(owner)?),
            };
            if !self.parameter_slots.insert((owner.clone(), row.index)) {
                return Err(referential("duplicate parameter owner/index in type graph"));
            }
            let data = TypeParameterData::new(owner, row.index, row.name.as_str(), self.kind_id(row.kind)?)
                .with_variance(map_variance(row.variance.clone()))
                .with_mode(map_parameter_mode(row.mode.clone()));
            Ok(self.store.intern_type_parameter(data))
        })();
        self.end(node);
        let id = result?;
        self.parameters[index] = Some(id);
        Ok(id)
    }

    fn resolve_row(&mut self, index: usize) -> Result<RecordRowId, ArtifactError> {
        if let Some(id) = self
            .rows
            .get(index)
            .ok_or_else(|| referential(format!("record row {index} is out of bounds")))?
        {
            return Ok(*id);
        }
        let node = GraphNode::Row(index);
        self.begin(node)?;
        let graph = self.graph;
        let row = graph
            .rows
            .get(index)
            .ok_or_else(|| referential(format!("record row {index} is out of bounds")))?;
        let result = (|| {
            let mut fields = self.allocations.vector::<RecordRowField>(row.fields.len()).map_err(ArtifactError::Allocation)?;
            for field in &row.fields {
                fields.push(RecordRowField {
                    key: self.field_key(&field.key)?,
                    ty: self.type_id(field.ty)?,
                    presence: map_presence(field.presence.clone()),
                });
            }
            let tail = match row.tail.clone() {
                RowTailV1::Closed => RecordRowTail::Closed,
                RowTailV1::Unconstrained { key_type } => RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained {
                    key_type: self.type_id(key_type)?,
                }),
                RowTailV1::Typed { key_type, value_type } => RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                    key_type: self.type_id(key_type)?,
                    value_type: self.type_id(value_type)?,
                }),
                RowTailV1::Parameter { parameter } => RecordRowTail::Parameter(self.parameter_id(parameter)?),
                RowTailV1::Projection { ty } => RecordRowTail::Projection(self.type_id(ty)?),
            };
            self.store.record_row_checked(fields, tail).map_err(|error| referential(error.to_string()))
        })();
        self.end(node);
        let id = result?;
        self.rows[index] = Some(id);
        Ok(id)
    }

    fn resolve_family(&mut self, index: usize) -> Result<crate::types::family::FamilyTypeId, ArtifactError> {
        if let Some(id) = self
            .families
            .get(index)
            .ok_or_else(|| referential(format!("family row {index} is out of bounds")))?
        {
            return Ok(*id);
        }
        let node = GraphNode::Family(index);
        self.begin(node)?;
        let graph = self.graph;
        let family = graph
            .families
            .get(index)
            .ok_or_else(|| referential(format!("family row {index} is out of bounds")))?;
        let result = (|| {
            let mut members = self
                .allocations
                .vector::<FamilyMemberType>(family.members.len())
                .map_err(ArtifactError::Allocation)?;
            for member in &family.members {
                let operation = FamilyOperationShape::new(
                    map_selector_kind(member.operation.kind.clone()),
                    member.operation.slots.iter().map(map_selector_slot).collect::<Vec<_>>().into_boxed_slice(),
                );
                members.push(FamilyMemberType {
                    operation,
                    member_kind: match member.kind.clone() {
                        FamilyMemberKindV1::Value => FamilyMemberTypeKind::Value,
                        FamilyMemberKindV1::Callable => FamilyMemberTypeKind::Callable,
                    },
                    ty: self.type_id(member.ty)?,
                });
            }
            let family_ty = self
                .store
                .family_type(members)
                .map_err(|error| referential(format!("invalid family type: {error:?}")))?;
            match self.store.get(family_ty) {
                TypeData::Family(id) => Ok(*id),
                _ => Err(referential("family interner did not produce a family form")),
            }
        })();
        self.end(node);
        let id = result?;
        self.families[index] = Some(id);
        Ok(id)
    }

    fn resolve_variant(&mut self, index: usize) -> Result<VariantTypeId, ArtifactError> {
        if let Some(id) = self
            .variants
            .get(index)
            .ok_or_else(|| referential(format!("variant row {index} is out of bounds")))?
        {
            return Ok(*id);
        }
        let node = GraphNode::Variant(index);
        self.begin(node)?;
        let graph = self.graph;
        let variant = graph
            .variants
            .get(index)
            .ok_or_else(|| referential(format!("variant row {index} is out of bounds")))?;
        let result: Result<VariantTypeId, ArtifactError> = (|| {
            let owner = self.declaration(&variant.owner)?;
            let selector = parse_selector(&variant.selector)?;
            Ok(self.store.intern_variant_identity(VariantId::new(owner, selector)))
        })();
        self.end(node);
        let id = result?;
        self.variants[index] = Some(id);
        Ok(id)
    }

    fn resolve_static_domain(&mut self, index: usize) -> Result<StaticDomainId, ArtifactError> {
        if let Some(id) = self
            .static_domains
            .get(index)
            .ok_or_else(|| referential(format!("static domain row {index} is out of bounds")))?
        {
            return Ok(*id);
        }
        let node = GraphNode::StaticDomain(index);
        self.begin(node)?;
        let graph = self.graph;
        let domain = graph
            .static_domains
            .get(index)
            .ok_or_else(|| referential(format!("static domain row {index} is out of bounds")))?;
        let result: Result<StaticDomainId, ArtifactError> = (|| {
            let data = match domain {
                StaticDomainV1::ConstInt => StaticDomainData::ConstInt,
                StaticDomainV1::Nominal { declaration } => StaticDomainData::Nominal {
                    declaration: self.declaration(declaration)?,
                },
                StaticDomainV1::Associated { declaration, arguments } => {
                    let owner = self.declaration(&declaration.owner)?;
                    let declaration = crate::identity::AssociatedStaticDomainDeclarationId::new(owner, declaration.name.clone());
                    let mut converted = self.allocations.vector::<StaticArgument>(arguments.len()).map_err(ArtifactError::Allocation)?;
                    for argument in arguments {
                        converted.push(self.static_argument(argument)?);
                    }
                    StaticDomainData::Associated {
                        declaration,
                        owner_arguments: converted.into_boxed_slice(),
                    }
                }
            };
            Ok(self.store.intern_static_domain(data))
        })();
        self.end(node);
        let id = result?;
        self.static_domains[index] = Some(id);
        Ok(id)
    }

    fn resolve_static_constructor(&mut self, index: usize) -> Result<crate::types::id::AlgebraicStaticConstructorId, ArtifactError> {
        if let Some(id) = self
            .static_constructors
            .get(index)
            .ok_or_else(|| referential(format!("static constructor row {index} is out of bounds")))?
        {
            return Ok(*id);
        }
        let node = GraphNode::StaticConstructor(index);
        self.begin(node)?;
        let graph = self.graph;
        let constructor = graph
            .static_constructors
            .get(index)
            .ok_or_else(|| referential(format!("static constructor row {index} is out of bounds")))?;
        let result: Result<crate::types::id::AlgebraicStaticConstructorId, ArtifactError> = (|| {
            let domain = self.resolve_static_domain(constructor.domain.0 as usize)?;
            let declaration = self.declaration(&constructor.declaration)?;
            let parameter_kinds = self.resolve_kinds(&constructor.parameter_kinds)?.into_boxed_slice();
            Ok(self
                .store
                .intern_static_constructor(domain, declaration, constructor.case_name.as_str(), parameter_kinds))
        })();
        self.end(node);
        let id = result?;
        self.static_constructors[index] = Some(id);
        Ok(id)
    }

    fn resolve_scoped(&mut self, index: usize) -> Result<crate::types::id::ScopedTypeId, ArtifactError> {
        if let Some(id) = self
            .scoped
            .get(index)
            .ok_or_else(|| referential(format!("scoped type row {index} is out of bounds")))?
        {
            return Ok(*id);
        }
        let node = GraphNode::Scoped(index);
        self.begin(node)?;
        let graph = self.graph;
        let form = graph
            .scoped
            .get(index)
            .ok_or_else(|| referential(format!("scoped type row {index} is out of bounds")))?;
        let result: Result<crate::types::id::ScopedTypeId, ArtifactError> = (|| {
            let data = match form {
                ScopedTypeV1::Bound { depth, index } => ScopedTypeData::Bound { depth: *depth, index: *index },
                ScopedTypeV1::Free { ty } => ScopedTypeData::Free(self.type_id(*ty)?),
                ScopedTypeV1::Applied { origin, arguments } => {
                    let mut converted = self
                        .allocations
                        .vector::<crate::types::id::ScopedTypeId>(arguments.len())
                        .map_err(ArtifactError::Allocation)?;
                    for argument in arguments {
                        converted.push(self.scoped_id(*argument)?);
                    }
                    ScopedTypeData::Applied {
                        origin: self.scoped_id(*origin)?,
                        arguments: converted.into_boxed_slice(),
                    }
                }
                ScopedTypeV1::Union { members } => ScopedTypeData::Union(self.resolve_scoped_ids(members)?.into_boxed_slice()),
                ScopedTypeV1::Tuple { elements } => ScopedTypeData::Tuple(self.scoped_elements(elements)?.into_boxed_slice()),
                ScopedTypeV1::Record { fields } => ScopedTypeData::Record(self.scoped_fields(fields)?.into_boxed_slice()),
                ScopedTypeV1::OpenRecord { row } => {
                    let (fields, tail) = self.scoped_row(row)?;
                    let tail = match tail {
                        ScopedRecordRowTail::Closed => return Err(referential("open-record form has a closed row tail")),
                        ScopedRecordRowTail::Bound { depth, index } => ScopedRecordTail::Bound { depth, index },
                        ScopedRecordRowTail::FreeParameter(parameter) => ScopedRecordTail::FreeParameter(parameter),
                        ScopedRecordRowTail::Anonymous(remainder) => ScopedRecordTail::Anonymous(remainder),
                        ScopedRecordRowTail::Projection(_) => return Err(referential("open-record form has a projected row tail")),
                    };
                    ScopedTypeData::OpenRecord(ScopedOpenRecord { fields, tail })
                }
                ScopedTypeV1::RecordRow { row } => {
                    let (fields, tail) = self.scoped_row(row)?;
                    ScopedTypeData::RecordRow(ScopedRecordRow { fields, tail })
                }
                ScopedTypeV1::Callable { callable } => {
                    let mut parameters = self
                        .allocations
                        .vector::<ScopedCallableParameter>(callable.parameters.len())
                        .map_err(ArtifactError::Allocation)?;
                    for parameter in &callable.parameters {
                        parameters.push(ScopedCallableParameter {
                            label: parameter.label.as_deref().map(str::to_owned).map(String::into_boxed_str),
                            ty: self.scoped_id(parameter.ty)?,
                            rest: parameter.rest,
                        });
                    }
                    ScopedTypeData::Callable(ScopedCallableType {
                        parameters: parameters.into_boxed_slice(),
                        return_type: self.scoped_id(callable.result)?,
                    })
                }
                ScopedTypeV1::Pack { pack } => ScopedTypeData::Pack(ScopedPackType {
                    elements: self.scoped_elements(&pack.elements)?.into_boxed_slice(),
                    positional_remainder: self.scoped_remainder(pack.positionals.clone())?,
                    labeled_remainder: self.scoped_remainder(pack.labeled.clone())?,
                }),
                ScopedTypeV1::TypePack { members } => ScopedTypeData::TypePack(self.resolve_scoped_ids(members)?.into_boxed_slice()),
                ScopedTypeV1::LabeledTypePack { members } => {
                    let mut converted = self
                        .allocations
                        .vector::<ScopedLabeledTypePackElement>(members.len())
                        .map_err(ArtifactError::Allocation)?;
                    for member in members {
                        converted.push(ScopedLabeledTypePackElement {
                            label: member.label.as_str().into(),
                            element: self.scoped_id(member.ty)?,
                        });
                    }
                    ScopedTypeData::LabeledTypePack(converted.into_boxed_slice())
                }
                ScopedTypeV1::CompleteTypePack { positionals, labeled } => ScopedTypeData::CompleteTypePack {
                    positionals: self.scoped_id(*positionals)?,
                    labeled: self.scoped_id(*labeled)?,
                },
                ScopedTypeV1::TypePackConcat { members } => ScopedTypeData::TypePackConcat(self.resolve_scoped_ids(members)?.into_boxed_slice()),
                ScopedTypeV1::LabeledTypePackConcat { members } => ScopedTypeData::LabeledTypePackConcat(self.resolve_scoped_ids(members)?.into_boxed_slice()),
                ScopedTypeV1::TupleFromPacks { positionals, labeled } => ScopedTypeData::TupleFromPacks {
                    positionals: self.scoped_id(*positionals)?,
                    labeled: self.scoped_id(*labeled)?,
                },
                ScopedTypeV1::PackFromPacks { positionals, labeled } => ScopedTypeData::PackFromPacks {
                    positionals: self.scoped_id(*positionals)?,
                    labeled: self.scoped_id(*labeled)?,
                },
                ScopedTypeV1::CallableFromPacks { positionals, labeled, result } => ScopedTypeData::CallableFromPacks {
                    positionals: self.scoped_id(*positionals)?,
                    labeled: self.scoped_id(*labeled)?,
                    return_type: self.scoped_id(*result)?,
                },
                ScopedTypeV1::PackProjection { subject, lane } => ScopedTypeData::PackProjection {
                    subject: self.scoped_id(*subject)?,
                    lane: match lane {
                        PackLaneV1::Positionals => ScopedPackProjectionLane::Positionals,
                        PackLaneV1::Labeled => ScopedPackProjectionLane::Labeled,
                        PackLaneV1::Complete => ScopedPackProjectionLane::Complete,
                    },
                },
                ScopedTypeV1::Lambda { lambda } => ScopedTypeData::Lambda(self.resolve_lambda(lambda.0 as usize)?),
            };
            Ok(self.store.arena_mut().intern_scoped(data))
        })();
        self.end(node);
        let id = result?;
        self.scoped[index] = Some(id);
        Ok(id)
    }

    fn resolve_scoped_ids(&mut self, ids: &[ArtifactScopedTypeId]) -> Result<Vec<crate::types::id::ScopedTypeId>, ArtifactError> {
        let mut result = self
            .allocations
            .vector::<crate::types::id::ScopedTypeId>(ids.len())
            .map_err(ArtifactError::Allocation)?;
        for id in ids {
            result.push(self.scoped_id(*id)?);
        }
        Ok(result)
    }

    fn scoped_elements(&mut self, elements: &[wire::ScopedElementV1]) -> Result<Vec<ScopedTupleElement>, ArtifactError> {
        let mut result = self
            .allocations
            .vector::<ScopedTupleElement>(elements.len())
            .map_err(ArtifactError::Allocation)?;
        for element in elements {
            result.push(ScopedTupleElement {
                label: element.label.as_deref().map(str::to_owned).map(String::into_boxed_str),
                ty: self.scoped_id(element.ty)?,
            });
        }
        Ok(result)
    }

    fn scoped_fields(&mut self, fields: &[wire::ScopedFieldV1]) -> Result<Vec<ScopedRecordField>, ArtifactError> {
        let mut result = self.allocations.vector::<ScopedRecordField>(fields.len()).map_err(ArtifactError::Allocation)?;
        for field in fields {
            result.push(ScopedRecordField {
                key: self.field_key(&field.key)?,
                ty: self.scoped_id(field.ty)?,
                presence: map_presence(field.presence.clone()),
            });
        }
        Ok(result)
    }

    fn scoped_row(&mut self, row: &wire::ScopedRowV1) -> Result<(Box<[ScopedRecordField]>, ScopedRecordRowTail), ArtifactError> {
        let fields = self.scoped_fields(&row.fields)?.into_boxed_slice();
        let tail = match row.tail.clone() {
            ScopedRowTailV1::Closed => ScopedRecordRowTail::Closed,
            ScopedRowTailV1::Bound { depth, index } => ScopedRecordRowTail::Bound { depth, index },
            ScopedRowTailV1::Parameter { parameter } => ScopedRecordRowTail::FreeParameter(self.parameter_id(parameter)?),
            ScopedRowTailV1::Unconstrained { key_type } => ScopedRecordRowTail::Anonymous(ScopedAnonymousRecordRemainder::Unconstrained {
                key_type: self.scoped_id(key_type)?,
            }),
            ScopedRowTailV1::Typed { key_type, value_type } => ScopedRecordRowTail::Anonymous(ScopedAnonymousRecordRemainder::Typed {
                key_type: self.scoped_id(key_type)?,
                value_type: self.scoped_id(value_type)?,
            }),
            ScopedRowTailV1::Projection { ty } => ScopedRecordRowTail::Projection(self.type_id(ty)?),
        };
        Ok((fields, tail))
    }

    fn scoped_remainder(&mut self, remainder: ScopedRemainderV1) -> Result<ScopedPackRemainder, ArtifactError> {
        Ok(match remainder {
            ScopedRemainderV1::None => ScopedPackRemainder::None,
            ScopedRemainderV1::Unconstrained => ScopedPackRemainder::Unconstrained,
            ScopedRemainderV1::Typed { ty } => ScopedPackRemainder::Typed(self.scoped_id(ty)?),
        })
    }

    fn resolve_lambda(&mut self, index: usize) -> Result<TypeLambdaId, ArtifactError> {
        if let Some(id) = self
            .lambdas
            .get(index)
            .ok_or_else(|| referential(format!("lambda row {index} is out of bounds")))?
        {
            return Ok(*id);
        }
        let node = GraphNode::Lambda(index);
        self.begin(node)?;
        let graph = self.graph;
        let lambda = graph
            .lambdas
            .get(index)
            .ok_or_else(|| referential(format!("lambda row {index} is out of bounds")))?;
        let result: Result<TypeLambdaId, ArtifactError> = (|| {
            if lambda.parameter_kinds.len() != lambda.parameter_modes.len() {
                return Err(referential("lambda parameter kind/mode counts differ"));
            }
            let parameter_kinds = self.resolve_kinds(&lambda.parameter_kinds)?.into_boxed_slice();
            let mut parameter_modes = self
                .allocations
                .vector::<GenericParameterMode>(lambda.parameter_modes.len())
                .map_err(ArtifactError::Allocation)?;
            for mode in &lambda.parameter_modes {
                parameter_modes.push(map_parameter_mode(mode.clone()));
            }
            let body = self.scoped_id(lambda.body)?;
            let result_kind = self.kind_id(lambda.result_kind)?;
            let id = self
                .store
                .arena_mut()
                .intern_lambda_with_modes(parameter_kinds, parameter_modes.into_boxed_slice(), body, result_kind, None);
            Ok(id)
        })();
        self.end(node);
        let id = result?;
        self.lambdas[index] = Some(id);
        Ok(id)
    }

    fn field_key(&self, key: &FieldKeyV1) -> Result<FieldKey, ArtifactError> {
        Ok(match key {
            FieldKeyV1::Symbol { value } => FieldKey::symbol(value.as_str()),
            FieldKeyV1::String { value } => FieldKey::string(value.as_str()),
            FieldKeyV1::Int { decimal } => FieldKey::int(parse_static_int(decimal)?),
            FieldKeyV1::Bool { value } => FieldKey::bool(*value),
        })
    }

    fn callable(&self, reference: &phalcom_type_meta::identity::StableCallableRef) -> Result<CallableId, ArtifactError> {
        relocate_stable_callable(reference, self.projects)
    }

    fn implementation(&self, reference: &phalcom_type_meta::identity::StableImplRef) -> Result<ImplId, ArtifactError> {
        relocate_stable_impl(reference, self.projects)
    }

    fn finish(self) -> Result<ReconstructedTypeGraph, ArtifactError> {
        fn unwrap<T>(values: Vec<Option<T>>, family: &str) -> Result<Box<[T]>, ArtifactError> {
            let mut result = Vec::with_capacity(values.len());
            for (index, value) in values.into_iter().enumerate() {
                result.push(value.ok_or_else(|| referential(format!("{family} row {index} was not reconstructed")))?);
            }
            Ok(result.into_boxed_slice())
        }
        Ok(ReconstructedTypeGraph {
            store: self.store,
            types: unwrap(self.types, "type")?,
            kinds: unwrap(self.kinds, "kind")?,
            parameters: unwrap(self.parameters, "parameter")?,
            rows: unwrap(self.rows, "record row")?,
            families: unwrap(self.families, "family")?,
            variants: unwrap(self.variants, "variant")?,
            static_domains: unwrap(self.static_domains, "static domain")?,
            static_constructors: unwrap(self.static_constructors, "static constructor")?,
            scoped: unwrap(self.scoped, "scoped type")?,
            lambdas: unwrap(self.lambdas, "lambda")?,
        })
    }
}

pub(crate) fn relocate_stable_module(
    reference: &phalcom_type_meta::identity::StableModuleRef,
    projects: &ProjectUniverse,
) -> Result<crate::identity::ModuleId, ArtifactError> {
    resolve_stable_module(reference, projects).ok_or_else(|| referential("unresolved stable module identity"))
}

pub(crate) fn relocate_stable_declaration(
    reference: &phalcom_type_meta::identity::StableDeclarationRef,
    projects: &ProjectUniverse,
) -> Result<DeclarationId, ArtifactError> {
    resolve_stable_declaration(reference, projects).ok_or_else(|| referential("unresolved stable declaration identity"))
}

pub(crate) fn relocate_stable_variant(
    reference: &phalcom_type_meta::identity::StableVariantRef,
    projects: &ProjectUniverse,
) -> Result<VariantId, ArtifactError> {
    Ok(VariantId::new(
        relocate_stable_declaration(&reference.owner, projects)?,
        parse_selector(&reference.selector)?,
    ))
}

pub(crate) fn relocate_stable_impl(reference: &phalcom_type_meta::identity::StableImplRef, projects: &ProjectUniverse) -> Result<ImplId, ArtifactError> {
    Ok(ImplId::new(
        relocate_stable_module(&reference.module, projects)?,
        ImplLocalId(reference.local_id),
    ))
}

pub(crate) fn relocate_stable_callable(
    reference: &phalcom_type_meta::identity::StableCallableRef,
    projects: &ProjectUniverse,
) -> Result<CallableId, ArtifactError> {
    let owner = match &reference.owner {
        StableCallableOwnerRef::Declaration(owner) => CallableOwnerId::Declaration(relocate_stable_declaration(owner, projects)?),
        StableCallableOwnerRef::Variant(owner) => CallableOwnerId::Variant(relocate_stable_variant(owner, projects)?),
        StableCallableOwnerRef::Conformance(owner) => CallableOwnerId::Conformance(relocate_stable_impl(owner, projects)?),
    };
    Ok(CallableId::new(owner, parse_selector(&reference.selector)?, map_dispatch_side(reference.side)))
}

pub(crate) fn map_dispatch_side(side: StableDispatchSide) -> DispatchSide {
    match side {
        StableDispatchSide::Instance => DispatchSide::Instance,
        StableDispatchSide::Class => DispatchSide::Class,
    }
}

fn map_variance(variance: VarianceV1) -> Variance {
    match variance {
        VarianceV1::Covariant => Variance::Covariant,
        VarianceV1::Contravariant => Variance::Contravariant,
        VarianceV1::Invariant => Variance::Invariant,
    }
}

fn map_parameter_mode(mode: ParameterModeV1) -> GenericParameterMode {
    match mode {
        ParameterModeV1::Single => GenericParameterMode::Single,
        ParameterModeV1::PositionalPack => GenericParameterMode::PositionalPack,
        ParameterModeV1::LabeledPack => GenericParameterMode::LabeledPack,
        ParameterModeV1::CompletePack => GenericParameterMode::CompletePack,
    }
}

fn map_rest_mode(mode: wire::RestModeV1) -> RestMode {
    match mode {
        wire::RestModeV1::None => RestMode::None,
        wire::RestModeV1::Positional => RestMode::Positional,
        wire::RestModeV1::Labeled => RestMode::Labeled,
        wire::RestModeV1::Complete => RestMode::Complete,
    }
}

fn map_selector_kind(kind: SelectorKindV1) -> SelectorKind {
    match kind {
        SelectorKindV1::Getter => SelectorKind::Getter,
        SelectorKindV1::Setter => SelectorKind::Setter,
        SelectorKindV1::Method => SelectorKind::Method,
        SelectorKindV1::SubscriptGet => SelectorKind::SubscriptGet,
        SelectorKindV1::SubscriptSet => SelectorKind::SubscriptSet,
    }
}

fn map_selector_slot(slot: &SelectorSlotV1) -> SelectorSlot {
    match slot {
        SelectorSlotV1::Positional => SelectorSlot::Positional,
        SelectorSlotV1::Label { name } => SelectorSlot::Label(name.clone()),
    }
}

fn map_presence(presence: PresenceV1) -> StructuralMemberPresence {
    match presence {
        PresenceV1::Required => StructuralMemberPresence::Required,
        PresenceV1::Optional => StructuralMemberPresence::Optional,
    }
}

fn map_member_selector(selector: TypeMemberSelectorV1) -> TypeMemberSelector {
    match selector {
        TypeMemberSelectorV1::Named { name } if name == "At" => TypeMemberSelector::Subscript,
        TypeMemberSelectorV1::Named { name } => TypeMemberSelector::Named(name.into_boxed_str()),
        TypeMemberSelectorV1::Subscript => TypeMemberSelector::Subscript,
    }
}

pub(crate) fn parse_selector(encoded: &str) -> Result<Selector, ArtifactError> {
    let selector = Selector::decode(encoded);
    if selector.encode() != encoded {
        return Err(referential(format!("noncanonical stable selector: {encoded:?}")));
    }
    Ok(selector)
}

fn parse_static_int(decimal: &str) -> Result<StaticInt, ArtifactError> {
    let value = BigInt::from_str(decimal).map_err(|_| referential(format!("invalid static integer: {decimal:?}")))?;
    if value.to_string() != decimal {
        return Err(referential(format!("noncanonical static integer: {decimal:?}")));
    }
    Ok(StaticInt::new(value))
}

fn referential(message: impl Into<String>) -> ArtifactError {
    ArtifactError::Validation(ValidationError::ReferentialIntegrity(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_compiler_artifact::semantic::type_store::{
        ArtifactKindId, ArtifactScopedTypeId, ArtifactTypeId, ArtifactTypeLambdaId, KindV1, LambdaV1, ScopedTypeV1, TupleElementV1, TupleRemainderV1,
        TypeEntryV1, TypeGraphV1,
    };

    fn graph(types: Vec<TypeEntryV1>, kinds: Vec<KindV1>, scoped: Vec<ScopedTypeV1>, lambdas: Vec<LambdaV1>) -> TypeGraphV1 {
        TypeGraphV1 {
            types,
            kinds,
            parameters: vec![],
            current_parameters: vec![],
            rows: vec![],
            families: vec![],
            variants: vec![],
            static_domains: vec![],
            static_constructors: vec![],
            scoped,
            lambdas,
            variance_overrides: vec![],
        }
    }

    #[test]
    fn relocates_forward_type_and_kind_rows_through_nested_lambda() {
        let graph = graph(
            vec![
                TypeEntryV1 {
                    kind: ArtifactKindId(1),
                    form: TypeV1::Unit,
                },
                TypeEntryV1 {
                    kind: ArtifactKindId(0),
                    form: TypeV1::Lambda {
                        lambda: ArtifactTypeLambdaId(0),
                    },
                },
                TypeEntryV1 {
                    kind: ArtifactKindId(1),
                    form: TypeV1::Tuple {
                        tuple: wire::TupleV1 {
                            elements: vec![TupleElementV1 {
                                label: None,
                                ty: ArtifactTypeId(0),
                            }],
                            positionals: TupleRemainderV1::None,
                            labeled: TupleRemainderV1::None,
                        },
                    },
                },
            ],
            vec![
                KindV1::Arrow {
                    parameters: vec![ArtifactKindId(1)],
                    result: ArtifactKindId(1),
                },
                KindV1::Type,
            ],
            vec![ScopedTypeV1::Free { ty: ArtifactTypeId(2) }],
            vec![LambdaV1 {
                parameter_kinds: vec![ArtifactKindId(1)],
                parameter_modes: vec![ParameterModeV1::Single],
                body: ArtifactScopedTypeId(0),
                result_kind: ArtifactKindId(1),
            }],
        );
        let mut budget = AllocationBudget::new(1024 * 1024);
        let imported = reconstruct_type_graph(&graph, &Default::default(), &ProjectUniverse::new(), &mut budget).unwrap();

        let TypeData::Tuple(tuple) = imported.store.get(imported.types[2]) else {
            panic!("forward tuple row was not reconstructed");
        };
        assert_eq!(tuple.elements[0].ty, imported.types[0]);

        let TypeData::Lambda(lambda_id) = imported.store.get(imported.types[1]) else {
            panic!("lambda row was not reconstructed");
        };
        let lambda = imported.store.arena().get_lambda(*lambda_id);
        assert_eq!(imported.store.get_kind(lambda.parameter_kinds[0]), &KindData::Type);
        assert_eq!(lambda.result_kind, imported.kinds[1]);
        assert!(matches!(imported.store.arena().get_scoped(lambda.body), ScopedTypeData::Free(ty) if *ty == imported.types[2]));
    }

    #[test]
    fn rejects_cycles_that_current_type_store_cannot_represent() {
        let graph = graph(
            vec![TypeEntryV1 {
                kind: ArtifactKindId(0),
                form: TypeV1::Union {
                    members: vec![ArtifactTypeId(0)],
                },
            }],
            vec![KindV1::Type],
            vec![],
            vec![],
        );
        let mut budget = AllocationBudget::new(1024 * 1024);
        let error = reconstruct_type_graph(&graph, &Default::default(), &ProjectUniverse::new(), &mut budget).unwrap_err();
        assert!(matches!(error, ArtifactError::Validation(ValidationError::ReferentialIntegrity(message)) if message.contains("dependency cycle")));
    }

    #[test]
    fn rejects_scoped_lambda_cycles_before_interning_them() {
        let graph = graph(
            vec![TypeEntryV1 {
                kind: ArtifactKindId(0),
                form: TypeV1::Lambda {
                    lambda: ArtifactTypeLambdaId(0),
                },
            }],
            vec![KindV1::Type],
            vec![ScopedTypeV1::Lambda {
                lambda: ArtifactTypeLambdaId(0),
            }],
            vec![LambdaV1 {
                parameter_kinds: vec![],
                parameter_modes: vec![],
                body: ArtifactScopedTypeId(0),
                result_kind: ArtifactKindId(0),
            }],
        );
        let mut budget = AllocationBudget::new(1024 * 1024);
        let error = reconstruct_type_graph(&graph, &Default::default(), &ProjectUniverse::new(), &mut budget).unwrap_err();
        assert!(matches!(error, ArtifactError::Validation(ValidationError::ReferentialIntegrity(message)) if message.contains("dependency cycle")));
    }

    #[test]
    fn rejects_invalid_tuple_shape_without_panicking() {
        let graph = graph(
            vec![
                TypeEntryV1 {
                    kind: ArtifactKindId(0),
                    form: TypeV1::Unit,
                },
                TypeEntryV1 {
                    kind: ArtifactKindId(0),
                    form: TypeV1::Tuple {
                        tuple: wire::TupleV1 {
                            elements: vec![
                                TupleElementV1 {
                                    label: Some("x".into()),
                                    ty: ArtifactTypeId(0),
                                },
                                TupleElementV1 {
                                    label: Some("x".into()),
                                    ty: ArtifactTypeId(0),
                                },
                            ],
                            positionals: TupleRemainderV1::None,
                            labeled: TupleRemainderV1::None,
                        },
                    },
                },
            ],
            vec![KindV1::Type],
            vec![],
            vec![],
        );
        let mut budget = AllocationBudget::new(1024 * 1024);
        let error = reconstruct_type_graph(&graph, &Default::default(), &ProjectUniverse::new(), &mut budget).unwrap_err();
        assert!(matches!(error, ArtifactError::Validation(ValidationError::ReferentialIntegrity(message)) if message.contains("duplicate label")));
    }
}
