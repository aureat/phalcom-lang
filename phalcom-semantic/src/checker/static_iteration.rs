use crate::checker::typed_expr::TypedExpression;
use crate::checker::context::CheckingContext;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::identity::ExpressionId;
use crate::structural::{OrderedProductOrigin, StructuralClassification, StructuralClassifier};
use crate::types::denotation::SemanticDenotation;
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::row::RecordRowTail;
use crate::types::store::{TypeData, TypeStore};
use phalcom_common::range::SourceRange;
use std::collections::{BTreeMap, BTreeSet};

/// The semantic mode of static structural iteration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaticIterationMode {
    PackValues,
    TypePackTerms,
    OrderedProductValues {
        origin: OrderedProductOrigin,
    },
    OrderedProductTypes {
        origin: OrderedProductOrigin,
    },
    UnorderedProductValues,
    UnorderedProductFields,
    ReflectedPack,
}

/// The item domain of static structural iteration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaticIterationItemDomain {
    Uniform(TypeKnowledge),
    FiniteExact(Box<[TypeKnowledge]>),
    Unknown(TypeKnowledge),
}

impl StaticIterationItemDomain {
    /// Consolidates the item domain into one representative element knowledge for body checking.
    pub fn consolidated_item_knowledge(&self, store: &mut TypeStore) -> TypeKnowledge {
        match self {
            Self::Uniform(k) | Self::Unknown(k) => k.clone(),
            Self::FiniteExact(items) => {
                if items.is_empty() {
                    TypeKnowledge::established(store.unit(), EvidenceOrigin::Flow)
                } else if items.len() == 1 {
                    items[0].clone()
                } else {
                    let mut distinct_types = BTreeSet::new();
                    let mut has_non_known = false;
                    for item in items.iter() {
                        if let Some(ty) = item.ty() {
                            distinct_types.insert(ty);
                        } else {
                            has_non_known = true;
                        }
                    }
                    if has_non_known || distinct_types.is_empty() {
                        items[0].clone()
                    } else if distinct_types.len() == 1 {
                        TypeKnowledge::established(*distinct_types.iter().next().unwrap(), EvidenceOrigin::Flow)
                    } else {
                        let members: Vec<TypeId> = distinct_types.into_iter().collect();
                        let union_ty = store.union(&members);
                        TypeKnowledge::established(union_ty, EvidenceOrigin::Flow)
                    }
                }
            }
        }
    }
}

/// Checked semantic product for a `for` lane analyzed under `EvaluationPhase::Static`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticIterationLaneResolution {
    pub source_expression: ExpressionId,
    pub source_range: SourceRange,
    pub source_type: TypeId,
    pub mode: StaticIterationMode,
    pub item_domain: StaticIterationItemDomain,
}

/// Map from loop source expression to its static structural iteration lane resolution.
pub type StaticIterationLaneResolutionIndex = BTreeMap<ExpressionId, StaticIterationLaneResolution>;

/// Resolves a static structural iteration lane for a `for` loop in static evaluation phase.
pub fn resolve_static_iteration_lane(
    ctx: &mut CheckingContext<'_>,
    source_expr_id: ExpressionId,
    source_range: SourceRange,
    iter_typed: &TypedExpression,
) -> Option<StaticIterationLaneResolution> {
    let (Some(data), Some(enums)) = (ctx.data_table, ctx.enum_table) else {
        return None;
    };

    // First check if this is iterating over a type expression:
    if let Some(SemanticDenotation::TypeForm(term) | SemanticDenotation::StaticTerm(term)) = iter_typed.denotation {
        let classifier = StructuralClassifier::new(data, enums);
        let classification = classifier.classify(ctx.store, term);
        return match classification {
            StructuralClassification::OrderedProduct(_) => {
                let structure = classifier.ordered_product(ctx.store, term).ok().flatten();
                let (origin, item_domain) = if let Some(struct_info) = structure {
                    let mut items = Vec::new();
                    for entry in struct_info.positional_entries.iter().chain(struct_info.labeled_entries.iter()) {
                        if let Some(entry_ty) = entry.ty {
                            let desc = crate::checker::expression::type_form_descriptor_type(ctx, entry_ty)
                                .unwrap_or_else(|| ctx.store.unit());
                            items.push(TypeKnowledge::established(desc, EvidenceOrigin::DeclarationSemantics));
                        }
                    }
                    (struct_info.origin, StaticIterationItemDomain::FiniteExact(items.into_boxed_slice()))
                } else {
                    (
                        OrderedProductOrigin::Tuple,
                        StaticIterationItemDomain::Unknown(TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)),
                    )
                };
                Some(StaticIterationLaneResolution {
                    source_expression: source_expr_id,
                    source_range,
                    source_type: term,
                    mode: StaticIterationMode::OrderedProductTypes { origin },
                    item_domain,
                })
            }
            StructuralClassification::UnorderedProduct(_) => {
                let structure = classifier.unordered_product(ctx.store, term).ok().flatten();
                if let Some(struct_info) = structure {
                    if struct_info.fields.is_empty() && !matches!(struct_info.tail, RecordRowTail::Closed) {
                        ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::StaticIterationOpenTailNotFinite,
                            "cannot iterate over unordered product with open tail in static for",
                            source_range,
                        ));
                        return None;
                    }
                    let mut items = Vec::new();
                    for field in struct_info.fields.iter() {
                        if let Some(field_ty) = field.ty {
                            let desc = crate::checker::expression::type_form_descriptor_type(ctx, field_ty)
                                .unwrap_or_else(|| ctx.store.unit());
                            items.push(TypeKnowledge::established(desc, EvidenceOrigin::DeclarationSemantics));
                        }
                    }
                    Some(StaticIterationLaneResolution {
                        source_expression: source_expr_id,
                        source_range,
                        source_type: term,
                        mode: StaticIterationMode::UnorderedProductFields,
                        item_domain: StaticIterationItemDomain::FiniteExact(items.into_boxed_slice()),
                    })
                } else {
                    ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::StaticIterationNotStructural,
                        format!("static for requires structural product or pack, but found `{}`", ctx.store.format_type(term)),
                        source_range,
                    ));
                    None
                }
            }
            _ => {
                ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::StaticIterationNotStructural,
                    format!("static for requires structural product or pack, but found `{}`", ctx.store.format_type(term)),
                    source_range,
                ));
                None
            }
        };
    }

    // Otherwise, it is a value expression:
    let source_ty = iter_typed.knowledge.ty()?;

    // Check if the type is explicitly a Pack in TypeData:
    if let TypeData::Pack(items) = ctx.store.get(source_ty).clone() {
        let domain_items = items
            .elements
            .iter()
            .map(|elem| TypeKnowledge::established(elem.ty, EvidenceOrigin::DeclarationSemantics))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        return Some(StaticIterationLaneResolution {
            source_expression: source_expr_id,
            source_range,
            source_type: source_ty,
            mode: StaticIterationMode::PackValues,
            item_domain: StaticIterationItemDomain::FiniteExact(domain_items),
        });
    }

    // Check if it's the nominal Pack type:
    let is_nominal_pack = match ctx.store.get(source_ty) {
        TypeData::Nominal { declaration } => *declaration == ctx.core_ids.pack,
        _ => false,
    };
    if is_nominal_pack {
        return Some(StaticIterationLaneResolution {
            source_expression: source_expr_id,
            source_range,
            source_type: source_ty,
            mode: StaticIterationMode::PackValues,
            item_domain: StaticIterationItemDomain::Unknown(TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)),
        });
    }

    let classifier = StructuralClassifier::new(data, enums);
    let classification = classifier.classify(ctx.store, source_ty);
    match classification {
        StructuralClassification::OrderedProduct(_) => {
            let structure = classifier.ordered_product(ctx.store, source_ty).ok().flatten();
            if let Some(struct_info) = structure {
                let is_pack = matches!(struct_info.origin, OrderedProductOrigin::Pack | OrderedProductOrigin::PackFromPacks);
                let mode = if is_pack {
                    StaticIterationMode::PackValues
                } else {
                    StaticIterationMode::OrderedProductValues {
                        origin: struct_info.origin.clone(),
                    }
                };
                let mut items = Vec::new();
                for entry in struct_info.positional_entries.iter().chain(struct_info.labeled_entries.iter()) {
                    let entry_k = entry
                        .ty
                        .map(|t| TypeKnowledge::established(t, EvidenceOrigin::DeclarationSemantics))
                        .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause));
                    items.push(entry_k);
                }
                Some(StaticIterationLaneResolution {
                    source_expression: source_expr_id,
                    source_range,
                    source_type: source_ty,
                    mode,
                    item_domain: StaticIterationItemDomain::FiniteExact(items.into_boxed_slice()),
                })
            } else {
                ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::StaticIterationNotStructural,
                    format!("static for requires structural product or pack, but found `{}`", ctx.store.format_type(source_ty)),
                    source_range,
                ));
                None
            }
        }
        StructuralClassification::UnorderedProduct(_) => {
            let structure = classifier.unordered_product(ctx.store, source_ty).ok().flatten();
            if let Some(struct_info) = structure {
                if struct_info.fields.is_empty() && !matches!(struct_info.tail, RecordRowTail::Closed) {
                    ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::StaticIterationOpenTailNotFinite,
                        "cannot iterate over unordered product with open tail in static for",
                        source_range,
                    ));
                    return None;
                }
                let mut items = Vec::new();
                for field in struct_info.fields.iter() {
                    let field_k = field
                        .ty
                        .map(|t| TypeKnowledge::established(t, EvidenceOrigin::DeclarationSemantics))
                        .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause));
                    items.push(field_k);
                }
                Some(StaticIterationLaneResolution {
                    source_expression: source_expr_id,
                    source_range,
                    source_type: source_ty,
                    mode: StaticIterationMode::UnorderedProductFields,
                    item_domain: StaticIterationItemDomain::FiniteExact(items.into_boxed_slice()),
                })
            } else {
                ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::StaticIterationNotStructural,
                    format!("static for requires structural product or pack, but found `{}`", ctx.store.format_type(source_ty)),
                    source_range,
                ));
                None
            }
        }
        _ => {
            ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::StaticIterationNotStructural,
                format!("static for requires structural product or pack, but found `{}`", ctx.store.format_type(source_ty)),
                source_range,
            ));
            None
        }
    }
}
