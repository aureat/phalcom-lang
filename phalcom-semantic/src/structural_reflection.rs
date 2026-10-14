//! Canonical compiler-derived structural reflection projector.
//!
//! Evaluates Product, OrderedProduct, UnorderedProduct, Sum, and domain-native
//! UnorderedShape members to reduced `StaticValueRef` values or domain-neutral
//! `MemberProjection::Stuck` outcomes without fabricating TypeStore nodes or ImplIds.

use crate::core_surface::identity::StructuralUniverseIds;
use crate::data_semantics::{DataProductShapeMaterializationError, DataProductShapeState, DataSemanticTable};
use crate::declarations::DeclarationTypeTable;
use crate::enum_semantics::{AlgebraicCasePayloadKind, EnumSemanticTable};
use crate::field_key::FieldKey;
use crate::generic_evidence::GenericConformanceEvidence;
use crate::identity::{DeclarationId, VariantId};
use crate::member_projection::{MemberDeclarationIdentity, MemberProjection, MemberProjectionArgument, MemberProjectionSubject, StaticMemberProjectionOutcome};
use crate::static_domain::StaticValueDomain;
use crate::static_value::{StaticValueData, StaticValueRef, StaticValueStore, StaticValueStoreError};
use crate::structural::{OrderedProductStructure, StructuralClassification, StructuralClassifier, SumStructureError, UnorderedProductStructure};
use crate::types::id::{KindId, TypeId};
use crate::types::row::{AnonymousRecordRemainder, RecordRowTail, StructuralMemberPresence};
use crate::types::store::{AnonymousTupleRemainder, TypeData, TypeStore};
use crate::types::type_member::{ConstTerm, IntrinsicTypeOperation, TypeMemberArgument};
use phalcom_common::selector::{Selector, SelectorBase};

/// Errors occurring during canonical structural reflection projection.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum StructuralReflectionError {
    #[error("static value store error: {0}")]
    ValueStore(StaticValueStoreError),
    #[error("data shape materialization error: {0}")]
    DataMaterialization(DataProductShapeMaterializationError),
    #[error("sum structure error: {0}")]
    SumStructure(SumStructureError),
    #[error("missing declaration info for {0:?}")]
    MissingDeclaration(DeclarationId),
    #[error("missing variant info for {0:?}")]
    MissingVariant(VariantId),
    #[error("type error: {0}")]
    TypeFormation(Box<str>),
}

impl From<StaticValueStoreError> for StructuralReflectionError {
    fn from(err: StaticValueStoreError) -> Self {
        Self::ValueStore(err)
    }
}

impl From<DataProductShapeMaterializationError> for StructuralReflectionError {
    fn from(err: DataProductShapeMaterializationError) -> Self {
        Self::DataMaterialization(err)
    }
}

impl From<SumStructureError> for StructuralReflectionError {
    fn from(err: SumStructureError) -> Self {
        Self::SumStructure(err)
    }
}

fn selector_name(sel: &Selector) -> Option<&str> {
    match &sel.base {
        SelectorBase::Named(name) => Some(name.as_str()),
        SelectorBase::Subscript => None,
    }
}

/// Authority for projecting structural capabilities and reflection data.
pub struct StructuralReflectionProjector<'a> {
    pub classifier: StructuralClassifier<'a>,
    pub ids: &'a StructuralUniverseIds,
    pub types: &'a mut TypeStore,
    pub values: &'a mut StaticValueStore,
    pub declarations: &'a DeclarationTypeTable,
    pub data: &'a DataSemanticTable,
    pub enums: &'a EnumSemanticTable,
}

impl<'a> StructuralReflectionProjector<'a> {
    pub fn new(
        classifier: StructuralClassifier<'a>,
        ids: &'a StructuralUniverseIds,
        types: &'a mut TypeStore,
        values: &'a mut StaticValueStore,
        declarations: &'a DeclarationTypeTable,
        data: &'a DataSemanticTable,
        enums: &'a EnumSemanticTable,
    ) -> Self {
        Self {
            classifier,
            ids,
            types,
            values,
            declarations,
            data,
            enums,
        }
    }

    /// Evaluates `Product::Kind` for `subject`.
    pub fn project_product_kind(&mut self, subject: TypeId) -> Result<StaticMemberProjectionOutcome, StructuralReflectionError> {
        let classification = self.classifier.classify(self.types, subject);
        match classification {
            StructuralClassification::OrderedProduct(_) => {
                let val = self.construct_product_kind_ordered()?;
                Ok(StaticMemberProjectionOutcome::ReducedValue(val))
            }
            StructuralClassification::UnorderedProduct(_) => {
                let val = self.construct_product_kind_unordered()?;
                Ok(StaticMemberProjectionOutcome::ReducedValue(val))
            }
            _ => {
                if self.is_abstract(subject) {
                    Ok(StaticMemberProjectionOutcome::Stuck(MemberProjection {
                        subject: MemberProjectionSubject::Type(subject),
                        member: MemberDeclarationIdentity::TraitAssociated {
                            trait_ref: crate::traits::TraitRef::new(self.ids.product.clone(), Vec::new()),
                            requirement: crate::traits::AssociatedTypeRequirementId::new(self.ids.product.clone(), 0),
                        },
                        arguments: Box::new([]),
                        evidence: None,
                        result_domain: StaticValueDomain::ValueType(self.declarations.form(&self.ids.product_kind).unwrap_or(subject)),
                    }))
                } else {
                    Ok(StaticMemberProjectionOutcome::Invalid("subject does not conform to Product".into()))
                }
            }
        }
    }

    /// Evaluates `Product::Shape` for `subject`.
    pub fn project_product_shape(&mut self, subject: TypeId) -> Result<StaticMemberProjectionOutcome, StructuralReflectionError> {
        let classification = self.classifier.classify(self.types, subject);
        match classification {
            StructuralClassification::OrderedProduct(_) => {
                let Some(structure) = self.classifier.ordered_product(self.types, subject)? else {
                    return Ok(StaticMemberProjectionOutcome::Invalid("ordered product classification failed".into()));
                };
                if structure.state == DataProductShapeState::Symbolic || structure.symbolic_lanes.is_some() {
                    return Ok(StaticMemberProjectionOutcome::Stuck(MemberProjection {
                        subject: MemberProjectionSubject::Type(subject),
                        member: MemberDeclarationIdentity::TraitAssociated {
                            trait_ref: crate::traits::TraitRef::new(self.ids.product.clone(), Vec::new()),
                            requirement: crate::traits::AssociatedTypeRequirementId::new(self.ids.product.clone(), 1),
                        },
                        arguments: Box::new([]),
                        evidence: None,
                        result_domain: StaticValueDomain::ValueType(self.declarations.form(&self.ids.product_shape).unwrap_or(subject)),
                    }));
                }
                let shape_val = self.project_ordered_shape_from_structure(&structure)?;
                let product_shape_val = self.wrap_product_shape_ordered(shape_val)?;
                Ok(StaticMemberProjectionOutcome::ReducedValue(product_shape_val))
            }
            StructuralClassification::UnorderedProduct(_) => {
                let Some(structure) = self.classifier.unordered_product(self.types, subject)? else {
                    return Ok(StaticMemberProjectionOutcome::Invalid("unordered product classification failed".into()));
                };
                if structure.state == DataProductShapeState::Symbolic {
                    return Ok(StaticMemberProjectionOutcome::Stuck(MemberProjection {
                        subject: MemberProjectionSubject::Type(subject),
                        member: MemberDeclarationIdentity::TraitAssociated {
                            trait_ref: crate::traits::TraitRef::new(self.ids.product.clone(), Vec::new()),
                            requirement: crate::traits::AssociatedTypeRequirementId::new(self.ids.product.clone(), 1),
                        },
                        arguments: Box::new([]),
                        evidence: None,
                        result_domain: StaticValueDomain::ValueType(self.declarations.form(&self.ids.product_shape).unwrap_or(subject)),
                    }));
                }
                let shape_val = self.project_unordered_shape_from_structure(&structure)?;
                let product_shape_val = self.wrap_product_shape_unordered(shape_val)?;
                Ok(StaticMemberProjectionOutcome::ReducedValue(product_shape_val))
            }
            _ => {
                if self.is_abstract(subject) {
                    Ok(StaticMemberProjectionOutcome::Stuck(MemberProjection {
                        subject: MemberProjectionSubject::Type(subject),
                        member: MemberDeclarationIdentity::TraitAssociated {
                            trait_ref: crate::traits::TraitRef::new(self.ids.product.clone(), Vec::new()),
                            requirement: crate::traits::AssociatedTypeRequirementId::new(self.ids.product.clone(), 1),
                        },
                        arguments: Box::new([]),
                        evidence: None,
                        result_domain: StaticValueDomain::ValueType(self.declarations.form(&self.ids.product_shape).unwrap_or(subject)),
                    }))
                } else {
                    Ok(StaticMemberProjectionOutcome::Invalid("subject does not conform to Product".into()))
                }
            }
        }
    }

    /// Evaluates an `OrderedProduct` member (`OrderedShape`, `Entries`, `PositionalEntries`, `LabeledEntries`).
    pub fn project_ordered_product_member(&mut self, subject: TypeId, member_name: &str) -> Result<StaticMemberProjectionOutcome, StructuralReflectionError> {
        let Some(structure) = self.classifier.ordered_product(self.types, subject)? else {
            if self.is_abstract(subject) {
                let req_index = match member_name {
                    "OrderedShape" => 0,
                    "Entries" => 1,
                    "PositionalEntries" => 2,
                    "LabeledEntries" => 3,
                    _ => 0,
                };
                return Ok(StaticMemberProjectionOutcome::Stuck(MemberProjection {
                    subject: MemberProjectionSubject::Type(subject),
                    member: MemberDeclarationIdentity::TraitAssociated {
                        trait_ref: crate::traits::TraitRef::new(self.ids.ordered_product.clone(), Vec::new()),
                        requirement: crate::traits::AssociatedTypeRequirementId::new(self.ids.ordered_product.clone(), req_index),
                    },
                    arguments: Box::new([]),
                    evidence: None,
                    result_domain: StaticValueDomain::ValueType(self.declarations.form(&self.ids.ordered_shape).unwrap_or(subject)),
                }));
            }
            return Ok(StaticMemberProjectionOutcome::Invalid("subject does not conform to OrderedProduct".into()));
        };

        if structure.state == DataProductShapeState::Symbolic || structure.symbolic_lanes.is_some() {
            let req_index = match member_name {
                "OrderedShape" => 0,
                "Entries" => 1,
                "PositionalEntries" => 2,
                "LabeledEntries" => 3,
                _ => 0,
            };
            return Ok(StaticMemberProjectionOutcome::Stuck(MemberProjection {
                subject: MemberProjectionSubject::Type(subject),
                member: MemberDeclarationIdentity::TraitAssociated {
                    trait_ref: crate::traits::TraitRef::new(self.ids.ordered_product.clone(), Vec::new()),
                    requirement: crate::traits::AssociatedTypeRequirementId::new(self.ids.ordered_product.clone(), req_index),
                },
                arguments: Box::new([]),
                evidence: None,
                result_domain: StaticValueDomain::ValueType(self.declarations.form(&self.ids.ordered_shape).unwrap_or(subject)),
            }));
        }

        match member_name {
            "OrderedShape" => {
                let val = self.project_ordered_shape_from_structure(&structure)?;
                Ok(StaticMemberProjectionOutcome::ReducedValue(val))
            }
            "Entries" => {
                let val = self.project_ordered_entries_from_structure(&structure)?;
                Ok(StaticMemberProjectionOutcome::ReducedValue(val))
            }
            "PositionalEntries" => {
                let val = self.project_ordered_positionals_from_structure(&structure)?;
                Ok(StaticMemberProjectionOutcome::ReducedValue(val))
            }
            "LabeledEntries" => {
                let val = self.project_ordered_labeled_from_structure(&structure)?;
                Ok(StaticMemberProjectionOutcome::ReducedValue(val))
            }
            other => Ok(StaticMemberProjectionOutcome::Invalid(
                format!("unknown OrderedProduct member `{other}`").into(),
            )),
        }
    }

    /// Evaluates an `UnorderedProduct` member (`UnorderedShape`).
    pub fn project_unordered_product_member(&mut self, subject: TypeId, member_name: &str) -> Result<StaticMemberProjectionOutcome, StructuralReflectionError> {
        let Some(structure) = self.classifier.unordered_product(self.types, subject)? else {
            if self.is_abstract(subject) {
                return Ok(StaticMemberProjectionOutcome::Stuck(MemberProjection {
                    subject: MemberProjectionSubject::Type(subject),
                    member: MemberDeclarationIdentity::TraitAssociated {
                        trait_ref: crate::traits::TraitRef::new(self.ids.unordered_product.clone(), Vec::new()),
                        requirement: crate::traits::AssociatedTypeRequirementId::new(self.ids.unordered_product.clone(), 0),
                    },
                    arguments: Box::new([]),
                    evidence: None,
                    result_domain: StaticValueDomain::ValueType(self.declarations.form(&self.ids.unordered_shape).unwrap_or(subject)),
                }));
            }
            return Ok(StaticMemberProjectionOutcome::Invalid("subject does not conform to UnorderedProduct".into()));
        };

        if structure.state == DataProductShapeState::Symbolic {
            return Ok(StaticMemberProjectionOutcome::Stuck(MemberProjection {
                subject: MemberProjectionSubject::Type(subject),
                member: MemberDeclarationIdentity::TraitAssociated {
                    trait_ref: crate::traits::TraitRef::new(self.ids.unordered_product.clone(), Vec::new()),
                    requirement: crate::traits::AssociatedTypeRequirementId::new(self.ids.unordered_product.clone(), 0),
                },
                arguments: Box::new([]),
                evidence: None,
                result_domain: StaticValueDomain::ValueType(self.declarations.form(&self.ids.unordered_shape).unwrap_or(subject)),
            }));
        }

        match member_name {
            "UnorderedShape" => {
                let val = self.project_unordered_shape_from_structure(&structure)?;
                Ok(StaticMemberProjectionOutcome::ReducedValue(val))
            }
            other => Ok(StaticMemberProjectionOutcome::Invalid(
                format!("unknown UnorderedProduct member `{other}`").into(),
            )),
        }
    }

    /// Evaluates `Sum::Variants` for `subject`.
    pub fn project_sum_variants(&mut self, subject: TypeId) -> Result<StaticMemberProjectionOutcome, StructuralReflectionError> {
        let Some(structure) = self.classifier.sum(self.types, subject)? else {
            if self.is_abstract(subject) {
                return Ok(StaticMemberProjectionOutcome::Stuck(MemberProjection {
                    subject: MemberProjectionSubject::Type(subject),
                    member: MemberDeclarationIdentity::TraitAssociated {
                        trait_ref: crate::traits::TraitRef::new(self.ids.sum.clone(), Vec::new()),
                        requirement: crate::traits::AssociatedTypeRequirementId::new(self.ids.sum.clone(), 0),
                    },
                    arguments: Box::new([]),
                    evidence: None,
                    result_domain: StaticValueDomain::ValueType(self.declarations.form(&self.ids.variant).unwrap_or(subject)),
                }));
            }
            return Ok(StaticMemberProjectionOutcome::Invalid("subject does not conform to Sum".into()));
        };

        let mut variant_values = Vec::with_capacity(structure.variants.len());
        for specialized in structure.variants.iter() {
            let name_str = match &specialized.declaration.variant.selector.base {
                phalcom_common::selector::SelectorBase::Named(n) => n.as_str(),
                phalcom_common::selector::SelectorBase::Subscript => "[]",
            };
            let name_sym = self.values.intern_symbol(name_str);
            let shape_val = self.project_variant_shape(specialized)?;
            let variant_val = self.construct_variant(name_sym, shape_val)?;
            variant_values.push(variant_val);
        }

        let pack_val = self.values.intern_pack(variant_values, Vec::new())?;
        Ok(StaticMemberProjectionOutcome::ReducedValue(pack_val))
    }

    /// Evaluates domain-native `UnorderedShape` type members (`Lookup`, `View`, `Tail`).
    pub fn project_unordered_shape_member(
        &mut self,
        shape_val: StaticValueRef,
        operation: IntrinsicTypeOperation,
        args: &[MemberProjectionArgument],
    ) -> Result<StaticMemberProjectionOutcome, StructuralReflectionError> {
        match operation {
            IntrinsicTypeOperation::UnorderedShapeLookup => {
                let Some(key_arg) = args.first() else {
                    return Ok(StaticMemberProjectionOutcome::Invalid("Lookup requires 1 argument".into()));
                };
                let key = match key_arg {
                    MemberProjectionArgument::Value(val) => match self.values.field_key_of(*val) {
                        Ok(k) => k,
                        Err(_) => {
                            return Ok(StaticMemberProjectionOutcome::Stuck(MemberProjection {
                                subject: MemberProjectionSubject::Value(shape_val),
                                member: MemberDeclarationIdentity::Intrinsic(operation),
                                arguments: args.to_vec().into_boxed_slice(),
                                evidence: None,
                                result_domain: StaticValueDomain::ValueType(self.declarations.form(&self.ids.field_lookup).unwrap_or(self.types.unit())),
                            }));
                        }
                    },
                    MemberProjectionArgument::Kinded(TypeMemberArgument::Const(ConstTerm::Int(i))) => FieldKey::int(i.clone()),
                    _ => {
                        return Ok(StaticMemberProjectionOutcome::Stuck(MemberProjection {
                            subject: MemberProjectionSubject::Value(shape_val),
                            member: MemberDeclarationIdentity::Intrinsic(operation),
                            arguments: args.to_vec().into_boxed_slice(),
                            evidence: None,
                            result_domain: StaticValueDomain::ValueType(self.declarations.form(&self.ids.field_lookup).unwrap_or(self.types.unit())),
                        }));
                    }
                };

                let StaticValueData::Data(data_id) = *self.values.get(shape_val)? else {
                    return Ok(StaticMemberProjectionOutcome::Invalid("expected UnorderedShape value".into()));
                };
                let data_node = self.values.get_data(data_id)?.clone();
                let fields_ref = data_node.components[0].1;
                let tail_ref = data_node.components[1].1;

                let StaticValueData::Record(rec_id) = *self.values.get(fields_ref)? else {
                    return Ok(StaticMemberProjectionOutcome::Invalid("expected Record fields".into()));
                };
                let fields = self.values.get_record(rec_id)?.to_vec();

                if let Some(matching) = fields.iter().find(|f| f.key == key) {
                    let StaticValueData::Data(fi_id) = *self.values.get(matching.value)? else {
                        return Ok(StaticMemberProjectionOutcome::Invalid("expected FieldInfo".into()));
                    };
                    let fi_node = self.values.get_data(fi_id)?.clone();
                    let val_ty_ref = fi_node.components[0].1;
                    let presence_ref = fi_node.components[1].1;

                    let StaticValueData::Variant(pres_var_id) = *self.values.get(presence_ref)? else {
                        return Ok(StaticMemberProjectionOutcome::Invalid("expected Presence variant".into()));
                    };
                    let pres_var_node = self.values.get_variant(pres_var_id)?.clone();
                    let is_required = selector_name(&pres_var_node.variant.selector) == Some("Required");

                    let outcome_val = if is_required {
                        self.construct_field_lookup_required(val_ty_ref)?
                    } else {
                        self.construct_field_lookup_optional(val_ty_ref)?
                    };
                    return Ok(StaticMemberProjectionOutcome::ReducedValue(outcome_val));
                }

                // Field not found in fixed record; check tail
                let StaticValueData::Variant(tail_var_id) = *self.values.get(tail_ref)? else {
                    return Ok(StaticMemberProjectionOutcome::Invalid("expected UnorderedTail variant".into()));
                };
                let tail_var = self.values.get_variant(tail_var_id)?.clone();
                match selector_name(&tail_var.variant.selector) {
                    Some("Closed") => {
                        let absent_val = self.construct_field_lookup_absent()?;
                        Ok(StaticMemberProjectionOutcome::ReducedValue(absent_val))
                    }
                    Some("Typed") => {
                        let key_ty_ref = tail_var.fields[0].1;
                        let val_ty_ref = tail_var.fields[1].1;
                        if self.key_matches_type(&key, key_ty_ref)? {
                            let opt_val = self.construct_field_lookup_optional(val_ty_ref)?;
                            Ok(StaticMemberProjectionOutcome::ReducedValue(opt_val))
                        } else {
                            let absent_val = self.construct_field_lookup_absent()?;
                            Ok(StaticMemberProjectionOutcome::ReducedValue(absent_val))
                        }
                    }
                    Some("Unconstrained") => {
                        let key_ty_ref = tail_var.fields[0].1;
                        if self.key_matches_type(&key, key_ty_ref)? {
                            let unk_val = self.construct_field_lookup_unknown()?;
                            Ok(StaticMemberProjectionOutcome::ReducedValue(unk_val))
                        } else {
                            let absent_val = self.construct_field_lookup_absent()?;
                            Ok(StaticMemberProjectionOutcome::ReducedValue(absent_val))
                        }
                    }
                    Some("Captured") => {
                        let rest_ref = tail_var.fields[0].1;
                        self.project_unordered_shape_member(rest_ref, operation, args)
                    }
                    _ => Ok(StaticMemberProjectionOutcome::Invalid("unknown UnorderedTail variant".into())),
                }
            }
            IntrinsicTypeOperation::UnorderedShapeView => {
                let StaticValueData::Data(data_id) = *self.values.get(shape_val)? else {
                    return Ok(StaticMemberProjectionOutcome::Invalid("expected UnorderedShape value".into()));
                };
                let data_node = self.values.get_data(data_id)?.clone();
                let fields_ref = data_node.components[0].1;
                let tail_ref = data_node.components[1].1;

                let StaticValueData::Record(rec_id) = *self.values.get(fields_ref)? else {
                    return Ok(StaticMemberProjectionOutcome::Invalid("expected Record fields".into()));
                };
                let fields = self.values.get_record(rec_id)?.to_vec();

                if fields.is_empty() {
                    let empty_view = self.construct_unordered_shape_view_empty(tail_ref)?;
                    Ok(StaticMemberProjectionOutcome::ReducedValue(empty_view))
                } else {
                    let first = &fields[0];
                    let StaticValueData::Data(fi_id) = *self.values.get(first.value)? else {
                        return Ok(StaticMemberProjectionOutcome::Invalid("expected FieldInfo".into()));
                    };
                    let fi_node = self.values.get_data(fi_id)?.clone();
                    let val_ty_ref = fi_node.components[0].1;
                    let presence_ref = fi_node.components[1].1;

                    let product_field = self.construct_product_field(&first.key, val_ty_ref, presence_ref)?;

                    let rest_fields = fields[1..].iter().map(|f| (f.key.clone(), f.value)).collect::<Vec<_>>();
                    let rest_record = self.values.intern_record(rest_fields)?;
                    let rest_shape = self.construct_unordered_shape(rest_record, tail_ref)?;

                    let field_view = self.construct_unordered_shape_view_field(product_field, rest_shape)?;
                    Ok(StaticMemberProjectionOutcome::ReducedValue(field_view))
                }
            }
            IntrinsicTypeOperation::UnorderedShapeTail => {
                let StaticValueData::Data(data_id) = *self.values.get(shape_val)? else {
                    return Ok(StaticMemberProjectionOutcome::Invalid("expected UnorderedShape value".into()));
                };
                let data_node = self.values.get_data(data_id)?;
                let tail_ref = data_node.components[1].1;
                Ok(StaticMemberProjectionOutcome::ReducedValue(tail_ref))
            }
            _ => Ok(StaticMemberProjectionOutcome::Invalid("unsupported operation on UnorderedShape".into())),
        }
    }

    /// Dispatches a member projection request to the appropriate structural reflection evaluator.
    pub fn project_member(
        &mut self,
        subject: &MemberProjectionSubject,
        member: &MemberDeclarationIdentity,
        arguments: &[MemberProjectionArgument],
        _evidence: Option<&GenericConformanceEvidence>,
    ) -> Result<StaticMemberProjectionOutcome, StructuralReflectionError> {
        match member {
            MemberDeclarationIdentity::TraitAssociated { trait_ref, requirement } => {
                let MemberProjectionSubject::Type(ty) = subject else {
                    return Ok(StaticMemberProjectionOutcome::Invalid("structural trait member requires Type subject".into()));
                };
                if trait_ref.declaration == self.ids.product {
                    if requirement.index == 0 {
                        self.project_product_kind(*ty)
                    } else if requirement.index == 1 {
                        self.project_product_shape(*ty)
                    } else {
                        Ok(StaticMemberProjectionOutcome::Invalid("invalid Product requirement index".into()))
                    }
                } else if trait_ref.declaration == self.ids.ordered_product {
                    let member_name = match requirement.index {
                        0 => "OrderedShape",
                        1 => "Entries",
                        2 => "PositionalEntries",
                        3 => "LabeledEntries",
                        _ => return Ok(StaticMemberProjectionOutcome::Invalid("invalid OrderedProduct requirement index".into())),
                    };
                    self.project_ordered_product_member(*ty, member_name)
                } else if trait_ref.declaration == self.ids.unordered_product {
                    if requirement.index == 0 {
                        self.project_unordered_product_member(*ty, "UnorderedShape")
                    } else {
                        Ok(StaticMemberProjectionOutcome::Invalid("invalid UnorderedProduct requirement index".into()))
                    }
                } else if trait_ref.declaration == self.ids.sum {
                    if requirement.index == 0 {
                        self.project_sum_variants(*ty)
                    } else {
                        Ok(StaticMemberProjectionOutcome::Invalid("invalid Sum requirement index".into()))
                    }
                } else {
                    Ok(StaticMemberProjectionOutcome::Invalid("not a canonical structural trait".into()))
                }
            }
            MemberDeclarationIdentity::Intrinsic(operation) => {
                let MemberProjectionSubject::Value(val_ref) = subject else {
                    return Ok(StaticMemberProjectionOutcome::Invalid(
                        "intrinsic operation requires StaticValue subject".into(),
                    ));
                };
                self.project_unordered_shape_member(*val_ref, *operation, arguments)
            }
            MemberDeclarationIdentity::TypeMember(decl_id) => {
                if decl_id.owner == self.ids.unordered_shape {
                    let MemberProjectionSubject::Value(val_ref) = subject else {
                        return Ok(StaticMemberProjectionOutcome::Invalid(
                            "UnorderedShape member requires StaticValue subject".into(),
                        ));
                    };
                    let op = match decl_id.selector.name().as_ref() {
                        "Lookup" => IntrinsicTypeOperation::UnorderedShapeLookup,
                        "View" => IntrinsicTypeOperation::UnorderedShapeView,
                        "Tail" => IntrinsicTypeOperation::UnorderedShapeTail,
                        other => {
                            return Ok(StaticMemberProjectionOutcome::Invalid(
                                format!("unknown UnorderedShape member `{other}`").into(),
                            ));
                        }
                    };
                    self.project_unordered_shape_member(*val_ref, op, arguments)
                } else {
                    Ok(StaticMemberProjectionOutcome::Invalid("unsupported type member owner".into()))
                }
            }
        }
    }

    // --- Internal construction helpers ---

    fn is_abstract(&self, ty: TypeId) -> bool {
        matches!(
            self.types.get(ty),
            TypeData::Parameter(_)
                | TypeData::TypeMemberProjection(_)
                | TypeData::AssociatedProjection { .. }
                | TypeData::TupleFromPacks { .. }
                | TypeData::PackFromPacks { .. }
        )
    }

    fn key_matches_type(&mut self, key: &FieldKey, key_ty_ref: StaticValueRef) -> Result<bool, StructuralReflectionError> {
        let StaticValueData::Type(key_ty) = *self.values.get(key_ty_ref)? else {
            return Ok(false);
        };
        let field_key_decl = DeclarationId::new(crate::core_surface::identity::universe_module(&["collections", "record"]), "FieldKey".into());
        let int_decl = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Int);
        let str_decl = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::String);
        let sym_decl = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Symbol);
        let bool_decl = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Bool);

        let origin = self.types.nominal_origin_declaration(key_ty);
        if origin == Some(&field_key_decl) {
            return Ok(true);
        }
        match key {
            FieldKey::Int(_) => Ok(origin == Some(&int_decl)),
            FieldKey::String(_) => Ok(origin == Some(&str_decl)),
            FieldKey::Symbol(_) => Ok(origin == Some(&sym_decl)),
            FieldKey::Bool(_) => Ok(origin == Some(&bool_decl)),
        }
    }

    fn project_ordered_shape_from_structure(&mut self, structure: &OrderedProductStructure) -> Result<StaticValueRef, StructuralReflectionError> {
        let entries_val = self.project_ordered_entries_from_structure(structure)?;
        let pos_tail = self.construct_ordered_tail(structure.positional_remainder, true)?;
        let lab_tail = self.construct_ordered_tail(structure.labeled_remainder, false)?;
        self.construct_ordered_shape(entries_val, pos_tail, lab_tail)
    }

    fn project_ordered_entries_from_structure(&mut self, structure: &OrderedProductStructure) -> Result<StaticValueRef, StructuralReflectionError> {
        let mut positionals = Vec::with_capacity(structure.positional_entries.len());
        for entry in structure.positional_entries.iter() {
            let ty = entry
                .ty
                .ok_or_else(|| StructuralReflectionError::TypeFormation("entry type is absent".into()))?;
            let ty_val = self.values.intern_type(ty, self.types)?;
            positionals.push(ty_val);
        }

        let mut labeled = Vec::with_capacity(structure.labeled_entries.len());
        for entry in structure.labeled_entries.iter() {
            let ty = entry
                .ty
                .ok_or_else(|| StructuralReflectionError::TypeFormation("entry type is absent".into()))?;
            let ty_val = self.values.intern_type(ty, self.types)?;
            let label_sym = self.values.intern_symbol_id(entry.label.as_deref().unwrap_or(""));
            labeled.push((label_sym, ty_val));
        }

        self.values.intern_pack(positionals, labeled).map_err(StructuralReflectionError::from)
    }

    fn project_ordered_positionals_from_structure(&mut self, structure: &OrderedProductStructure) -> Result<StaticValueRef, StructuralReflectionError> {
        let mut positionals = Vec::with_capacity(structure.positional_entries.len());
        for entry in structure.positional_entries.iter() {
            let ty = entry
                .ty
                .ok_or_else(|| StructuralReflectionError::TypeFormation("entry type is absent".into()))?;
            let ty_val = self.values.intern_type(ty, self.types)?;
            positionals.push(ty_val);
        }
        self.values.intern_pack(positionals, Vec::new()).map_err(StructuralReflectionError::from)
    }

    fn project_ordered_labeled_from_structure(&mut self, structure: &OrderedProductStructure) -> Result<StaticValueRef, StructuralReflectionError> {
        let mut labeled = Vec::with_capacity(structure.labeled_entries.len());
        for entry in structure.labeled_entries.iter() {
            let ty = entry
                .ty
                .ok_or_else(|| StructuralReflectionError::TypeFormation("entry type is absent".into()))?;
            let ty_val = self.values.intern_type(ty, self.types)?;
            let label_sym = self.values.intern_symbol_id(entry.label.as_deref().unwrap_or(""));
            labeled.push((label_sym, ty_val));
        }
        self.values.intern_pack(Vec::new(), labeled).map_err(StructuralReflectionError::from)
    }

    fn project_unordered_shape_from_structure(&mut self, structure: &UnorderedProductStructure) -> Result<StaticValueRef, StructuralReflectionError> {
        let mut record_fields = Vec::with_capacity(structure.fields.len());
        for entry in structure.fields.iter() {
            let ty = entry
                .ty
                .ok_or_else(|| StructuralReflectionError::TypeFormation("field type is absent".into()))?;
            let ty_val = self.values.intern_type(ty, self.types)?;
            let presence_val = self.construct_presence(entry.presence)?;
            let fi_val = self.construct_field_info(ty_val, presence_val)?;
            record_fields.push((entry.key.clone(), fi_val));
        }
        let fields_record = self.values.intern_record(record_fields)?;
        let tail_val = self.construct_unordered_tail(&structure.tail)?;
        self.construct_unordered_shape(fields_record, tail_val)
    }

    fn project_variant_shape(&mut self, specialized: &crate::structural::SpecializedVariantStructure) -> Result<StaticValueRef, StructuralReflectionError> {
        match specialized.payload_kind {
            AlgebraicCasePayloadKind::Singleton | AlgebraicCasePayloadKind::ExplicitEmpty => {
                let empty_pack = self.values.intern_pack(Vec::new(), Vec::new())?;
                let closed_pos = self.construct_ordered_tail(AnonymousTupleRemainder::None, true)?;
                let closed_lab = self.construct_ordered_tail(AnonymousTupleRemainder::None, false)?;
                let ordered_shape = self.construct_ordered_shape(empty_pack, closed_pos, closed_lab)?;
                self.wrap_product_shape_ordered(ordered_shape)
            }
            AlgebraicCasePayloadKind::Tuple => {
                let mut positionals = Vec::with_capacity(specialized.exact_case.fields.len());
                for f in specialized.exact_case.fields.iter() {
                    let ty = f.specialized_type.unwrap_or_else(|| self.types.unit());
                    let ty_val = self.values.intern_type(ty, self.types)?;
                    positionals.push(ty_val);
                }
                let entries_pack = self.values.intern_pack(positionals, Vec::new())?;
                let closed_pos = self.construct_ordered_tail(AnonymousTupleRemainder::None, true)?;
                let closed_lab = self.construct_ordered_tail(AnonymousTupleRemainder::None, false)?;
                let ordered_shape = self.construct_ordered_shape(entries_pack, closed_pos, closed_lab)?;
                self.wrap_product_shape_ordered(ordered_shape)
            }
            AlgebraicCasePayloadKind::Record => {
                let mut record_fields = Vec::with_capacity(specialized.exact_case.fields.len());
                for f in specialized.exact_case.fields.iter() {
                    let label = f.external_label.as_deref().unwrap_or(f.local_name.as_ref());
                    let key = FieldKey::symbol(label);
                    let ty = f.specialized_type.unwrap_or_else(|| self.types.unit());
                    let ty_val = self.values.intern_type(ty, self.types)?;
                    let presence = self.construct_presence(StructuralMemberPresence::Required)?;
                    let fi = self.construct_field_info(ty_val, presence)?;
                    record_fields.push((key, fi));
                }
                let fields_rec = self.values.intern_record(record_fields)?;
                let tail = self.construct_unordered_tail(&RecordRowTail::Closed)?;
                let unordered_shape = self.construct_unordered_shape(fields_rec, tail)?;
                self.wrap_product_shape_unordered(unordered_shape)
            }
        }
    }

    fn construct_product_kind_ordered(&mut self) -> Result<StaticValueRef, StructuralReflectionError> {
        let enum_info = self
            .enums
            .enum_info(&self.ids.product_kind)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.product_kind.clone()))?;
        let variant_id = enum_info
            .variants
            .iter()
            .find(|v| selector_name(&v.selector) == Some("Ordered"))
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.product_kind.clone()))?;
        let variant_info = self
            .enums
            .variant_info(variant_id)
            .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
        let enum_ty = self.declarations.form(&self.ids.product_kind).unwrap();
        let exact_case_ty = self
            .types
            .exact_case_type(variant_id, enum_ty)
            .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
        self.values
            .intern_variant(variant_info, exact_case_ty, Vec::new(), self.types)
            .map_err(Into::into)
    }

    fn construct_product_kind_unordered(&mut self) -> Result<StaticValueRef, StructuralReflectionError> {
        let enum_info = self
            .enums
            .enum_info(&self.ids.product_kind)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.product_kind.clone()))?;
        let variant_id = enum_info
            .variants
            .iter()
            .find(|v| selector_name(&v.selector) == Some("Unordered"))
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.product_kind.clone()))?;
        let variant_info = self
            .enums
            .variant_info(variant_id)
            .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
        let enum_ty = self.declarations.form(&self.ids.product_kind).unwrap();
        let exact_case_ty = self
            .types
            .exact_case_type(variant_id, enum_ty)
            .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
        self.values
            .intern_variant(variant_info, exact_case_ty, Vec::new(), self.types)
            .map_err(Into::into)
    }

    fn wrap_product_shape_ordered(&mut self, ordered_shape_ref: StaticValueRef) -> Result<StaticValueRef, StructuralReflectionError> {
        let enum_info = self
            .enums
            .enum_info(&self.ids.product_shape)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.product_shape.clone()))?;
        let variant_id = enum_info
            .variants
            .iter()
            .find(|v| selector_name(&v.selector) == Some("Ordered"))
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.product_shape.clone()))?;
        let variant_info = self
            .enums
            .variant_info(variant_id)
            .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
        let enum_ty = self.declarations.form(&self.ids.product_shape).unwrap();
        let exact_case_ty = self
            .types
            .exact_case_type(variant_id, enum_ty)
            .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
        let fields = vec![(variant_info.fields[0].id.clone(), ordered_shape_ref)];
        self.values.intern_variant(variant_info, exact_case_ty, fields, self.types).map_err(Into::into)
    }

    fn wrap_product_shape_unordered(&mut self, unordered_shape_ref: StaticValueRef) -> Result<StaticValueRef, StructuralReflectionError> {
        let enum_info = self
            .enums
            .enum_info(&self.ids.product_shape)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.product_shape.clone()))?;
        let variant_id = enum_info
            .variants
            .iter()
            .find(|v| selector_name(&v.selector) == Some("Unordered"))
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.product_shape.clone()))?;
        let variant_info = self
            .enums
            .variant_info(variant_id)
            .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
        let enum_ty = self.declarations.form(&self.ids.product_shape).unwrap();
        let exact_case_ty = self
            .types
            .exact_case_type(variant_id, enum_ty)
            .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
        let fields = vec![(variant_info.fields[0].id.clone(), unordered_shape_ref)];
        self.values.intern_variant(variant_info, exact_case_ty, fields, self.types).map_err(Into::into)
    }

    fn construct_ordered_shape(
        &mut self,
        entries_ref: StaticValueRef,
        pos_tail_ref: StaticValueRef,
        lab_tail_ref: StaticValueRef,
    ) -> Result<StaticValueRef, StructuralReflectionError> {
        let data_info = self
            .data
            .data_info(&self.ids.ordered_shape)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.ordered_shape.clone()))?;
        let ordered_shape_ty = self.declarations.form(&self.ids.ordered_shape).unwrap();
        let components = vec![
            (data_info.components[0].id.clone(), entries_ref),
            (data_info.components[1].id.clone(), pos_tail_ref),
            (data_info.components[2].id.clone(), lab_tail_ref),
        ];
        self.values.intern_data(data_info, ordered_shape_ty, components, self.types).map_err(Into::into)
    }

    fn construct_ordered_tail(&mut self, remainder: AnonymousTupleRemainder, _is_positional: bool) -> Result<StaticValueRef, StructuralReflectionError> {
        let enum_info = self
            .enums
            .enum_info(&self.ids.ordered_tail)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.ordered_tail.clone()))?;
        let tail_form = self.declarations.form(&self.ids.ordered_tail).unwrap();
        let unit_ty = self.types.unit();
        let applied_ty = self.types.apply_type_form(tail_form, &[unit_ty]).unwrap_or(tail_form);

        match remainder {
            AnonymousTupleRemainder::None => {
                let variant_id = enum_info
                    .variants
                    .iter()
                    .find(|v| selector_name(&v.selector) == Some("Closed"))
                    .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.ordered_tail.clone()))?;
                let variant_info = self
                    .enums
                    .variant_info(variant_id)
                    .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
                let exact_case_ty = self
                    .types
                    .exact_case_type(variant_id, applied_ty)
                    .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
                self.values
                    .intern_variant(variant_info, exact_case_ty, Vec::new(), self.types)
                    .map_err(Into::into)
            }
            AnonymousTupleRemainder::Typed(elem_ty) => {
                let variant_id = enum_info
                    .variants
                    .iter()
                    .find(|v| selector_name(&v.selector) == Some("Typed"))
                    .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.ordered_tail.clone()))?;
                let variant_info = self
                    .enums
                    .variant_info(variant_id)
                    .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
                let exact_case_ty = self
                    .types
                    .exact_case_type(variant_id, applied_ty)
                    .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
                let elem_val = self.values.intern_type(elem_ty, self.types)?;
                let fields = vec![(variant_info.fields[0].id.clone(), elem_val)];
                self.values.intern_variant(variant_info, exact_case_ty, fields, self.types).map_err(Into::into)
            }
            AnonymousTupleRemainder::Unconstrained => {
                let variant_id = enum_info
                    .variants
                    .iter()
                    .find(|v| selector_name(&v.selector) == Some("Unconstrained"))
                    .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.ordered_tail.clone()))?;
                let variant_info = self
                    .enums
                    .variant_info(variant_id)
                    .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
                let exact_case_ty = self
                    .types
                    .exact_case_type(variant_id, applied_ty)
                    .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
                self.values
                    .intern_variant(variant_info, exact_case_ty, Vec::new(), self.types)
                    .map_err(Into::into)
            }
        }
    }

    fn construct_presence(&mut self, presence: StructuralMemberPresence) -> Result<StaticValueRef, StructuralReflectionError> {
        let enum_info = self
            .enums
            .enum_info(&self.ids.presence)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.presence.clone()))?;
        let var_name = match presence {
            StructuralMemberPresence::Required => "Required",
            StructuralMemberPresence::Optional => "Optional",
        };
        let variant_id = enum_info
            .variants
            .iter()
            .find(|v| selector_name(&v.selector) == Some(var_name))
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.presence.clone()))?;
        let variant_info = self
            .enums
            .variant_info(variant_id)
            .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
        let enum_ty = self.declarations.form(&self.ids.presence).unwrap();
        let exact_case_ty = self
            .types
            .exact_case_type(variant_id, enum_ty)
            .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
        self.values
            .intern_variant(variant_info, exact_case_ty, Vec::new(), self.types)
            .map_err(Into::into)
    }

    fn construct_field_info(&mut self, val_ty_ref: StaticValueRef, presence_ref: StaticValueRef) -> Result<StaticValueRef, StructuralReflectionError> {
        let data_info = self
            .data
            .data_info(&self.ids.field_info)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.field_info.clone()))?;
        let fi_ty = self.declarations.form(&self.ids.field_info).unwrap();
        let components = vec![
            (data_info.components[0].id.clone(), val_ty_ref),
            (data_info.components[1].id.clone(), presence_ref),
        ];
        self.values.intern_data(data_info, fi_ty, components, self.types).map_err(Into::into)
    }

    fn construct_product_field(
        &mut self,
        key: &FieldKey,
        val_ty_ref: StaticValueRef,
        presence_ref: StaticValueRef,
    ) -> Result<StaticValueRef, StructuralReflectionError> {
        let data_info = self
            .data
            .data_info(&self.ids.product_field)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.product_field.clone()))?;
        let pf_ty = self.declarations.form(&self.ids.product_field).unwrap();
        let key_val = self.field_key_to_static_value(key)?;
        let components = vec![
            (data_info.components[0].id.clone(), key_val),
            (data_info.components[1].id.clone(), val_ty_ref),
            (data_info.components[2].id.clone(), presence_ref),
        ];
        self.values.intern_data(data_info, pf_ty, components, self.types).map_err(Into::into)
    }

    fn construct_unordered_tail(&mut self, tail: &RecordRowTail) -> Result<StaticValueRef, StructuralReflectionError> {
        let enum_info = self
            .enums
            .enum_info(&self.ids.unordered_tail)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.unordered_tail.clone()))?;
        let enum_ty = self.declarations.form(&self.ids.unordered_tail).unwrap();
        match tail {
            RecordRowTail::Closed => {
                let variant_id = enum_info
                    .variants
                    .iter()
                    .find(|v| selector_name(&v.selector) == Some("Closed"))
                    .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.unordered_tail.clone()))?;
                let variant_info = self
                    .enums
                    .variant_info(variant_id)
                    .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
                let exact_case_ty = self
                    .types
                    .exact_case_type(variant_id, enum_ty)
                    .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
                self.values
                    .intern_variant(variant_info, exact_case_ty, Vec::new(), self.types)
                    .map_err(Into::into)
            }
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed { key_type, value_type }) => {
                let variant_id = enum_info
                    .variants
                    .iter()
                    .find(|v| selector_name(&v.selector) == Some("Typed"))
                    .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.unordered_tail.clone()))?;
                let variant_info = self
                    .enums
                    .variant_info(variant_id)
                    .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
                let exact_case_ty = self
                    .types
                    .exact_case_type(variant_id, enum_ty)
                    .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
                let key_ty_val = self.values.intern_type(*key_type, self.types)?;
                let val_ty_val = self.values.intern_type(*value_type, self.types)?;
                let fields = vec![(variant_info.fields[0].id.clone(), key_ty_val), (variant_info.fields[1].id.clone(), val_ty_val)];
                self.values.intern_variant(variant_info, exact_case_ty, fields, self.types).map_err(Into::into)
            }
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type }) => {
                let variant_id = enum_info
                    .variants
                    .iter()
                    .find(|v| selector_name(&v.selector) == Some("Unconstrained"))
                    .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.unordered_tail.clone()))?;
                let variant_info = self
                    .enums
                    .variant_info(variant_id)
                    .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
                let exact_case_ty = self
                    .types
                    .exact_case_type(variant_id, enum_ty)
                    .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
                let key_ty_val = self.values.intern_type(*key_type, self.types)?;
                let fields = vec![(variant_info.fields[0].id.clone(), key_ty_val)];
                self.values.intern_variant(variant_info, exact_case_ty, fields, self.types).map_err(Into::into)
            }
            RecordRowTail::Parameter(_) | RecordRowTail::Projection(_) => {
                let variant_id = enum_info
                    .variants
                    .iter()
                    .find(|v| selector_name(&v.selector) == Some("Unconstrained"))
                    .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.unordered_tail.clone()))?;
                let variant_info = self
                    .enums
                    .variant_info(variant_id)
                    .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
                let exact_case_ty = self
                    .types
                    .exact_case_type(variant_id, enum_ty)
                    .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
                let field_key_decl = DeclarationId::new(crate::core_surface::identity::universe_module(&["collections", "record"]), "FieldKey".into());
                let field_key_ty = self
                    .declarations
                    .form(&field_key_decl)
                    .unwrap_or_else(|| self.types.nominal_form(field_key_decl, KindId::TYPE));
                let key_ty_val = self.values.intern_type(field_key_ty, self.types)?;
                let fields = vec![(variant_info.fields[0].id.clone(), key_ty_val)];
                self.values.intern_variant(variant_info, exact_case_ty, fields, self.types).map_err(Into::into)
            }
        }
    }

    fn construct_unordered_shape(&mut self, fields_ref: StaticValueRef, tail_ref: StaticValueRef) -> Result<StaticValueRef, StructuralReflectionError> {
        let data_info = self
            .data
            .data_info(&self.ids.unordered_shape)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.unordered_shape.clone()))?;
        let us_ty = self.declarations.form(&self.ids.unordered_shape).unwrap();
        let components = vec![(data_info.components[0].id.clone(), fields_ref), (data_info.components[1].id.clone(), tail_ref)];
        self.values.intern_data(data_info, us_ty, components, self.types).map_err(Into::into)
    }

    fn construct_field_lookup_required(&mut self, val_ty_ref: StaticValueRef) -> Result<StaticValueRef, StructuralReflectionError> {
        let enum_info = self
            .enums
            .enum_info(&self.ids.field_lookup)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.field_lookup.clone()))?;
        let variant_id = enum_info
            .variants
            .iter()
            .find(|v| selector_name(&v.selector) == Some("Required"))
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.field_lookup.clone()))?;
        let variant_info = self
            .enums
            .variant_info(variant_id)
            .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
        let enum_ty = self.declarations.form(&self.ids.field_lookup).unwrap();
        let exact_case_ty = self
            .types
            .exact_case_type(variant_id, enum_ty)
            .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
        let fields = vec![(variant_info.fields[0].id.clone(), val_ty_ref)];
        self.values.intern_variant(variant_info, exact_case_ty, fields, self.types).map_err(Into::into)
    }

    fn construct_field_lookup_optional(&mut self, val_ty_ref: StaticValueRef) -> Result<StaticValueRef, StructuralReflectionError> {
        let enum_info = self
            .enums
            .enum_info(&self.ids.field_lookup)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.field_lookup.clone()))?;
        let variant_id = enum_info
            .variants
            .iter()
            .find(|v| selector_name(&v.selector) == Some("Optional"))
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.field_lookup.clone()))?;
        let variant_info = self
            .enums
            .variant_info(variant_id)
            .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
        let enum_ty = self.declarations.form(&self.ids.field_lookup).unwrap();
        let exact_case_ty = self
            .types
            .exact_case_type(variant_id, enum_ty)
            .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
        let fields = vec![(variant_info.fields[0].id.clone(), val_ty_ref)];
        self.values.intern_variant(variant_info, exact_case_ty, fields, self.types).map_err(Into::into)
    }

    fn construct_field_lookup_absent(&mut self) -> Result<StaticValueRef, StructuralReflectionError> {
        let enum_info = self
            .enums
            .enum_info(&self.ids.field_lookup)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.field_lookup.clone()))?;
        let variant_id = enum_info
            .variants
            .iter()
            .find(|v| selector_name(&v.selector) == Some("Absent"))
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.field_lookup.clone()))?;
        let variant_info = self
            .enums
            .variant_info(variant_id)
            .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
        let enum_ty = self.declarations.form(&self.ids.field_lookup).unwrap();
        let exact_case_ty = self
            .types
            .exact_case_type(variant_id, enum_ty)
            .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
        self.values
            .intern_variant(variant_info, exact_case_ty, Vec::new(), self.types)
            .map_err(Into::into)
    }

    fn construct_field_lookup_unknown(&mut self) -> Result<StaticValueRef, StructuralReflectionError> {
        let enum_info = self
            .enums
            .enum_info(&self.ids.field_lookup)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.field_lookup.clone()))?;
        let variant_id = enum_info
            .variants
            .iter()
            .find(|v| selector_name(&v.selector) == Some("Unknown"))
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.field_lookup.clone()))?;
        let variant_info = self
            .enums
            .variant_info(variant_id)
            .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
        let enum_ty = self.declarations.form(&self.ids.field_lookup).unwrap();
        let exact_case_ty = self
            .types
            .exact_case_type(variant_id, enum_ty)
            .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
        self.values
            .intern_variant(variant_info, exact_case_ty, Vec::new(), self.types)
            .map_err(Into::into)
    }

    fn construct_unordered_shape_view_empty(&mut self, tail_ref: StaticValueRef) -> Result<StaticValueRef, StructuralReflectionError> {
        let enum_info = self
            .enums
            .enum_info(&self.ids.unordered_shape_view)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.unordered_shape_view.clone()))?;
        let variant_id = enum_info
            .variants
            .iter()
            .find(|v| selector_name(&v.selector) == Some("Empty"))
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.unordered_shape_view.clone()))?;
        let variant_info = self
            .enums
            .variant_info(variant_id)
            .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
        let enum_ty = self.declarations.form(&self.ids.unordered_shape_view).unwrap();
        let exact_case_ty = self
            .types
            .exact_case_type(variant_id, enum_ty)
            .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
        let fields = vec![(variant_info.fields[0].id.clone(), tail_ref)];
        self.values.intern_variant(variant_info, exact_case_ty, fields, self.types).map_err(Into::into)
    }

    fn construct_unordered_shape_view_field(
        &mut self,
        field_ref: StaticValueRef,
        rest_shape_ref: StaticValueRef,
    ) -> Result<StaticValueRef, StructuralReflectionError> {
        let enum_info = self
            .enums
            .enum_info(&self.ids.unordered_shape_view)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.unordered_shape_view.clone()))?;
        let variant_id = enum_info
            .variants
            .iter()
            .find(|v| selector_name(&v.selector) == Some("Field"))
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.unordered_shape_view.clone()))?;
        let variant_info = self
            .enums
            .variant_info(variant_id)
            .ok_or_else(|| StructuralReflectionError::MissingVariant(variant_id.clone()))?;
        let enum_ty = self.declarations.form(&self.ids.unordered_shape_view).unwrap();
        let exact_case_ty = self
            .types
            .exact_case_type(variant_id, enum_ty)
            .map_err(|_| StructuralReflectionError::TypeFormation("exact case type error".into()))?;
        let fields = vec![
            (variant_info.fields[0].id.clone(), field_ref),
            (variant_info.fields[1].id.clone(), rest_shape_ref),
        ];
        self.values.intern_variant(variant_info, exact_case_ty, fields, self.types).map_err(Into::into)
    }

    fn construct_variant(&mut self, name_sym_ref: StaticValueRef, shape_ref: StaticValueRef) -> Result<StaticValueRef, StructuralReflectionError> {
        let data_info = self
            .data
            .data_info(&self.ids.variant)
            .ok_or_else(|| StructuralReflectionError::MissingDeclaration(self.ids.variant.clone()))?;
        let v_ty = self.declarations.form(&self.ids.variant).unwrap();
        let components = vec![
            (data_info.components[0].id.clone(), name_sym_ref),
            (data_info.components[1].id.clone(), shape_ref),
        ];
        self.values.intern_data(data_info, v_ty, components, self.types).map_err(Into::into)
    }

    fn field_key_to_static_value(&mut self, key: &FieldKey) -> Result<StaticValueRef, StructuralReflectionError> {
        match key {
            FieldKey::String(s) => Ok(self.values.intern_string(s)),
            FieldKey::Symbol(s) => Ok(self.values.intern_symbol(s)),
            FieldKey::Int(i) => Ok(self.values.intern_int(i.to_bigint())),
            FieldKey::Bool(b) => Ok(self.values.intern_bool(*b)),
        }
    }
}
