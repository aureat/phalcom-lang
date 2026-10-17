use super::*;
use crate::match_semantics as native;
use crate::types::proof::*;
use crate::types::rigid::{LocalConstraint, LocalType, RigidOrigin};
use phalcom_compiler_artifact::semantic::world as wire;

pub(super) fn export_callable_proofs(
    analysis: &crate::checker::analysis::CallableAnalysis,
    ordinals: &CallableAnalysisOrdinalMap,
    types: &TypeExportContext,
) -> Result<
    (
        Option<wire::CallableProofGraphV1>,
        Vec<wire::CallableMatchResolutionV1>,
        Vec<wire::CallablePatternSiteResolutionV1>,
    ),
    ArtifactError,
> {
    analysis
        .local_proof_graph
        .validate()
        .map_err(|_| referential("published callable proof graph is invalid"))?;
    let mut export = ProofExport { types, ordinals };
    let graph = &analysis.local_proof_graph;
    let graph = if graph.scopes().is_empty() && graph.rigids().is_empty() {
        None
    } else {
        Some(wire::CallableProofGraphV1 {
            owner: to_stable_callable(&analysis.callable),
            scopes: graph
                .scopes()
                .iter()
                .map(|scope| wire::CallableProofScopeV1 {
                    parent: scope.parent.map(|parent| parent.0),
                })
                .collect(),
            rigids: graph
                .rigids()
                .iter()
                .map(|rigid| {
                    Ok(wire::CallableRigidV1 {
                        scope: rigid.scope.0,
                        kind: types.kind(rigid.kind)?,
                        origin: match &rigid.origin {
                            RigidOrigin::Synthetic => wire::CallableRigidOriginV1::Synthetic,
                            RigidOrigin::VariantParameter { variant, parameter } => wire::CallableRigidOriginV1::VariantParameter {
                                variant: to_stable_variant(variant),
                                parameter: types.parameter(*parameter)?,
                            },
                        },
                    })
                })
                .collect::<Result<_, ArtifactError>>()?,
        })
    };
    let matches = analysis
        .match_resolutions
        .iter()
        .map(|(expression, resolution)| {
            if expression != &resolution.expression {
                return Err(referential("callable match expression disagrees with its index key"));
            }
            Ok(wire::CallableMatchResolutionV1 {
                expression: *ordinals
                    .expressions
                    .get(expression)
                    .ok_or_else(|| referential("callable match expression is absent from its owner"))?,
                scrutinee: export_type_knowledge(&resolution.scrutinee, types)?,
                initial_space: export.space(&resolution.initial_space)?,
                arms: resolution
                    .arms
                    .iter()
                    .map(|arm| {
                        Ok(wire::CallableMatchArmV1 {
                            arm_index: arm.arm_index,
                            pattern: export.pattern(&arm.pattern)?,
                            reachable_space: export.space(&arm.reachable_space)?,
                            residual_after: export.space(&arm.residual_after)?,
                            bindings: export.bindings(&arm.bindings)?,
                            proof: export.proof(&arm.proof)?,
                            usefulness: usefulness(arm.usefulness),
                            branch_result: export_type_knowledge(&arm.branch_result, types)?,
                        })
                    })
                    .collect::<Result<_, ArtifactError>>()?,
                result: export_type_knowledge(&resolution.result, types)?,
                exhaustiveness: match &resolution.exhaustiveness {
                    native::ExhaustivenessResult::Proven => wire::CallableExhaustivenessV1::Proven,
                    native::ExhaustivenessResult::Blocked(reason) => wire::CallableExhaustivenessV1::Blocked {
                        reason: export_callable_block_reason(reason, types)?,
                    },
                    native::ExhaustivenessResult::Missing(witnesses) => wire::CallableExhaustivenessV1::Missing {
                        witnesses: witnesses.iter().map(|witness| export.witness(witness)).collect::<Result<_, _>>()?,
                    },
                },
            })
        })
        .collect::<Result<_, ArtifactError>>()?;
    let sites = analysis
        .pattern_site_resolutions
        .iter()
        .map(|(key, site)| {
            if key.kind != site.kind || key.range != site.range {
                return Err(referential("callable pattern site disagrees with its index key"));
            }
            Ok(wire::CallablePatternSiteResolutionV1 {
                ordinal: ordinals.pattern_sites[key],
                kind: match site.kind {
                    native::PatternSiteKind::IfLet => wire::CallablePatternSiteKindV1::IfLet,
                    native::PatternSiteKind::WhileLet => wire::CallablePatternSiteKindV1::WhileLet,
                    native::PatternSiteKind::ForLane => wire::CallablePatternSiteKindV1::ForLane,
                },
                range: span(site.range)?,
                subject: export_type_knowledge(&site.subject, types)?,
                pattern: export.pattern(&site.pattern)?,
                bindings: export.bindings(&site.bindings)?,
                usefulness: usefulness(site.usefulness),
            })
        })
        .collect::<Result<_, ArtifactError>>()?;
    Ok((graph, matches, sites))
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_type_meta::allocation::AllocationBudget;
    use std::sync::Arc;

    #[test]
    fn callable_proof_alpha_renaming_preserves_wire_projection_and_body_fingerprint() {
        use crate::identity::{BindingId, CallableId, DeclarationId, DispatchSide, ModuleId};
        use crate::types::{KindId, RigidArena};
        use phalcom_common::range::SourceRange;
        use phalcom_common::selector::Selector;
        let store = crate::types::TypeStore::new();
        let callable = CallableId::method(
            DeclarationId::new(ModuleId::universe_root(), "AlphaProof".into()),
            Selector::getter("run").unwrap(),
            DispatchSide::Instance,
        );
        let build = |noise: usize, independent: bool| {
            let mut arena = RigidArena::new();
            for _ in 0..noise {
                let scope = arena.fresh_scope(None);
                arena.fresh(scope, KindId::TYPE, RigidOrigin::Synthetic);
            }
            let scope = arena.fresh_scope(None);
            let first = arena.fresh(scope, KindId::TYPE, RigidOrigin::Synthetic);
            let second = if independent {
                let scope = arena.fresh_scope(None);
                arena.fresh(scope, KindId::TYPE, RigidOrigin::Synthetic)
            } else {
                first
            };
            let range = SourceRange { start: 0, end: 2 };
            let knowledge = TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence);
            let sites = [(
                native::PatternSiteKey::new(native::PatternSiteKind::IfLet, range),
                native::PatternSiteResolution {
                    kind: native::PatternSiteKind::IfLet,
                    range,
                    subject: knowledge.clone(),
                    pattern: native::PatternResolution::Wildcard,
                    bindings: vec![native::PatternBindingResolution {
                        binding: BindingId(0),
                        name: "value".into(),
                        knowledge: knowledge.clone(),
                        local_type: Some(LocalType::Union(vec![LocalType::Rigid(first), LocalType::Rigid(second)].into_boxed_slice())),
                        source: range,
                    }]
                    .into_boxed_slice(),
                    usefulness: native::PatternUsefulness::Useful,
                },
            )]
            .into();
            let proofs = crate::types::proof::freeze_callable_proofs(&arena, &store, &native::MatchResolutionIndex::new(), &sites).unwrap();
            let mut analysis = super::super::callable_analysis_status_tests::empty_callable_analysis(callable.clone());
            analysis
                .bindings
                .insert_source(
                    BindingId(0),
                    crate::checker::BindingState::new(BindingId(0), "value", range, None, knowledge, false),
                )
                .unwrap();
            analysis
                .binding_origins
                .insert_source(BindingId(0), crate::checker::analysis::CallableBindingOrigin::BodyLocal)
                .unwrap();
            analysis.match_resolutions = Arc::new(proofs.matches);
            analysis.pattern_site_resolutions = Arc::new(proofs.sites);
            analysis.local_proof_graph = Arc::new(proofs.graph);
            analysis
        };
        let first = build(0, true);
        let renamed = build(19, true);
        let shared = build(7, false);
        let mut types = TypeExportContext::new(&store);
        crate::artifact::roots::register_callable_analysis_roots(&mut types, &first, &store).unwrap();
        types.close_graph(&store, &Default::default()).unwrap();
        let project = |analysis| export_callable_proofs(analysis, &CallableAnalysisOrdinalMap::new(&callable, analysis).unwrap(), &types).unwrap();
        assert_eq!(project(&first), project(&renamed));
        assert_ne!(project(&first), project(&shared));
        let fingerprint = crate::db::fingerprint::callable_body_product_fingerprint;
        assert_eq!(fingerprint(&first), fingerprint(&renamed));
        assert_ne!(fingerprint(&first), fingerprint(&shared));
    }

    #[test]
    fn callable_proof_source_publication_round_trips_gadt_authority_and_budget_rollback() {
        let source: Arc<str> = Arc::from(
            r#"
data Inner<U> { @variant Boxed(_ value: U) -> Inner<U> }
data Expr<T> { @variant Wrap<U>(_ value: Inner<U>) -> Expr<List<U>> }
class Eval { eval<T>(_ value: Expr<T>) { match value { Expr::Wrap(Inner::Boxed(x)) => 1 Expr::Wrap(Inner::Boxed(y)) => 2 } } }
"#,
        );
        let parsed = phalcom_ast::parse_source(&source, 0).unwrap();
        let module = crate::identity::ModuleId::universe(crate::identity::ModulePath::from_components(vec![
            crate::identity::ModuleComponent::from_identifier("proof").unwrap(),
        ]));
        let analysis = crate::analyze_single_module(module, source, Arc::new(parsed)).unwrap();
        let snapshot = &analysis.snapshot;
        let callable = snapshot
            .callable_analyses
            .values()
            .find(|analysis| !analysis.match_resolutions.is_empty())
            .expect("real source produces a retained match");
        assert!(callable.internal_incidents.is_empty(), "{:?}", callable.internal_incidents);
        assert!(!callable.local_proof_graph.rigids().is_empty());
        let mut authorities = BTreeMap::new();
        for (binding, state) in &callable.bindings {
            if let Some(site) = snapshot.source_index.source_site_for_binding(&callable.callable, *binding) {
                authorities.insert(
                    *binding,
                    CallableBindingSourceAuthority::Source {
                        site: site.id.clone(),
                        range: site.range,
                    },
                );
            } else if state.parameter.is_some() {
                authorities.insert(*binding, CallableBindingSourceAuthority::GeneratedReceiver);
            }
        }
        let limits = ValidationLimits::default();
        let full_types = TypeExportContext::new_full(&snapshot.store).unwrap();
        let full_record = export_callable_analysis_record(&callable.callable, callable, &authorities, &full_types, &limits).unwrap();
        let full_graph = full_types.project_graph(&snapshot.store, &limits).unwrap();
        let contracts = export_structural_primary_contracts(
            &full_types,
            &snapshot.data_semantics,
            &snapshot.enum_semantics,
            &snapshot.declarations,
            &snapshot.hierarchy,
            &snapshot.type_aliases,
            &limits,
        )
        .unwrap();
        full_record.validate_proof_authority(&full_graph, &contracts, &limits).unwrap();
        let mut types = TypeExportContext::new(&snapshot.store);
        crate::artifact::roots::register_callable_analysis_roots(&mut types, callable, &snapshot.store).unwrap();
        types.close_graph(&snapshot.store, &limits).unwrap();
        let record = export_callable_analysis_record(&callable.callable, callable, &authorities, &types, &limits).unwrap();
        record.validate_proof_graph(&limits).unwrap();
        let graph = types.project_graph(&snapshot.store, &limits).unwrap();
        let (_fresh_store, relocation) = crate::types::TypeStore::import_type_graph(&graph, &limits).unwrap();
        let mut budget = AllocationBudget::new(1 << 20);
        let restored =
            crate::artifact::import::proof::import_callable_proofs(&record, crate::identity::BodyId(801), &relocation, &limits, &mut budget).unwrap();
        assert!(restored.matches.is_frozen());
        assert!(restored.sites.is_frozen());
        assert_eq!(restored.graph.scopes(), callable.local_proof_graph.scopes());
        assert_eq!(restored.graph.rigids().len(), callable.local_proof_graph.rigids().len());
        let source_match = callable.match_resolutions.values().next().unwrap();
        let restored_match = restored.matches.values().next().unwrap();
        assert_eq!(restored_match.expression.owner, crate::identity::BodyId(801));
        let native::PatternResolution::Variant(source_pattern) = &source_match.arms[0].pattern else {
            panic!("source variant")
        };
        let native::PatternResolution::Variant(restored_pattern) = &restored_match.arms[0].pattern else {
            panic!("restored variant")
        };
        let source_case = source_pattern.candidates[0].case_instantiation.as_ref().unwrap();
        let restored_case = restored_pattern.candidates[0].case_instantiation.as_ref().unwrap();
        assert_eq!(restored_match.arms.len(), 2);
        let native::PatternResolution::Variant(second) = &restored_match.arms[1].pattern else {
            panic!("second observation")
        };
        assert_ne!(restored_case.scope, second.candidates[0].case_instantiation.as_ref().unwrap().scope);
        let native::PatternResolution::Variant(inner) = restored_pattern.candidates[0].fields[0].child.as_ref() else {
            panic!("nested observation")
        };
        assert!(
            restored
                .graph
                .scope_contains(restored_case.scope, inner.candidates[0].case_instantiation.as_ref().unwrap().scope)
        );
        assert_eq!(source_case.scope, restored_case.scope);
        assert_eq!(
            source_case.local_rigids.values().copied().collect::<Vec<_>>(),
            restored_case.local_rigids.values().copied().collect::<Vec<_>>()
        );
        assert!(restored_case.local_rigids.is_frozen());
        assert!(restored_pattern.candidates[0].proof.local_bindings.is_frozen());
        assert_eq!(source_case.result_type.free_rigids(), restored_case.result_type.free_rigids());
        for (source, restored) in callable.local_proof_graph.rigids().iter().zip(restored.graph.rigids()) {
            assert_eq!(restored.kind, relocation.kind(types.kind(source.kind).unwrap()).unwrap());
            match (&source.origin, &restored.origin) {
                (
                    RigidOrigin::VariantParameter { variant, parameter },
                    RigidOrigin::VariantParameter {
                        variant: fresh_variant,
                        parameter: fresh_parameter,
                    },
                ) => {
                    assert_eq!(variant, fresh_variant);
                    assert_eq!(*fresh_parameter, relocation.parameter(types.parameter(*parameter).unwrap()).unwrap());
                }
                (RigidOrigin::Synthetic, RigidOrigin::Synthetic) => {}
                _ => panic!("rigid origin changed"),
            }
        }
        let exact = budget.requested();
        let mut fits = AllocationBudget::new(exact);
        crate::artifact::import::proof::import_callable_proofs(&record, crate::identity::BodyId(0), &relocation, &limits, &mut fits).unwrap();
        assert_eq!(fits.requested(), exact);
        let mut short = AllocationBudget::new(exact - 1);
        assert!(crate::artifact::import::proof::import_callable_proofs(&record, crate::identity::BodyId(0), &relocation, &limits, &mut short).is_err());
        assert_eq!(short.requested(), 0);
        let mut malformed = record.clone();
        malformed.local_proof_graph.as_mut().unwrap().rigids[0].kind = phalcom_compiler_artifact::semantic::type_store::ArtifactKindId(u32::MAX);
        let mut rollback = AllocationBudget::new(1 << 20);
        assert!(crate::artifact::import::proof::import_callable_proofs(&malformed, crate::identity::BodyId(0), &relocation, &limits, &mut rollback).is_err());
        assert_eq!(rollback.requested(), 0);
    }
}

fn span(range: phalcom_common::range::SourceRange) -> Result<phalcom_compiler_artifact::module_world::SpanV1, ArtifactError> {
    Ok(phalcom_compiler_artifact::module_world::SpanV1 {
        start: u64::try_from(range.start).map_err(|_| referential("pattern range start overflows"))?,
        end: u64::try_from(range.end).map_err(|_| referential("pattern range end overflows"))?,
    })
}
fn usefulness(value: native::PatternUsefulness) -> wire::CallablePatternUsefulnessV1 {
    match value {
        native::PatternUsefulness::Useful => wire::CallablePatternUsefulnessV1::Useful,
        native::PatternUsefulness::Redundant => wire::CallablePatternUsefulnessV1::Redundant,
        native::PatternUsefulness::Impossible => wire::CallablePatternUsefulnessV1::Impossible,
    }
}

struct ProofExport<'a> {
    types: &'a TypeExportContext,
    ordinals: &'a CallableAnalysisOrdinalMap,
}
impl ProofExport<'_> {
    fn local(&mut self, term: &FrozenLocalType) -> Result<wire::CallableLocalTypeV1, ArtifactError> {
        Ok(match term {
            LocalType::Canonical(ty) => wire::CallableLocalTypeV1::Canonical { ty: self.types.ty(*ty)? },
            LocalType::Rigid(rigid) => wire::CallableLocalTypeV1::Rigid { rigid: rigid.0 },
            LocalType::Applied { origin, arguments } => wire::CallableLocalTypeV1::Applied {
                origin: Box::new(self.local(origin)?),
                arguments: arguments.iter().map(|term| self.local(term)).collect::<Result<_, _>>()?,
            },
            LocalType::ExactCase { variant, enum_type } => wire::CallableLocalTypeV1::ExactCase {
                variant: to_stable_variant(variant),
                enum_type: Box::new(self.local(enum_type)?),
            },
            LocalType::Union(terms) => wire::CallableLocalTypeV1::Union {
                members: terms.iter().map(|term| self.local(term)).collect::<Result<_, _>>()?,
            },
            LocalType::Tuple(elements) => wire::CallableLocalTypeV1::Tuple {
                elements: elements
                    .iter()
                    .map(|element| {
                        Ok(wire::CallableLocalTupleElementV1 {
                            label: element.label.clone(),
                            ty: self.local(&element.ty)?,
                        })
                    })
                    .collect::<Result<_, ArtifactError>>()?,
            },
            LocalType::Record(fields) => wire::CallableLocalTypeV1::Record {
                fields: fields
                    .iter()
                    .map(|field| {
                        Ok(wire::CallableLocalRecordFieldV1 {
                            key: match &field.key {
                                ProofFieldKey::Symbol(value) => {
                                    phalcom_compiler_artifact::semantic::type_store::FieldKeyV1::Symbol { value: value.to_string() }
                                }
                                ProofFieldKey::String(value) => {
                                    phalcom_compiler_artifact::semantic::type_store::FieldKeyV1::String { value: value.to_string() }
                                }
                                ProofFieldKey::Int(value) => phalcom_compiler_artifact::semantic::type_store::FieldKeyV1::Int { decimal: value.to_string() },
                                ProofFieldKey::Bool(value) => phalcom_compiler_artifact::semantic::type_store::FieldKeyV1::Bool { value: *value },
                            },
                            ty: self.local(&field.ty)?,
                        })
                    })
                    .collect::<Result<_, ArtifactError>>()?,
            },
            LocalType::Callable { parameters, return_type } => wire::CallableLocalTypeV1::Callable {
                parameters: parameters
                    .iter()
                    .map(|parameter| {
                        Ok(wire::CallableLocalParameterV1 {
                            label: parameter.label.clone(),
                            ty: self.local(&parameter.ty)?,
                            rest: export_rest_mode(parameter.rest),
                        })
                    })
                    .collect::<Result<_, ArtifactError>>()?,
                return_type: Box::new(self.local(return_type)?),
            },
        })
    }
    fn constraint(&mut self, constraint: &FrozenLocalConstraint) -> Result<wire::CallableLocalConstraintV1, ArtifactError> {
        Ok(match constraint {
            LocalConstraint::Subtype { lower, upper } => wire::CallableLocalConstraintV1::Subtype {
                lower: self.local(lower)?,
                upper: self.local(upper)?,
            },
            LocalConstraint::Equivalent { left, right } => wire::CallableLocalConstraintV1::Equivalent {
                left: self.local(left)?,
                right: self.local(right)?,
            },
        })
    }
    fn proof(&mut self, proof: &FrozenBranchProof) -> Result<wire::CallableBranchProofV1, ArtifactError> {
        let mut bindings = proof
            .bindings
            .iter()
            .map(|(parameter, ty)| {
                Ok(wire::CallableProofBindingV1 {
                    parameter: self.types.parameter(*parameter)?,
                    ty: self.types.ty(*ty)?,
                })
            })
            .collect::<Result<Vec<_>, ArtifactError>>()?;
        bindings.sort_unstable_by_key(|binding| binding.parameter);
        let mut local_bindings = proof
            .local_bindings
            .iter()
            .map(|(parameter, ty)| {
                Ok(wire::CallableLocalBindingV1 {
                    parameter: self.types.parameter(*parameter)?,
                    ty: self.local(ty)?,
                })
            })
            .collect::<Result<Vec<_>, ArtifactError>>()?;
        local_bindings.sort_unstable_by_key(|binding| binding.parameter);
        Ok(wire::CallableBranchProofV1 {
            bindings,
            local_bindings,
            equalities: proof
                .equalities
                .iter()
                .map(|constraint| {
                    Ok(match constraint {
                        crate::types::TypeConstraint::Equal(left, right) => wire::CallableProofConstraintV1::Equal {
                            left: self.types.ty(*left)?,
                            right: self.types.ty(*right)?,
                        },
                        crate::types::TypeConstraint::Subtype(lower, upper) => wire::CallableProofConstraintV1::Subtype {
                            lower: self.types.ty(*lower)?,
                            upper: self.types.ty(*upper)?,
                        },
                        crate::types::TypeConstraint::HasMember(ty, selector) => wire::CallableProofConstraintV1::HasMember {
                            ty: self.types.ty(*ty)?,
                            selector: export_exact_selector(selector),
                        },
                    })
                })
                .collect::<Result<_, ArtifactError>>()?,
            local_equalities: proof
                .local_equalities
                .iter()
                .map(|constraint| self.constraint(constraint))
                .collect::<Result<_, _>>()?,
        })
    }
    fn case(&mut self, case: &FrozenCaseInstantiation) -> Result<wire::CallableCaseInstantiationV1, ArtifactError> {
        let mut local_rigids = case
            .local_rigids
            .iter()
            .map(|(parameter, rigid)| {
                Ok(wire::CallableCaseRigidV1 {
                    parameter: self.types.parameter(*parameter)?,
                    rigid: rigid.0,
                })
            })
            .collect::<Result<Vec<_>, ArtifactError>>()?;
        local_rigids.sort_unstable_by_key(|binder| binder.parameter);
        Ok(wire::CallableCaseInstantiationV1 {
            variant: to_stable_variant(&case.variant),
            scope: case.scope.0,
            local_rigids,
            payload_types: case.payload_types.iter().map(|term| self.local(term)).collect::<Result<_, _>>()?,
            result_type: self.local(&case.result_type)?,
            constraints: case
                .constraints
                .iter()
                .map(|constraint| self.constraint(constraint))
                .collect::<Result<_, _>>()?,
        })
    }
    fn bindings(&mut self, bindings: &[FrozenPatternBinding]) -> Result<Vec<wire::CallablePatternBindingV1>, ArtifactError> {
        bindings
            .iter()
            .map(|binding| {
                Ok(wire::CallablePatternBindingV1 {
                    binding: *self
                        .ordinals
                        .bindings
                        .get(&binding.binding)
                        .ok_or_else(|| referential("pattern binding is absent from its callable owner"))?,
                    name: binding.name.clone(),
                    knowledge: export_type_knowledge(&binding.knowledge, self.types)?,
                    local_type: binding.local_type.as_ref().map(|term| self.local(term)).transpose()?,
                    source: span(binding.source)?,
                })
            })
            .collect()
    }
    fn pattern(&mut self, pattern: &FrozenPatternResolution) -> Result<wire::CallablePatternV1, ArtifactError> {
        use native::PatternResolution as P;
        Ok(match pattern {
            P::Wildcard => wire::CallablePatternV1::Wildcard,
            P::Char(value) => wire::CallablePatternV1::Char { value: *value as u32 },
            P::Binding { binding, name, knowledge } => wire::CallablePatternV1::Binding {
                binding: *self
                    .ordinals
                    .bindings
                    .get(binding)
                    .ok_or_else(|| referential("pattern binding is absent from its callable owner"))?,
                name: name.clone(),
                knowledge: export_type_knowledge(knowledge, self.types)?,
            },
            P::Variant(variant) => wire::CallablePatternV1::Variant {
                owner: variant.owner.as_ref().map(to_stable_declaration),
                family: variant.family.as_ref().map(|family| wire::AssociatedFamilyIdentityV1 {
                    owner: to_stable_declaration(&family.owner),
                    selector: TypeMemberSelectorV1::Named {
                        name: family.base_name.to_string(),
                    },
                }),
                owner_candidates: variant.owner_candidates.iter().map(to_stable_declaration).collect(),
                selector: match &variant.selector {
                    native::VariantSelectorConstraint::Exact(selector) => wire::CallableVariantSelectorV1::Exact {
                        selector: export_exact_selector(selector),
                    },
                    native::VariantSelectorConstraint::Pattern(pattern) => wire::CallableVariantSelectorV1::Pattern {
                        pattern: export_selector_pattern(pattern),
                    },
                    native::VariantSelectorConstraint::WholeFamily => wire::CallableVariantSelectorV1::WholeFamily,
                },
                candidates: variant
                    .candidates
                    .iter()
                    .map(|candidate| {
                        Ok(wire::CallableVariantCandidateV1 {
                            variant: to_stable_variant(&candidate.variant),
                            exact_case: self.types.ty(candidate.exact_case)?,
                            fields: candidate
                                .fields
                                .iter()
                                .map(|field| {
                                    Ok(wire::CallableFieldPatternV1 {
                                        variant: to_stable_variant(&field.field.variant),
                                        field: field.field.index,
                                        field_type: export_type_knowledge(&field.field_type, self.types)?,
                                        local_type: field.local_type.as_ref().map(|term| self.local(term)).transpose()?,
                                        child: Box::new(self.pattern(&field.child)?),
                                    })
                                })
                                .collect::<Result<_, ArtifactError>>()?,
                            proof: self.proof(&candidate.proof)?,
                            case_instantiation: candidate.case_instantiation.as_ref().map(|case| self.case(case)).transpose()?,
                        })
                    })
                    .collect::<Result<_, ArtifactError>>()?,
                blocked: variant
                    .blocked
                    .as_ref()
                    .map(|reason| export_callable_block_reason(reason, self.types))
                    .transpose()?,
            },
            P::Or(or) => wire::CallablePatternV1::Or {
                alternatives: or.alternatives.iter().map(|pattern| self.pattern(pattern)).collect::<Result<_, _>>()?,
            },
            P::Tuple(elements) => wire::CallablePatternV1::Tuple {
                elements: elements.iter().map(|pattern| self.pattern(pattern)).collect::<Result<_, _>>()?,
            },
            P::List(list) => wire::CallablePatternV1::List {
                prefix: list.prefix.iter().map(|pattern| self.pattern(pattern)).collect::<Result<_, _>>()?,
                rest: list.rest.as_ref().map(|rest| self.pattern(rest).map(Box::new)).transpose()?,
            },
            P::Record(fields) => wire::CallablePatternV1::Record {
                fields: fields
                    .iter()
                    .map(|field| {
                        Ok(wire::CallableRecordPatternV1 {
                            label: field.label.clone(),
                            child: Box::new(self.pattern(&field.child)?),
                        })
                    })
                    .collect::<Result<_, ArtifactError>>()?,
            },
            P::Map(entries) => wire::CallablePatternV1::Map {
                entries: entries
                    .iter()
                    .map(|entry| {
                        Ok(wire::CallableMapPatternV1 {
                            key: match &entry.key {
                                phalcom_ast::ast::MapPatternKey::Symbol(value) => wire::CallableMapPatternKeyV1::Symbol { value: value.clone().into() },
                                phalcom_ast::ast::MapPatternKey::String(value) => wire::CallableMapPatternKeyV1::String { value: value.clone().into() },
                                phalcom_ast::ast::MapPatternKey::Int { digits, radix } => wire::CallableMapPatternKeyV1::Int {
                                    digits: digits.clone().into(),
                                    radix: *radix,
                                },
                            },
                            child: Box::new(self.pattern(&entry.child)?),
                        })
                    })
                    .collect::<Result<_, ArtifactError>>()?,
            },
        })
    }
    fn space(&mut self, space: &native::PatternSpaceSummary) -> Result<wire::CallablePatternSpaceV1, ArtifactError> {
        use native::PatternSpaceSummary as S;
        Ok(match space {
            S::Empty => wire::CallablePatternSpaceV1::Empty,
            S::List => wire::CallablePatternSpaceV1::List,
            S::Char(value) => wire::CallablePatternSpaceV1::Char { value: *value as u32 },
            S::Blocked(reason) => wire::CallablePatternSpaceV1::Blocked {
                reason: export_callable_block_reason(reason, self.types)?,
            },
            S::Opaque(ty) => wire::CallablePatternSpaceV1::Opaque { ty: self.types.ty(*ty)? },
            S::Union(members) => wire::CallablePatternSpaceV1::Union {
                members: members.iter().map(|space| self.space(space)).collect::<Result<_, _>>()?,
            },
            S::Tuple(elements) => wire::CallablePatternSpaceV1::Tuple {
                elements: elements.iter().map(|space| self.space(space)).collect::<Result<_, _>>()?,
            },
            S::Variant { variant, exact_case, fields } => wire::CallablePatternSpaceV1::Variant {
                variant: to_stable_variant(variant),
                exact_case: self.types.ty(*exact_case)?,
                fields: fields.iter().map(|space| self.space(space)).collect::<Result<_, _>>()?,
            },
        })
    }
    fn witness(&mut self, witness: &native::CoverageWitness) -> Result<wire::CallableCoverageWitnessV1, ArtifactError> {
        use native::CoverageWitness as W;
        Ok(match witness {
            W::Wildcard => wire::CallableCoverageWitnessV1::Wildcard,
            W::Char(value) => wire::CallableCoverageWitnessV1::Char { value: *value as u32 },
            W::Opaque(ty) => wire::CallableCoverageWitnessV1::Opaque { ty: self.types.ty(*ty)? },
            W::Tuple(elements) => wire::CallableCoverageWitnessV1::Tuple {
                elements: elements.iter().map(|witness| self.witness(witness)).collect::<Result<_, _>>()?,
            },
            W::List(elements) => wire::CallableCoverageWitnessV1::List {
                elements: elements.iter().map(|witness| self.witness(witness)).collect::<Result<_, _>>()?,
            },
            W::Variant { variant, exact_case, fields } => wire::CallableCoverageWitnessV1::Variant {
                variant: to_stable_variant(variant),
                exact_case: self.types.ty(*exact_case)?,
                fields: fields.iter().map(|witness| self.witness(witness)).collect::<Result<_, _>>()?,
            },
        })
    }
}
