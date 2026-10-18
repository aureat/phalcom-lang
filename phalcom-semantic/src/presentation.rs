//! Pure presentation projections over compiler-owned semantic products.

use crate::advisory::ValueShape;
use crate::advisory::{AdvisoryFact, AdvisoryTargetResolution};
use crate::checker::causal::CausalInvalidity;
use crate::checker::{AnalysisStatus, BindingConsistency, CallableAnalysis, CallableAnalysisStatus, ExpressionAnalysis, ExpressionAnalysisIndex};
use crate::identity::{BindingId, CallableId, ExpressionId, FieldId, ModuleId, SourceSiteId};
use crate::signature::CallableSemanticSignature;
use crate::source_index::SourceSemanticIndex;
use crate::source_index::interval::{RangeEntry, RangeIndex};
use crate::source_index::{CallableSourceInfo, SourceCallableKind};
use crate::types::evidence::TypeKnowledge;
use crate::types::id::TypeId;
use crate::types::store::TypeStore;
use phalcom_common::range::SourceRange;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError, BudgetedSortedMap};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// Canonical formal presentation state for one semantic site.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormalPresentation {
    /// A formally established type with canonical source spelling.
    Known(String),
    /// An explicit or analyzed dynamic boundary.
    Dynamic,
    /// Formal analysis could not establish a type.
    Unknown,
    /// The site is invalid because a semantic diagnostic owns it.
    Invalid,
    /// The site depends on a blocked semantic product.
    Blocked,
    /// Analysis was cancelled before a formal result was ready.
    Cancelled,
    /// Analysis exceeded its configured budget.
    BudgetExceeded,
    /// Analysis failed internally.
    InternalFailure,
    /// Analysis produced a bounded partial product.
    Partial,
}

impl FormalPresentation {
    /// Returns text suitable for the formal section of an editor surface.
    pub fn text(&self) -> String {
        match self {
            Self::Known(ty) => ty.clone(),
            Self::Dynamic => "Dynamic".to_string(),
            Self::Unknown => "Unknown".to_string(),
            Self::Invalid => "Invalid".to_string(),
            Self::Blocked => "Blocked".to_string(),
            Self::Cancelled => "Cancelled".to_string(),
            Self::BudgetExceeded => "Budget exceeded".to_string(),
            Self::InternalFailure => "Internal failure".to_string(),
            Self::Partial => "Partial".to_string(),
        }
    }
}

/// Pure canonical formatter backed by one immutable type store.
pub struct TypePresenter<'a> {
    store: &'a TypeStore,
}

impl<'a> TypePresenter<'a> {
    /// Creates a presenter over an existing compiler-owned type store.
    pub fn new(store: &'a TypeStore) -> Self {
        Self { store }
    }

    /// Formats one canonical type without performing inference or relation checks.
    pub fn present_type(&self, ty: TypeId) -> String {
        self.store.format_type(ty)
    }

    /// Formats formal knowledge while preserving its epistemic state.
    pub fn present_knowledge(&self, knowledge: &TypeKnowledge) -> FormalPresentation {
        match knowledge {
            TypeKnowledge::Known(evidence) => FormalPresentation::Known(self.present_type(evidence.ty())),
            TypeKnowledge::Dynamic(_) => FormalPresentation::Dynamic,
            TypeKnowledge::Unknown(_) => FormalPresentation::Unknown,
        }
    }

    /// Maps expression analysis status before consulting its knowledge payload.
    pub fn present_expression(&self, expression: &ExpressionAnalysis) -> FormalPresentation {
        match &expression.status {
            AnalysisStatus::Ready => self.present_knowledge(&expression.knowledge),
            AnalysisStatus::Invalid(_) => FormalPresentation::Invalid,
            AnalysisStatus::Suppressed(_) => FormalPresentation::Blocked,
            AnalysisStatus::Blocked(_) => FormalPresentation::Blocked,
            AnalysisStatus::DynamicBoundary(_) => FormalPresentation::Dynamic,
            AnalysisStatus::Cancelled => FormalPresentation::Cancelled,
            AnalysisStatus::BudgetExceeded(_) => FormalPresentation::BudgetExceeded,
            AnalysisStatus::InternalFailure(_) => FormalPresentation::InternalFailure,
        }
    }

    /// Maps callable analysis status to a presentation-only state.
    pub fn present_callable_status(&self, status: CallableAnalysisStatus) -> FormalPresentation {
        match status {
            CallableAnalysisStatus::Complete => FormalPresentation::Known("Ready".to_string()),
            CallableAnalysisStatus::Partial => FormalPresentation::Partial,
            CallableAnalysisStatus::Blocked => FormalPresentation::Blocked,
            CallableAnalysisStatus::Cancelled => FormalPresentation::Cancelled,
            CallableAnalysisStatus::BudgetExceeded => FormalPresentation::BudgetExceeded,
            CallableAnalysisStatus::InternalFailure(_) => FormalPresentation::InternalFailure,
        }
    }
}

/// Protocol-neutral rendering of compiler-owned advisory runtime shapes.
///
/// This formatter deliberately emits language-level text only. It does not
/// depend on, or construct, any editor protocol type.
pub struct AdvisoryPresenter;

impl AdvisoryPresenter {
    /// Presents one canonical advisory shape deterministically.
    pub fn present_shape(shape: &ValueShape) -> String {
        match shape {
            ValueShape::Unknown => "?".to_string(),
            ValueShape::Never => "Never".to_string(),
            ValueShape::Unit => "Unit".to_string(),
            ValueShape::Instance(declaration) => declaration.name.to_string(),
            ValueShape::ClassObject(declaration) => format!("{} class", declaration.name),
            ValueShape::Module(module) => module.to_string(),
            ValueShape::Tuple(elements) => format!("({})", elements.iter().map(Self::present_shape).collect::<Vec<_>>().join(", ")),
            ValueShape::ExactList(elements) => format!("List<{}>", Self::present_shape(&ValueShape::bounded_union(elements.iter().cloned()))),
            ValueShape::Record(fields) => format!(
                "{{{}}}",
                fields
                    .iter()
                    .map(|(label, value)| format!("{label}: {}", Self::present_shape(value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ValueShape::List(element) => format!("List<{}>", Self::present_shape(element)),
            ValueShape::Set(element) => format!("Set<{}>", Self::present_shape(element)),
            ValueShape::Map { key, value } => format!("Map<{}, {}>", Self::present_shape(key), Self::present_shape(value)),
            ValueShape::Range(element) => format!("Range<{}>", Self::present_shape(element)),
            ValueShape::Callable(_) => "Callable".to_string(),
            ValueShape::Selector(selector) => format!("#{selector}"),
            ValueShape::SelectorPattern(pattern) => format!("#{pattern}"),
            ValueShape::Family { spec, .. } => match spec {
                phalcom_ast::ast::NormalizedSelectorSpec::Exact(selector) => format!("Family<#{selector}>"),
                phalcom_ast::ast::NormalizedSelectorSpec::Pattern(pattern) => format!("Family<#{pattern}>"),
            },
            ValueShape::Method(callable) => format!("Method<{}>", callable.selector),
            ValueShape::MethodFamily(family) => format!("MethodFamily<#{pattern}>", pattern = family.pattern),
            ValueShape::BoundMethod { method, .. } => format!("BoundMethod<{}>", method.selector),
            ValueShape::BoundMethodFamily { family, .. } => format!("BoundMethodFamily<#{pattern}>", pattern = family.pattern),
            ValueShape::Union(alternatives) => alternatives.iter().map(Self::present_shape).collect::<Vec<_>>().join(" | "),
        }
    }
}

/// Protocol-neutral presentation of one callable parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterPresentation {
    pub index: u32,
    pub name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub rest: phalcom_ast::ast::RestMode,
    pub type_: FormalPresentation,
}

/// Protocol-neutral presentation of one canonical callable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallablePresentation {
    pub callable: CallableId,
    pub selector: String,
    pub kind: SourceCallableKind,
    pub owner_name: Box<str>,
    pub parameters: Arc<[ParameterPresentation]>,
    pub return_type: FormalPresentation,
    pub documentation: Option<Arc<str>>,
}

impl CallablePresentation {
    /// Projects canonical signature/source products into editor-neutral text.
    pub fn from_signature(signature: &CallableSemanticSignature, source: Option<&CallableSourceInfo>, presenter: &TypePresenter<'_>) -> Self {
        let parameters = signature
            .parameters
            .iter()
            .map(|parameter| ParameterPresentation {
                index: parameter.index(),
                name: parameter.local_name.clone(),
                external_label: parameter.external_label.clone(),
                rest: parameter.rest,
                type_: present_declared_type(&parameter.declared_type, presenter),
            })
            .collect::<Vec<_>>();
        Self {
            callable: signature.callable.clone(),
            selector: signature.selector.to_string(),
            kind: source.map_or(SourceCallableKind::Method, |source| source.kind),
            owner_name: signature.owner.name.clone(),
            parameters: Arc::from(parameters.into_boxed_slice()),
            return_type: presenter.present_knowledge(&signature.published_return_knowledge()),
            documentation: None,
        }
    }
}

/// Protocol-neutral presentation of one canonical field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldPresentation {
    pub field: FieldId,
    pub owner_name: Box<str>,
    pub name: Box<str>,
    pub type_: FormalPresentation,
    pub mutable: bool,
    pub documentation: Option<Arc<str>>,
}

impl FieldPresentation {
    /// Projects canonical field signature into editor-neutral text.
    pub fn from_signature(signature: &crate::signature::FieldSemanticSignature, presenter: &TypePresenter<'_>) -> Self {
        Self {
            field: signature.field.clone(),
            owner_name: signature.owner.name.clone(),
            name: signature.name.clone(),
            type_: present_declared_type(&signature.declared_type, presenter),
            mutable: signature.mutable,
            documentation: None,
        }
    }
}

pub(crate) fn present_declared_type(fact: &crate::declaration_type::DeclaredTypeFact, presenter: &TypePresenter<'_>) -> FormalPresentation {
    match &fact.state {
        crate::declaration_type::DeclaredTypeState::Known(term) => present_type_term(term, presenter),
        crate::declaration_type::DeclaredTypeState::Dynamic(_) => FormalPresentation::Dynamic,
        crate::declaration_type::DeclaredTypeState::Unknown(_) => FormalPresentation::Unknown,
    }
}

fn present_type_term(term: &crate::types::TypeTerm, presenter: &TypePresenter<'_>) -> FormalPresentation {
    match term {
        crate::types::TypeTerm::Canonical(ty) => FormalPresentation::Known(presenter.present_type(*ty)),
        crate::types::TypeTerm::SelfType(_) | crate::types::TypeTerm::Infer(_) => FormalPresentation::Unknown,
    }
}

/// Stable identity for a formal type site.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FormalSiteId {
    Expression {
        callable: CallableId,
        expression: ExpressionId,
    },
    Binding {
        callable: CallableId,
        binding: crate::identity::BindingId,
    },
    Callable(CallableId),
}

/// Immutable projection of a compiler-owned formal result at one source site.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormalTypeSite {
    /// Canonical source module owning the range.
    pub module: ModuleId,
    /// Source range of the expression or callable body.
    pub range: SourceRange,
    /// Semantic site identity.
    pub site: FormalSiteId,
    /// Formal state and canonical type text.
    pub presentation: FormalPresentation,
}

/// Immutable presentation projection. It stores no independent invalidation state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SemanticPresentationIndex {
    sites: BTreeMap<FormalSiteId, FormalTypeSite>,
    binding_sites: BTreeMap<(ModuleId, SourceRange), FormalSiteId>,
    expression_sites: BTreeMap<ModuleId, Vec<FormalTypeSite>>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FormalFactRef {
    Callable(CallableId),
    Expression {
        callable: CallableId,
        expression: ExpressionId,
    },
    Binding {
        callable: CallableId,
        binding: crate::identity::BindingId,
    },
}

impl FormalFactRef {
    /// Returns the canonical module owning this formal fact.
    pub fn module(&self) -> &ModuleId {
        match self {
            FormalFactRef::Callable(c) => c.module(),
            FormalFactRef::Expression { callable, .. } => callable.module(),
            FormalFactRef::Binding { callable, .. } => callable.module(),
        }
    }
}

/// Machine-readable formal readiness/validity state attached to a projected
/// source fact. This is separate from rendered type text and preserves causal
/// invalidity without making advisory observations formal evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FormalFactStatus {
    Ready,
    Unknown,
    Dynamic,
    Invalid,
    InvalidMultiple,
    Blocked,
    Cancelled,
    BudgetExceeded,
    InternalFailure,
    Partial,
}

/// Snapshot-owned source attachment for one machine-readable formal fact.
///
/// This record contains identity and location only. Callers retrieve formal
/// knowledge/status from the keyed checker product; no rendered string is used
/// as semantic truth.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormalFactSite {
    pub module: ModuleId,
    pub range: SourceRange,
    pub fact: FormalFactRef,
    pub status: FormalFactStatus,
    /// Causal invalidity is projected as bounded machine-readable state; it
    /// is never reconstructed from rendered presentation text.
    pub causal_invalidity: CausalInvalidity,
    /// Persistent binding contract and reconciliation outcome, when this site
    /// is a binding. Expression/callable sites leave this absent.
    pub contract: Option<FormalContractRelation>,
}

fn compare_formal_fact_sites(left: &FormalFactSite, right: &FormalFactSite) -> std::cmp::Ordering {
    left.module
        .cmp(&right.module)
        .then_with(|| left.range.start.cmp(&right.range.start))
        .then_with(|| left.range.len().cmp(&right.range.len()))
        .then_with(|| left.fact.cmp(&right.fact))
}

/// Formal binding-contract relation retained by the source projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormalContractRelation {
    pub ty: TypeId,
    pub origin: crate::checker::binding::BindingContractOrigin,
    pub consistency: BindingConsistency,
}

/// Read-only composition of source, formal, and advisory products at one
/// indexed source position. Each channel remains independently optional and
/// retains its own authority/status semantics.
#[derive(Clone, Debug)]
pub struct SemanticSiteView<'a> {
    /// Snapshot-local source site selected by the source occurrence index.
    pub source_site: Option<SourceSiteId>,
    /// Keyed formal fact attached at this position, if any.
    pub formal: Option<&'a FormalFactSite>,
    /// Advisory runtime-shape fact attached to the selected source site.
    pub advisory: Option<&'a AdvisoryFact>,
    /// Advisory canonical target attached to the selected source site.
    pub target: Option<&'a AdvisoryTargetResolution>,
}

/// Formal presentation facts and interval index for one source module.
#[derive(Clone, Debug, Default)]
pub struct ModuleFormalProjection {
    by_fact: FormalFactLookup,
    sites: Arc<[FormalFactSite]>,
    intervals: RangeIndex<usize>,
    binding_attachments: BindingAttachmentLookup,
}

#[derive(Clone, Debug)]
enum FormalFactLookup {
    Source(BTreeMap<FormalFactRef, FormalFactSite>),
    Frozen(Box<[usize]>),
}

impl Default for FormalFactLookup {
    fn default() -> Self {
        Self::Source(BTreeMap::new())
    }
}

#[derive(Clone, Debug)]
enum BindingAttachmentLookup {
    Source(BTreeMap<(CallableId, BindingId), SourceRange>),
    Frozen(Box<[(usize, SourceRange)]>),
}

impl Default for BindingAttachmentLookup {
    fn default() -> Self {
        Self::Source(BTreeMap::new())
    }
}

#[derive(Debug)]
pub(crate) enum ModuleFormalProjectionBuildError {
    Allocation(AllocationError),
    DuplicateFact,
}

#[derive(Debug)]
pub(crate) enum FormalSemanticProjectionBuildError {
    Allocation(AllocationError),
    DuplicateFact,
    DuplicateModule,
}

impl ModuleFormalProjection {
    /// Creates a module formal projection from already-built formal fact sites.
    pub fn new(sites: Vec<FormalFactSite>) -> Self {
        Self::new_with_binding_attachments(sites, BTreeMap::new())
    }

    fn new_with_binding_attachments(
        sites: Vec<FormalFactSite>,
        binding_attachments: BTreeMap<(CallableId, BindingId), SourceRange>,
    ) -> Self {
        let mut by_fact = BTreeMap::new();
        for site in &sites {
            by_fact.insert(site.fact.clone(), site.clone());
        }
        let sites_arc: Arc<[FormalFactSite]> = Arc::from(sites.into_boxed_slice());
        let intervals = RangeIndex::new(
            sites_arc
                .iter()
                .enumerate()
                .map(|(index, site)| RangeEntry::new(site.range, index, formal_fact_priority(&site.fact))),
        );
        Self {
            by_fact: FormalFactLookup::Source(by_fact),
            sites: sites_arc,
            intervals,
            binding_attachments: BindingAttachmentLookup::Source(binding_attachments),
        }
    }

    fn new_frozen_with_allocation_budget(
        sites: Vec<FormalFactSite>,
        binding_ranges: &crate::parent_map::ParentMap<CallableId, crate::ordered_map::OrderedMap<BindingId, SourceRange>>,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, ModuleFormalProjectionBuildError> {
        let mut fact_indices = allocations
            .vector::<usize>(sites.len())
            .map_err(ModuleFormalProjectionBuildError::Allocation)?;
        fact_indices.extend(0..sites.len());
        fact_indices.sort_unstable_by(|&left, &right| sites[left].fact.cmp(&sites[right].fact));
        if fact_indices.windows(2).any(|pair| sites[pair[0]].fact == sites[pair[1]].fact) {
            return Err(ModuleFormalProjectionBuildError::DuplicateFact);
        }

        let binding_attachment_count = sites
            .iter()
            .filter_map(|site| match &site.fact {
                FormalFactRef::Binding { callable, binding } => binding_ranges.get(callable)?.get(binding),
                FormalFactRef::Callable(_) | FormalFactRef::Expression { .. } => None,
            })
            .count();
        let mut binding_attachments = allocations
            .vector::<(usize, SourceRange)>(binding_attachment_count)
            .map_err(ModuleFormalProjectionBuildError::Allocation)?;
        for (index, site) in sites.iter().enumerate() {
            if let FormalFactRef::Binding { callable, binding } = &site.fact
                && let Some(range) = binding_ranges.get(callable).and_then(|ranges| ranges.get(binding))
            {
                binding_attachments.push((index, *range));
            }
        }

        let interval_entries = sites
            .iter()
            .enumerate()
            .map(|(index, site)| RangeEntry::new(site.range, index, formal_fact_priority(&site.fact)));
        let intervals = RangeIndex::new_with_allocation_budget(interval_entries, sites.len(), allocations)
            .map_err(ModuleFormalProjectionBuildError::Allocation)?;
        let fact_indices = allocations
            .boxed_slice(fact_indices)
            .map_err(ModuleFormalProjectionBuildError::Allocation)?;
        let binding_attachments = allocations
            .boxed_slice(binding_attachments)
            .map_err(ModuleFormalProjectionBuildError::Allocation)?;
        let sites = allocations.arc_slice(sites).map_err(ModuleFormalProjectionBuildError::Allocation)?;

        Ok(Self {
            by_fact: FormalFactLookup::Frozen(fact_indices),
            sites,
            intervals,
            binding_attachments: BindingAttachmentLookup::Frozen(binding_attachments),
        })
    }

    /// Returns one formal site by canonical fact identity.
    pub fn get(&self, fact: &FormalFactRef) -> Option<&FormalFactSite> {
        match &self.by_fact {
            FormalFactLookup::Source(by_fact) => by_fact.get(fact),
            FormalFactLookup::Frozen(indices) => {
                let position = indices.binary_search_by(|&index| self.sites[index].fact.cmp(fact)).ok()?;
                self.sites.get(indices[position])
            }
        }
    }

    /// Returns most-specific formal site at a source position.
    pub fn fact_at(&self, offset: usize) -> Option<&FormalFactSite> {
        let index = self.intervals.index_at(offset)?;
        self.sites.get(index)
    }

    /// Returns all formal sites in this module shard.
    pub fn sites(&self) -> &[FormalFactSite] {
        &self.sites
    }

    /// Returns the exact compiler-owned declaration site for one binding.
    /// A formal fact may exist without a source attachment (for example for
    /// an implicit receiver), so its presentation range is not a substitute.
    pub fn binding_attachment_range(&self, callable: &CallableId, binding: BindingId) -> Option<SourceRange> {
        match &self.binding_attachments {
            BindingAttachmentLookup::Source(attachments) => attachments.get(&(callable.clone(), binding)).copied(),
            BindingAttachmentLookup::Frozen(attachments) => {
                let fact_index = match &self.by_fact {
                    FormalFactLookup::Source(_) => return None,
                    FormalFactLookup::Frozen(indices) => indices
                        .binary_search_by(|&index| match &self.sites[index].fact {
                            FormalFactRef::Callable(_) | FormalFactRef::Expression { .. } => std::cmp::Ordering::Less,
                            FormalFactRef::Binding {
                                callable: candidate,
                                binding: candidate_binding,
                            } => candidate.cmp(callable).then_with(|| candidate_binding.cmp(&binding)),
                        })
                        .ok()
                        .map(|position| indices[position])?,
                };
                attachments
                    .binary_search_by_key(&fact_index, |(index, _)| *index)
                    .ok()
                    .map(|position| attachments[position].1)
            }
        }
    }
}

/// Indexed machine-readable formal source projection with persistent module roots.
#[derive(Clone, Debug, Default)]
pub struct FormalSemanticProjection {
    modules: Option<im::OrdMap<ModuleId, Arc<ModuleFormalProjection>>>,
    frozen_modules: Option<Arc<BudgetedSortedMap<ModuleId, Arc<ModuleFormalProjection>>>>,
    retired_modules: Option<im::OrdSet<ModuleId>>,
}

impl FormalSemanticProjection {
    /// Builds projection from immutable callable products without re-analysis.
    pub fn from_callable_analyses(analyses: &HashMap<CallableId, Arc<CallableAnalysis>>) -> Self {
        Self::from_callable_analyses_with_source_index(analyses, None)
    }

    /// Builds module-specific formal projection from callable products and current source index.
    pub fn build_module_projection(
        module: &ModuleId,
        callable_ids: impl IntoIterator<Item = CallableId>,
        analyses: &HashMap<CallableId, Arc<CallableAnalysis>>,
        source_index: Option<&SourceSemanticIndex>,
    ) -> ModuleFormalProjection {
        Self::build_module_projection_with_binding_ranges(module, callable_ids, analyses, source_index, None)
    }

    fn build_module_projection_with_binding_ranges(
        module: &ModuleId,
        callable_ids: impl IntoIterator<Item = CallableId>,
        analyses: &HashMap<CallableId, Arc<CallableAnalysis>>,
        source_index: Option<&SourceSemanticIndex>,
        binding_ranges: Option<&BTreeMap<CallableId, BTreeMap<BindingId, SourceRange>>>,
    ) -> ModuleFormalProjection {
        let mut sites = Vec::new();
        let mut binding_attachments = BTreeMap::new();
        let module_index = source_index.and_then(|index| index.module(module));
        for callable in callable_ids {
            let Some(analysis) = analyses.get(&callable) else {
                continue;
            };
            debug_assert_eq!(analysis.callable.module(), module);
            let callable_fact = FormalFactRef::Callable(analysis.callable.clone());
            sites.push(FormalFactSite {
                module: module.clone(),
                range: module_index
                    .and_then(|index| index.structure.callable_body_ranges.get(&analysis.callable).copied())
                    .unwrap_or(analysis.body_range),
                fact: callable_fact,
                status: callable_status(analysis.status),
                causal_invalidity: callable_causal_invalidity(analysis),
                contract: None,
            });
            for expression in analysis.expressions.values() {
                sites.push(FormalFactSite {
                    module: module.clone(),
                    range: source_index
                        .and_then(|index| index.source_site_for_expression(&analysis.callable, expression.id).map(|site| site.range))
                        .unwrap_or(expression.range),
                    fact: FormalFactRef::Expression {
                        callable: analysis.callable.clone(),
                        expression: expression.id,
                    },
                    status: expression_status(&expression.status),
                    causal_invalidity: expression.causal_invalidity,
                    contract: None,
                });
            }
            for binding in analysis.bindings.values() {
                let attached_range = binding_ranges
                    .and_then(|callables| callables.get(&analysis.callable))
                    .and_then(|bindings| bindings.get(&binding.binding))
                    .copied()
                    .or_else(|| source_index.and_then(|index| index.source_site_for_binding(&analysis.callable, binding.binding).map(|site| site.range)));
                if let Some(range) = attached_range {
                    binding_attachments.insert((analysis.callable.clone(), binding.binding), range);
                }
                sites.push(FormalFactSite {
                    module: module.clone(),
                    range: attached_range.unwrap_or(binding.range),
                    fact: FormalFactRef::Binding {
                        callable: analysis.callable.clone(),
                        binding: binding.binding,
                    },
                    status: binding_status(binding),
                    causal_invalidity: binding.causal_invalidity,
                    contract: binding.contract.as_ref().map(|contract| FormalContractRelation {
                        ty: contract.ty,
                        origin: contract.origin,
                        consistency: binding.consistency.clone(),
                    }),
                });
            }
        }
        sites.sort_unstable_by(compare_formal_fact_sites);
        ModuleFormalProjection::new_with_binding_attachments(sites, binding_attachments)
    }

    /// Builds projection from immutable callable products while taking source
    /// ranges from the current source index. Reused semantic products may
    /// retain historical ranges; presentation must not publish those stale
    /// positions.
    pub fn from_callable_analyses_with_source_index(analyses: &HashMap<CallableId, Arc<CallableAnalysis>>, source_index: Option<&SourceSemanticIndex>) -> Self {
        Self::from_callable_analyses_with_attachments(analyses, source_index, None)
    }

    /// Builds a source-free formal projection from owner-qualified portable
    /// binding declaration ranges carried by a semantic artifact.
    pub fn from_callable_analyses_with_binding_ranges(
        analyses: &HashMap<CallableId, Arc<CallableAnalysis>>,
        binding_ranges: &BTreeMap<CallableId, BTreeMap<BindingId, SourceRange>>,
    ) -> Self {
        Self::from_callable_analyses_with_attachments(analyses, None, Some(binding_ranges))
    }

    /// Builds a source-free formal projection from an imported callable table
    /// and owner-qualified declaration ranges carried by the artifact.
    pub fn from_callable_analysis_table_with_binding_ranges(
        analyses: &crate::checker::CallableAnalysisTable,
        binding_ranges: &BTreeMap<CallableId, BTreeMap<BindingId, SourceRange>>,
    ) -> Self {
        let mut callables_by_module: BTreeMap<ModuleId, Vec<CallableId>> = BTreeMap::new();
        for callable in analyses.keys() {
            callables_by_module.entry(callable.module().clone()).or_default().push(callable.clone());
        }
        let mut modules = im::OrdMap::new();
        for (module, mut callable_ids) in callables_by_module {
            callable_ids.sort();
            let mut sites = Vec::new();
            let mut binding_attachments = BTreeMap::new();
            for callable in callable_ids {
                let Some(analysis) = analyses.get(&callable) else { continue };
                sites.push(FormalFactSite {
                    module: module.clone(),
                    range: analysis.body_range,
                    fact: FormalFactRef::Callable(analysis.callable.clone()),
                    status: callable_status(analysis.status),
                    causal_invalidity: callable_causal_invalidity(analysis),
                    contract: None,
                });
                for expression in analysis.expressions.values() {
                    sites.push(FormalFactSite {
                        module: module.clone(),
                        range: expression.range,
                        fact: FormalFactRef::Expression {
                            callable: analysis.callable.clone(),
                            expression: expression.id,
                        },
                        status: expression_status(&expression.status),
                        causal_invalidity: expression.causal_invalidity,
                        contract: None,
                    });
                }
                for binding in analysis.bindings.values() {
                    let attached_range = binding_ranges.get(&analysis.callable).and_then(|ranges| ranges.get(&binding.binding)).copied();
                    if let Some(range) = attached_range {
                        binding_attachments.insert((analysis.callable.clone(), binding.binding), range);
                    }
                    sites.push(FormalFactSite {
                        module: module.clone(),
                        range: attached_range.unwrap_or(binding.range),
                        fact: FormalFactRef::Binding {
                            callable: analysis.callable.clone(),
                            binding: binding.binding,
                        },
                        status: binding_status(binding),
                        causal_invalidity: binding.causal_invalidity,
                        contract: binding.contract.as_ref().map(|contract| FormalContractRelation {
                            ty: contract.ty,
                            origin: contract.origin,
                            consistency: binding.consistency.clone(),
                        }),
                    });
                }
            }
            sites.sort_unstable_by(compare_formal_fact_sites);
            modules.insert(
                module,
                Arc::new(ModuleFormalProjection::new_with_binding_attachments(sites, binding_attachments)),
            );
        }
        Self {
            modules: Some(modules),
            ..Self::default()
        }
    }

    /// Builds the immutable artifact projection using only precharged vectors
    /// and frozen sorted module authority.
    pub(crate) fn from_callable_analysis_table_with_binding_ranges_and_allocation_budget(
        analyses: &crate::checker::CallableAnalysisTable,
        binding_ranges: &crate::parent_map::ParentMap<CallableId, crate::ordered_map::OrderedMap<BindingId, SourceRange>>,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, FormalSemanticProjectionBuildError> {
        let mut callables = allocations
            .vector::<&CallableId>(analyses.len())
            .map_err(FormalSemanticProjectionBuildError::Allocation)?;
        callables.extend(analyses.keys());
        callables.sort_unstable_by(|left, right| left.module().cmp(right.module()).then_with(|| left.cmp(right)));

        let module_count = callables
            .windows(2)
            .filter(|pair| pair[0].module() != pair[1].module())
            .count()
            + usize::from(!callables.is_empty());
        let mut frozen_modules = allocations
            .vector::<(ModuleId, Arc<ModuleFormalProjection>)>(module_count)
            .map_err(FormalSemanticProjectionBuildError::Allocation)?;

        let mut start = 0;
        while start < callables.len() {
            let module = callables[start].module();
            let mut end = start + 1;
            while end < callables.len() && callables[end].module() == module {
                end += 1;
            }

            let callable_group = &callables[start..end];
            let site_count = callable_group
                .iter()
                .filter_map(|callable| analyses.get(callable))
                .map(|analysis| 1 + analysis.expressions.len() + analysis.bindings.len())
                .sum();
            let mut sites = allocations
                .vector::<FormalFactSite>(site_count)
                .map_err(FormalSemanticProjectionBuildError::Allocation)?;

            for callable in callable_group {
                let Some(analysis) = analyses.get(callable) else { continue };
                let owned_module = module
                    .clone_with_allocation_budget(allocations)
                    .map_err(FormalSemanticProjectionBuildError::Allocation)?;
                let owned_callable = analysis
                    .callable
                    .clone_with_allocation_budget(allocations)
                    .map_err(FormalSemanticProjectionBuildError::Allocation)?;
                sites.push(FormalFactSite {
                    module: owned_module,
                    range: analysis.body_range,
                    fact: FormalFactRef::Callable(owned_callable),
                    status: callable_status(analysis.status),
                    causal_invalidity: callable_causal_invalidity(analysis),
                    contract: None,
                });

                for expression in analysis.expressions.values() {
                    let owned_module = module
                        .clone_with_allocation_budget(allocations)
                        .map_err(FormalSemanticProjectionBuildError::Allocation)?;
                    let owned_callable = analysis
                        .callable
                        .clone_with_allocation_budget(allocations)
                        .map_err(FormalSemanticProjectionBuildError::Allocation)?;
                    sites.push(FormalFactSite {
                        module: owned_module,
                        range: expression.range,
                        fact: FormalFactRef::Expression {
                            callable: owned_callable,
                            expression: expression.id,
                        },
                        status: expression_status(&expression.status),
                        causal_invalidity: expression.causal_invalidity,
                        contract: None,
                    });
                }

                for binding in analysis.bindings.values() {
                    let attached_range = binding_ranges
                        .get(&analysis.callable)
                        .and_then(|ranges| ranges.get(&binding.binding))
                        .copied();
                    let owned_module = module
                        .clone_with_allocation_budget(allocations)
                        .map_err(FormalSemanticProjectionBuildError::Allocation)?;
                    let owned_callable = analysis
                        .callable
                        .clone_with_allocation_budget(allocations)
                        .map_err(FormalSemanticProjectionBuildError::Allocation)?;
                    sites.push(FormalFactSite {
                        module: owned_module,
                        range: attached_range.unwrap_or(binding.range),
                        fact: FormalFactRef::Binding {
                            callable: owned_callable,
                            binding: binding.binding,
                        },
                        status: binding_status(binding),
                        causal_invalidity: binding.causal_invalidity,
                        contract: binding
                            .contract
                            .as_ref()
                            .map(|contract| -> Result<_, FormalSemanticProjectionBuildError> {
                                Ok(FormalContractRelation {
                                    ty: contract.ty,
                                    origin: contract.origin,
                                    consistency: binding
                                        .consistency
                                        .clone_with_allocation_budget(allocations)
                                        .map_err(FormalSemanticProjectionBuildError::Allocation)?,
                                })
                            })
                            .transpose()?,
                    });
                }
            }

            sites.sort_unstable_by(compare_formal_fact_sites);
            let projection = ModuleFormalProjection::new_frozen_with_allocation_budget(sites, binding_ranges, allocations)
                .map_err(|error| match error {
                    ModuleFormalProjectionBuildError::Allocation(error) => {
                        FormalSemanticProjectionBuildError::Allocation(error)
                    }
                    ModuleFormalProjectionBuildError::DuplicateFact => FormalSemanticProjectionBuildError::DuplicateFact,
                })?;
            let projection = allocations
                .arc(projection)
                .map_err(FormalSemanticProjectionBuildError::Allocation)?;
            let module = module
                .clone_with_allocation_budget(allocations)
                .map_err(FormalSemanticProjectionBuildError::Allocation)?;
            frozen_modules.push((module, projection));
            start = end;
        }

        let frozen_modules = BudgetedSortedMap::from_sorted_entries(frozen_modules)
            .map_err(|_| FormalSemanticProjectionBuildError::DuplicateModule)?;
        let frozen_modules = allocations
            .arc(frozen_modules)
            .map_err(FormalSemanticProjectionBuildError::Allocation)?;
        Ok(Self {
            modules: None,
            frozen_modules: Some(frozen_modules),
            retired_modules: None,
        })
    }

    fn from_callable_analyses_with_attachments(
        analyses: &HashMap<CallableId, Arc<CallableAnalysis>>,
        source_index: Option<&SourceSemanticIndex>,
        binding_ranges: Option<&BTreeMap<CallableId, BTreeMap<BindingId, SourceRange>>>,
    ) -> Self {
        let mut callables_by_module: BTreeMap<ModuleId, Vec<CallableId>> = BTreeMap::new();
        for callable in analyses.keys() {
            callables_by_module.entry(callable.module().clone()).or_default().push(callable.clone());
        }
        if let Some(source_index) = source_index {
            for (mod_id, _) in source_index.modules() {
                callables_by_module.entry(mod_id.clone()).or_default();
            }
        }
        let mut modules = im::OrdMap::new();
        for (module, mut callable_ids) in callables_by_module {
            callable_ids.sort();
            let projection = Self::build_module_projection_with_binding_ranges(&module, callable_ids, analyses, source_index, binding_ranges);
            modules.insert(module, Arc::new(projection));
        }
        Self {
            modules: Some(modules),
            ..Self::default()
        }
    }

    /// Replaces or adds one module's formal projection shard.
    pub fn replace_module(&mut self, module: ModuleId, projection: Arc<ModuleFormalProjection>) {
        if let Some(retired) = &mut self.retired_modules {
            retired.remove(&module);
        }
        self.modules.get_or_insert_with(im::OrdMap::new).insert(module, projection);
    }

    /// Retires one module's formal projection shard.
    pub fn retire_module(&mut self, module: &ModuleId) {
        if let Some(modules) = &mut self.modules {
            modules.remove(module);
        }
        if self.frozen_modules.as_ref().is_some_and(|modules| modules.contains_key(module)) {
            self.retired_modules.get_or_insert_with(im::OrdSet::new).insert(module.clone());
        }
    }

    /// Returns one module's formal projection shard.
    pub fn module(&self, module: &ModuleId) -> Option<&ModuleFormalProjection> {
        self.modules
            .as_ref()
            .and_then(|modules| modules.get(module))
            .map(AsRef::as_ref)
            .or_else(|| {
                if self.retired_modules.as_ref().is_some_and(|retired| retired.contains(module)) {
                    None
                } else {
                    self.frozen_modules.as_ref()?.get(module).map(AsRef::as_ref)
                }
            })
    }

    /// Returns one module's formal projection shard Arc.
    pub fn module_arc(&self, module: &ModuleId) -> Option<Arc<ModuleFormalProjection>> {
        self.modules
            .as_ref()
            .and_then(|modules| modules.get(module))
            .cloned()
            .or_else(|| {
                if self.retired_modules.as_ref().is_some_and(|retired| retired.contains(module)) {
                    None
                } else {
                    self.frozen_modules.as_ref()?.get(module).cloned()
                }
            })
    }

    /// Returns iterator over module formal projections.
    pub fn modules(&self) -> impl Iterator<Item = (&ModuleId, &Arc<ModuleFormalProjection>)> {
        let source = self.modules.as_ref().into_iter().flat_map(|modules| modules.iter());
        let frozen = self
            .frozen_modules
            .iter()
            .flat_map(|modules| modules.iter())
            .filter(|(module, _)| {
                !self.modules.as_ref().is_some_and(|m| m.contains_key(module))
                    && !self.retired_modules.as_ref().is_some_and(|r| r.contains(module))
            });
        source.chain(frozen)
    }

    /// Returns the number of module projection shards without traversing them.
    pub fn module_count(&self) -> usize {
        self.modules.as_ref().map_or(0, |modules| modules.len())
            + self
                .frozen_modules
                .as_ref()
                .map_or(0, |modules| {
                    modules
                        .iter()
                        .filter(|(module, _)| {
                            !self.modules.as_ref().is_some_and(|m| m.contains_key(module))
                                && !self.retired_modules.as_ref().is_some_and(|r| r.contains(module))
                        })
                        .count()
                })
    }

    /// Returns one formal site by canonical fact identity.
    pub fn get(&self, fact: &FormalFactRef) -> Option<&FormalFactSite> {
        let module = fact.module();
        self.module(module)?.get(fact)
    }

    /// Returns an exact source-owned formal binding attachment by owner.
    pub fn binding_attachment_range(&self, callable: &CallableId, binding: BindingId) -> Option<SourceRange> {
        self.module(callable.module())?.binding_attachment_range(callable, binding)
    }

    /// Returns most-specific formal site at a source position.
    pub fn fact_at(&self, module: &ModuleId, offset: usize) -> Option<&FormalFactSite> {
        self.module(module)?.fact_at(offset)
    }

    /// Number of machine-readable formal site records across all modules.
    pub fn len(&self) -> usize {
        self.modules().map(|(_, module)| module.sites.len()).sum()
    }

    /// Whether no formal source facts are published.
    pub fn is_empty(&self) -> bool {
        self.module_count() == 0
    }
}

fn expression_status(status: &AnalysisStatus) -> FormalFactStatus {
    match status {
        AnalysisStatus::Ready => FormalFactStatus::Ready,
        AnalysisStatus::Invalid(_) => FormalFactStatus::Invalid,
        AnalysisStatus::Suppressed(_) | AnalysisStatus::Blocked(_) => FormalFactStatus::Blocked,
        AnalysisStatus::DynamicBoundary(_) => FormalFactStatus::Dynamic,
        AnalysisStatus::Cancelled => FormalFactStatus::Cancelled,
        AnalysisStatus::BudgetExceeded(_) => FormalFactStatus::BudgetExceeded,
        AnalysisStatus::InternalFailure(_) => FormalFactStatus::InternalFailure,
    }
}

fn callable_status(status: CallableAnalysisStatus) -> FormalFactStatus {
    match status {
        CallableAnalysisStatus::Complete => FormalFactStatus::Ready,
        CallableAnalysisStatus::Partial => FormalFactStatus::Partial,
        CallableAnalysisStatus::Blocked => FormalFactStatus::Blocked,
        CallableAnalysisStatus::Cancelled => FormalFactStatus::Cancelled,
        CallableAnalysisStatus::BudgetExceeded => FormalFactStatus::BudgetExceeded,
        CallableAnalysisStatus::InternalFailure(_) => FormalFactStatus::InternalFailure,
    }
}

fn binding_status(binding: &crate::checker::BindingState) -> FormalFactStatus {
    match binding.causal_invalidity {
        CausalInvalidity::One(_) => FormalFactStatus::Invalid,
        CausalInvalidity::Multiple => FormalFactStatus::InvalidMultiple,
        CausalInvalidity::Clean => match &binding.current {
            TypeKnowledge::Known(_) => FormalFactStatus::Ready,
            TypeKnowledge::Unknown(_) => FormalFactStatus::Unknown,
            TypeKnowledge::Dynamic(_) => FormalFactStatus::Dynamic,
        },
    }
}

fn callable_causal_invalidity(analysis: &CallableAnalysis) -> CausalInvalidity {
    analysis
        .expressions
        .values()
        .map(|expression| expression.causal_invalidity)
        .chain(analysis.bindings.values().map(|binding| binding.causal_invalidity))
        .fold(CausalInvalidity::Clean, CausalInvalidity::join)
}

fn formal_fact_priority(fact: &FormalFactRef) -> u8 {
    match fact {
        FormalFactRef::Expression { .. } => 0,
        FormalFactRef::Binding { .. } => 1,
        FormalFactRef::Callable(_) => 2,
    }
}

impl SemanticPresentationIndex {
    /// Projects expression, binding, and callable status products from one analysis.
    pub fn from_callable_analysis(module: ModuleId, analysis: &CallableAnalysis, presenter: &TypePresenter<'_>) -> Self {
        let mut index = Self::default();
        index.insert_callable(module, analysis, presenter);
        index
    }

    /// Adds one callable analysis to this projection builder.
    pub fn insert_callable(&mut self, module: ModuleId, analysis: &CallableAnalysis, presenter: &TypePresenter<'_>) {
        let callable_site = FormalSiteId::Callable(analysis.callable.clone());
        self.sites.insert(
            callable_site.clone(),
            FormalTypeSite {
                module: module.clone(),
                range: analysis.body_range,
                site: callable_site,
                presentation: presenter.present_callable_status(analysis.status),
            },
        );
        self.insert_expressions(module.clone(), &analysis.callable, &analysis.expressions, presenter);
        self.insert_bindings(module, &analysis.callable, &analysis.bindings, presenter);
    }

    /// Returns one projected site by canonical semantic identity.
    pub fn get(&self, site: &FormalSiteId) -> Option<&FormalTypeSite> {
        self.sites.get(site)
    }

    /// Returns exact formal binding site by module and exact token range.
    pub fn get_binding_site(&self, module: &ModuleId, range: &SourceRange) -> Option<&FormalTypeSite> {
        let site_id = self.binding_sites.get(&(module.clone(), *range))?;
        self.sites.get(site_id)
    }

    /// Finds the most specific (smallest containing) expression site at the given source offset.
    pub fn find_expression_at(&self, module: &ModuleId, offset: usize) -> Option<&FormalTypeSite> {
        let list = self.expression_sites.get(module)?;
        let mut best: Option<&FormalTypeSite> = None;
        for site in list {
            if site.range.contains(offset) {
                match best {
                    None => best = Some(site),
                    Some(curr) => {
                        if site.range.len() < curr.range.len() || (site.range.len() == curr.range.len() && site.site < curr.site) {
                            best = Some(site);
                        }
                    }
                }
            }
        }
        best
    }

    /// Returns number of projected formal sites.
    pub fn len(&self) -> usize {
        self.sites.len()
    }

    /// Returns whether the projection contains no sites.
    pub fn is_empty(&self) -> bool {
        self.sites.is_empty()
    }

    fn insert_expressions(&mut self, module: ModuleId, callable: &CallableId, expressions: &ExpressionAnalysisIndex, presenter: &TypePresenter<'_>) {
        for expression in expressions.values() {
            let site = FormalSiteId::Expression {
                callable: callable.clone(),
                expression: expression.id,
            };
            let formal_site = FormalTypeSite {
                module: module.clone(),
                range: expression.range,
                site: site.clone(),
                presentation: expression_presentation(expression, presenter),
            };
            self.sites.insert(site, formal_site.clone());
            self.expression_sites.entry(module.clone()).or_default().push(formal_site);
        }
    }

    fn insert_bindings(
        &mut self,
        module: ModuleId,
        callable: &CallableId,
        bindings: &crate::checker::analysis::BindingAnalysisIndex,
        presenter: &TypePresenter<'_>,
    ) {
        for state in bindings.values() {
            let site = FormalSiteId::Binding {
                callable: callable.clone(),
                binding: state.binding,
            };
            let formal_site = FormalTypeSite {
                module: module.clone(),
                range: state.range,
                site: site.clone(),
                presentation: presenter.present_knowledge(&state.current),
            };
            self.sites.insert(site.clone(), formal_site);
            self.binding_sites.insert((module.clone(), state.range), site);
        }
    }
}

fn expression_presentation(expression: &ExpressionAnalysis, presenter: &TypePresenter<'_>) -> FormalPresentation {
    presenter.present_expression(expression)
}
