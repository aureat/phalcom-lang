//! Pure presentation projections over compiler-owned semantic products.

use crate::checker::{AnalysisStatus, CallableAnalysis, CallableAnalysisStatus, ExpressionAnalysis, ExpressionAnalysisIndex};
use crate::identity::{CallableId, ExpressionId, ModuleId};
use crate::types::evidence::TypeKnowledge;
use crate::types::id::TypeId;
use crate::types::store::TypeStore;
use phalcom_common::range::SourceRange;
use std::collections::BTreeMap;

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
            TypeKnowledge::Known(evidence) => FormalPresentation::Known(self.present_type(evidence.ty)),
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
        }
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
