//! Projection of checker-authorized static expressions to canonical Record keys.

use crate::db::{CancellationToken, QueryBudget, QueryKey, QueryOutcome};
use crate::field_key::FieldKey;
use crate::static_constant::{CheckedStaticExpression, StaticConstantId, StaticConstantTable};
use crate::static_definition::StaticDefinitionTable;
use crate::static_eval::{StaticEvalError, StaticEvalOutcome, StaticEvaluator};
use crate::static_value::{StaticValueStore, StaticValueStoreError};
use crate::types::store::TypeStore;
use phalcom_ast::ast::{
    AnonymousRecordRemainderSyntax, AnonymousRemainderSyntax, Expr, GenericArgumentSyntax, KindSyntax, RecordKeySyntax, TypeAnnotation, TypeAnnotationExpr,
    TypeCallableItem, TypeMemberArgumentSyntax, TypeTupleItem,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::marker::PhantomData;
use std::sync::Arc;

/// Stable identity for one computed Record key within its publishing owner.
///
/// `ordinal` is assigned by deterministic source traversal within `owner`.
/// It is deliberately independent of source ranges and expression-local IDs.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RecordKeyUseId {
    pub owner: QueryKey,
    pub ordinal: u32,
}

impl RecordKeyUseId {
    pub fn new(owner: QueryKey, ordinal: u32) -> Self {
        Self { owner, ordinal }
    }
}

/// One computed key expression found in a type annotation. Ordinals follow a
/// deterministic depth-first source traversal and can be offset when an owner
/// contains several annotations.
#[derive(Clone, Copy, Debug)]
pub struct TypeRecordKeyExpressionSite<'a> {
    pub ordinal: u32,
    pub syntax: &'a RecordKeySyntax,
    pub expression: &'a Expr,
}

pub fn type_record_key_expressions(annotation: &TypeAnnotation, first_ordinal: u32) -> Vec<TypeRecordKeyExpressionSite<'_>> {
    fn visit_kind<'a>(kind: &'a KindSyntax, next: &mut u32, sites: &mut Vec<TypeRecordKeyExpressionSite<'a>>) {
        match kind {
            KindSyntax::StaticDomain { form, .. } => visit_annotation(form, next, sites),
            KindSyntax::Arrow { parameter, result, .. } => {
                visit_kind(parameter, next, sites);
                visit_kind(result, next, sites);
            }
            KindSyntax::Grouped { inner, .. } => visit_kind(inner, next, sites),
            KindSyntax::Type(_)
            | KindSyntax::RecordRow(_)
            | KindSyntax::RecordFields(_)
            | KindSyntax::TypePack(_)
            | KindSyntax::LabeledTypePack(_)
            | KindSyntax::CompleteTypePack(_)
            | KindSyntax::Invalid { .. } => {}
        }
    }

    fn visit_remainder<'a>(remainder: &'a AnonymousRecordRemainderSyntax, next: &mut u32, sites: &mut Vec<TypeRecordKeyExpressionSite<'a>>) {
        match remainder {
            AnonymousRecordRemainderSyntax::Unconstrained(_) => {}
            AnonymousRecordRemainderSyntax::TypedValue { value_type, .. } => visit_annotation(value_type, next, sites),
            AnonymousRecordRemainderSyntax::TypedKey { key_type, .. } => visit_annotation(key_type, next, sites),
            AnonymousRecordRemainderSyntax::Typed { key_type, value_type, .. } => {
                visit_annotation(key_type, next, sites);
                visit_annotation(value_type, next, sites);
            }
        }
    }

    fn visit_type_remainder<'a>(remainder: &'a AnonymousRemainderSyntax, next: &mut u32, sites: &mut Vec<TypeRecordKeyExpressionSite<'a>>) {
        if let AnonymousRemainderSyntax::Typed { constraint, .. } = remainder {
            visit_annotation(constraint, next, sites);
        }
    }

    fn visit_argument<'a>(argument: &'a GenericArgumentSyntax, next: &mut u32, sites: &mut Vec<TypeRecordKeyExpressionSite<'a>>) {
        match argument {
            GenericArgumentSyntax::Single(annotation) => visit_annotation(annotation, next, sites),
            GenericArgumentSyntax::Expansion(expansion) => visit_annotation(&expansion.operand, next, sites),
            GenericArgumentSyntax::Remainder(remainder) => visit_type_remainder(&remainder.remainder, next, sites),
            GenericArgumentSyntax::Value(expression) => {
                if let Expr::TypeForm(annotation) = expression.as_ref() {
                    visit_annotation(annotation, next, sites);
                }
            }
            GenericArgumentSyntax::InferenceHole { .. } => {}
        }
    }

    fn visit_annotation<'a>(annotation: &'a TypeAnnotation, next: &mut u32, sites: &mut Vec<TypeRecordKeyExpressionSite<'a>>) {
        match &annotation.expr {
            TypeAnnotationExpr::Application { origin, arguments, .. } => {
                visit_annotation(origin, next, sites);
                for argument in arguments {
                    visit_argument(argument, next, sites);
                }
            }
            TypeAnnotationExpr::ExactEnumCase {
                enum_target,
                generic_arguments,
                ..
            } => {
                visit_annotation(enum_target, next, sites);
                for argument in generic_arguments {
                    visit_annotation(argument, next, sites);
                }
            }
            TypeAnnotationExpr::Union { members, .. } => {
                for member in members {
                    visit_annotation(member, next, sites);
                }
            }
            TypeAnnotationExpr::Tuple { elements, .. } => {
                for element in elements {
                    match element {
                        TypeTupleItem::Element(element) => visit_annotation(&element.ty, next, sites),
                        TypeTupleItem::Expansion(expansion) => visit_annotation(&expansion.operand, next, sites),
                        TypeTupleItem::Remainder(remainder) => {
                            if let AnonymousRemainderSyntax::Typed { constraint, .. } = &remainder.remainder {
                                visit_annotation(constraint, next, sites);
                            }
                        }
                    }
                }
            }
            TypeAnnotationExpr::Callable { parameters, result, .. } => {
                for parameter in parameters {
                    match parameter {
                        TypeCallableItem::Parameter(parameter) => visit_annotation(&parameter.ty, next, sites),
                        TypeCallableItem::Expansion(expansion) => visit_annotation(&expansion.operand, next, sites),
                        TypeCallableItem::Remainder(remainder) => {
                            if let AnonymousRemainderSyntax::Typed { constraint, .. } = &remainder.remainder {
                                visit_annotation(constraint, next, sites);
                            }
                        }
                    }
                }
                visit_annotation(result, next, sites);
            }
            TypeAnnotationExpr::Record { fields, tail, .. } => {
                for field in fields {
                    if let RecordKeySyntax::Computed { expr, .. } = &field.key {
                        let ordinal = *next;
                        *next = (*next).saturating_add(1);
                        sites.push(TypeRecordKeyExpressionSite {
                            ordinal,
                            syntax: &field.key,
                            expression: expr,
                        });
                    }
                    visit_annotation(&field.ty, next, sites);
                }
                if let Some(phalcom_ast::ast::RecordRowTailSyntax::Anonymous(remainder)) = tail {
                    visit_remainder(remainder, next, sites);
                }
            }
            TypeAnnotationExpr::TypeLambda { parameters, body, .. } => {
                for parameter in parameters {
                    if let Some(kind) = &parameter.kind {
                        visit_kind(kind, next, sites);
                    }
                }
                visit_annotation(body, next, sites);
            }
            TypeAnnotationExpr::AssociatedTypeProjection { subject, .. } => visit_annotation(subject, next, sites),
            TypeAnnotationExpr::QualifiedAssociatedTypeProjection { subject, trait_ref, .. } => {
                visit_annotation(subject, next, sites);
                visit_annotation(&trait_ref.trait_type, next, sites);
                for specialization in &trait_ref.associated_specializations {
                    visit_annotation(&specialization.value, next, sites);
                }
            }
            TypeAnnotationExpr::TypeMemberProjection { subject, arguments, .. } => {
                visit_annotation(subject, next, sites);
                for argument in arguments {
                    if let TypeMemberArgumentSyntax::TypeForm(annotation) = argument {
                        visit_annotation(annotation, next, sites);
                    }
                }
            }
            TypeAnnotationExpr::Reference(_)
            | TypeAnnotationExpr::Unit { .. }
            | TypeAnnotationExpr::Dynamic { .. }
            | TypeAnnotationExpr::Never { .. }
            | TypeAnnotationExpr::SelfType { .. }
            | TypeAnnotationExpr::Invalid { .. } => {}
        }
    }

    let mut next = first_ordinal;
    let mut sites = Vec::new();
    visit_annotation(annotation, &mut next, &mut sites);
    sites
}

/// Checks and projects every computed key expression within one annotation.
/// The owner supplies ordinary-checker and P2 evaluator closures so this
/// helper creates no parallel checking, name-resolution, or evaluation path.
pub fn project_type_record_keys<'a>(
    annotation: &'a TypeAnnotation,
    owner: QueryKey,
    first_ordinal: u32,
    check: impl FnMut(&TypeRecordKeyExpressionSite<'a>) -> QueryOutcome<Arc<CheckedStaticExpression>>,
    project: impl FnMut(&CheckedStaticExpression) -> QueryOutcome<StaticKeyProjection>,
) -> QueryOutcome<ProjectedRecordKeySet<'a>> {
    project_type_record_key_sites(type_record_key_expressions(annotation, first_ordinal), owner, check, project)
}

/// Variant for owners that publish several type annotations under one query
/// key. The iterator order is the deterministic owner traversal order.
pub fn project_type_record_key_annotations<'a>(
    annotations: impl IntoIterator<Item = &'a TypeAnnotation>,
    owner: QueryKey,
    check: impl FnMut(&TypeRecordKeyExpressionSite<'a>) -> QueryOutcome<Arc<CheckedStaticExpression>>,
    project: impl FnMut(&CheckedStaticExpression) -> QueryOutcome<StaticKeyProjection>,
) -> QueryOutcome<ProjectedRecordKeySet<'a>> {
    let mut sites = Vec::new();
    let mut next_ordinal = 0_u32;
    for annotation in annotations {
        let annotation_sites = type_record_key_expressions(annotation, next_ordinal);
        if let Some(last) = annotation_sites.last() {
            next_ordinal = last.ordinal.saturating_add(1);
        }
        sites.extend(annotation_sites);
    }
    project_type_record_key_sites(sites, owner, check, project)
}

fn project_type_record_key_sites<'a>(
    sites: Vec<TypeRecordKeyExpressionSite<'a>>,
    owner: QueryKey,
    mut check: impl FnMut(&TypeRecordKeyExpressionSite<'a>) -> QueryOutcome<Arc<CheckedStaticExpression>>,
    mut project: impl FnMut(&CheckedStaticExpression) -> QueryOutcome<StaticKeyProjection>,
) -> QueryOutcome<ProjectedRecordKeySet<'a>> {
    let mut projections = ProjectedRecordKeySet::new(owner);
    for site in &sites {
        let checked = match check(site) {
            QueryOutcome::Ready(checked) => checked,
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Failed(error) => return QueryOutcome::Failed(error),
        };
        let projection = match project(&checked) {
            QueryOutcome::Ready(projection) => projection,
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Failed(error) => return QueryOutcome::Failed(error),
        };
        if let Err(use_id) = projections.insert(site.syntax, site.ordinal, checked, projection) {
            return QueryOutcome::Failed(format!("duplicate computed Record key identity {use_id:?}"));
        }
    }
    QueryOutcome::Ready(projections)
}

/// Outcome of reducing a checked static expression at a Record-key site.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaticKeyProjection {
    /// Evaluation produced one canonical key from the closed FieldKey domain.
    Concrete(FieldKey),
    /// Evaluation is valid but requires a later static stage to reduce.
    DeferredToP7,
    /// The checked expression or reduced value cannot serve as a Record key.
    Invalid(StaticKeyProjectionError),
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum StaticKeyProjectionError {
    #[error("static key expression has checker diagnostics")]
    CheckerDiagnostics,
    #[error("selected static constant `{0}` has no reduced current value")]
    ConstantNotReduced(StaticConstantId),
    #[error("static key expression produced a kinded result")]
    KindedResult,
    #[error("static key evaluation failed: {0}")]
    Evaluation(StaticEvalError),
    #[error("static key evaluation was recursive")]
    Recursive,
    #[error("static key evaluation is unknown: {0:?}")]
    Unknown(crate::types::evidence::UnknownReason),
    #[error("static key evaluation is blocked: {0:?}")]
    Blocked(crate::types::outcome::BlockReason),
    #[error("static key evaluation crosses a dynamic boundary: {0:?}")]
    Dynamic(crate::types::outcome::DynamicBoundaryObligation),
    #[error("reduced static value is not a Record key: {0}")]
    InvalidFieldKey(StaticValueStoreError),
}

/// Checked projection facts supplied by the semantic owner before type-form
/// construction. The syntax-address index is short-lived plumbing only; the
/// retained identity is the owner-relative [`RecordKeyUseId`].
#[derive(Clone, Debug)]
pub struct ProjectedRecordKeyFact {
    pub use_id: RecordKeyUseId,
    pub checked: Arc<CheckedStaticExpression>,
    pub projection: StaticKeyProjection,
}

/// Owner-local set of already checked key projections for one annotation
/// borrow. It carries no query/product identity of its own.
#[derive(Debug)]
pub struct ProjectedRecordKeySet<'a> {
    owner: QueryKey,
    by_syntax_address: HashMap<usize, RecordKeyUseId>,
    facts: BTreeMap<RecordKeyUseId, ProjectedRecordKeyFact>,
    _syntax_lifetime: PhantomData<&'a RecordKeySyntax>,
}

impl<'a> ProjectedRecordKeySet<'a> {
    pub fn new(owner: QueryKey) -> Self {
        Self {
            owner,
            by_syntax_address: HashMap::new(),
            facts: BTreeMap::new(),
            _syntax_lifetime: PhantomData,
        }
    }

    /// Adds one fact for a computed key. The caller supplies the deterministic
    /// owner-local traversal ordinal used in its stable identity.
    pub fn insert(
        &mut self,
        syntax: &'a RecordKeySyntax,
        ordinal: u32,
        checked: Arc<CheckedStaticExpression>,
        projection: StaticKeyProjection,
    ) -> Result<RecordKeyUseId, RecordKeyUseId> {
        if !matches!(syntax, RecordKeySyntax::Computed { .. }) {
            let use_id = RecordKeyUseId::new(self.owner.clone(), ordinal);
            return Err(use_id);
        }
        let use_id = RecordKeyUseId::new(self.owner.clone(), ordinal);
        let address = std::ptr::from_ref(syntax) as usize;
        if self.facts.contains_key(&use_id) || self.by_syntax_address.contains_key(&address) {
            return Err(use_id);
        }
        self.by_syntax_address.insert(address, use_id.clone());
        self.facts.insert(
            use_id.clone(),
            ProjectedRecordKeyFact {
                use_id: use_id.clone(),
                checked,
                projection,
            },
        );
        Ok(use_id)
    }

    pub fn owner(&self) -> &QueryKey {
        &self.owner
    }

    pub fn fact_for(&self, syntax: &RecordKeySyntax) -> Option<&ProjectedRecordKeyFact> {
        let address = std::ptr::from_ref(syntax) as usize;
        let use_id = self.by_syntax_address.get(&address)?;
        self.facts.get(use_id)
    }

    pub fn facts(&self) -> impl Iterator<Item = &ProjectedRecordKeyFact> {
        self.facts.values()
    }

    /// All exact semantic dependencies selected while checking projected keys.
    pub fn dependencies(&self) -> BTreeSet<crate::checker::analysis::SemanticDependency> {
        let mut dependencies = self
            .facts
            .values()
            .flat_map(|fact| fact.checked.dependencies.iter().cloned())
            .collect::<BTreeSet<_>>();
        dependencies.extend(self.facts.values().flat_map(|fact| {
            fact.checked
                .static_constant_reads
                .iter()
                .cloned()
                .map(crate::static_constant::StaticConstantId)
                .map(crate::checker::analysis::SemanticDependency::StaticConstant)
        }));
        dependencies
    }
}

/// Reduces one ordinary-checker-approved expression to an owned canonical
/// `FieldKey`. Constant dependencies come from checker-selected binding IDs
/// retained in the checked expression; this function does no name lookup.
pub fn project_checked_static_key(
    checked: &CheckedStaticExpression,
    constants: &StaticConstantTable,
    definitions: &StaticDefinitionTable,
    types: &TypeStore,
    data_semantics: &crate::data_semantics::DataSemanticTable,
    enum_semantics: &crate::enum_semantics::EnumSemanticTable,
    values: &mut StaticValueStore,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
) -> QueryOutcome<StaticKeyProjection> {
    if !checked.diagnostics.is_empty() {
        return QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::CheckerDiagnostics));
    }

    for symbol in checked.static_constant_reads.iter() {
        let id = StaticConstantId(symbol.clone());
        let Some(product) = constants.get(&id) else {
            return QueryOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
        };
        if product.value.is_none() {
            return QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::ConstantNotReduced(id)));
        }
    }

    let outcome = {
        let mut evaluator = StaticEvaluator::for_checked_expression(checked, values, budget, cancellation)
            .with_static_constants(constants)
            .with_static_definitions(definitions)
            .with_type_store(types)
            .with_semantic_tables(types, data_semantics, enum_semantics);
        evaluator.evaluate()
    };

    match outcome {
        StaticEvalOutcome::Reduced(value) => match values.field_key_of(value) {
            Ok(key) => QueryOutcome::Ready(StaticKeyProjection::Concrete(key)),
            Err(error) => QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::InvalidFieldKey(error))),
        },
        StaticEvalOutcome::Symbolic(_) => QueryOutcome::Ready(StaticKeyProjection::DeferredToP7),
        StaticEvalOutcome::ReducedKinded(_) => QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::KindedResult)),
        StaticEvalOutcome::Invalid(error) => QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::Evaluation(error))),
        StaticEvalOutcome::Recursive(_) => QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::Recursive)),
        StaticEvalOutcome::BudgetExceeded(report) => QueryOutcome::BudgetExceeded(report),
        StaticEvalOutcome::Cancelled => QueryOutcome::Cancelled,
        StaticEvalOutcome::Unknown(reason) => QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::Unknown(reason))),
        StaticEvalOutcome::Blocked(reason) => QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::Blocked(reason))),
        StaticEvalOutcome::Dynamic(obligation) => {
            QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::Dynamic(obligation)))
        }
    }
}

/// Retains all checker-produced semantic dependencies for an owner query.
pub fn exact_static_key_dependencies(checked: &CheckedStaticExpression) -> Arc<[crate::checker::analysis::SemanticDependency]> {
    checked.dependencies.clone()
}
