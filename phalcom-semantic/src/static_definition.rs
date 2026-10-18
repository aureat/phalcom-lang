//! Semantic static definitions, signatures, and alias projection.
//!
//! (LANG007.C4.P2 Task 3)

use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic, SemanticSourceSpan};
use crate::identity::{BindingId, DeclarationId, ExpressionId, ModuleId, VariantId};
use crate::member_projection::MemberProjection;
use crate::static_domain::{StaticValueDomain, resolve_static_domain};
use crate::type_alias::TypeAliasInfo;
use crate::types::annotation::{
    GenericBinderSite, TypeFormationOutcome, TypeFormationSite, TypeResolver, lower_scoped_type_alias_form, resolve_generic_signature, resolve_type_form,
};
use crate::types::id::{KindId, TypeParameterId};
use crate::types::parameter::{GenericSignature, TypeParameterOwner};
use crate::types::store::{TypeData, TypeStore};
use phalcom_ast::ast::{Expr, GenericConstraintSyntax, LexicalBlock, StaticDefinitionBody, TypeAnnotation, TypeDefinitionDef, WhereClauseSyntax};
use phalcom_common::range::SourceRange;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// A parameter of a static definition with its semantic static domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticParameter {
    pub name: String,
    pub domain: StaticValueDomain,
    pub mode: crate::types::parameter::GenericParameterMode,
    pub index: u32,
    /// Allocated only when `domain` is in the kinded lane (`StaticValueDomain::Kind`).
    /// Ordinary value parameters (Int, Bool, etc.) have `None`.
    pub type_parameter_id: Option<TypeParameterId>,
    pub range: SourceRange,
}

/// The checked signature of a static definition.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticDefinitionSignature {
    pub parameters: Box<[StaticParameter]>,
    pub result_domain: StaticValueDomain,
    /// C3 kinded generic signature bridging kinded parameters only.
    pub generic_signature: Option<GenericSignature>,
    pub where_clause: Option<WhereClauseSyntax>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StaticBinaryOp {
    Add, Sub, Mul, Div, Mod, BitAnd, BitOr, BitXor, Shl, Shr, Eq, NotEq, Less, LessEq, Greater, GreaterEq, And, Or,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StaticUnaryOp {
    Neg, Not, BitNot,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OperationalTarget {
    Declaration(DeclarationId),
    DataConstructor(DeclarationId),
    VariantConstructor(VariantId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationalNode {
    Constant(crate::static_value::StaticValueRef),
    ReadParameter(u32),
    ReadLocal(u32),
    ReadStaticConstant(phalcom_modules::linker::SymbolId),
    WriteLocal(u32, Box<OperationalNode>),
    TypeForm(crate::types::id::TypeId),
    Binary { op: StaticBinaryOp, left: Box<OperationalNode>, right: Box<OperationalNode> },
    Unary { op: StaticUnaryOp, operand: Box<OperationalNode> },
    StaticApplication { application_ordinal: u32, arguments: Vec<OperationalNode> },
    StaticConstruction { target: OperationalTarget, result_type: crate::types::id::TypeId, arguments: Vec<OperationalNode> },
    MemberProjection { target: Box<OperationalNode>, member: Box<str> },
    If { condition: Box<OperationalNode>, then_branch: Box<OperationalNode>, else_branch: Option<Box<OperationalNode>> },
    Sequence { result_type: crate::types::id::TypeId, elements: Vec<OperationalNode> },
    Tuple { elements: Vec<OperationalNode> },
    Block { local_count: u32, statements: Vec<OperationalNode>, result: Option<Box<OperationalNode>> },
    Return(Option<Box<OperationalNode>>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanonicalCheckedStaticApplication {
    pub id: StaticApplicationId,
    pub source_range: SourceRange,
    pub target: DeclarationId,
    pub signature: StaticDefinitionSignature,
    pub arguments: Box<[CanonicalStaticApplicationArgument]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalStaticApplicationArgument {
    KindedTerm { parameter_index: u32, term: crate::types::id::TypeId },
    KindedExpression { parameter_index: u32, application_ordinal: u32 },
    Value { parameter_index: u32 },
}

/// Source-independent executable owner derived from one checked static body.
/// The body-local application rows remain the authority for target and checked
/// signature; operational nodes carry only their ordinal within this owner.
#[derive(Clone, Debug, PartialEq)]
pub struct CanonicalStaticProgram {
    pub owner: StaticBodyId,
    pub parameter_count: u32,
    pub local_count: u32,
    pub root: OperationalNode,
    pub applications: Arc<[CanonicalCheckedStaticApplication]>,
    pub kinded_parameters: Arc<[u32]>,
    pub static_constant_reads: Arc<[phalcom_modules::linker::SymbolId]>,
    pub dependencies: Arc<[crate::checker::analysis::SemanticDependency]>,
    pub result: StaticValueDomain,
    pub input_fingerprint: crate::db::key::InputFingerprint,
    pub module_binding_fingerprint: Option<crate::db::key::ProductFingerprint>,
}

/// Compatibility name for the operational root carried by existing artifact
/// codec helpers. The canonical program is the owner, not a detached template.
pub type OperationalTemplate = CanonicalStaticProgram;

/// Body representation of a static definition.
#[derive(Clone, Debug)]
pub enum StaticDefinitionBodyRepr {
    /// Legacy/direct type form body.
    TypeForm(TypeAnnotation),
    /// Ordinary expression body.
    Expression(Expr),
    /// Lexical block body.
    Block(LexicalBlock),
    /// Source-independent operational template for persistent compiler artifacts.
    Operational(std::sync::Arc<OperationalTemplate>),
}

impl StaticDefinitionBodyRepr {
    pub fn range(&self) -> SourceRange {
        match self {
            Self::TypeForm(ann) => ann.range,
            Self::Expression(expr) => expr.range(),
            Self::Block(block) => block.range,
            Self::Operational(_) => phalcom_common::range::SourceRange::default(),
        }
    }
}

/// Checked semantic static definition product.
#[derive(Clone, Debug)]
pub struct StaticDefinition {
    pub id: DeclarationId,
    pub signature: StaticDefinitionSignature,
    pub body: StaticDefinitionBodyRepr,
    pub source_range: SourceRange,
}

/// Stable owner identity for checked static-body facts.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticBodyId(pub DeclarationId);

impl StaticBodyId {
    pub fn clone_with_allocation_budget(
        &self,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, phalcom_type_meta::allocation::AllocationError> {
        Ok(Self(self.0.clone_with_allocation_budget(allocations)?))
    }
}

/// Query-local identity for one nested static-definition application in a
/// checked static body. The ordinal follows the body's structural walk; source
/// ranges remain provenance and are not used as target identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticApplicationId {
    pub body: StaticBodyId,
    pub ordinal: u32,
}

impl StaticApplicationId {
    pub fn clone_with_allocation_budget(
        &self,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, phalcom_type_meta::allocation::AllocationError> {
        Ok(Self {
            body: self.body.clone_with_allocation_budget(allocations)?,
            ordinal: self.ordinal,
        })
    }
}

/// One already-checked argument aligned with its target parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedStaticApplicationArgument {
    /// A kinded argument resolved through the canonical TypeStore.
    Kinded { parameter_index: u32, term: crate::types::id::TypeId },
    /// A kinded argument whose TypeForm is itself a checked static application.
    KindedExpression { parameter_index: u32, expression: ExpressionId },
    /// An ordinary value argument whose expression has ordinary checker facts.
    Value { parameter_index: u32, expression: ExpressionId },
}

/// Exact checked target and signature for one nested static application.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckedStaticApplication {
    pub id: StaticApplicationId,
    /// Enclosing checked TypeForm expression.
    pub expression: ExpressionId,
    /// Application syntax site, retained only as provenance for diagnostics and
    /// the evaluator's AST continuation lookup.
    pub source_range: SourceRange,
    pub target: DeclarationId,
    pub signature: StaticDefinitionSignature,
    /// Arguments are retained in target parameter order.
    pub arguments: Box<[CheckedStaticApplicationArgument]>,
}

/// Stable payload identity selected by ordinary constructor checking.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedStaticPayloadId {
    DataComponent(crate::identity::DataComponentId),
    VariantField(crate::identity::VariantFieldId),
}

/// One source argument paired with the canonical payload it initializes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedStaticConstructionArgument {
    pub expression: ExpressionId,
    pub payload: CheckedStaticPayloadId,
}

/// Exact ordinary-checker constructor selection for one static-body expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedStaticConstruction {
    pub expression: ExpressionId,
    pub target: crate::identity::InvocationTargetId,
    pub result_type: crate::types::id::TypeId,
    pub arguments: Box<[CheckedStaticConstructionArgument]>,
}

/// Canonical member-projection request attached to one checked static-body site.
pub type CheckedStaticMemberProjectionIndex = BTreeMap<ExpressionId, MemberProjection>;

/// Ordinary checker products and evaluator layout for one static definition body.
#[derive(Clone, Debug)]
pub struct CheckedStaticBody {
    pub owner: StaticBodyId,
    pub root: StaticDefinitionBodyRepr,
    pub expressions: Arc<crate::checker::analysis::ExpressionAnalysisIndex>,
    pub bindings: Arc<crate::checker::analysis::BindingAnalysisIndex>,
    pub static_operations: Arc<crate::static_intrinsic::StaticOperationResolutionIndex>,
    /// Exact target/signature/domain facts for nested static applications.
    pub static_applications: Arc<[CheckedStaticApplication]>,
    /// Exact checked constructor selection and payload mapping by expression.
    pub static_constructions: Arc<BTreeMap<ExpressionId, CheckedStaticConstruction>>,
    /// Canonical structural member-projection requests by checked expression site.
    pub member_projections: Arc<CheckedStaticMemberProjectionIndex>,
    /// Ordinary associated-resolution facts for expressions owned by this body.
    pub associated_resolutions: Arc<crate::checker::associated::AssociatedResolutionIndex>,
    pub sequence_literals: Arc<crate::checker::analysis::SequenceLiteralResolutionIndex>,
    pub match_resolutions: Arc<crate::match_semantics::MatchResolutionIndex>,
    pub pattern_site_resolutions: Arc<crate::match_semantics::PatternSiteResolutionIndex>,
    pub static_iteration_lanes: Arc<crate::checker::static_iteration::StaticIterationLaneResolutionIndex>,
    pub proof_predicates: Arc<crate::checker::proof_predicate::ProofPredicateResolutionIndex>,
    pub parameter_bindings: Arc<BTreeMap<u32, BindingId>>,
    pub kinded_parameters: Arc<BTreeMap<u32, TypeParameterId>>,
    pub local_slots: Arc<BTreeMap<BindingId, u32>>,
    /// Body-local IDs seeded from stable module constant identities.
    pub static_constant_bindings: Arc<BTreeMap<BindingId, phalcom_modules::linker::SymbolId>>,
    /// Stable constants read by this checked body, in identity order.
    pub static_constant_reads: Arc<[phalcom_modules::linker::SymbolId]>,
    /// Content fingerprint of the module binding facts used to seed this body.
    pub module_binding_fingerprint: Option<crate::db::key::ProductFingerprint>,
    /// Fingerprint of definition source/signature plus phase-readable inputs.
    pub input_fingerprint: crate::db::key::InputFingerprint,
    pub dependencies: Arc<[crate::checker::analysis::SemanticDependency]>,
    pub result: StaticValueDomain,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}

/// Product wrapper published by static definition queries.
#[derive(Clone, Debug)]
pub struct StaticDefinitionProduct {
    pub definition: Arc<StaticDefinition>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
    /// Optimized projection for Type-valued definitions compatible with C3 alias machinery.
    pub alias_info: Option<Arc<TypeAliasInfo>>,
    /// Source-independent executable authority derived from the checked body.
    /// Imported products always carry this and have no source checked body.
    pub canonical_program: Option<Arc<CanonicalStaticProgram>>,
    /// Composed snapshot view of the distinct cached ordinary-checker body.
    pub checked_body: Option<Arc<CheckedStaticBody>>,
}

/// Table storing static definitions by declaration ID and module.
#[derive(Clone, Debug, Default)]
pub struct StaticDefinitionTable {
    parent: Option<Arc<Self>>,
    definitions: crate::ordered_map::OrderedMap<DeclarationId, Arc<StaticDefinitionProduct>>,
    module_definitions: BTreeMap<ModuleId, BTreeSet<DeclarationId>>,
}

impl StaticDefinitionTable {
    pub(crate) fn from_frozen_entries_with_allocation_budget(
        entries: Vec<(DeclarationId, Arc<StaticDefinitionProduct>)>,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, crate::ordered_map::FrozenOrderedMapError> {
        Ok(Self {
            definitions: crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(entries, allocations)?,
            ..Self::default()
        })
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// Attaches the immutable canonical definitions without copying their entries.
    pub fn with_parent(parent: Arc<Self>) -> Self {
        Self {
            parent: Some(parent),
            ..Self::default()
        }
    }

    /// Number of workspace-owned definitions, excluding the canonical parent.
    pub fn local_len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.local_len() == 0 && self.parent.as_ref().is_none_or(|parent| parent.is_empty())
    }

    pub fn insert(&mut self, product: Arc<StaticDefinitionProduct>) {
        let decl = product.definition.id.clone();
        assert!(
            !self.parent.as_ref().is_some_and(|parent| parent.contains_key(&decl)),
            "cannot overwrite a canonical static definition"
        );
        self.module_definitions.entry(decl.module.clone()).or_default().insert(decl.clone());
        self.definitions.insert(decl, product);
    }

    pub fn remove_module(&mut self, module: &ModuleId) {
        if let Some(declarations) = self.module_definitions.remove(module) {
            for declaration in declarations {
                self.definitions.remove(&declaration);
            }
        }
    }

    pub fn remove(&mut self, declaration: &DeclarationId) {
        if self.definitions.remove(declaration).is_some() {
            if let Some(decls) = self.module_definitions.get_mut(&declaration.module) {
                decls.remove(declaration);
                if decls.is_empty() {
                    self.module_definitions.remove(&declaration.module);
                }
            }
        }
    }

    pub fn get(&self, declaration: &DeclarationId) -> Option<&Arc<StaticDefinitionProduct>> {
        self.definitions.get(declaration).or_else(|| self.parent.as_ref().and_then(|parent| parent.get(declaration)))
    }

    pub fn contains_key(&self, declaration: &DeclarationId) -> bool {
        self.get(declaration).is_some()
    }

    /// Iterates canonical and local definitions in stable parent-then-local order.
    pub fn iter(&self) -> Box<dyn Iterator<Item = (&DeclarationId, &Arc<StaticDefinitionProduct>)> + '_> {
        match &self.parent {
            Some(parent) => Box::new(parent.iter().chain(self.definitions.iter())),
            None => Box::new(self.definitions.iter()),
        }
    }
}

/// Resolves a static definition from its AST declaration into a semantic product,
/// allocating TypeParameterIds only for kinded parameters and creating a C3 TypeAliasInfo
/// projection if compatible.
pub fn resolve_static_definition(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    def: &TypeDefinitionDef,
    module_id: &ModuleId,
    dependencies: Box<[DeclarationId]>,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> StaticDefinitionProduct {
    let declaration_id = DeclarationId::new(module_id.clone(), def.name.clone().into());

    let mut parameters = Vec::with_capacity(def.parameters.len());
    let mut kinded_params = Vec::new();
    let mut kinded_indices = Vec::new();
    let mut value_param_names = BTreeSet::new();

    for (i, param) in def.parameters.iter().enumerate() {
        let domain = if let Some(domain_syntax) = &param.domain {
            match resolve_static_domain(store, declarations, resolver, site, domain_syntax, diagnostics) {
                TypeFormationOutcome::Ready(domain) => domain,
                _ => StaticValueDomain::Kind(KindId::TYPE),
            }
        } else {
            StaticValueDomain::Kind(KindId::TYPE)
        };

        if domain.is_kind() {
            kinded_params.push(param.to_generic_parameter());
            kinded_indices.push(i);
        } else {
            value_param_names.insert(param.name.clone());
        }

        let mode = match param.mode {
            None => crate::types::parameter::GenericParameterMode::Single,
            Some(phalcom_ast::ast::ExpansionMode::Positional) => crate::types::parameter::GenericParameterMode::PositionalPack,
            Some(phalcom_ast::ast::ExpansionMode::Labeled) => crate::types::parameter::GenericParameterMode::LabeledPack,
            Some(phalcom_ast::ast::ExpansionMode::Complete) => crate::types::parameter::GenericParameterMode::CompletePack,
        };

        parameters.push(StaticParameter {
            name: param.name.clone(),
            domain,
            mode,
            index: i as u32,
            type_parameter_id: None,
            range: param.range,
        });
    }

    // Diagnose any value parameters appearing in type-level where constraints
    let sanitized_where_clause = if let Some(where_clause) = &def.where_clause {
        let mut valid_constraints = Vec::new();
        let mut had_value_param = false;
        for constraint in &where_clause.constraints {
            let mut mentions_value_param = false;
            match constraint {
                GenericConstraintSyntax::Subtype { lower, upper, range }
                | GenericConstraintSyntax::Equivalent {
                    left: lower,
                    right: upper,
                    range,
                } => {
                    if let phalcom_ast::ast::TypeAnnotationExpr::Reference(r) = &lower.expr {
                        if value_param_names.contains(&r.root) {
                            mentions_value_param = true;
                            diagnostics.push(SemanticDiagnostic::error_in(
                                module_id.clone(),
                                DiagnosticCode::AnnotationUnsupported,
                                format!("value parameter `{}` cannot appear in type-level constraints", r.root),
                                *range,
                            ));
                        }
                    }
                    if let phalcom_ast::ast::TypeAnnotationExpr::Reference(r) = &upper.expr {
                        if value_param_names.contains(&r.root) {
                            mentions_value_param = true;
                            diagnostics.push(SemanticDiagnostic::error_in(
                                module_id.clone(),
                                DiagnosticCode::AnnotationUnsupported,
                                format!("value parameter `{}` cannot appear in type-level constraints", r.root),
                                *range,
                            ));
                        }
                    }
                }
                GenericConstraintSyntax::Conforms { subject, range, .. } => {
                    if let phalcom_ast::ast::TypeAnnotationExpr::Reference(r) = &subject.expr {
                        if value_param_names.contains(&r.root) {
                            mentions_value_param = true;
                            diagnostics.push(SemanticDiagnostic::error_in(
                                module_id.clone(),
                                DiagnosticCode::AnnotationUnsupported,
                                format!("value parameter `{}` cannot appear in trait conformance constraints", r.root),
                                *range,
                            ));
                        }
                    }
                }
                GenericConstraintSyntax::KindFact { subject, range, .. } => {
                    if value_param_names.contains(subject) {
                        mentions_value_param = true;
                        diagnostics.push(SemanticDiagnostic::error_in(
                            module_id.clone(),
                            DiagnosticCode::AnnotationUnsupported,
                            format!("value parameter `{}` cannot appear in kind fact constraints", subject),
                            *range,
                        ));
                    }
                }
                GenericConstraintSyntax::Invalid { .. } => {}
            }
            if mentions_value_param {
                had_value_param = true;
            } else {
                valid_constraints.push(constraint.clone());
            }
        }
        if had_value_param && valid_constraints.is_empty() {
            None
        } else if had_value_param {
            Some(WhereClauseSyntax {
                constraints: valid_constraints,
                range: where_clause.range,
            })
        } else {
            Some(where_clause.clone())
        }
    } else {
        None
    };

    let generic_signature = if kinded_params.is_empty() {
        None
    } else {
        let outcome = resolve_generic_signature(
            store,
            declarations,
            resolver,
            site,
            TypeParameterOwner::Declaration(declaration_id.clone()),
            GenericBinderSite::TypeAlias,
            &kinded_params,
            sanitized_where_clause.as_ref(),
            diagnostics,
        );
        match outcome {
            TypeFormationOutcome::Ready(sig) => {
                for (&idx, &param_id) in kinded_indices.iter().zip(sig.parameters.iter()) {
                    parameters[idx].type_parameter_id = Some(param_id);
                }
                Some(sig)
            }
            TypeFormationOutcome::Dynamic => {
                diagnostics.push(SemanticDiagnostic::error_in(
                    module_id.clone(),
                    DiagnosticCode::AnnotationUnsupported,
                    "generic signature depends on a dynamic type-form boundary",
                    def.range,
                ));
                None
            }
            _ => None,
        }
    };

    let mut result_domain = if let Some(domain_syntax) = &def.result_domain {
        match resolve_static_domain(store, declarations, resolver, site, domain_syntax, diagnostics) {
            TypeFormationOutcome::Ready(domain) => domain,
            _ => StaticValueDomain::Kind(KindId::TYPE),
        }
    } else {
        StaticValueDomain::Kind(KindId::TYPE)
    };

    let body_repr = match &def.body {
        StaticDefinitionBody::Expression(expr) => match &**expr {
            Expr::TypeForm(ann) => StaticDefinitionBodyRepr::TypeForm((**ann).clone()),
            _ => StaticDefinitionBodyRepr::Expression((**expr).clone()),
        },
        StaticDefinitionBody::Block(block) => StaticDefinitionBodyRepr::Block(block.clone()),
    };

    // Compatibility projection for transparent type aliases
    let explicit_result_kind = def.result_domain.is_some();
    let alias_info = if let StaticValueDomain::Kind(expected_kind) = result_domain {
        if let StaticDefinitionBodyRepr::TypeForm(ref ann) = body_repr {
            if kinded_params.len() == parameters.len() {
                let form_outcome = match generic_signature.as_ref() {
                    Some(sig) => lower_scoped_type_alias_form(store, declarations, resolver, site, sig, ann, diagnostics),
                    None => resolve_type_form(store, declarations, resolver, site, ann, diagnostics),
                };
                match form_outcome {
                    TypeFormationOutcome::Ready(form_type_id) => {
                        let actual_result_kind = match store.get(form_type_id) {
                            TypeData::Lambda(lambda_id) => store.arena().get_lambda(*lambda_id).result_kind,
                            _ => store.kind_of(form_type_id),
                        };
                        if explicit_result_kind && expected_kind != actual_result_kind {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                module_id.clone(),
                                DiagnosticCode::AnnotationUnresolved,
                                format!(
                                    "static definition body has kind {}, expected {}",
                                    store.format_kind(actual_result_kind),
                                    store.format_kind(expected_kind)
                                ),
                                def.body.range(),
                            ));
                            None
                        } else {
                            if !explicit_result_kind {
                                result_domain = StaticValueDomain::Kind(actual_result_kind);
                            }
                            let actual_kind = store.kind_of(form_type_id);
                            Some(Arc::new(TypeAliasInfo {
                                declaration: declaration_id.clone(),
                                kind: actual_kind,
                                kind_shape: store.format_kind(actual_kind).into_boxed_str(),
                                generic_signature: generic_signature.clone(),
                                form: form_type_id,
                                structural_form: store.format_type(form_type_id).into_boxed_str(),
                                dependencies,
                                source: SemanticSourceSpan::new(module_id.clone(), def.range),
                            }))
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let signature = StaticDefinitionSignature {
        parameters: parameters.into_boxed_slice(),
        result_domain,
        generic_signature,
        where_clause: def.where_clause.clone(),
    };

    let definition = Arc::new(StaticDefinition {
        id: declaration_id,
        signature,
        body: body_repr,
        source_range: def.range,
    });

    StaticDefinitionProduct {
        definition,
        diagnostics: Arc::from(diagnostics.as_slice()),
        alias_info,
        canonical_program: None,
        checked_body: None,
    }
}
