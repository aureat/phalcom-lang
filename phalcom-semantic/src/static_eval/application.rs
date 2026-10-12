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
}

/// Normalized specialization and proof environment key.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticSpecializationEnvironmentKey {
    pub argument_bindings: Box<[(StaticResidualParameter, StaticSpecializationArgument)]>,
    pub type_bindings: Box<[(TypeParameterId, TypeId)]>,
    pub constraints: Box<[CanonicalStaticProofFact]>,
}

impl StaticSpecializationEnvironmentKey {
    pub fn new(
        type_bindings: Box<[(TypeParameterId, TypeId)]>,
        constraints: Box<[CanonicalStaticProofFact]>,
    ) -> Self {
        Self {
            argument_bindings: Box::new([]),
            type_bindings,
            constraints,
        }
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

    pub fn merge(&self, supplied: &Self) -> Result<Self, StaticSpecializationEnvironmentError> {
        Self::normalized(
            self.argument_bindings.iter().cloned().chain(supplied.argument_bindings.iter().cloned()),
            self.type_bindings.iter().copied().chain(supplied.type_bindings.iter().copied()),
            self.constraints.iter().cloned().chain(supplied.constraints.iter().cloned()),
        )
    }

    pub fn is_empty(&self) -> bool {
        self.argument_bindings.is_empty() && self.type_bindings.is_empty() && self.constraints.is_empty()
    }

    /// Projects canonical facts from active branch proof environments into a normalized key.
    pub fn from_branch_proofs<'a>(
        environments: impl IntoIterator<Item = &'a crate::match_semantics::BranchProofEnvironment>,
    ) -> Result<Self, StaticSpecializationEnvironmentError> {
        let mut bindings = Vec::new();
        let mut constraints = Vec::new();

        for env in environments {
            for (param, ty) in &env.bindings {
                bindings.push((*param, *ty));
            }
            for eq in env.equalities.iter() {
                match eq {
                    crate::types::constraint::TypeConstraint::Equal(t1, t2) => {
                        constraints.push(CanonicalStaticProofFact::Equal(*t1, *t2));
                    }
                    crate::types::constraint::TypeConstraint::Subtype(lower, upper) => {
                        constraints.push(CanonicalStaticProofFact::Subtype { lower: *lower, upper: *upper });
                    }
                    crate::types::constraint::TypeConstraint::HasMember(_, _) => {}
                }
            }
        }
        Self::normalized([], bindings, constraints)
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
    pub environment: StaticSpecializationEnvironmentKey,
}

impl StaticApplicationKey {
    pub fn new(
        definition: DeclarationId,
        arguments: impl IntoIterator<Item = impl Into<StaticArgumentIdentity>>,
        environment: StaticSpecializationEnvironmentKey,
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
            StaticSpecializationEnvironmentKey::default(),
        );
        let key2 = StaticApplicationKey::new(
            decl.clone(),
            [StaticArgumentIdentity::Value(val2)],
            StaticSpecializationEnvironmentKey::default(),
        );
        assert_eq!(key1, key2, "same definition and value argument produce equal keys");

        let val_diff = values.intern_int(99);
        let key_diff_arg = StaticApplicationKey::new(
            decl.clone(),
            [StaticArgumentIdentity::Value(val_diff)],
            StaticSpecializationEnvironmentKey::default(),
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
        assert_eq!(key_a.constraints.len(), 1);
        assert_eq!(key_a.constraints[0], CanonicalStaticProofFact::Equal(t1, t2));
    }
}
