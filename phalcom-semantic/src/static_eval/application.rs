//! Exact static application identity, specialization environment, cycle representation, and memo outcomes.

use crate::generic_evidence::GenericConformanceKey;
use crate::identity::DeclarationId;
use crate::static_eval::residual::{StaticEvalTerm, StaticResidualParameter, StaticResidualRef};
use crate::static_value::StaticValueRef;
use crate::types::id::{TypeId, TypeParameterId};
use phalcom_common::range::SourceRange;
use std::fmt;

/// Summary of an argument participating in an exact cycle diagnostic.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum StaticArgumentSummary {
    Kinded(TypeId),
    Value(StaticValueRef),
    Residual(StaticResidualRef),
    Text(Box<str>),
}

/// One activation frame in an exact recursive static application cycle.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StaticCycleFrame {
    pub definition: DeclarationId,
    pub arguments: Box<[StaticArgumentSummary]>,
    pub source_range: Option<SourceRange>,
}

/// Canonical payload for recursive cycle detection.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StaticCycle {
    pub applications: Box<[StaticCycleFrame]>,
    pub cycle_start: u32,
}

impl StaticCycle {
    pub fn new(applications: Box<[StaticCycleFrame]>, cycle_start: u32) -> Self {
        Self {
            applications,
            cycle_start,
        }
    }

    pub fn empty() -> Self {
        Self {
            applications: Box::new([]),
            cycle_start: 0,
        }
    }
}

impl fmt::Display for StaticCycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "static evaluation entered an exact recursive cycle\n")?;
        for (i, frame) in self.applications.iter().enumerate() {
            let marker = if i == self.cycle_start as usize {
                "  ↻ "
            } else if i == 0 {
                "  "
            } else {
                "  → "
            };
            write!(f, "{}{:?}", marker, frame.definition)?;
            if !frame.arguments.is_empty() {
                write!(f, "<")?;
                for (j, arg) in frame.arguments.iter().enumerate() {
                    if j > 0 {
                        write!(f, ", ")?;
                    }
                    match arg {
                        StaticArgumentSummary::Kinded(id) => write!(f, "Type({:?})", id)?,
                        StaticArgumentSummary::Value(v) => write!(f, "Value({:?})", v)?,
                        StaticArgumentSummary::Residual(r) => write!(f, "Residual({:?})", r)?,
                        StaticArgumentSummary::Text(s) => write!(f, "{}", s)?,
                    }
                }
                write!(f, ">")?;
            }
            writeln!(f)?;
        }
        write!(f, "cycle begins at application {} above", self.cycle_start.saturating_add(1))
    }
}

/// Canonical static proof proposition that can specialize an application.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CanonicalStaticProofFact {
    Equal(TypeId, TypeId),
    Subtype { lower: TypeId, upper: TypeId },
    Conforms { key: GenericConformanceKey },
}

/// Checked branch refinements captured by the evaluator for one residual.
/// This type is crate-private and is stored outside the public residual payload.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct StaticTrustedBranchContext {
    type_bindings: Box<[(TypeParameterId, TypeId)]>,
    constraints: Box<[CanonicalStaticProofFact]>,
}

impl StaticTrustedBranchContext {
    pub(crate) fn from_parts(
        type_bindings: impl IntoIterator<Item = (TypeParameterId, TypeId)>,
        constraints: impl IntoIterator<Item = CanonicalStaticProofFact>,
    ) -> Self {
        use std::collections::{BTreeMap, BTreeSet};
        let type_bindings = type_bindings.into_iter().collect::<BTreeMap<_, _>>().into_iter().collect::<Vec<_>>().into_boxed_slice();
        let constraints = constraints
            .into_iter()
            .map(|fact| match fact {
                CanonicalStaticProofFact::Equal(left, right) if right < left => CanonicalStaticProofFact::Equal(right, left),
                fact => fact,
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self { type_bindings, constraints }
    }

    pub(crate) fn type_bindings(&self) -> &[(TypeParameterId, TypeId)] {
        &self.type_bindings
    }

    pub(crate) fn constraints(&self) -> &[CanonicalStaticProofFact] {
        &self.constraints
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.type_bindings.is_empty() && self.constraints.is_empty()
    }

    pub(crate) fn overlay_branch_proofs<'a>(
        &self,
        environments: impl IntoIterator<Item = &'a crate::match_semantics::BranchProofEnvironment>,
    ) -> Self {
        use std::collections::{BTreeMap, BTreeSet};

        let mut types: BTreeMap<_, _> = self.type_bindings.iter().copied().collect();
        let mut constraints: BTreeSet<_> = self.constraints.iter().cloned().collect();
        for environment in environments {
            for (parameter, ty) in &environment.bindings {
                types.insert(*parameter, *ty);
            }
            for equality in environment.equalities.iter() {
                match equality {
                    crate::types::constraint::TypeConstraint::Equal(left, right) => {
                        let (left, right) = if left <= right { (*left, *right) } else { (*right, *left) };
                        constraints.insert(CanonicalStaticProofFact::Equal(left, right));
                    }
                    crate::types::constraint::TypeConstraint::Subtype(lower, upper) => {
                        constraints.insert(CanonicalStaticProofFact::Subtype { lower: *lower, upper: *upper });
                    }
                    crate::types::constraint::TypeConstraint::HasMember(_, _) => {}
                }
            }
        }
        Self::from_parts(types, constraints)
    }

    pub(crate) fn substitute(
        &self,
        substitution: &crate::types::substitution::TypeSubstitution,
        types: &mut crate::types::store::TypeStore,
    ) -> Result<Self, StaticSpecializationEnvironmentError> {
        use std::collections::{BTreeMap, BTreeSet};

        let mut bindings = BTreeMap::new();
        for (parameter, ty) in self.type_bindings.iter() {
            let value = substitution
                .try_apply(types, *ty)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            let parameter_type = types.parameter_form(*parameter);
            let specialized_parameter = substitution
                .try_apply(types, parameter_type)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            // A later concrete specialization that disagrees with the checked
            // branch refinement makes that refinement inapplicable.
            if specialized_parameter == parameter_type || specialized_parameter == value {
                bindings.insert(*parameter, value);
            }
        }

        let mut constraints = BTreeSet::new();
        for fact in self.constraints.iter() {
            match fact {
                CanonicalStaticProofFact::Equal(left, right) => {
                    let left = substitution
                        .try_apply(types, *left)
                        .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                    let right = substitution
                        .try_apply(types, *right)
                        .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                    let (left, right) = if left <= right { (left, right) } else { (right, left) };
                    constraints.insert(CanonicalStaticProofFact::Equal(left, right));
                }
                CanonicalStaticProofFact::Subtype { lower, upper } => {
                    let lower = substitution
                        .try_apply(types, *lower)
                        .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                    let upper = substitution
                        .try_apply(types, *upper)
                        .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                    constraints.insert(CanonicalStaticProofFact::Subtype { lower, upper });
                }
                CanonicalStaticProofFact::Conforms { key } => {
                    let subject = match key.subject {
                        crate::types::parameter::TypeTerm::Canonical(ty) => crate::types::parameter::TypeTerm::Canonical(
                            substitution
                                .try_apply(types, ty)
                                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?,
                        ),
                        ref other => other.clone(),
                    };
                    let mut arguments = Vec::with_capacity(key.trait_ref.arguments.len());
                    for argument in key.trait_ref.arguments.iter() {
                        arguments.push(
                            substitution
                                .try_apply(types, *argument)
                                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?,
                        );
                    }
                    constraints.insert(CanonicalStaticProofFact::Conforms {
                        key: crate::generic_evidence::GenericConformanceKey::new(
                            subject,
                            crate::traits::TraitRef::new(key.trait_ref.declaration.clone(), arguments),
                        ),
                    });
                }
            }
        }
        Ok(Self::from_parts(bindings, constraints))
    }
}

/// Exact argument captured for one formal static parameter.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StaticSpecializationArgument {
    Kinded(TypeId),
    Value(StaticValueRef),
}

/// Failure to compose two canonical specialization environments.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum StaticSpecializationEnvironmentError {
    #[error("conflicting exact arguments for one static parameter")]
    ConflictingArgumentBinding,
    #[error("conflicting canonical type bindings for one type parameter")]
    ConflictingTypeBinding,
    #[error("argument domain mismatch for static parameter")]
    ArgumentDomainMismatch,
    #[error("static value belongs to a foreign store lineage")]
    ForeignStaticValueStore,
    #[error("malformed specialization environment")]
    MalformedEnvironment,
    #[error("specialization composition was cancelled")]
    Cancelled,
    #[error("specialization composition exceeded its query budget: {0:?}")]
    BudgetExceeded(crate::types::outcome::BudgetReport),
    #[error("specialization substitution closure did not converge")]
    NonConvergentSubstitution,
    #[error("unsupported canonical substitution in specialization environment")]
    UnsupportedCanonicalSubstitution,
}

/// Canonical ambient environment for static application identity (cycles & memoization).
///
/// Contains only ambient type bindings and constraints that can affect evaluation,
/// strictly excluding exact formal argument bindings of caller activations.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticApplicationEnvironmentKey {
    type_bindings: Box<[(TypeParameterId, TypeId)]>,
    constraints: Box<[CanonicalStaticProofFact]>,
    trusted_branch_context: StaticTrustedBranchContext,
}

impl StaticApplicationEnvironmentKey {
    pub fn normalized(
        type_bindings: impl IntoIterator<Item = (TypeParameterId, TypeId)>,
        constraints: impl IntoIterator<Item = CanonicalStaticProofFact>,
    ) -> Result<Self, StaticSpecializationEnvironmentError> {
        use std::collections::{BTreeMap, BTreeSet};

        let mut normalized_types = BTreeMap::new();
        for (parameter, ty) in type_bindings {
            if let Some(existing) = normalized_types.insert(parameter, ty)
                && existing != ty
            {
                return Err(StaticSpecializationEnvironmentError::ConflictingTypeBinding);
            }
        }

        let normalized_constraints = constraints
            .into_iter()
            .map(|fact| match fact {
                CanonicalStaticProofFact::Equal(left, right) if right < left => CanonicalStaticProofFact::Equal(right, left),
                fact => fact,
            })
            .collect::<BTreeSet<_>>();

        Ok(Self {
            type_bindings: normalized_types.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            constraints: normalized_constraints.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            trusted_branch_context: StaticTrustedBranchContext::default(),
        })
    }

    pub fn type_bindings(&self) -> &[(TypeParameterId, TypeId)] {
        &self.type_bindings
    }

    pub fn constraints(&self) -> &[CanonicalStaticProofFact] {
        &self.constraints
    }

    pub fn is_empty(&self) -> bool {
        self.type_bindings.is_empty() && self.constraints.is_empty() && self.trusted_branch_context.is_empty()
    }

    pub fn to_specialization_key(&self) -> StaticSpecializationEnvironmentKey {
        StaticSpecializationEnvironmentKey {
            argument_bindings: Box::new([]),
            type_bindings: self.type_bindings.clone(),
            constraints: self.constraints.clone(),
        }
    }

    pub(crate) fn with_trusted_branch_context(mut self, context: StaticTrustedBranchContext) -> Self {
        self.trusted_branch_context = context;
        self
    }
}

/// Normalized specialization and proof environment key.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticSpecializationEnvironmentKey {
    argument_bindings: Box<[(StaticResidualParameter, StaticSpecializationArgument)]>,
    type_bindings: Box<[(TypeParameterId, TypeId)]>,
    constraints: Box<[CanonicalStaticProofFact]>,
}

impl StaticSpecializationEnvironmentKey {
    pub fn argument_bindings(&self) -> &[(StaticResidualParameter, StaticSpecializationArgument)] {
        &self.argument_bindings
    }

    pub fn type_bindings(&self) -> &[(TypeParameterId, TypeId)] {
        &self.type_bindings
    }

    pub fn constraints(&self) -> &[CanonicalStaticProofFact] {
        &self.constraints
    }

    pub fn normalized(
        argument_bindings: impl IntoIterator<Item = (StaticResidualParameter, StaticSpecializationArgument)>,
        type_bindings: impl IntoIterator<Item = (TypeParameterId, TypeId)>,
        constraints: impl IntoIterator<Item = CanonicalStaticProofFact>,
    ) -> Result<Self, StaticSpecializationEnvironmentError> {
        use std::collections::{BTreeMap, BTreeSet};

        let mut normalized_arguments = BTreeMap::new();
        for (parameter, argument) in argument_bindings {
            if let Some(existing) = normalized_arguments.insert(parameter, argument)
                && existing != argument
            {
                return Err(StaticSpecializationEnvironmentError::ConflictingArgumentBinding);
            }
        }

        let mut normalized_types = BTreeMap::new();
        for (parameter, ty) in type_bindings {
            if let Some(existing) = normalized_types.insert(parameter, ty)
                && existing != ty
            {
                return Err(StaticSpecializationEnvironmentError::ConflictingTypeBinding);
            }
        }

        let normalized_constraints = constraints
            .into_iter()
            .map(|fact| match fact {
                CanonicalStaticProofFact::Equal(left, right) if right < left => CanonicalStaticProofFact::Equal(right, left),
                fact => fact,
            })
            .collect::<BTreeSet<_>>();

        Ok(Self {
            argument_bindings: normalized_arguments.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            type_bindings: normalized_types.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            constraints: normalized_constraints.into_iter().collect::<Vec<_>>().into_boxed_slice(),
        })
    }

    pub fn application_identity_projection(&self) -> StaticApplicationEnvironmentKey {
        StaticApplicationEnvironmentKey {
            type_bindings: self.type_bindings.clone(),
            constraints: self.constraints.clone(),
            trusted_branch_context: StaticTrustedBranchContext::default(),
        }
    }

    pub(crate) fn with_additional_type_bindings(&self, bindings: impl IntoIterator<Item = (TypeParameterId, TypeId)>) -> Self {
        let type_bindings = self.type_bindings.iter().copied().chain(bindings).collect::<Vec<_>>().into_boxed_slice();
        Self {
            argument_bindings: self.argument_bindings.clone(),
            type_bindings,
            constraints: self.constraints.clone(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.argument_bindings.is_empty()
            && self.type_bindings.is_empty()
            && self.constraints.is_empty()
    }

    /// Strict merge of two environments at the same composition layer.
    ///
    /// Conflicting bindings fail closed with an error.
    pub fn merge(&self, supplied: &Self) -> Result<Self, StaticSpecializationEnvironmentError> {
        Self::normalized(
            self.argument_bindings.iter().cloned().chain(supplied.argument_bindings.iter().cloned()),
            self.type_bindings.iter().copied().chain(supplied.type_bindings.iter().copied()),
            self.constraints.iter().cloned().chain(supplied.constraints.iter().cloned()),
        )
    }

    /// Overlays exact arguments from the current activation frame onto inherited context.
    ///
    /// If the current activation supplies an argument for a formal that was already bound
    /// in an outer frame, the current activation's argument shadows the outer binding.
    /// Conflicting duplicate arguments within the supplied batch fail closed.
    pub fn overlay_arguments(
        &self,
        arguments: impl IntoIterator<Item = (StaticResidualParameter, StaticSpecializationArgument)>,
    ) -> Result<Self, StaticSpecializationEnvironmentError> {
        use std::collections::BTreeMap;

        let mut args_map: BTreeMap<_, _> = self.argument_bindings.iter().cloned().collect();
        let mut batch_seen = BTreeMap::new();

        for (param, arg) in arguments {
            if let Some(existing) = batch_seen.insert(param.clone(), arg)
                && existing != arg
            {
                return Err(StaticSpecializationEnvironmentError::ConflictingArgumentBinding);
            }
            args_map.insert(param, arg);
        }

        Ok(Self {
            argument_bindings: args_map.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            type_bindings: self.type_bindings.clone(),
            constraints: self.constraints.clone(),
        })
    }

    /// Adds caller-supplied branch propositions to a normalized environment.
    /// These propositions do not carry evaluator proof provenance.
    pub fn overlay_branch_proofs<'a>(
        &self,
        environments: impl IntoIterator<Item = &'a crate::match_semantics::BranchProofEnvironment>,
    ) -> Result<Self, StaticSpecializationEnvironmentError> {
        use std::collections::{BTreeMap, BTreeSet};

        let mut bindings: BTreeMap<_, _> = self.type_bindings.iter().copied().collect();
        let mut constraints: BTreeSet<_> = self.constraints.iter().cloned().collect();
        for environment in environments {
            for (parameter, ty) in &environment.bindings {
                bindings.insert(*parameter, *ty);
            }
            for equality in environment.equalities.iter() {
                match equality {
                    crate::types::constraint::TypeConstraint::Equal(left, right) => {
                        let (left, right) = if left <= right { (*left, *right) } else { (*right, *left) };
                        constraints.insert(CanonicalStaticProofFact::Equal(left, right));
                    }
                    crate::types::constraint::TypeConstraint::Subtype(lower, upper) => {
                        constraints.insert(CanonicalStaticProofFact::Subtype { lower: *lower, upper: *upper });
                    }
                    crate::types::constraint::TypeConstraint::HasMember(_, _) => {}
                }
            }
        }
        Self::normalized(
            self.argument_bindings.iter().cloned(),
            bindings,
            constraints,
        )
    }

    /// Projects caller-supplied branch propositions into a normalized key.
    /// The projected propositions are not checked evaluator proof evidence.
    pub fn from_branch_proofs<'a>(
        environments: impl IntoIterator<Item = &'a crate::match_semantics::BranchProofEnvironment>,
    ) -> Result<Self, StaticSpecializationEnvironmentError> {
        use std::collections::{BTreeMap, BTreeSet};

        let mut bindings_map = BTreeMap::new();
        let mut constraints = BTreeSet::new();

        for env in environments {
            for (param, ty) in &env.bindings {
                bindings_map.insert(*param, *ty);
            }
            for eq in env.equalities.iter() {
                match eq {
                    crate::types::constraint::TypeConstraint::Equal(t1, t2) => {
                        let (min, max) = if t1 <= t2 { (*t1, *t2) } else { (*t2, *t1) };
                        constraints.insert(CanonicalStaticProofFact::Equal(min, max));
                    }
                    crate::types::constraint::TypeConstraint::Subtype(lower, upper) => {
                        constraints.insert(CanonicalStaticProofFact::Subtype { lower: *lower, upper: *upper });
                    }
                    crate::types::constraint::TypeConstraint::HasMember(_, _) => {}
                }
            }
        }

        Ok(Self {
            argument_bindings: Box::new([]),
            type_bindings: bindings_map.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            constraints: constraints.into_iter().collect::<Vec<_>>().into_boxed_slice(),
        })
    }

    /// Composes a captured environment with supplied specialization information.
    ///
    /// Computes substitution closure before conflict checking.
    pub fn compose_with_supplied(
        &self,
        supplied: &Self,
        types: &mut crate::types::store::TypeStore,
        budget: &mut crate::types::outcome::QueryBudget,
        cancellation: &crate::db::CancellationToken,
    ) -> Result<Self, StaticSpecializationEnvironmentError> {
        use std::collections::{BTreeMap, BTreeSet, HashMap};
        use crate::types::substitution::TypeSubstitution;

        if cancellation.is_cancelled() {
            return Err(StaticSpecializationEnvironmentError::Cancelled);
        }
        budget
            .charge_step()
            .map_err(StaticSpecializationEnvironmentError::BudgetExceeded)?;

        // 1. Gather all type bindings from supplied and captured to build substitution closure
        let mut raw_subst_map: HashMap<TypeParameterId, TypeId> = HashMap::new();
        for (param, ty) in self.type_bindings.iter().chain(supplied.type_bindings.iter()) {
            raw_subst_map.insert(*param, *ty);
        }

        // Compute the transitive closure. The pass bound is derived from the
        // number of bindings; recursive type growth fails closed instead of
        // publishing a partially closed substitution.
        let resolved_subst_map = close_substitution_map(
            raw_subst_map,
            types,
            Some(budget),
            Some(cancellation),
        )?;
        let subst = TypeSubstitution::from_bindings(resolved_subst_map);

        // 2. Substitute through type bindings and check conflicts
        let mut final_types = BTreeMap::new();
        for (param, ty) in &self.type_bindings {
            let substituted = subst.try_apply(types, *ty)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            if let Some(&existing) = final_types.get(param) {
                if existing != substituted {
                    return Err(StaticSpecializationEnvironmentError::ConflictingTypeBinding);
                }
            } else {
                final_types.insert(*param, substituted);
            }
        }
        for (param, ty) in &supplied.type_bindings {
            let substituted = subst.try_apply(types, *ty)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            if let Some(&existing) = final_types.get(param) {
                if existing != substituted {
                    return Err(StaticSpecializationEnvironmentError::ConflictingTypeBinding);
                }
            } else {
                final_types.insert(*param, substituted);
            }
        }

        // 3. Substitute through argument bindings and check conflicts
        let mut final_arguments = BTreeMap::new();
        for (param, arg) in &self.argument_bindings {
            let substituted_arg = match arg {
                StaticSpecializationArgument::Kinded(k) => {
                    let substituted = subst.try_apply(types, *k)
                        .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                    StaticSpecializationArgument::Kinded(substituted)
                }
                StaticSpecializationArgument::Value(v) => StaticSpecializationArgument::Value(*v),
            };
            final_arguments.insert(param.clone(), substituted_arg);
        }
        for (param, arg) in &supplied.argument_bindings {
            let substituted_arg = match arg {
                StaticSpecializationArgument::Kinded(k) => {
                    let substituted = subst.try_apply(types, *k)
                        .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                    StaticSpecializationArgument::Kinded(substituted)
                }
                StaticSpecializationArgument::Value(v) => StaticSpecializationArgument::Value(*v),
            };
            if let Some(&existing) = final_arguments.get(param) {
                if existing != substituted_arg {
                    return Err(StaticSpecializationEnvironmentError::ConflictingArgumentBinding);
                }
            } else {
                final_arguments.insert(param.clone(), substituted_arg);
            }
        }

        // 4. Substitute through proof constraints and union
        let mut final_constraints = BTreeSet::new();
        for fact in self.constraints.iter().chain(supplied.constraints.iter()) {
            match fact {
                CanonicalStaticProofFact::Equal(t1, t2) => {
                    let sub_t1 = subst.try_apply(types, *t1)
                        .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                    let sub_t2 = subst.try_apply(types, *t2)
                        .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                    let (min, max) = if sub_t1 <= sub_t2 { (sub_t1, sub_t2) } else { (sub_t2, sub_t1) };
                    final_constraints.insert(CanonicalStaticProofFact::Equal(min, max));
                }
                CanonicalStaticProofFact::Subtype { lower, upper } => {
                    let sub_lower = subst.try_apply(types, *lower)
                        .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                    let sub_upper = subst.try_apply(types, *upper)
                        .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                    final_constraints.insert(CanonicalStaticProofFact::Subtype { lower: sub_lower, upper: sub_upper });
                }
                CanonicalStaticProofFact::Conforms { key } => {
                    final_constraints.insert(CanonicalStaticProofFact::Conforms { key: key.clone() });
                }
            }
        }

        Ok(Self {
            argument_bindings: final_arguments.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            type_bindings: final_types.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            constraints: final_constraints.into_iter().collect::<Vec<_>>().into_boxed_slice(),
        })
    }

    /// Computes the transitive substitution closure from this environment's type bindings.
    pub fn substitution_closure(
        &self,
        types: &mut crate::types::store::TypeStore,
    ) -> Result<crate::types::substitution::TypeSubstitution, StaticSpecializationEnvironmentError> {
        use std::collections::HashMap;
        use crate::types::substitution::TypeSubstitution;

        let mut raw_subst_map: HashMap<TypeParameterId, TypeId> = HashMap::new();
        for (param, ty) in self.type_bindings.iter() {
            raw_subst_map.insert(*param, *ty);
        }

        let resolved_subst_map = close_substitution_map(raw_subst_map, types, None, None)?;
        Ok(TypeSubstitution::from_bindings(resolved_subst_map))
    }
}

fn close_substitution_map(
    mut resolved_subst_map: std::collections::HashMap<TypeParameterId, TypeId>,
    types: &mut crate::types::store::TypeStore,
    mut budget: Option<&mut crate::types::outcome::QueryBudget>,
    cancellation: Option<&crate::db::CancellationToken>,
) -> Result<std::collections::HashMap<TypeParameterId, TypeId>, StaticSpecializationEnvironmentError> {
    use crate::types::substitution::TypeSubstitution;

    let max_iterations = resolved_subst_map.len().saturating_add(1);
    for _ in 0..max_iterations {
        if cancellation.is_some_and(crate::db::CancellationToken::is_cancelled) {
            return Err(StaticSpecializationEnvironmentError::Cancelled);
        }
        if let Some(budget) = budget.as_deref_mut() {
            budget
                .charge_step()
                .map_err(StaticSpecializationEnvironmentError::BudgetExceeded)?;
        }

        let current_subst = TypeSubstitution::from_bindings(resolved_subst_map.clone());
        let mut changed = false;
        for target_ty in resolved_subst_map.values_mut() {
            let new_ty = current_subst
                .try_apply(types, *target_ty)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            if new_ty != *target_ty {
                *target_ty = new_ty;
                changed = true;
            }
        }
        if !changed {
            return Ok(resolved_subst_map);
        }
    }

    Err(StaticSpecializationEnvironmentError::NonConvergentSubstitution)
}

impl From<StaticApplicationEnvironmentKey> for StaticSpecializationEnvironmentKey {
    fn from(key: StaticApplicationEnvironmentKey) -> Self {
        key.to_specialization_key()
    }
}

/// Session-local semantic argument identity for exact static applications.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StaticArgumentIdentity {
    Kinded(TypeId),
    Value(StaticValueRef),
    Residual(StaticResidualRef),
}

impl From<StaticEvalTerm> for StaticArgumentIdentity {
    fn from(term: StaticEvalTerm) -> Self {
        match term {
            StaticEvalTerm::Kinded(t) => Self::Kinded(t),
            StaticEvalTerm::Value(v) => Self::Value(v),
            StaticEvalTerm::Residual(r) => Self::Residual(r),
        }
    }
}

impl From<StaticArgumentIdentity> for StaticEvalTerm {
    fn from(arg: StaticArgumentIdentity) -> Self {
        match arg {
            StaticArgumentIdentity::Kinded(t) => Self::Kinded(t),
            StaticArgumentIdentity::Value(v) => Self::Value(v),
            StaticArgumentIdentity::Residual(r) => Self::Residual(r),
        }
    }
}

impl From<crate::static_eval::machine::StaticArgument> for StaticArgumentIdentity {
    fn from(arg: crate::static_eval::machine::StaticArgument) -> Self {
        match arg {
            crate::static_eval::machine::StaticArgument::Kinded(t) => Self::Kinded(t),
            crate::static_eval::machine::StaticArgument::Value(v) => Self::Value(v),
            crate::static_eval::machine::StaticArgument::Residual(r) => Self::Residual(r),
        }
    }
}

/// Exact semantic key identifying one static definition application.
///
/// Distinct source sites calling the same definition with the same arguments
/// and specialization environment share this identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticApplicationKey {
    pub definition: DeclarationId,
    pub arguments: Box<[StaticArgumentIdentity]>,
    pub environment: StaticApplicationEnvironmentKey,
}

impl StaticApplicationKey {
    pub fn new(
        definition: DeclarationId,
        arguments: impl IntoIterator<Item = impl Into<StaticArgumentIdentity>>,
        environment: StaticApplicationEnvironmentKey,
    ) -> Self {
        Self {
            definition,
            arguments: arguments.into_iter().map(Into::into).collect::<Vec<_>>().into_boxed_slice(),
            environment,
        }
    }
}

/// Evaluator-local memoized result for an exact static application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaticMemoResult {
    Reduced(StaticEvalTerm),
    Symbolic(StaticResidualRef),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::match_semantics::BranchProofEnvironment;
    use crate::static_value::StaticValueStore;
    use crate::types::constraint::TypeConstraint;
    use phalcom_modules::identity::ModuleId;

    #[test]
    fn exact_application_key_equality_and_order_independence() {
        let decl = DeclarationId::new(ModuleId::universe_root(), "Count".into());
        let mut values = StaticValueStore::new();
        let val1 = values.intern_int(42);
        let val2 = values.intern_int(42);
        assert_eq!(val1, val2);

        let key1 = StaticApplicationKey::new(
            decl.clone(),
            [StaticArgumentIdentity::Value(val1)],
            StaticApplicationEnvironmentKey::default(),
        );
        let key2 = StaticApplicationKey::new(
            decl.clone(),
            [StaticArgumentIdentity::Value(val2)],
            StaticApplicationEnvironmentKey::default(),
        );
        assert_eq!(key1, key2, "same definition and value argument produce equal keys");

        let val_diff = values.intern_int(99);
        let key_diff_arg = StaticApplicationKey::new(
            decl.clone(),
            [StaticArgumentIdentity::Value(val_diff)],
            StaticApplicationEnvironmentKey::default(),
        );
        assert_ne!(key1, key_diff_arg, "different arguments produce unequal keys");
    }

    #[test]
    fn specialization_environment_from_branch_proofs_normalizes_equalities() {
        let t1 = TypeId(10);
        let t2 = TypeId(20);

        let mut env_a = BranchProofEnvironment::default();
        env_a.equalities = Box::new([TypeConstraint::Equal(t1, t2)]);

        let mut env_b = BranchProofEnvironment::default();
        env_b.equalities = Box::new([TypeConstraint::Equal(t2, t1)]);

        let key_a = StaticSpecializationEnvironmentKey::from_branch_proofs([&env_a]).unwrap();
        let key_b = StaticSpecializationEnvironmentKey::from_branch_proofs([&env_b]).unwrap();

        assert_eq!(key_a, key_b, "symmetric equality constraints normalize identically");
        assert_eq!(key_a.constraints().len(), 1);
        assert_eq!(key_a.constraints()[0], CanonicalStaticProofFact::Equal(t1, t2));
    }
}
