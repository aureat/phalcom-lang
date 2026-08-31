//! Pure presentation projections over compiler-owned semantic products.

use crate::advisory::ValueShape;
use crate::advisory::{AdvisoryFact, AdvisoryTargetResolution};
use crate::checker::causal::CausalInvalidity;
use crate::checker::{AnalysisStatus, BindingConsistency, CallableAnalysis, CallableAnalysisStatus, ExpressionAnalysis, ExpressionAnalysisIndex};
use crate::identity::{CallableId, ExpressionId, FieldId, ModuleId, SourceSiteId};
use crate::signature::CallableSemanticSignature;
use crate::source_index::SourceSemanticIndex;
use crate::source_index::interval::{RangeEntry, RangeIndex};
use crate::source_index::{CallableSourceInfo, SourceCallableKind};
use crate::types::evidence::TypeKnowledge;
use crate::types::id::TypeId;
use crate::types::store::TypeStore;
use phalcom_common::range::SourceRange;
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
                "#{{{}}}",
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

/// Machine-readable identity of one formal source fact.
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

/// Indexed machine-readable formal source projection.
#[derive(Clone, Debug, Default)]
pub struct FormalSemanticProjection {
    by_fact: BTreeMap<FormalFactRef, FormalFactSite>,
    by_module: BTreeMap<ModuleId, Arc<[FormalFactSite]>>,
    intervals: BTreeMap<ModuleId, RangeIndex<usize>>,
}

impl FormalSemanticProjection {
    /// Builds projection from immutable callable products without re-analysis.
    pub fn from_callable_analyses(analyses: &HashMap<CallableId, Arc<CallableAnalysis>>) -> Self {
        Self::from_callable_analyses_with_source_index(analyses, None)
    }

    /// Builds projection from immutable callable products while taking source
    /// ranges from the current source index. Reused semantic products may
    /// retain historical ranges; presentation must not publish those stale
    /// positions.
    pub fn from_callable_analyses_with_source_index(analyses: &HashMap<CallableId, Arc<CallableAnalysis>>, source_index: Option<&SourceSemanticIndex>) -> Self {
        let mut sites = Vec::new();
        let mut ordered = analyses.values().cloned().collect::<Vec<_>>();
        ordered.sort_by(|left, right| left.callable.cmp(&right.callable));
        for analysis in ordered {
            let module = analysis.callable.module().clone();
            let module_index = source_index.and_then(|index| index.module(&module));
            let callable_fact = FormalFactRef::Callable(analysis.callable.clone());
            sites.push(FormalFactSite {
                module: module.clone(),
                range: module_index
                    .and_then(|index| index.structure.callable_body_ranges.get(&analysis.callable).copied())
                    .unwrap_or(analysis.body_range),
                fact: callable_fact,
                status: callable_status(analysis.status),
                causal_invalidity: callable_causal_invalidity(&analysis),
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
                sites.push(FormalFactSite {
                    module: module.clone(),
                    range: source_index
                        .and_then(|index| index.source_site_for_binding(&analysis.callable, binding.binding).map(|site| site.range))
                        .unwrap_or(binding.range),
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
        sites.sort_by_key(|site| (site.module.clone(), site.range.start, site.range.len(), site.fact.clone()));
        let mut by_fact = BTreeMap::new();
        let mut grouped = BTreeMap::<ModuleId, Vec<FormalFactSite>>::new();
        for site in sites {
            by_fact.insert(site.fact.clone(), site.clone());
            grouped.entry(site.module.clone()).or_default().push(site);
        }
        let mut by_module = BTreeMap::new();
        let mut intervals = BTreeMap::new();
        for (module, module_sites) in grouped {
            let module_sites: Arc<[FormalFactSite]> = Arc::from(module_sites);
            let ranges = RangeIndex::new(
                module_sites
                    .iter()
                    .enumerate()
                    .map(|(index, site)| RangeEntry::new(site.range, index, formal_fact_priority(&site.fact))),
            );
            by_module.insert(module.clone(), module_sites);
            intervals.insert(module, ranges);
        }
        Self { by_fact, by_module, intervals }
    }

    /// Returns one formal site by canonical fact identity.
    pub fn get(&self, fact: &FormalFactRef) -> Option<&FormalFactSite> {
        self.by_fact.get(fact)
    }

    /// Returns most-specific formal site at a source position.
    pub fn fact_at(&self, module: &ModuleId, offset: usize) -> Option<&FormalFactSite> {
        let index = self.intervals.get(module)?.index_at(offset)?;
        self.by_module.get(module)?.get(index)
    }

    /// Number of machine-readable formal site records.
    pub fn len(&self) -> usize {
        self.by_fact.len()
    }

    /// Whether no formal source facts are published.
    pub fn is_empty(&self) -> bool {
        self.by_fact.is_empty()
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
