//! Immutable callable-owned proof identities. Checker allocator IDs stop here.

use super::TypeStore;
use super::case_instantiation::CaseInstantiation;
use super::id::{KindId, RigidScopeId, RigidTypeVariableId};
use super::rigid::{LocalCallableParameter, LocalConstraint, LocalRecordField, LocalTupleElement, LocalType, RigidArena, RigidOrigin};
use crate::match_semantics::*;
use std::collections::BTreeMap;

/// An ordinal is meaningful only in its enclosing callable proof graph.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProofScopeOrdinal(pub u32);

/// Distinct from the checker's query-local `RigidTypeVariableId`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProofRigidOrdinal(pub u32);

/// Content-owned frozen keys avoid unchecked Arc-slice and big-integer allocation
/// during proof import. Integers use their canonical decimal spelling.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ProofFieldKey {
    Symbol(Box<str>),
    String(Box<str>),
    Int(Box<str>),
    Bool(bool),
}

pub type FrozenLocalType = LocalType<ProofRigidOrdinal, ProofFieldKey>;
pub type FrozenLocalConstraint = LocalConstraint<ProofRigidOrdinal, ProofFieldKey>;
pub type FrozenCaseInstantiation = CaseInstantiation<ProofRigidOrdinal, ProofScopeOrdinal, ProofFieldKey>;
pub type FrozenPatternResolution = PatternResolution<ProofRigidOrdinal, ProofScopeOrdinal, ProofFieldKey>;
pub type FrozenMatchResolution = MatchResolution<ProofRigidOrdinal, ProofScopeOrdinal, ProofFieldKey>;
pub type FrozenMatchArmResolution = MatchArmResolution<ProofRigidOrdinal, ProofScopeOrdinal, ProofFieldKey>;
pub type FrozenBranchProof = BranchProofEnvironment<ProofRigidOrdinal, ProofFieldKey>;
pub type FrozenPatternBinding = PatternBindingResolution<ProofRigidOrdinal, ProofFieldKey>;
pub type FrozenMatchResolutionIndex = MatchResolutionIndex<ProofRigidOrdinal, ProofScopeOrdinal, ProofFieldKey>;
pub type FrozenPatternSiteResolutionIndex = PatternSiteResolutionIndex<ProofRigidOrdinal, ProofScopeOrdinal, ProofFieldKey>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CallableProofScope {
    pub parent: Option<ProofScopeOrdinal>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CallableRigid {
    pub scope: ProofScopeOrdinal,
    pub kind: KindId,
    pub origin: RigidOrigin,
}

/// Definitions for every local rigid referenced by a published callable.
/// The buffers have no public mutation API; imported buffers are budgeted.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct CallableProofGraph {
    pub(crate) scopes: Vec<CallableProofScope>,
    pub(crate) rigids: Vec<CallableRigid>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProofPublicationError {
    MissingScope,
    MissingRigid,
    CyclicScope,
    InvisibleRigid,
    InvalidCaseBinder,
    OrdinalOverflow,
}

impl CallableProofGraph {
    pub fn scopes(&self) -> &[CallableProofScope] {
        &self.scopes
    }
    pub fn rigids(&self) -> &[CallableRigid] {
        &self.rigids
    }

    pub fn scope_contains(&self, outer: ProofScopeOrdinal, inner: ProofScopeOrdinal) -> bool {
        let mut current = Some(inner);
        for _ in 0..self.scopes.len() {
            let Some(id) = current else { return false };
            if id == outer {
                return (id.0 as usize) < self.scopes.len();
            }
            current = self.scopes.get(id.0 as usize).and_then(|scope| scope.parent);
        }
        false
    }

    pub fn validate(&self) -> Result<(), ProofPublicationError> {
        // Publication ordinals are parent-before-child, which also proves acyclicity.
        for (index, scope) in self.scopes.iter().enumerate() {
            if scope.parent.is_some_and(|parent| parent.0 as usize >= index) {
                return Err(ProofPublicationError::CyclicScope);
            }
        }
        for rigid in &self.rigids {
            if rigid.scope.0 as usize >= self.scopes.len() {
                return Err(ProofPublicationError::MissingScope);
            }
        }
        Ok(())
    }
}

pub struct FrozenCallableProofs {
    pub graph: CallableProofGraph,
    pub matches: FrozenMatchResolutionIndex,
    pub sites: FrozenPatternSiteResolutionIndex,
}

/// Freeze only retained semantic occurrences, never arena allocation history.
pub(crate) fn freeze_callable_proofs(
    arena: &RigidArena,
    store: &TypeStore,
    matches: &MatchResolutionIndex,
    sites: &PatternSiteResolutionIndex,
) -> Result<FrozenCallableProofs, ProofPublicationError> {
    let mut freezer = ProofFreezer {
        arena,
        store,
        graph: CallableProofGraph::default(),
        scopes: BTreeMap::new(),
        rigids: BTreeMap::new(),
        visiting: Vec::new(),
    };
    let mut frozen_matches = FrozenMatchResolutionIndex::new();
    for (expression, resolution) in matches {
        let mut arms = Vec::with_capacity(resolution.arms.len());
        for arm in &resolution.arms {
            arms.push(MatchArmResolution {
                arm_index: arm.arm_index,
                pattern: freezer.pattern(&arm.pattern, None)?,
                reachable_space: arm.reachable_space.clone(),
                residual_after: arm.residual_after.clone(),
                bindings: freezer.bindings(&arm.bindings)?,
                proof: freezer.proof(&arm.proof, None)?,
                usefulness: arm.usefulness,
                branch_result: arm.branch_result.clone(),
            });
        }
        frozen_matches
            .insert_source(
                *expression,
                MatchResolution {
                    expression: resolution.expression,
                    scrutinee: resolution.scrutinee.clone(),
                    initial_space: resolution.initial_space.clone(),
                    arms: arms.into_boxed_slice(),
                    result: resolution.result.clone(),
                    exhaustiveness: resolution.exhaustiveness.clone(),
                },
            )
            .expect("new publication map is mutable");
    }
    let mut frozen_sites = FrozenPatternSiteResolutionIndex::new();
    for (key, site) in sites {
        frozen_sites
            .insert_source(
                *key,
                PatternSiteResolution {
                    kind: site.kind,
                    range: site.range,
                    subject: site.subject.clone(),
                    pattern: freezer.pattern(&site.pattern, None)?,
                    bindings: freezer.bindings(&site.bindings)?,
                    usefulness: site.usefulness,
                },
            )
            .expect("new publication map is mutable");
    }
    freezer.graph.validate()?;
    Ok(FrozenCallableProofs {
        graph: freezer.graph,
        matches: frozen_matches,
        sites: frozen_sites,
    })
}

struct ProofFreezer<'a> {
    arena: &'a RigidArena,
    store: &'a TypeStore,
    graph: CallableProofGraph,
    scopes: BTreeMap<RigidScopeId, ProofScopeOrdinal>,
    rigids: BTreeMap<RigidTypeVariableId, ProofRigidOrdinal>,
    visiting: Vec<RigidScopeId>,
}

impl ProofFreezer<'_> {
    fn scope(&mut self, id: RigidScopeId) -> Result<ProofScopeOrdinal, ProofPublicationError> {
        self.scope_at(id, None)
    }

    fn scope_at(&mut self, id: RigidScopeId, enclosing: Option<RigidScopeId>) -> Result<ProofScopeOrdinal, ProofPublicationError> {
        if let Some(ordinal) = self.scopes.get(&id) {
            let ordinal = *ordinal;
            if let Some(enclosing) = enclosing.filter(|parent| *parent != id) {
                let enclosing = self.scope(enclosing)?;
                if !self.graph.scope_contains(enclosing, ordinal) {
                    return Err(ProofPublicationError::InvisibleRigid);
                }
            }
            return Ok(ordinal);
        }
        if self.visiting.contains(&id) {
            return Err(ProofPublicationError::CyclicScope);
        }
        self.visiting.push(id);
        // Query construction can leave a nested observation at an arena root.
        // Its retained recursive pattern supplies the semantic enclosing opening.
        // Explicit arena ancestry is retained, never replaced by this context.
        let parent = self
            .arena
            .scope_parent(id)
            .ok_or(ProofPublicationError::MissingScope)?
            .or(enclosing.filter(|parent| *parent != id));
        let parent = parent.map(|parent| self.scope(parent)).transpose()?;
        self.visiting.pop();
        let ordinal = ProofScopeOrdinal(u32::try_from(self.graph.scopes.len()).map_err(|_| ProofPublicationError::OrdinalOverflow)?);
        self.graph.scopes.push(CallableProofScope { parent });
        self.scopes.insert(id, ordinal);
        Ok(ordinal)
    }

    fn rigid(&mut self, id: RigidTypeVariableId, visible_from: Option<RigidScopeId>) -> Result<ProofRigidOrdinal, ProofPublicationError> {
        let definition = self.arena.variable(id).ok_or(ProofPublicationError::MissingRigid)?.clone();
        let scope = self.scope(definition.scope)?;
        if let Some(visible_from) = visible_from {
            let visible_from = self.scope(visible_from)?;
            if !self.graph.scope_contains(scope, visible_from) {
                return Err(ProofPublicationError::InvisibleRigid);
            }
        }
        if let Some(ordinal) = self.rigids.get(&id) {
            return Ok(*ordinal);
        }
        let ordinal = ProofRigidOrdinal(u32::try_from(self.graph.rigids.len()).map_err(|_| ProofPublicationError::OrdinalOverflow)?);
        self.graph.rigids.push(CallableRigid {
            scope,
            kind: definition.kind,
            origin: definition.origin,
        });
        self.rigids.insert(id, ordinal);
        Ok(ordinal)
    }

    fn local(&mut self, term: &LocalType, scope: Option<RigidScopeId>) -> Result<FrozenLocalType, ProofPublicationError> {
        Ok(match term {
            LocalType::Canonical(ty) => LocalType::Canonical(*ty),
            LocalType::Rigid(id) => LocalType::Rigid(self.rigid(*id, scope)?),
            LocalType::Applied { origin, arguments } => LocalType::Applied {
                origin: Box::new(self.local(origin, scope)?),
                arguments: self.locals(arguments, scope)?,
            },
            LocalType::ExactCase { variant, enum_type } => LocalType::ExactCase {
                variant: variant.clone(),
                enum_type: Box::new(self.local(enum_type, scope)?),
            },
            LocalType::Union(terms) => LocalType::Union(self.locals(terms, scope)?),
            LocalType::Tuple(elements) => LocalType::Tuple(
                elements
                    .iter()
                    .map(|element| {
                        Ok(LocalTupleElement {
                            label: element.label.clone(),
                            ty: self.local(&element.ty, scope)?,
                        })
                    })
                    .collect::<Result<_, _>>()?,
            ),
            LocalType::Record(fields) => LocalType::Record(
                fields
                    .iter()
                    .map(|field| {
                        Ok(LocalRecordField {
                            key: match &field.key {
                                crate::field_key::FieldKey::Symbol(value) => ProofFieldKey::Symbol(value.as_ref().into()),
                                crate::field_key::FieldKey::String(value) => ProofFieldKey::String(value.as_ref().into()),
                                crate::field_key::FieldKey::Int(value) => ProofFieldKey::Int(value.to_string().into()),
                                crate::field_key::FieldKey::Bool(value) => ProofFieldKey::Bool(*value),
                            },
                            ty: self.local(&field.ty, scope)?,
                        })
                    })
                    .collect::<Result<_, _>>()?,
            ),
            LocalType::Callable { parameters, return_type } => LocalType::Callable {
                parameters: parameters
                    .iter()
                    .map(|parameter| {
                        Ok(LocalCallableParameter {
                            label: parameter.label.clone(),
                            ty: self.local(&parameter.ty, scope)?,
                            rest: parameter.rest,
                        })
                    })
                    .collect::<Result<_, _>>()?,
                return_type: Box::new(self.local(return_type, scope)?),
            },
        })
    }

    fn locals(&mut self, terms: &[LocalType], scope: Option<RigidScopeId>) -> Result<Box<[FrozenLocalType]>, ProofPublicationError> {
        terms.iter().map(|term| self.local(term, scope)).collect()
    }

    fn constraint(&mut self, constraint: &LocalConstraint, scope: Option<RigidScopeId>) -> Result<FrozenLocalConstraint, ProofPublicationError> {
        Ok(match constraint {
            LocalConstraint::Subtype { lower, upper } => LocalConstraint::Subtype {
                lower: self.local(lower, scope)?,
                upper: self.local(upper, scope)?,
            },
            LocalConstraint::Equivalent { left, right } => LocalConstraint::Equivalent {
                left: self.local(left, scope)?,
                right: self.local(right, scope)?,
            },
        })
    }

    fn proof(&mut self, proof: &BranchProofEnvironment, scope: Option<RigidScopeId>) -> Result<FrozenBranchProof, ProofPublicationError> {
        Ok(BranchProofEnvironment {
            bindings: proof.bindings.clone(),
            equalities: proof.equalities.clone(),
            local_bindings: proof
                .local_bindings
                .iter()
                .map(|(parameter, ty)| Ok((*parameter, self.local(ty, scope)?)))
                .collect::<Result<_, _>>()?,
            local_equalities: proof
                .local_equalities
                .iter()
                .map(|constraint| self.constraint(constraint, scope))
                .collect::<Result<_, _>>()?,
        })
    }

    fn register_case(&mut self, case: &CaseInstantiation, enclosing: Option<RigidScopeId>) -> Result<ProofScopeOrdinal, ProofPublicationError> {
        let scope = self.scope_at(case.scope, enclosing)?;
        let mut parameters = case.local_rigids.iter().collect::<Vec<_>>();
        parameters.sort_by_key(|(parameter, _)| {
            let data = self.store.type_parameter(**parameter);
            (data.owner.clone(), data.index)
        });
        for (&parameter, &id) in parameters {
            let definition = self.arena.variable(id).ok_or(ProofPublicationError::MissingRigid)?;
            if definition.scope != case.scope
                || definition.kind != self.store.type_parameter(parameter).kind
                || !matches!(&definition.origin, RigidOrigin::VariantParameter { variant, parameter: origin } if variant == &case.variant && *origin == parameter)
            {
                return Err(ProofPublicationError::InvalidCaseBinder);
            }
            self.rigid(id, Some(case.scope))?;
        }
        Ok(scope)
    }

    fn case(&mut self, case: &CaseInstantiation, enclosing: Option<RigidScopeId>) -> Result<FrozenCaseInstantiation, ProofPublicationError> {
        let scope = self.register_case(case, enclosing)?;
        Ok(CaseInstantiation {
            variant: case.variant.clone(),
            scope,
            local_rigids: case
                .local_rigids
                .iter()
                .map(|(parameter, id)| Ok((*parameter, self.rigid(*id, Some(case.scope))?)))
                .collect::<Result<_, _>>()?,
            payload_types: self.locals(&case.payload_types, Some(case.scope))?,
            result_type: self.local(&case.result_type, Some(case.scope))?,
            constraints: case
                .constraints
                .iter()
                .map(|constraint| self.constraint(constraint, Some(case.scope)))
                .collect::<Result<_, _>>()?,
        })
    }

    fn bindings(&mut self, bindings: &[PatternBindingResolution]) -> Result<Box<[FrozenPatternBinding]>, ProofPublicationError> {
        bindings
            .iter()
            .map(|binding| {
                Ok(PatternBindingResolution {
                    binding: binding.binding,
                    name: binding.name.clone(),
                    knowledge: binding.knowledge.clone(),
                    source: binding.source,
                    local_type: binding.local_type.as_ref().map(|term| self.local(term, None)).transpose()?,
                })
            })
            .collect()
    }

    fn pattern(&mut self, pattern: &PatternResolution, enclosing: Option<RigidScopeId>) -> Result<FrozenPatternResolution, ProofPublicationError> {
        Ok(match pattern {
            PatternResolution::Wildcard => PatternResolution::Wildcard,
            PatternResolution::Char(value) => PatternResolution::Char(*value),
            PatternResolution::Binding { binding, name, knowledge } => PatternResolution::Binding {
                binding: *binding,
                name: name.clone(),
                knowledge: knowledge.clone(),
            },
            PatternResolution::Variant(variant) => {
                // Register candidate openings in stable identity order, preserving the checked candidate sequence.
                let mut ordered = variant.candidates.iter().collect::<Vec<_>>();
                ordered.sort_by(|left, right| left.variant.cmp(&right.variant));
                for candidate in ordered {
                    if let Some(case) = &candidate.case_instantiation {
                        self.register_case(case, enclosing)?;
                    }
                }
                let mut candidates = Vec::with_capacity(variant.candidates.len());
                for candidate in &variant.candidates {
                    let case = candidate.case_instantiation.as_ref().map(|case| self.case(case, enclosing)).transpose()?;
                    let scope = candidate.case_instantiation.as_ref().map(|case| case.scope).or(enclosing);
                    let mut fields = Vec::with_capacity(candidate.fields.len());
                    for field in &candidate.fields {
                        fields.push(ResolvedFieldPattern {
                            field: field.field.clone(),
                            field_type: field.field_type.clone(),
                            local_type: field.local_type.as_ref().map(|term| self.local(term, scope)).transpose()?,
                            child: Box::new(self.pattern(&field.child, scope)?),
                        });
                    }
                    candidates.push(ResolvedVariantCandidate {
                        variant: candidate.variant.clone(),
                        exact_case: candidate.exact_case,
                        fields: fields.into_boxed_slice(),
                        proof: self.proof(&candidate.proof, scope)?,
                        case_instantiation: case,
                    });
                }
                PatternResolution::Variant(ResolvedVariantPattern {
                    owner: variant.owner.clone(),
                    family: variant.family.clone(),
                    owner_candidates: variant.owner_candidates.clone(),
                    selector: variant.selector.clone(),
                    candidates: candidates.into_boxed_slice(),
                    blocked: variant.blocked.clone(),
                })
            }
            PatternResolution::Or(or) => PatternResolution::Or(ResolvedOrPattern {
                alternatives: self.patterns(&or.alternatives, enclosing)?,
            }),
            PatternResolution::Tuple(elements) => PatternResolution::Tuple(self.patterns(elements, enclosing)?),
            PatternResolution::List(list) => PatternResolution::List(ResolvedListPattern {
                prefix: self.patterns(&list.prefix, enclosing)?,
                rest: list.rest.as_ref().map(|rest| self.pattern(rest, enclosing).map(Box::new)).transpose()?,
            }),
            PatternResolution::Record(fields) => PatternResolution::Record(
                fields
                    .iter()
                    .map(|field| {
                        Ok(ResolvedRecordFieldPattern {
                            label: field.label.clone(),
                            child: Box::new(self.pattern(&field.child, enclosing)?),
                        })
                    })
                    .collect::<Result<_, _>>()?,
            ),
            PatternResolution::Map(entries) => PatternResolution::Map(
                entries
                    .iter()
                    .map(|entry| {
                        Ok(ResolvedMapEntryPattern {
                            key: entry.key.clone(),
                            child: Box::new(self.pattern(&entry.child, enclosing)?),
                        })
                    })
                    .collect::<Result<_, _>>()?,
            ),
        })
    }

    fn patterns(&mut self, patterns: &[PatternResolution], enclosing: Option<RigidScopeId>) -> Result<Box<[FrozenPatternResolution]>, ProofPublicationError> {
        patterns.iter().map(|pattern| self.pattern(pattern, enclosing)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::BindingId;
    use crate::types::{TypeKnowledge, UnknownReason};
    use phalcom_common::range::SourceRange;

    #[test]
    fn callable_proof_publication_preserves_nested_constructor_visibility() {
        use std::sync::Arc;
        let source: Arc<str> = Arc::from(
            r#"
data Inner<U> { @variant Boxed(_ value: U) -> Inner<U> }
data Outer<T> { @variant Wrap<A>(_ inner: Inner<A>) -> Outer<List<A>> }
class Eval { eval<T>(_ value: Outer<T>) { match value { Outer::Wrap(Inner::Boxed(x)) => 1 } } }
"#,
        );
        let parsed = phalcom_ast::parse_source(&source, 0).unwrap();
        let analysis = crate::analyze_single_module(crate::identity::ModuleId::universe_root(), source, Arc::new(parsed)).unwrap();
        let callable = analysis
            .snapshot
            .callable_analyses
            .values()
            .find(|analysis| !analysis.match_resolutions.is_empty() || !analysis.internal_incidents.is_empty())
            .unwrap();
        assert!(callable.internal_incidents.is_empty(), "{:?}", callable.internal_incidents);
        let pattern = &callable.match_resolutions.values().next().unwrap().arms[0].pattern;
        let PatternResolution::Variant(outer_pattern) = pattern else {
            panic!("outer pattern")
        };
        let outer = outer_pattern.candidates[0].case_instantiation.as_ref().unwrap();
        let PatternResolution::Variant(inner_pattern) = outer_pattern.candidates[0].fields[0].child.as_ref() else {
            panic!("inner pattern")
        };
        let inner = inner_pattern.candidates[0].case_instantiation.as_ref().unwrap();
        assert!(callable.local_proof_graph.scope_contains(outer.scope, inner.scope));
        assert_ne!(outer.scope, inner.scope);
    }

    fn freeze_history(noise: usize, independent: bool) -> FrozenCallableProofs {
        let mut arena = RigidArena::new();
        for _ in 0..noise {
            let scope = arena.fresh_scope(None);
            arena.fresh(scope, KindId::TYPE, RigidOrigin::Synthetic);
        }
        let parent = arena.fresh_scope(None);
        let first_scope = arena.fresh_scope(Some(parent));
        let first = arena.fresh(first_scope, KindId::TYPE, RigidOrigin::Synthetic);
        let second = if independent {
            let scope = arena.fresh_scope(Some(parent));
            arena.fresh(scope, KindId::TYPE, RigidOrigin::Synthetic)
        } else {
            first
        };
        let range = SourceRange { start: 1, end: 2 };
        let sites = [(
            PatternSiteKey::new(PatternSiteKind::IfLet, range),
            PatternSiteResolution {
                kind: PatternSiteKind::IfLet,
                range,
                subject: TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                pattern: PatternResolution::Wildcard,
                bindings: vec![PatternBindingResolution {
                    binding: BindingId(0),
                    name: "local".into(),
                    knowledge: TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                    source: range,
                    local_type: Some(LocalType::Union(vec![LocalType::Rigid(first), LocalType::Rigid(second)].into_boxed_slice())),
                }]
                .into_boxed_slice(),
                usefulness: PatternUsefulness::Useful,
            },
        )]
        .into();
        freeze_callable_proofs(&arena, &TypeStore::new(), &MatchResolutionIndex::new(), &sites).unwrap()
    }

    #[test]
    fn callable_proof_freeze_ignores_query_allocator_history_and_preserves_independent_nested_openings() {
        let first = freeze_history(0, true);
        let noisy = freeze_history(19, true);
        assert_eq!(first.graph, noisy.graph);
        assert_eq!(first.sites, noisy.sites);
        assert_eq!(first.graph.rigids.len(), 2);
        assert_ne!(first.graph.rigids[0].scope, first.graph.rigids[1].scope);
        assert_eq!(first.graph.scopes.len(), 3);
        assert_eq!(first.graph.scopes[1].parent, Some(ProofScopeOrdinal(0)));
        assert_eq!(first.graph.scopes[2].parent, Some(ProofScopeOrdinal(0)));
        assert_ne!(first.graph, freeze_history(0, false).graph);
    }

    #[test]
    fn callable_proof_freeze_rejects_missing_and_invisible_query_definitions() {
        let mut arena = RigidArena::new();
        let outer = arena.fresh_scope(None);
        let inner = arena.fresh_scope(Some(outer));
        let sibling = arena.fresh_scope(None);
        let rigid = arena.fresh(outer, KindId::TYPE, RigidOrigin::Synthetic);
        let mut freezer = ProofFreezer {
            arena: &arena,
            store: &TypeStore::new(),
            graph: Default::default(),
            scopes: Default::default(),
            rigids: Default::default(),
            visiting: Vec::new(),
        };
        assert_eq!(freezer.rigid(rigid, Some(inner)), Ok(ProofRigidOrdinal(0)));
        assert_eq!(freezer.rigid(rigid, Some(sibling)), Err(ProofPublicationError::InvisibleRigid));
        assert_eq!(freezer.rigid(RigidTypeVariableId(91), None), Err(ProofPublicationError::MissingRigid));
        assert_eq!(freezer.scope(RigidScopeId(143)), Err(ProofPublicationError::MissingScope));
    }
}
