use super::*;
use crate::match_semantics as native;
use crate::types::TypeKnowledge;
use crate::types::proof::*;
use crate::types::rigid::{LocalCallableParameter, LocalConstraint, LocalRecordField, LocalTupleElement, LocalType, RigidOrigin};
use phalcom_compiler_artifact::semantic::world as wire;

pub(in crate::artifact) struct ImportedCallableProofs {
    pub graph: Arc<CallableProofGraph>,
    pub matches: Arc<FrozenMatchResolutionIndex>,
    pub sites: Arc<FrozenPatternSiteResolutionIndex>,
}

pub(in crate::artifact) fn import_callable_proofs(
    record: &wire::CallableAnalysisRecordV1,
    body: crate::identity::BodyId,
    relocation: &TypeRelocation,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCallableProofs, ArtifactError> {
    record.validate_proof_graph(limits)?;
    let mut child = allocations.child(allocations.remaining());
    let mut import = ProofImport {
        relocation,
        allocations: &mut child,
    };
    let graph = if let Some(graph) = &record.local_proof_graph {
        CallableProofGraph {
            scopes: import.map(&graph.scopes, |_, scope| {
                Ok(CallableProofScope {
                    parent: scope.parent.map(ProofScopeOrdinal),
                })
            })?,
            rigids: import.map(&graph.rigids, |import, rigid| {
                Ok(CallableRigid {
                    scope: ProofScopeOrdinal(rigid.scope),
                    kind: import.relocation.kind(rigid.kind)?,
                    origin: match &rigid.origin {
                        wire::CallableRigidOriginV1::Synthetic => RigidOrigin::Synthetic,
                        wire::CallableRigidOriginV1::VariantParameter { variant, parameter } => RigidOrigin::VariantParameter {
                            variant: import_stable_variant_with_allocation_budget(variant, import.allocations)?,
                            parameter: import.relocation.parameter(*parameter)?,
                        },
                    },
                })
            })?,
        }
    } else {
        CallableProofGraph::default()
    };
    graph.validate().map_err(|_| referential("imported callable proof graph is invalid"))?;
    let graph = import.allocations.arc(graph).map_err(allocation_error)?;
    let rows = import.map(&record.match_resolutions, |import, resolution| {
        let expression = crate::identity::ExpressionId::new(body, crate::identity::LocalExpressionId(resolution.expression));
        Ok((
            expression,
            native::MatchResolution {
                expression,
                scrutinee: import.knowledge(&resolution.scrutinee)?,
                initial_space: import.space(&resolution.initial_space)?,
                arms: import.map_boxed(&resolution.arms, |import, arm| {
                    Ok(native::MatchArmResolution {
                        arm_index: arm.arm_index,
                        pattern: import.pattern(&arm.pattern)?,
                        reachable_space: import.space(&arm.reachable_space)?,
                        residual_after: import.space(&arm.residual_after)?,
                        bindings: import.bindings(&arm.bindings)?,
                        proof: import.proof(&arm.proof)?,
                        usefulness: usefulness(&arm.usefulness),
                        branch_result: import.knowledge(&arm.branch_result)?,
                    })
                })?,
                result: import.knowledge(&resolution.result)?,
                exhaustiveness: match &resolution.exhaustiveness {
                    wire::CallableExhaustivenessV1::Proven => native::ExhaustivenessResult::Proven,
                    wire::CallableExhaustivenessV1::Blocked { reason } => native::ExhaustivenessResult::Blocked(import.reason(reason)?),
                    wire::CallableExhaustivenessV1::Missing { witnesses } => {
                        native::ExhaustivenessResult::Missing(import.map_boxed(witnesses, |import, witness| import.witness(witness))?)
                    }
                },
            },
        ))
    })?;
    let matches = import_analysis_map(rows, import.allocations, "imported match expressions are not canonical")?;
    let matches = import.allocations.arc(matches).map_err(allocation_error)?;
    let rows = import.map(&record.pattern_site_resolutions, |import, site| {
        let kind = match site.kind {
            wire::CallablePatternSiteKindV1::IfLet => native::PatternSiteKind::IfLet,
            wire::CallablePatternSiteKindV1::WhileLet => native::PatternSiteKind::WhileLet,
            wire::CallablePatternSiteKindV1::ForLane => native::PatternSiteKind::ForLane,
        };
        let range = import_callable_range(&site.range)?;
        Ok((
            native::PatternSiteKey::new(kind, range),
            native::PatternSiteResolution {
                kind,
                range,
                subject: import.knowledge(&site.subject)?,
                pattern: import.pattern(&site.pattern)?,
                bindings: import.bindings(&site.bindings)?,
                usefulness: usefulness(&site.usefulness),
            },
        ))
    })?;
    let sites = import_analysis_map(rows, import.allocations, "imported pattern sites are not canonical")?;
    let sites = import.allocations.arc(sites).map_err(allocation_error)?;
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(ImportedCallableProofs { graph, matches, sites })
}

fn usefulness(value: &wire::CallablePatternUsefulnessV1) -> native::PatternUsefulness {
    match value {
        wire::CallablePatternUsefulnessV1::Useful => native::PatternUsefulness::Useful,
        wire::CallablePatternUsefulnessV1::Redundant => native::PatternUsefulness::Redundant,
        wire::CallablePatternUsefulnessV1::Impossible => native::PatternUsefulness::Impossible,
    }
}
struct ProofImport<'a> {
    relocation: &'a TypeRelocation,
    allocations: &'a mut AllocationBudget,
}
impl ProofImport<'_> {
    fn map<T, U>(&mut self, source: &[T], mut convert: impl FnMut(&mut Self, &T) -> Result<U, ArtifactError>) -> Result<Vec<U>, ArtifactError> {
        let mut result = self.allocations.vector(source.len()).map_err(allocation_error)?;
        for element in source {
            result.push(convert(self, element)?);
        }
        Ok(result)
    }
    fn map_boxed<T, U>(&mut self, source: &[T], convert: impl FnMut(&mut Self, &T) -> Result<U, ArtifactError>) -> Result<Box<[U]>, ArtifactError> {
        let source = self.map(source, convert)?;
        self.allocations.boxed_slice(source).map_err(allocation_error)
    }
    fn knowledge(&mut self, value: &wire::TypeKnowledgeV1) -> Result<TypeKnowledge, ArtifactError> {
        import_type_knowledge_with_allocation_budget(value, self.relocation, self.allocations)
    }
    fn reason(&mut self, value: &wire::CallableBlockReasonV1) -> Result<crate::types::BlockReason, ArtifactError> {
        import_callable_block_reason(value, self.relocation, self.allocations)
    }
    fn label(&mut self, label: &Option<Box<str>>) -> Result<Option<Box<str>>, ArtifactError> {
        label
            .as_ref()
            .map(|label| self.allocations.boxed_str(label).map_err(allocation_error))
            .transpose()
    }
    fn local(&mut self, term: &wire::CallableLocalTypeV1) -> Result<FrozenLocalType, ArtifactError> {
        use wire::CallableLocalTypeV1 as L;
        Ok(match term {
            L::Canonical { ty } => LocalType::Canonical(self.relocation.ty(*ty)?),
            L::Rigid { rigid } => LocalType::Rigid(ProofRigidOrdinal(*rigid)),
            L::Applied { origin, arguments } => {
                let origin = self.local(origin)?;
                let origin = self.allocations.boxed(origin).map_err(allocation_error)?;
                LocalType::Applied {
                    origin,
                    arguments: self.map_boxed(arguments, |import, term| import.local(term))?,
                }
            }
            L::ExactCase { variant, enum_type } => {
                let variant = import_stable_variant_with_allocation_budget(variant, self.allocations)?;
                let enum_type = self.local(enum_type)?;
                LocalType::ExactCase {
                    variant,
                    enum_type: self.allocations.boxed(enum_type).map_err(allocation_error)?,
                }
            }
            L::Union { members } => LocalType::Union(self.map_boxed(members, |import, term| import.local(term))?),
            L::Tuple { elements } => LocalType::Tuple(self.map_boxed(elements, |import, element| {
                Ok(LocalTupleElement {
                    label: import.label(&element.label)?,
                    ty: import.local(&element.ty)?,
                })
            })?),
            L::Record { fields } => LocalType::Record(self.map_boxed(fields, |import, field| {
                Ok(LocalRecordField {
                    key: import.key(&field.key)?,
                    ty: import.local(&field.ty)?,
                })
            })?),
            L::Callable { parameters, return_type } => {
                let parameters = self.map_boxed(parameters, |import, parameter| {
                    Ok(LocalCallableParameter {
                        label: import.label(&parameter.label)?,
                        ty: import.local(&parameter.ty)?,
                        rest: match parameter.rest {
                            0 => phalcom_ast::ast::RestMode::None,
                            1 => phalcom_ast::ast::RestMode::Positional,
                            2 => phalcom_ast::ast::RestMode::Labeled,
                            3 => phalcom_ast::ast::RestMode::Complete,
                            _ => return Err(referential("invalid local callable rest mode")),
                        },
                    })
                })?;
                let return_type = self.local(return_type)?;
                LocalType::Callable {
                    parameters,
                    return_type: self.allocations.boxed(return_type).map_err(allocation_error)?,
                }
            }
        })
    }
    fn key(&mut self, key: &phalcom_compiler_artifact::semantic::type_store::FieldKeyV1) -> Result<ProofFieldKey, ArtifactError> {
        use phalcom_compiler_artifact::semantic::type_store::FieldKeyV1 as K;
        match key {
            K::Bool { value } => Ok(ProofFieldKey::Bool(*value)),
            K::Int { decimal } => Ok(ProofFieldKey::Int(self.allocations.boxed_str(decimal).map_err(allocation_error)?)),
            K::Symbol { value } => Ok(ProofFieldKey::Symbol(self.allocations.boxed_str(value).map_err(allocation_error)?)),
            K::String { value } => Ok(ProofFieldKey::String(self.allocations.boxed_str(value).map_err(allocation_error)?)),
        }
    }
    fn constraint(&mut self, constraint: &wire::CallableLocalConstraintV1) -> Result<FrozenLocalConstraint, ArtifactError> {
        Ok(match constraint {
            wire::CallableLocalConstraintV1::Subtype { lower, upper } => LocalConstraint::Subtype {
                lower: self.local(lower)?,
                upper: self.local(upper)?,
            },
            wire::CallableLocalConstraintV1::Equivalent { left, right } => LocalConstraint::Equivalent {
                left: self.local(left)?,
                right: self.local(right)?,
            },
        })
    }
    fn proof(&mut self, proof: &wire::CallableBranchProofV1) -> Result<FrozenBranchProof, ArtifactError> {
        let mut bindings = self.map(&proof.bindings, |import, binding| {
            Ok((import.relocation.parameter(binding.parameter)?, import.relocation.ty(binding.ty)?))
        })?;
        bindings.sort_unstable_by_key(|binding| binding.0);
        let bindings = import_analysis_map(bindings, self.allocations, "relocated proof bindings are duplicated")?;
        let mut local_bindings = self.map(&proof.local_bindings, |import, binding| {
            Ok((import.relocation.parameter(binding.parameter)?, import.local(&binding.ty)?))
        })?;
        local_bindings.sort_unstable_by_key(|binding| binding.0);
        let local_bindings = import_analysis_map(local_bindings, self.allocations, "relocated local proof bindings are duplicated")?;
        Ok(native::BranchProofEnvironment {
            bindings,
            local_bindings,
            equalities: self.map_boxed(&proof.equalities, |import, equality| {
                Ok(match equality {
                    wire::CallableProofConstraintV1::Equal { left, right } => {
                        crate::types::TypeConstraint::Equal(import.relocation.ty(*left)?, import.relocation.ty(*right)?)
                    }
                    wire::CallableProofConstraintV1::Subtype { lower, upper } => {
                        crate::types::TypeConstraint::Subtype(import.relocation.ty(*lower)?, import.relocation.ty(*upper)?)
                    }
                    wire::CallableProofConstraintV1::HasMember { ty, selector } => {
                        crate::types::TypeConstraint::HasMember(import.relocation.ty(*ty)?, import_exact_selector_with_budget(selector, import.allocations)?)
                    }
                })
            })?,
            local_equalities: self.map_boxed(&proof.local_equalities, |import, constraint| import.constraint(constraint))?,
        })
    }
    fn case(&mut self, case: &wire::CallableCaseInstantiationV1) -> Result<FrozenCaseInstantiation, ArtifactError> {
        let mut local_rigids = self.map(&case.local_rigids, |import, binder| {
            Ok((import.relocation.parameter(binder.parameter)?, ProofRigidOrdinal(binder.rigid)))
        })?;
        local_rigids.sort_unstable_by_key(|binding| binding.0);
        Ok(crate::types::CaseInstantiation {
            variant: import_stable_variant_with_allocation_budget(&case.variant, self.allocations)?,
            scope: ProofScopeOrdinal(case.scope),
            local_rigids: import_analysis_map(local_rigids, self.allocations, "relocated case rigids are duplicated")?,
            payload_types: self.map_boxed(&case.payload_types, |import, term| import.local(term))?,
            result_type: self.local(&case.result_type)?,
            constraints: self.map_boxed(&case.constraints, |import, constraint| import.constraint(constraint))?,
        })
    }
    fn bindings(&mut self, bindings: &[wire::CallablePatternBindingV1]) -> Result<Box<[FrozenPatternBinding]>, ArtifactError> {
        self.map_boxed(bindings, |import, binding| {
            Ok(native::PatternBindingResolution {
                binding: BindingId(binding.binding),
                name: import.allocations.boxed_str(&binding.name).map_err(allocation_error)?,
                knowledge: import.knowledge(&binding.knowledge)?,
                local_type: binding.local_type.as_ref().map(|term| import.local(term)).transpose()?,
                source: import_callable_range(&binding.source)?,
            })
        })
    }
    fn pattern(&mut self, pattern: &wire::CallablePatternV1) -> Result<FrozenPatternResolution, ArtifactError> {
        use wire::CallablePatternV1 as P;
        Ok(match pattern {
            P::Wildcard => native::PatternResolution::Wildcard,
            P::Char { value } => native::PatternResolution::Char(char::from_u32(*value).ok_or_else(|| referential("invalid pattern character"))?),
            P::Binding { binding, name, knowledge } => native::PatternResolution::Binding {
                binding: BindingId(*binding),
                name: self.allocations.boxed_str(name).map_err(allocation_error)?,
                knowledge: self.knowledge(knowledge)?,
            },
            P::Variant {
                owner,
                family,
                owner_candidates,
                selector,
                candidates,
                blocked,
            } => native::PatternResolution::Variant(native::ResolvedVariantPattern {
                owner: owner
                    .as_ref()
                    .map(|owner| import_stable_declaration_with_allocation_budget(owner, self.allocations))
                    .transpose()?,
                family: family
                    .as_ref()
                    .map(|family| {
                        Ok::<_, ArtifactError>(crate::identity::VariantFamilyId::new(
                            import_stable_declaration_with_allocation_budget(&family.owner, self.allocations)?,
                            match &family.selector {
                                phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1::Named { name } => {
                                    self.allocations.boxed_str(name).map_err(allocation_error)?
                                }
                                _ => return Err(referential("variant family requires a named selector")),
                            },
                        ))
                    })
                    .transpose()?,
                owner_candidates: self.map_boxed(owner_candidates, |import, owner| {
                    import_stable_declaration_with_allocation_budget(owner, import.allocations)
                })?,
                selector: match selector {
                    wire::CallableVariantSelectorV1::WholeFamily => native::VariantSelectorConstraint::WholeFamily,
                    wire::CallableVariantSelectorV1::Exact { selector } => {
                        native::VariantSelectorConstraint::Exact(import_exact_selector_with_budget(selector, self.allocations)?)
                    }
                    wire::CallableVariantSelectorV1::Pattern { pattern } => {
                        native::VariantSelectorConstraint::Pattern(import_selector_pattern_with_allocation_budget(pattern, self.allocations)?)
                    }
                },
                candidates: self.map_boxed(candidates, |import, candidate| {
                    Ok(native::ResolvedVariantCandidate {
                        variant: import_stable_variant_with_allocation_budget(&candidate.variant, import.allocations)?,
                        exact_case: import.relocation.ty(candidate.exact_case)?,
                        fields: import.map_boxed(&candidate.fields, |import, field| {
                            let child = import.pattern(&field.child)?;
                            Ok(native::ResolvedFieldPattern {
                                field: crate::identity::VariantFieldId::new(
                                    import_stable_variant_with_allocation_budget(&field.variant, import.allocations)?,
                                    field.field,
                                ),
                                field_type: import.knowledge(&field.field_type)?,
                                local_type: field.local_type.as_ref().map(|term| import.local(term)).transpose()?,
                                child: import.allocations.boxed(child).map_err(allocation_error)?,
                            })
                        })?,
                        proof: import.proof(&candidate.proof)?,
                        case_instantiation: candidate.case_instantiation.as_ref().map(|case| import.case(case)).transpose()?,
                    })
                })?,
                blocked: blocked.as_ref().map(|reason| self.reason(reason)).transpose()?,
            }),
            P::Or { alternatives } => native::PatternResolution::Or(native::ResolvedOrPattern {
                alternatives: self.map_boxed(alternatives, |import, pattern| import.pattern(pattern))?,
            }),
            P::Tuple { elements } => native::PatternResolution::Tuple(self.map_boxed(elements, |import, pattern| import.pattern(pattern))?),
            P::List { prefix, rest } => native::PatternResolution::List(native::ResolvedListPattern {
                prefix: self.map_boxed(prefix, |import, pattern| import.pattern(pattern))?,
                rest: rest
                    .as_ref()
                    .map(|pattern| {
                        let pattern = self.pattern(pattern)?;
                        self.allocations.boxed(pattern).map_err(allocation_error)
                    })
                    .transpose()?,
            }),
            P::Record { fields } => native::PatternResolution::Record(self.map_boxed(fields, |import, field| {
                let child = import.pattern(&field.child)?;
                Ok(native::ResolvedRecordFieldPattern {
                    label: import.allocations.boxed_str(&field.label).map_err(allocation_error)?,
                    child: import.allocations.boxed(child).map_err(allocation_error)?,
                })
            })?),
            P::Map { entries } => native::PatternResolution::Map(self.map_boxed(entries, |import, entry| {
                let child = import.pattern(&entry.child)?;
                Ok(native::ResolvedMapEntryPattern {
                    key: match &entry.key {
                        wire::CallableMapPatternKeyV1::Symbol { value } => {
                            phalcom_ast::ast::MapPatternKey::Symbol(import.allocations.string(value).map_err(allocation_error)?)
                        }
                        wire::CallableMapPatternKeyV1::String { value } => {
                            phalcom_ast::ast::MapPatternKey::String(import.allocations.string(value).map_err(allocation_error)?)
                        }
                        wire::CallableMapPatternKeyV1::Int { digits, radix } => phalcom_ast::ast::MapPatternKey::Int {
                            digits: import.allocations.string(digits).map_err(allocation_error)?,
                            radix: *radix,
                        },
                    },
                    child: import.allocations.boxed(child).map_err(allocation_error)?,
                })
            })?),
        })
    }
    fn space(&mut self, space: &wire::CallablePatternSpaceV1) -> Result<native::PatternSpaceSummary, ArtifactError> {
        use wire::CallablePatternSpaceV1 as S;
        Ok(match space {
            S::Empty => native::PatternSpaceSummary::Empty,
            S::List => native::PatternSpaceSummary::List,
            S::Char { value } => native::PatternSpaceSummary::Char(char::from_u32(*value).ok_or_else(|| referential("invalid pattern-space character"))?),
            S::Blocked { reason } => native::PatternSpaceSummary::Blocked(self.reason(reason)?),
            S::Opaque { ty } => native::PatternSpaceSummary::Opaque(self.relocation.ty(*ty)?),
            S::Union { members } => native::PatternSpaceSummary::Union(self.map_boxed(members, |import, space| import.space(space))?),
            S::Tuple { elements } => native::PatternSpaceSummary::Tuple(self.map_boxed(elements, |import, space| import.space(space))?),
            S::Variant { variant, exact_case, fields } => native::PatternSpaceSummary::Variant {
                variant: import_stable_variant_with_allocation_budget(variant, self.allocations)?,
                exact_case: self.relocation.ty(*exact_case)?,
                fields: self.map_boxed(fields, |import, space| import.space(space))?,
            },
        })
    }
    fn witness(&mut self, witness: &wire::CallableCoverageWitnessV1) -> Result<native::CoverageWitness, ArtifactError> {
        use wire::CallableCoverageWitnessV1 as W;
        Ok(match witness {
            W::Wildcard => native::CoverageWitness::Wildcard,
            W::Char { value } => native::CoverageWitness::Char(char::from_u32(*value).ok_or_else(|| referential("invalid coverage character"))?),
            W::Opaque { ty } => native::CoverageWitness::Opaque(self.relocation.ty(*ty)?),
            W::Tuple { elements } => native::CoverageWitness::Tuple(self.map_boxed(elements, |import, witness| import.witness(witness))?),
            W::List { elements } => native::CoverageWitness::List(self.map_boxed(elements, |import, witness| import.witness(witness))?),
            W::Variant { variant, exact_case, fields } => native::CoverageWitness::Variant {
                variant: import_stable_variant_with_allocation_budget(variant, self.allocations)?,
                exact_case: self.relocation.ty(*exact_case)?,
                fields: self.map_boxed(fields, |import, witness| import.witness(witness))?,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_compiler_artifact::semantic::type_store::FieldKeyV1;

    #[test]
    fn callable_proof_record_keys_import_with_exact_charged_content_backing() {
        let limits = ValidationLimits::default();
        let store = TypeStore::new();
        let types = crate::types::store::artifact::TypeExportContext::new_full(&store).unwrap();
        let graph = types.project_graph(&store, &limits).unwrap();
        let (_, relocation) = TypeStore::import_type_graph(&graph, &limits).unwrap();
        let keys = [
            FieldKeyV1::Symbol { value: "field".into() },
            FieldKeyV1::String { value: "a label".into() },
            FieldKeyV1::Int {
                decimal: "123456789012345678901234567890".into(),
            },
            FieldKeyV1::Bool { value: true },
        ];
        let build = |budget: &mut AllocationBudget| -> Result<Vec<ProofFieldKey>, ArtifactError> {
            let mut child = budget.child(budget.remaining());
            let mut import = ProofImport {
                relocation: &relocation,
                allocations: &mut child,
            };
            let keys = import.map(&keys, |import, key| import.key(key))?;
            budget.commit_child(child).map_err(allocation_error)?;
            Ok(keys)
        };
        let mut budget = AllocationBudget::new(4096);
        let imported = build(&mut budget).unwrap();
        assert_eq!(
            imported,
            vec![
                ProofFieldKey::Symbol("field".into()),
                ProofFieldKey::String("a label".into()),
                ProofFieldKey::Int("123456789012345678901234567890".into()),
                ProofFieldKey::Bool(true)
            ]
        );
        let exact = budget.requested();
        assert_eq!(build(&mut AllocationBudget::new(exact)).unwrap(), imported);
        let mut short = AllocationBudget::new(exact - 1);
        assert!(build(&mut short).is_err());
        assert_eq!(short.requested(), 0);
    }
}
