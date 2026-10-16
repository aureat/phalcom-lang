//! Canonical static constant identity, checked bodies, and query products.
//!
//! (LANG007.C4.P2 Task 8)

use crate::checker::analysis::{BindingAnalysisIndex, ExpressionAnalysisIndex, SemanticDependency};
use crate::checker::associated::AssociatedResolutionIndex;
use crate::db::key::InputFingerprint;
use crate::diagnostic::SemanticDiagnostic;
use crate::identity::{BindingId, ExpressionId, ModuleId};
use crate::static_definition::{CheckedStaticApplication, CheckedStaticConstruction};
use crate::static_eval::StaticEvalOutcome;
use crate::static_intrinsic::StaticOperationResolutionIndex;
use crate::static_value::StaticValueRef;
use crate::types::id::TypeId;
use phalcom_ast::ast::Expr;
use phalcom_common::range::SourceRange;
use phalcom_modules::linker::SymbolId;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Stable identity for a module-owned compile-time constant.
///
/// Wraps the canonical `SymbolId(module, name)`. Never uses VM Symbol, compiler slot,
/// or source range as target identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticConstantId(pub SymbolId);

impl StaticConstantId {
    pub fn new(module: ModuleId, name: impl Into<Box<str>>) -> Self {
        Self(SymbolId { module, name: name.into() })
    }

    pub fn symbol(&self) -> &SymbolId {
        &self.0
    }

    pub fn module(&self) -> &ModuleId {
        &self.0.module
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }
}

impl std::fmt::Display for StaticConstantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.0.module, self.0.name)
    }
}

/// Ordinary checker products and evaluator layout for one static expression.
#[derive(Clone, Debug)]
pub struct CheckedStaticExpression {
    pub root: Expr,
    pub expressions: Arc<ExpressionAnalysisIndex>,
    pub bindings: Arc<BindingAnalysisIndex>,
    pub static_operations: Arc<StaticOperationResolutionIndex>,
    pub static_applications: Arc<[CheckedStaticApplication]>,
    pub static_constructions: Arc<BTreeMap<ExpressionId, CheckedStaticConstruction>>,
    pub associated_resolutions: Arc<AssociatedResolutionIndex>,
    pub sequence_literals: Arc<crate::checker::analysis::SequenceLiteralResolutionIndex>,
    pub match_resolutions: Arc<crate::match_semantics::MatchResolutionIndex>,
    pub pattern_site_resolutions: Arc<crate::match_semantics::PatternSiteResolutionIndex>,
    pub proof_predicates: Arc<crate::checker::proof_predicate::ProofPredicateResolutionIndex>,
    pub local_slots: Arc<BTreeMap<BindingId, u32>>,
    pub static_constant_bindings: Arc<BTreeMap<BindingId, SymbolId>>,
    pub static_constant_reads: Arc<[SymbolId]>,
    pub dependencies: Arc<[SemanticDependency]>,
    pub result_type: Option<TypeId>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}

/// Owner metadata and targeted input identity for one constant initializer.
/// Its checked body uses the same expression product consumed by static-key
/// projection.
#[derive(Clone, Debug)]
pub struct CheckedStaticConstantInitializer {
    pub id: StaticConstantId,
    pub declaration_range: SourceRange,
    pub initializer_range: SourceRange,
    pub expression: CheckedStaticExpression,
    /// Exact constant reads made while reducing computed Record keys in this
    /// constant's declared type. These reads are also included in
    /// `expression.static_constant_reads`; keeping their origin separately
    /// lets the owner diagnose a static-formation cycle without treating an
    /// ordinary initializer cycle as a computed-key cycle.
    pub record_key_static_constant_reads: Arc<[SymbolId]>,
    pub input_fingerprint: InputFingerprint,
}

impl std::ops::Deref for CheckedStaticConstantInitializer {
    type Target = CheckedStaticExpression;

    fn deref(&self) -> &Self::Target {
        &self.expression
    }
}

/// Product published by static constant queries in the incremental DB and snapshots.
#[derive(Clone, Debug)]
pub struct StaticConstantProduct {
    pub id: StaticConstantId,
    pub declaration_range: SourceRange,
    pub initializer_range: SourceRange,
    /// Source-backed checker products are retained only on source-built products.
    /// Artifact-loaded products carry the same reusable facts in `authority`.
    pub checked: Option<Arc<CheckedStaticConstantInitializer>>,
    /// Source-independent query and semantic authority for this constant.
    pub authority: StaticConstantAuthority,
    pub outcome: StaticEvalOutcome,
    /// Owns every node referenced by a published symbolic outcome.
    pub residuals: Option<Arc<crate::static_eval::StaticResidualStore>>,
    pub value: Option<StaticValueRef>,
    pub lkg_value: Option<StaticValueRef>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}

/// Persistent, source-independent facts required to reuse and consume a checked constant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticConstantAuthority {
    pub input_fingerprint: InputFingerprint,
    pub dependencies: Arc<[SemanticDependency]>,
    pub static_constant_reads: Arc<[SymbolId]>,
    pub record_key_static_constant_reads: Arc<[SymbolId]>,
    pub result_type: Option<TypeId>,
}

impl StaticConstantAuthority {
    /// Copies the persistent authority from the source check before publication.
    pub fn from_checked(checked: &CheckedStaticConstantInitializer) -> Self {
        Self {
            input_fingerprint: checked.input_fingerprint,
            dependencies: checked.dependencies.clone(),
            static_constant_reads: checked.static_constant_reads.clone(),
            record_key_static_constant_reads: checked.record_key_static_constant_reads.clone(),
            result_type: checked.result_type,
        }
    }
}

impl StaticConstantProduct {
    pub fn id(&self) -> &StaticConstantId {
        &self.id
    }

    pub fn symbol(&self) -> &SymbolId {
        &self.id.0
    }

    pub fn is_successful(&self) -> bool {
        matches!(self.outcome, StaticEvalOutcome::Reduced(_))
    }

    pub fn value(&self) -> Option<StaticValueRef> {
        self.value
    }

    /// Converts this product's reduced static value into a backend-facing materialization descriptor.
    pub fn materialization_descriptor(&self, store: &crate::static_value::StaticValueStore) -> Option<crate::materialization::MaterializationDescriptor> {
        let val_ref = self.value()?;
        crate::materialization::MaterializationDescriptor::from_static_value(store, val_ref)
    }
}

/// Table storing static constants by canonical identity and module.
#[derive(Clone, Debug, Default)]
pub struct StaticConstantTable {
    parent: Option<Arc<Self>>,
    constants: BTreeMap<StaticConstantId, Arc<StaticConstantProduct>>,
    by_symbol: BTreeMap<SymbolId, StaticConstantId>,
    module_constants: BTreeMap<ModuleId, BTreeSet<StaticConstantId>>,
}

impl StaticConstantTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Attaches an immutable canonical constant table without copying its entries.
    pub fn with_parent(parent: Arc<Self>) -> Self {
        Self {
            parent: Some(parent),
            ..Self::default()
        }
    }

    /// Number of workspace-owned constants, excluding the canonical parent.
    pub fn local_len(&self) -> usize {
        self.constants.len()
    }

    pub fn insert(&mut self, product: Arc<StaticConstantProduct>) {
        let id = product.id.clone();
        assert!(
            !self.parent.as_ref().is_some_and(|parent| parent.get(&id).is_some()),
            "cannot overwrite a canonical static constant"
        );
        self.module_constants.entry(id.0.module.clone()).or_default().insert(id.clone());
        self.by_symbol.insert(id.0.clone(), id.clone());
        self.constants.insert(id, product);
    }

    pub fn get(&self, id: &StaticConstantId) -> Option<&Arc<StaticConstantProduct>> {
        self.constants.get(id).or_else(|| self.parent.as_ref().and_then(|parent| parent.get(id)))
    }

    pub fn get_by_symbol(&self, symbol: &SymbolId) -> Option<&Arc<StaticConstantProduct>> {
        self.by_symbol
            .get(symbol)
            .and_then(|id| self.constants.get(id))
            .or_else(|| self.parent.as_ref().and_then(|parent| parent.get_by_symbol(symbol)))
    }

    /// Iterates canonical and workspace constants in stable parent-then-local order.
    pub fn iter(&self) -> Box<dyn Iterator<Item = (&StaticConstantId, &Arc<StaticConstantProduct>)> + '_> {
        match &self.parent {
            Some(parent) => Box::new(parent.iter().chain(self.constants.iter())),
            None => Box::new(self.constants.iter()),
        }
    }

    pub fn module_constants<'a>(&'a self, module: &ModuleId) -> Box<dyn Iterator<Item = &'a Arc<StaticConstantProduct>> + 'a> {
        let local = self.module_constants
            .get(module)
            .into_iter()
            .flat_map(move |ids| ids.iter().filter_map(|id| self.constants.get(id)));
        match &self.parent {
            Some(parent) => Box::new(parent.module_constants(module).chain(local)),
            None => Box::new(local),
        }
    }

    pub fn remove_module(&mut self, module: &ModuleId) {
        if let Some(ids) = self.module_constants.remove(module) {
            for id in ids {
                self.by_symbol.remove(&id.0);
                self.constants.remove(&id);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.constants.len() + self.parent.as_ref().map_or(0, |parent| parent.len())
    }

    pub fn is_empty(&self) -> bool {
        self.constants.is_empty() && self.parent.as_ref().is_none_or(|parent| parent.is_empty())
    }
}
