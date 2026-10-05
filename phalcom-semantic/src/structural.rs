//! Canonical classification of types that have structural product or sum forms.
//!
//! This layer is intentionally below public structural capabilities and
//! reflection. It recognizes exact semantic type forms and declaration
//! products; it does not inspect runtime objects, source names, or conformance
//! evidence.

use crate::data_semantics::{DataProductShapeMaterializationError, DataProductShapeState, DataSemanticTable, DataShape, materialize_data_product_shape};
use crate::enum_semantics::{AlgebraicCasePayloadKind, EnumSemanticTable};
use crate::identity::{DataComponentId, DeclarationId, VariantId};
use crate::reflection::{ExactCaseTypeReflection, VariantReflection};
use crate::types::id::{KindId, TypeId};
use crate::types::store::{AnonymousTupleRemainder, TypeData, TypeStore};
use thiserror::Error;

/// The canonical ordered form represented by an exact semantic type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrderedProductForm {
    /// The canonical unit tuple.
    Unit,
    /// A finalized positional/labeled tuple type.
    Tuple,
    /// A tuple assembled from symbolic ordered type-pack lanes.
    TupleFromPacks,
    /// A finalized first-class Pack type.
    Pack,
    /// A Pack assembled from symbolic ordered type-pack lanes.
    PackFromPacks,
    /// A nominal tuple-shaped data declaration, preserving its owner identity.
    Data { owner: DeclarationId },
}

/// Shared, lower-level structural classification for a canonical type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StructuralClassification {
    /// The type has no structural product or sum form supported by this layer.
    Unsupported,
    /// The type has an ordered product form.
    OrderedProduct(OrderedProductForm),
    /// The type is a declared sum, identified by its canonical owner.
    Sum { owner: DeclarationId },
}

/// The canonical source category for an ordered structural type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrderedProductOrigin {
    /// Unit, which is the empty ordered tuple form.
    Unit,
    /// A tuple type.
    Tuple,
    /// A tuple type whose lanes are supplied through kinded pack terms.
    TupleFromPacks,
    /// A first-class Pack type.
    Pack,
    /// A Pack type whose lanes are supplied through kinded pack terms.
    PackFromPacks,
    /// A nominal tuple-shaped data declaration.
    Data { owner: DeclarationId },
}

/// One shallow ordered entry. Its type is a canonical reference and is not recursively reflected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderedProductEntry {
    /// The declared component identity for nominal data, when one exists.
    pub component: Option<DataComponentId>,
    /// The Symbol label for a labeled lane entry; `None` denotes positional lane membership.
    pub label: Option<Box<str>>,
    /// The canonical immediate component type, when known.
    pub ty: Option<TypeId>,
}

/// Shallow canonical facts for one ordered structural type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderedProductStructure {
    /// The exact canonical type that was classified.
    pub ty: TypeId,
    /// The source category, retaining nominal identity where applicable.
    pub origin: OrderedProductOrigin,
    /// Immediate positional entries in lane order.
    pub positional_entries: Box<[OrderedProductEntry]>,
    /// Immediate Symbol-labeled entries in lane order.
    pub labeled_entries: Box<[OrderedProductEntry]>,
    /// The canonical positional remainder state, when lanes are directly represented.
    pub positional_remainder: AnonymousTupleRemainder,
    /// The canonical labeled remainder state, when lanes are directly represented.
    pub labeled_remainder: AnonymousTupleRemainder,
    /// Symbolic lane terms retained by `TupleFromPacks` / `PackFromPacks`.
    pub symbolic_lanes: Option<(TypeId, TypeId)>,
    /// Whether nominal data shape materialization encountered unresolved entries or expansions.
    pub state: DataProductShapeState,
}

/// One declaration-ordered Variant specialized against an exact Sum type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpecializedVariantStructure {
    /// Existing protocol-neutral declaration reflection.
    pub declaration: VariantReflection,
    /// Immediate payload category, without publishing P4-dependent Record shape.
    pub payload_kind: AlgebraicCasePayloadKind,
    /// Exact-case relationship and immediately specialized field type references.
    pub exact_case: ExactCaseTypeReflection,
}

/// Shallow structural facts for one exact Sum type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SumStructure {
    /// The exact canonical sum type that was classified.
    pub ty: TypeId,
    /// Canonical declaration identity for the Sum.
    pub owner: DeclarationId,
    /// Variants in the declaration order published by `EnumInfo`.
    pub variants: Box<[SpecializedVariantStructure]>,
}

/// Failure to form a shallow Sum descriptor from inconsistent canonical products.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum SumStructureError {
    /// `EnumInfo` referenced a Variant that was absent from the same semantic table.
    #[error("sum declaration references missing variant {0:?}")]
    MissingVariant(VariantId),
    /// The exact case could not be formed for the owning Sum type.
    #[error("exact case type could not be formed for the owning sum")]
    ExactCaseTypeFormation,
}

/// Classifies exact type identities using canonical semantic declaration tables.
///
/// This facade does not create a query product or retain state. Callers that
/// need a durable or incremental product must own that dependency explicitly.
pub struct StructuralClassifier<'a> {
    data: &'a DataSemanticTable,
    enums: &'a EnumSemanticTable,
}

impl<'a> StructuralClassifier<'a> {
    /// Creates a classifier over one coherent set of canonical semantic owners.
    pub fn new(data: &'a DataSemanticTable, enums: &'a EnumSemanticTable) -> Self {
        Self { data, enums }
    }

    /// Classifies `ty` without resolving source names or consulting capability evidence.
    pub fn classify(&self, types: &TypeStore, ty: TypeId) -> StructuralClassification {
        if types.kind_of(ty) != KindId::TYPE {
            return StructuralClassification::Unsupported;
        }
        match types.get(ty) {
            TypeData::Unit => StructuralClassification::OrderedProduct(OrderedProductForm::Unit),
            TypeData::Tuple(_) => StructuralClassification::OrderedProduct(OrderedProductForm::Tuple),
            TypeData::TupleFromPacks { .. } => StructuralClassification::OrderedProduct(OrderedProductForm::TupleFromPacks),
            TypeData::Pack(_) => StructuralClassification::OrderedProduct(OrderedProductForm::Pack),
            TypeData::PackFromPacks { .. } => StructuralClassification::OrderedProduct(OrderedProductForm::PackFromPacks),
            TypeData::Nominal { .. } | TypeData::Applied { .. } => {
                let Some((owner, _)) = types.applied_nominal_static_parts(ty) else {
                    return StructuralClassification::Unsupported;
                };

                if self.enums.enum_info(&owner).is_some() {
                    return StructuralClassification::Sum { owner };
                }

                match self.data.data_info(&owner) {
                    Some(info) if info.shape == DataShape::Tuple => {
                        StructuralClassification::OrderedProduct(OrderedProductForm::Data { owner })
                    }
                    _ => StructuralClassification::Unsupported,
                }
            }
            _ => StructuralClassification::Unsupported,
        }
    }

    /// Forms the P10 shallow ordered descriptor for one exact structural type.
    ///
    /// The descriptor retains open remainder terms and never recursively
    /// reflects component types. Data components are specialized through the
    /// existing canonical data materializer.
    pub fn ordered_product(
        &self,
        types: &mut TypeStore,
        ty: TypeId,
    ) -> Result<Option<OrderedProductStructure>, DataProductShapeMaterializationError> {
        let classification = self.classify(types, ty);
        let mut structure = match classification {
            StructuralClassification::Unsupported | StructuralClassification::Sum { .. } => return Ok(None),
            StructuralClassification::OrderedProduct(OrderedProductForm::Unit) => OrderedProductStructureBuilder::new(ty, OrderedProductOrigin::Unit),
            StructuralClassification::OrderedProduct(OrderedProductForm::Tuple) => match types.get(ty).clone() {
                TypeData::Tuple(tuple) => {
                    let mut structure = OrderedProductStructureBuilder::new(ty, OrderedProductOrigin::Tuple);
                    structure.structure.positional_remainder = tuple.positional_remainder;
                    structure.structure.labeled_remainder = tuple.labeled_remainder;
                    for element in tuple.elements.iter() {
                        structure.push(None, element.label.clone(), Some(element.ty));
                    }
                    structure
                }
                _ => return Ok(None),
            },
            StructuralClassification::OrderedProduct(OrderedProductForm::TupleFromPacks) => match types.get(ty).clone() {
                TypeData::TupleFromPacks { positionals, labeled } => {
                    let mut structure = OrderedProductStructureBuilder::new(ty, OrderedProductOrigin::TupleFromPacks);
                    structure.structure.symbolic_lanes = Some((positionals, labeled));
                    structure.structure.state = DataProductShapeState::Symbolic;
                    structure
                }
                _ => return Ok(None),
            },
            StructuralClassification::OrderedProduct(OrderedProductForm::Pack) => match types.get(ty).clone() {
                TypeData::Pack(pack) => {
                    let mut structure = OrderedProductStructureBuilder::new(ty, OrderedProductOrigin::Pack);
                    structure.structure.positional_remainder = pack.positional_remainder;
                    structure.structure.labeled_remainder = pack.labeled_remainder;
                    for element in pack.elements.iter() {
                        structure.push(None, element.label.clone(), Some(element.ty));
                    }
                    structure
                }
                _ => return Ok(None),
            },
            StructuralClassification::OrderedProduct(OrderedProductForm::PackFromPacks) => match types.get(ty).clone() {
                TypeData::PackFromPacks { positionals, labeled } => {
                    let mut structure = OrderedProductStructureBuilder::new(ty, OrderedProductOrigin::PackFromPacks);
                    structure.structure.symbolic_lanes = Some((positionals, labeled));
                    structure.structure.state = DataProductShapeState::Symbolic;
                    structure
                }
                _ => return Ok(None),
            },
            StructuralClassification::OrderedProduct(OrderedProductForm::Data { owner }) => {
                let Some(info) = self.data.data_info(&owner) else {
                    return Ok(None);
                };
                let materialized = materialize_data_product_shape(types, info, ty)?;
                let mut structure = OrderedProductStructureBuilder::new(ty, OrderedProductOrigin::Data { owner });
                structure.structure.state = materialized.state;
                for component in materialized.components.iter() {
                    structure.push(component.component.clone(), component.label.clone(), component.ty);
                }
                structure
            }
        };

        // Unit is represented by its own TypeData tag and naturally has two empty lanes.
        if matches!(structure.structure.origin, OrderedProductOrigin::Unit) {
            structure.structure.positional_remainder = AnonymousTupleRemainder::None;
            structure.structure.labeled_remainder = AnonymousTupleRemainder::None;
        }
        Ok(Some(structure.finish()))
    }

    /// Forms shallow declaration-ordered Variant facts for an exact Sum type.
    ///
    /// The descriptor reuses the existing reflection and exact-case
    /// specialization owners. It does not carry branch-local GADT proof
    /// environments or recursively reflect field types.
    pub fn sum(&self, types: &mut TypeStore, ty: TypeId) -> Result<Option<SumStructure>, SumStructureError> {
        let StructuralClassification::Sum { owner } = self.classify(types, ty) else {
            return Ok(None);
        };
        let Some(info) = self.enums.enum_info(&owner) else {
            return Ok(None);
        };

        let mut variants = Vec::with_capacity(info.variants.len());
        for id in info.variants.iter() {
            let variant = self.enums.variant_info(id).ok_or_else(|| SumStructureError::MissingVariant(id.clone()))?;
            let exact_case_type = types.exact_case_type(id, ty).map_err(|_| SumStructureError::ExactCaseTypeFormation)?;
            variants.push(SpecializedVariantStructure {
                declaration: VariantReflection::from_variant_info(variant),
                payload_kind: variant.payload_kind,
                exact_case: ExactCaseTypeReflection::from_exact_case(exact_case_type, variant, ty, types),
            });
        }

        Ok(Some(SumStructure {
            ty,
            owner,
            variants: variants.into_boxed_slice(),
        }))
    }
}

struct OrderedProductStructureBuilder {
    structure: OrderedProductStructure,
    positional_entries: Vec<OrderedProductEntry>,
    labeled_entries: Vec<OrderedProductEntry>,
}

impl OrderedProductStructureBuilder {
    fn new(ty: TypeId, origin: OrderedProductOrigin) -> Self {
        Self {
            structure: OrderedProductStructure {
                ty,
                origin,
                positional_entries: Box::new([]),
                labeled_entries: Box::new([]),
                positional_remainder: AnonymousTupleRemainder::None,
                labeled_remainder: AnonymousTupleRemainder::None,
                symbolic_lanes: None,
                state: DataProductShapeState::Concrete,
            },
            positional_entries: Vec::new(),
            labeled_entries: Vec::new(),
        }
    }

    fn push(&mut self, component: Option<DataComponentId>, label: Option<Box<str>>, ty: Option<TypeId>) {
        let entry = OrderedProductEntry { component, label, ty };
        if entry.label.is_some() {
            self.labeled_entries.push(entry);
        } else {
            self.positional_entries.push(entry);
        }
    }

    fn finish(mut self) -> OrderedProductStructure {
        self.structure.positional_entries = self.positional_entries.into_boxed_slice();
        self.structure.labeled_entries = self.labeled_entries.into_boxed_slice();
        self.structure
    }
}
