//! Interned, normalized semantic residuals for open static computation.
//!
//! (LANG007.C4.P7)

use crate::checker::proof_predicate::ProofPredicateGoal;
use crate::field_key::FieldKey;
use crate::identity::{DataComponentId, DataConstructorId, DeclarationId, ExpressionId, VariantFieldId, VariantId};
use crate::member_projection::MemberProjection;
use crate::static_definition::StaticBodyId;
use crate::static_domain::StaticValueDomain;
use crate::static_eval::application::{StaticSpecializationEnvironmentKey, StaticTrustedBranchContext};
use crate::static_value::StaticValueRef;
use crate::types::id::TypeId;
use crate::types::outcome::{BudgetReport, QueryBudget};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_RESIDUAL_STORE_ID: AtomicU64 = AtomicU64::new(1);

/// Lineage identity of one append-only static residual store.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticResidualStoreId(u64);

impl StaticResidualStoreId {
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Index of a residual node within its allocating [`StaticResidualStore`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticResidualId(u32);

impl StaticResidualId {
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Store-qualified reference to an interned static residual.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticResidualRef {
    store: StaticResidualStoreId,
    residual: StaticResidualId,
}

impl StaticResidualRef {
    pub const fn new(store: StaticResidualStoreId, residual: StaticResidualId) -> Self {
        Self { store, residual }
    }

    pub const fn store_id(self) -> StaticResidualStoreId {
        self.store
    }

    pub const fn residual_id(self) -> StaticResidualId {
        self.residual
    }
}

/// Domain-neutral term envelope that can represent reduced values, kinds, or residuals.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StaticEvalTerm {
    Value(StaticValueRef),
    Kinded(TypeId),
    Residual(StaticResidualRef),
}

/// Alias for term representations inside residuals.
pub type StaticResidualTerm = StaticEvalTerm;

/// Stable semantic identity for an unreduced static definition parameter.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticResidualParameter {
    pub owner: DeclarationId,
    pub index: u32,
    pub domain: StaticValueDomain,
}

/// Unreduced static application of a definition to arguments.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticResidualApplication {
    pub definition: DeclarationId,
    pub arguments: Box<[StaticEvalTerm]>,
    pub environment: StaticSpecializationEnvironmentKey,
}

/// One arm of a residual match.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticResidualMatchArm {
    pub arm_index: u32,
    pub body: StaticEvalTerm,
}

/// Unreduced static match expression.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticResidualMatch {
    pub resolution_owner: StaticBodyId,
    pub expression: ExpressionId,
    pub scrutinee: StaticEvalTerm,
    pub arms: Box<[StaticResidualMatchArm]>,
}

/// Interned canonical payload of a static residual.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum StaticResidualData {
    Parameter {
        parameter: StaticResidualParameter,
    },
    Apply {
        application: StaticResidualApplication,
        result_domain: StaticValueDomain,
    },
    Proof {
        goal: ProofPredicateGoal,
        result_type: TypeId,
        environment: StaticSpecializationEnvironmentKey,
    },
    MemberProjection {
        projection: MemberProjection,
        result_domain: StaticValueDomain,
        environment: StaticSpecializationEnvironmentKey,
    },
    If {
        condition: StaticEvalTerm,
        then_term: StaticEvalTerm,
        else_term: StaticEvalTerm,
        result_domain: StaticValueDomain,
    },
    Match {
        semantics: StaticResidualMatch,
        result_domain: StaticValueDomain,
    },
    Pack {
        positionals: Box<[StaticEvalTerm]>,
        labels: Box<[Box<str>]>,
        labeled_values: Box<[StaticEvalTerm]>,
        result_type: TypeId,
    },
    Tuple {
        elements: Box<[StaticEvalTerm]>,
        result_type: TypeId,
    },
    Record {
        fields: Box<[(FieldKey, StaticEvalTerm)]>,
        result_type: TypeId,
    },
    Data {
        owner: DeclarationId,
        constructor: DataConstructorId,
        result_type: TypeId,
        components: Box<[(DataComponentId, StaticEvalTerm)]>,
    },
    Variant {
        variant: VariantId,
        result_type: TypeId,
        fields: Box<[(VariantFieldId, StaticEvalTerm)]>,
    },
}

impl StaticResidualData {
    fn payload_work_units(&self) -> usize {
        fn environment_units(environment: &StaticSpecializationEnvironmentKey) -> usize {
            environment
                .argument_bindings()
                .len()
                .saturating_add(environment.type_bindings().len())
                .saturating_add(environment.constraints().len())
        }

        match self {
            Self::Parameter { .. } => 0,
            Self::Apply { application, .. } => application.arguments.len().saturating_add(environment_units(&application.environment)),
            Self::Proof { environment, .. } => environment_units(environment),
            Self::MemberProjection { projection, environment, .. } => projection.arguments.len().saturating_add(environment_units(environment)),
            Self::If { .. } => 3,
            Self::Match { semantics, .. } => semantics.arms.len().saturating_add(1),
            Self::Pack {
                positionals,
                labels,
                labeled_values,
                ..
            } => positionals
                .len()
                .saturating_add(labeled_values.len())
                .saturating_add(labels.len())
                .saturating_add(labels.iter().map(|label| label.len()).fold(0usize, usize::saturating_add)),
            Self::Tuple { elements, .. } => elements.len(),
            Self::Record { fields, .. } => fields.len(),
            Self::Data { components, .. } => components.len(),
            Self::Variant { fields, .. } => fields.len(),
        }
    }

    /// Returns the semantic domain produced by evaluating or specializing this residual.
    pub fn result_domain(&self) -> StaticValueDomain {
        match self {
            Self::Parameter { parameter } => parameter.domain,
            Self::Apply { result_domain, .. } => *result_domain,
            Self::Proof { result_type, .. } => StaticValueDomain::ValueType(*result_type),
            Self::MemberProjection { result_domain, .. } => *result_domain,
            Self::If { result_domain, .. } => *result_domain,
            Self::Match { result_domain, .. } => *result_domain,
            Self::Pack { result_type, .. } => StaticValueDomain::ValueType(*result_type),
            Self::Tuple { result_type, .. } => StaticValueDomain::ValueType(*result_type),
            Self::Record { result_type, .. } => StaticValueDomain::ValueType(*result_type),
            Self::Data { result_type, .. } => StaticValueDomain::ValueType(*result_type),
            Self::Variant { result_type, .. } => StaticValueDomain::ValueType(*result_type),
        }
    }
}

/// Store-level error for static residual operations.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum StaticResidualStoreError {
    #[error("residual id {0} is not present in store")]
    UnknownResidual(u32),
    #[error("residual reference belongs to another store lineage")]
    ForeignStore,
    #[error("Pack residual has mismatched labeled lanes")]
    InvalidPackLanes,
    #[error("Pack residual has duplicate Symbol label `{0}`")]
    DuplicatePackLabel(Box<str>),
    #[error("static residual node budget exceeded: {0:?}")]
    BudgetExceeded(BudgetReport),
}

impl From<BudgetReport> for StaticResidualStoreError {
    fn from(report: BudgetReport) -> Self {
        Self::BudgetExceeded(report)
    }
}

/// Append-only arena for interned, canonical static residuals.
#[derive(Clone, Debug)]
pub struct StaticResidualStore {
    id: StaticResidualStoreId,
    residuals: Vec<StaticResidualData>,
    trusted_contexts: Vec<StaticTrustedBranchContext>,
    interner: HashMap<(StaticResidualData, StaticTrustedBranchContext), StaticResidualId>,
}

impl Default for StaticResidualStore {
    fn default() -> Self {
        Self::new()
    }
}

impl StaticResidualStore {
    pub fn new() -> Self {
        let raw = NEXT_RESIDUAL_STORE_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id: StaticResidualStoreId(raw),
            residuals: Vec::new(),
            trusted_contexts: Vec::new(),
            interner: HashMap::new(),
        }
    }

    pub fn id(&self) -> StaticResidualStoreId {
        self.id
    }

    pub fn get(&self, reference: StaticResidualRef) -> Result<&StaticResidualData, StaticResidualStoreError> {
        if reference.store != self.id {
            return Err(StaticResidualStoreError::ForeignStore);
        }
        let index = reference.residual.0 as usize;
        self.residuals.get(index).ok_or(StaticResidualStoreError::UnknownResidual(reference.residual.0))
    }

    pub(crate) fn trusted_context(&self, reference: StaticResidualRef) -> Result<&StaticTrustedBranchContext, StaticResidualStoreError> {
        if reference.store != self.id {
            return Err(StaticResidualStoreError::ForeignStore);
        }
        self.trusted_contexts
            .get(reference.residual.0 as usize)
            .ok_or(StaticResidualStoreError::UnknownResidual(reference.residual.0))
    }

    /// Interns a static residual node, charging the residual growth budget.
    pub fn intern(&mut self, data: StaticResidualData, budget: &mut QueryBudget) -> Result<StaticResidualRef, StaticResidualStoreError> {
        self.intern_with_trusted_context(data, StaticTrustedBranchContext::default(), budget)
    }

    pub(crate) fn intern_with_trusted_context(
        &mut self,
        data: StaticResidualData,
        trusted_context: StaticTrustedBranchContext,
        budget: &mut QueryBudget,
    ) -> Result<StaticResidualRef, StaticResidualStoreError> {
        // Charge for variable-size payloads before traversing or cloning an
        // interning key. Repeated attempts may be free in nodes, but not in work.
        let payload_units = data
            .payload_work_units()
            .saturating_add(trusted_context.type_bindings().len())
            .saturating_add(trusted_context.constraints().len());
        for _ in 0..payload_units.div_ceil(1024) {
            budget.charge_step().map_err(StaticResidualStoreError::from)?;
        }
        self.validate_data(&data)?;
        let key = (data.clone(), trusted_context.clone());
        if let Some(&existing) = self.interner.get(&key) {
            return Ok(StaticResidualRef::new(self.id, existing));
        }

        budget.charge_residual_node().map_err(StaticResidualStoreError::from)?;

        let id = StaticResidualId(self.residuals.len() as u32);
        self.residuals.push(data.clone());
        self.trusted_contexts.push(trusted_context);
        self.interner.insert(key, id);
        Ok(StaticResidualRef::new(self.id, id))
    }

    pub fn parameter(&mut self, parameter: StaticResidualParameter, budget: &mut QueryBudget) -> Result<StaticResidualRef, StaticResidualStoreError> {
        self.intern(StaticResidualData::Parameter { parameter }, budget)
    }

    pub fn apply(
        &mut self,
        application: StaticResidualApplication,
        result_domain: StaticValueDomain,
        budget: &mut QueryBudget,
    ) -> Result<StaticResidualRef, StaticResidualStoreError> {
        self.intern(StaticResidualData::Apply { application, result_domain }, budget)
    }

    pub fn proof(
        &mut self,
        mut goal: ProofPredicateGoal,
        result_type: TypeId,
        environment: StaticSpecializationEnvironmentKey,
        budget: &mut QueryBudget,
    ) -> Result<StaticResidualRef, StaticResidualStoreError> {
        if let ProofPredicateGoal::Equivalent { left, right } = &mut goal {
            if *right < *left {
                std::mem::swap(left, right);
            }
        }
        self.intern(
            StaticResidualData::Proof {
                goal,
                result_type,
                environment,
            },
            budget,
        )
    }

    pub fn member_projection(
        &mut self,
        projection: MemberProjection,
        result_domain: StaticValueDomain,
        environment: StaticSpecializationEnvironmentKey,
        budget: &mut QueryBudget,
    ) -> Result<StaticResidualRef, StaticResidualStoreError> {
        self.intern(
            StaticResidualData::MemberProjection {
                projection,
                result_domain,
                environment,
            },
            budget,
        )
    }

    pub fn if_then_else(
        &mut self,
        condition: StaticEvalTerm,
        then_term: StaticEvalTerm,
        else_term: StaticEvalTerm,
        result_domain: StaticValueDomain,
        budget: &mut QueryBudget,
    ) -> Result<StaticResidualRef, StaticResidualStoreError> {
        if then_term == else_term {
            if let StaticEvalTerm::Residual(r) = then_term {
                return Ok(r);
            }
        }
        self.intern(
            StaticResidualData::If {
                condition,
                then_term,
                else_term,
                result_domain,
            },
            budget,
        )
    }

    pub fn match_term(
        &mut self,
        semantics: StaticResidualMatch,
        result_domain: StaticValueDomain,
        budget: &mut QueryBudget,
    ) -> Result<StaticResidualRef, StaticResidualStoreError> {
        self.intern(StaticResidualData::Match { semantics, result_domain }, budget)
    }

    pub fn pack(
        &mut self,
        positionals: Box<[StaticEvalTerm]>,
        labels: Box<[Box<str>]>,
        labeled_values: Box<[StaticEvalTerm]>,
        result_type: TypeId,
        budget: &mut QueryBudget,
    ) -> Result<StaticResidualRef, StaticResidualStoreError> {
        self.intern(
            StaticResidualData::Pack {
                positionals,
                labels,
                labeled_values,
                result_type,
            },
            budget,
        )
    }

    pub fn tuple(
        &mut self,
        elements: Box<[StaticEvalTerm]>,
        result_type: TypeId,
        budget: &mut QueryBudget,
    ) -> Result<StaticResidualRef, StaticResidualStoreError> {
        self.intern(StaticResidualData::Tuple { elements, result_type }, budget)
    }

    pub fn record(
        &mut self,
        mut fields: Vec<(FieldKey, StaticEvalTerm)>,
        result_type: TypeId,
        budget: &mut QueryBudget,
    ) -> Result<StaticResidualRef, StaticResidualStoreError> {
        fields.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        self.intern(
            StaticResidualData::Record {
                fields: fields.into_boxed_slice(),
                result_type,
            },
            budget,
        )
    }

    pub fn data(
        &mut self,
        owner: DeclarationId,
        constructor: DataConstructorId,
        result_type: TypeId,
        components: Box<[(DataComponentId, StaticEvalTerm)]>,
        budget: &mut QueryBudget,
    ) -> Result<StaticResidualRef, StaticResidualStoreError> {
        self.intern(
            StaticResidualData::Data {
                owner,
                constructor,
                result_type,
                components,
            },
            budget,
        )
    }

    pub fn variant(
        &mut self,
        variant: VariantId,
        result_type: TypeId,
        fields: Box<[(VariantFieldId, StaticEvalTerm)]>,
        budget: &mut QueryBudget,
    ) -> Result<StaticResidualRef, StaticResidualStoreError> {
        self.intern(StaticResidualData::Variant { variant, result_type, fields }, budget)
    }

    /// Computes an allocation-order-independent semantic fingerprint of the residual node.
    ///
    /// Note (LANG007.C4.P7): Residual fingerprints represent store-lineage-local identity
    /// for identical canonical node structures and constituent type IDs. They guarantee
    /// allocation-order independence within compatible stores, but do not assert durable
    /// or cross-lineage universal semantic identity. Cross-session and incremental DB
    /// identity is deferred to T12/P8.
    pub fn content_fingerprint(&self, reference: StaticResidualRef) -> Result<u64, StaticResidualStoreError> {
        let node = self.get(reference)?;
        let mut hasher = DefaultHasher::new();
        self.fingerprint_node(node, &mut hasher)?;
        Ok(hasher.finish())
    }

    fn validate_term(&self, term: &StaticEvalTerm) -> Result<(), StaticResidualStoreError> {
        if let StaticEvalTerm::Residual(reference) = term {
            self.get(*reference)?;
        }
        Ok(())
    }

    fn validate_data(&self, data: &StaticResidualData) -> Result<(), StaticResidualStoreError> {
        let validate = |term: &StaticEvalTerm| self.validate_term(term);
        match data {
            StaticResidualData::Parameter { .. } | StaticResidualData::Proof { .. } | StaticResidualData::MemberProjection { .. } => {}
            StaticResidualData::Apply { application, .. } => {
                for argument in application.arguments.iter() {
                    validate(argument)?;
                }
            }
            StaticResidualData::If {
                condition,
                then_term,
                else_term,
                ..
            } => {
                validate(condition)?;
                validate(then_term)?;
                validate(else_term)?;
            }
            StaticResidualData::Match { semantics, .. } => {
                validate(&semantics.scrutinee)?;
                for arm in semantics.arms.iter() {
                    validate(&arm.body)?;
                }
            }
            StaticResidualData::Pack {
                positionals,
                labels,
                labeled_values,
                ..
            } => {
                if labels.len() != labeled_values.len() {
                    return Err(StaticResidualStoreError::InvalidPackLanes);
                }
                let mut seen = std::collections::HashSet::with_capacity(labels.len());
                for label in labels.iter() {
                    if !seen.insert(label.as_ref()) {
                        return Err(StaticResidualStoreError::DuplicatePackLabel(label.clone()));
                    }
                }
                for term in positionals.iter().chain(labeled_values.iter()) {
                    validate(term)?;
                }
            }
            StaticResidualData::Tuple { elements, .. } => {
                for element in elements.iter() {
                    validate(element)?;
                }
            }
            StaticResidualData::Record { fields, .. } => {
                for (_, value) in fields.iter() {
                    validate(value)?;
                }
            }
            StaticResidualData::Data { components, .. } => {
                for (_, component) in components.iter() {
                    validate(component)?;
                }
            }
            StaticResidualData::Variant { fields, .. } => {
                for (_, value) in fields.iter() {
                    validate(value)?;
                }
            }
        }
        Ok(())
    }

    fn fingerprint_term(&self, term: &StaticEvalTerm, hasher: &mut DefaultHasher) -> Result<(), StaticResidualStoreError> {
        match term {
            StaticEvalTerm::Value(v) => {
                0u8.hash(hasher);
                v.hash(hasher);
            }
            StaticEvalTerm::Kinded(k) => {
                1u8.hash(hasher);
                k.hash(hasher);
            }
            StaticEvalTerm::Residual(r) => {
                2u8.hash(hasher);
                let child = self.get(*r)?;
                self.fingerprint_node(child, hasher)?;
            }
        }
        Ok(())
    }

    fn fingerprint_node(&self, node: &StaticResidualData, hasher: &mut DefaultHasher) -> Result<(), StaticResidualStoreError> {
        match node {
            StaticResidualData::Parameter { parameter } => {
                10u8.hash(hasher);
                parameter.hash(hasher);
            }
            StaticResidualData::Apply { application, result_domain } => {
                11u8.hash(hasher);
                application.definition.hash(hasher);
                application.environment.hash(hasher);
                result_domain.hash(hasher);
                application.arguments.len().hash(hasher);
                for arg in application.arguments.iter() {
                    self.fingerprint_term(arg, hasher)?;
                }
            }
            StaticResidualData::Proof {
                goal,
                result_type,
                environment,
            } => {
                12u8.hash(hasher);
                result_type.hash(hasher);
                environment.hash(hasher);
                match goal {
                    ProofPredicateGoal::Equivalent { left, right } => {
                        0u8.hash(hasher);
                        let (min_t, max_t) = if left <= right { (left, right) } else { (right, left) };
                        min_t.hash(hasher);
                        max_t.hash(hasher);
                    }
                    ProofPredicateGoal::Subtype { lower, upper } => {
                        1u8.hash(hasher);
                        lower.hash(hasher);
                        upper.hash(hasher);
                    }
                    ProofPredicateGoal::Conforms { key } => {
                        2u8.hash(hasher);
                        key.hash(hasher);
                    }
                }
            }
            StaticResidualData::MemberProjection {
                projection,
                result_domain,
                environment,
            } => {
                13u8.hash(hasher);
                projection.hash(hasher);
                result_domain.hash(hasher);
                environment.hash(hasher);
            }
            StaticResidualData::If {
                condition,
                then_term,
                else_term,
                result_domain,
            } => {
                14u8.hash(hasher);
                result_domain.hash(hasher);
                self.fingerprint_term(condition, hasher)?;
                self.fingerprint_term(then_term, hasher)?;
                self.fingerprint_term(else_term, hasher)?;
            }
            StaticResidualData::Match { semantics, result_domain } => {
                15u8.hash(hasher);
                semantics.resolution_owner.hash(hasher);
                semantics.expression.hash(hasher);
                result_domain.hash(hasher);
                self.fingerprint_term(&semantics.scrutinee, hasher)?;
                semantics.arms.len().hash(hasher);
                for arm in semantics.arms.iter() {
                    arm.arm_index.hash(hasher);
                    self.fingerprint_term(&arm.body, hasher)?;
                }
            }
            StaticResidualData::Pack {
                positionals,
                labels,
                labeled_values,
                result_type,
            } => {
                16u8.hash(hasher);
                result_type.hash(hasher);
                positionals.len().hash(hasher);
                for p in positionals.iter() {
                    self.fingerprint_term(p, hasher)?;
                }
                labels.hash(hasher);
                labeled_values.len().hash(hasher);
                for lv in labeled_values.iter() {
                    self.fingerprint_term(lv, hasher)?;
                }
            }
            StaticResidualData::Tuple { elements, result_type } => {
                17u8.hash(hasher);
                result_type.hash(hasher);
                elements.len().hash(hasher);
                for e in elements.iter() {
                    self.fingerprint_term(e, hasher)?;
                }
            }
            StaticResidualData::Record { fields, result_type } => {
                18u8.hash(hasher);
                result_type.hash(hasher);
                fields.len().hash(hasher);
                for (k, v) in fields.iter() {
                    k.hash(hasher);
                    self.fingerprint_term(v, hasher)?;
                }
            }
            StaticResidualData::Data {
                owner,
                constructor,
                result_type,
                components,
            } => {
                19u8.hash(hasher);
                owner.hash(hasher);
                constructor.hash(hasher);
                result_type.hash(hasher);
                components.len().hash(hasher);
                for (cid, comp) in components.iter() {
                    cid.hash(hasher);
                    self.fingerprint_term(comp, hasher)?;
                }
            }
            StaticResidualData::Variant { variant, result_type, fields } => {
                20u8.hash(hasher);
                variant.hash(hasher);
                result_type.hash(hasher);
                fields.len().hash(hasher);
                for (fid, fval) in fields.iter() {
                    fid.hash(hasher);
                    self.fingerprint_term(fval, hasher)?;
                }
            }
        }
        Ok(())
    }
}
