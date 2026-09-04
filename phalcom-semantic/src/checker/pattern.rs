use phalcom_ast::ast::{Pattern, VariantPattern, VariantPatternMode};
use phalcom_ast::selector::{selector_from_exact_variant_pattern, selector_pattern_from_variant_pattern};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorBase, SelectorKind};
use std::collections::{BTreeMap, BTreeSet};

use crate::checker::context::CheckingContext;
use crate::checker::pattern_space::{PatternSpace, VariantSpace};
use crate::enum_semantics::VariantShape;
use crate::identity::{BindingId, DeclarationId, VariantFamilyId};
use crate::match_semantics::{
    PatternBindingResolution, PatternResolution, ResolvedFieldPattern, ResolvedListPattern, ResolvedOrPattern, ResolvedVariantCandidate,
    ResolvedVariantPattern, VariantSelectorConstraint,
};
use crate::types::annotation::TypeResolver;
use crate::types::denotation::ValueSemanticFact;
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::rigid::LocalType;
use crate::types::store::TypeData;

use crate::checker::coverage::CoverageSubject;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BindingMode {
    Live,
    Detached,
}

/// Resolves an AST pattern against one canonical + query-local subject.
///
/// The returned semantic product contains canonical branch bindings. Candidate
/// and or-pattern alternatives are analyzed with detached temporary identities,
/// then joined and committed exactly once to the surrounding branch scope.
pub(crate) fn resolve_pattern(
    ctx: &mut CheckingContext<'_>,
    pattern: &Pattern,
    subject: &CoverageSubject,
    bindings: &mut Vec<PatternBindingResolution>,
) -> (PatternResolution, PatternSpace) {
    resolve_pattern_with_mode(ctx, pattern, subject, bindings, BindingMode::Live)
}

fn resolve_pattern_with_mode(
    ctx: &mut CheckingContext<'_>,
    pattern: &Pattern,
    subject: &CoverageSubject,
    bindings: &mut Vec<PatternBindingResolution>,
    binding_mode: BindingMode,
) -> (PatternResolution, PatternSpace) {
    match pattern {
        Pattern::Wildcard { .. } => (PatternResolution::Wildcard, PatternSpace::Opaque(subject.canonical)),
        Pattern::Name { name, range } => {
            if let Some((var_res, var_space)) = try_resolve_contextual_singleton(ctx, name, *range, subject) {
                (PatternResolution::Variant(var_res), var_space)
            } else {
                bind_name_pattern(ctx, name, *range, subject, bindings, binding_mode)
            }
        }
        Pattern::Variant(variant_pat) => {
            let (res, space) = resolve_variant_pattern(ctx, variant_pat, subject, bindings, binding_mode);
            (PatternResolution::Variant(res), space)
        }
        Pattern::Or { alternatives, range } => {
            let mut resolved_alternatives = Vec::with_capacity(alternatives.len());
            let mut alternative_spaces = Vec::with_capacity(alternatives.len());
            let mut alternative_bindings = Vec::with_capacity(alternatives.len());
            let mut engine = crate::checker::coverage::CoverageEngine::new(subject.clone());
            let mut prior_coverage_alts = Vec::new();

            for alternative in alternatives {
                let mut local_bindings = Vec::new();
                let (resolution, space) = resolve_pattern_with_mode(ctx, alternative, subject, &mut local_bindings, BindingMode::Detached);

                let cov_alt = coverage_pattern_for_resolution(engine.arena_mut(), &resolution);
                let usefulness = engine.check_or_alternative(
                    ctx.declarations,
                    ctx.store,
                    &ctx.hierarchy,
                    &mut ctx.rigids,
                    ctx.enum_table,
                    &prior_coverage_alts,
                    cov_alt,
                );
                if usefulness == crate::match_semantics::PatternUsefulness::Redundant {
                    ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        crate::diagnostic::DiagnosticCode::MatchPatternOrRedundant,
                        "redundant or-pattern alternative: earlier alternatives already cover its reachable value space",
                        *range,
                    ));
                }
                prior_coverage_alts.push(cov_alt);

                resolved_alternatives.push(resolution);
                alternative_spaces.push(space);
                alternative_bindings.push(local_bindings);
            }

            let active_bindings: Vec<Vec<PatternBindingResolution>> = alternative_bindings.iter().filter(|alt| !alt.is_empty()).cloned().collect();
            let bindings_to_commit = if active_bindings.is_empty() {
                &alternative_bindings[..]
            } else {
                &active_bindings[..]
            };

            let replacements = commit_shared_bindings(
                ctx,
                bindings_to_commit,
                bindings,
                *range,
                binding_mode,
                crate::diagnostic::DiagnosticCode::MatchPatternOrBindingMismatch,
                "or-pattern alternatives must introduce the same binding names",
            );
            for resolution in &mut resolved_alternatives {
                remap_pattern_bindings(resolution, &replacements);
            }

            let mut covered = PatternSpace::Empty;
            for space in alternative_spaces {
                covered = covered.union(&space);
            }
            (
                PatternResolution::Or(ResolvedOrPattern {
                    alternatives: resolved_alternatives.into_boxed_slice(),
                }),
                covered.normalize(),
            )
        }
        Pattern::Tuple { elements, .. } => {
            let mut tuple_res = Vec::with_capacity(elements.len());
            let mut element_spaces = Vec::with_capacity(elements.len());

            for (i, elem) in elements.iter().enumerate() {
                let elem_ty = match ctx.store.get(subject.canonical) {
                    TypeData::Tuple(elems) => elems.get(i).map(|e| e.ty).unwrap_or(subject.canonical),
                    _ => ctx.core_type(&ctx.core_ids.object.clone()).unwrap_or(subject.canonical),
                };
                let elem_local_ty = match &subject.local {
                    LocalType::Tuple(elems) => elems.get(i).map(|e| e.ty.clone()),
                    _ => None,
                };
                let elem_subject = CoverageSubject::from_parts(elem_ty, elem_local_ty.unwrap_or(LocalType::Canonical(elem_ty)));
                let (elem_res, elem_space) = resolve_pattern_with_mode(ctx, elem, &elem_subject, bindings, binding_mode);
                tuple_res.push(elem_res);
                element_spaces.push(elem_space);
            }

            (
                PatternResolution::Tuple(tuple_res.into_boxed_slice()),
                PatternSpace::Tuple(element_spaces.into_boxed_slice()).normalize(),
            )
        }
        Pattern::List { elements, rest, .. } => {
            let elem_ty = ctx
                .store
                .applied_nominal_parts(subject.canonical)
                .and_then(|(declaration, arguments)| (declaration == ctx.core_ids.list && arguments.len() == 1).then(|| arguments[0]))
                .or_else(|| ctx.core_type(&ctx.core_ids.object.clone()))
                .unwrap_or(subject.canonical);
            let elem_local_ty = match &subject.local {
                LocalType::Applied { arguments, .. } if !arguments.is_empty() => Some(arguments[0].clone()),
                _ => None,
            };
            let mut prefix_res = Vec::with_capacity(elements.len());
            let mut prefix_spaces = Vec::with_capacity(elements.len());
            for elem in elements {
                let elem_subject = CoverageSubject::from_parts(elem_ty, elem_local_ty.clone().unwrap_or(LocalType::Canonical(elem_ty)));
                let (elem_res, elem_space) = resolve_pattern_with_mode(ctx, elem, &elem_subject, bindings, binding_mode);
                prefix_res.push(elem_res);
                prefix_spaces.push(elem_space);
            }
            let (rest_res, rest_space) = rest.as_ref().map_or((None, None), |rest_pattern| {
                // `*rest` binds the remaining sequence, not one element. Keep
                // its expected type at the canonical list root so binding
                // knowledge and residual-space elimination agree.
                let (rest_resolution, rest_space) = resolve_pattern_with_mode(ctx, rest_pattern, subject, bindings, binding_mode);
                (Some(Box::new(rest_resolution)), Some(rest_space))
            });

            (
                PatternResolution::List(ResolvedListPattern {
                    prefix: prefix_res.into_boxed_slice(),
                    rest: rest_res,
                }),
                PatternSpace::List(crate::checker::pattern_space::ListSpace {
                    prefix: prefix_spaces.into_boxed_slice(),
                    rest: rest_space.map(Box::new),
                }),
            )
        }
        Pattern::Record { entries, range } => resolve_record_pattern(ctx, entries, *range, subject, bindings, binding_mode),
        Pattern::Map { entries, range } => resolve_map_pattern(ctx, entries, *range, subject, bindings, binding_mode),
    }
}

fn resolve_record_pattern(
    ctx: &mut CheckingContext<'_>,
    entries: &[phalcom_ast::ast::RecordPatternEntry],
    _range: SourceRange,
    subject: &CoverageSubject,
    bindings: &mut Vec<PatternBindingResolution>,
    binding_mode: BindingMode,
) -> (PatternResolution, PatternSpace) {
    let known_row = match ctx.store.get(subject.canonical).clone() {
        TypeData::Record(row_id) => Some(ctx.store.record_row(row_id).clone()),
        _ => None,
    };
    let mut resolved = Vec::with_capacity(entries.len());
    let mut fields = Vec::with_capacity(entries.len());
    let mut impossible = false;

    for entry in entries {
        let field_ty = match &known_row {
            Some(row) => match row.find_field(&entry.label) {
                Some(ty) => Some(ty),
                None if matches!(row.tail, crate::types::row::RecordRowTail::Closed) => {
                    impossible = true;
                    ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        crate::diagnostic::DiagnosticCode::MatchPatternFieldMismatch,
                        format!("record field `{}` is not present in scrutinee type", entry.label),
                        entry.range,
                    ));
                    None
                }
                None => None,
            },
            None => None,
        };
        let child_ty = field_ty.unwrap_or_else(|| conservative_pattern_type(ctx, subject.canonical));
        let child_subject = record_field_subject(subject, &entry.label, child_ty);
        let (child, child_space) = resolve_pattern_with_mode(ctx, &entry.pattern, &child_subject, bindings, binding_mode);
        resolved.push(crate::match_semantics::ResolvedRecordFieldPattern {
            label: entry.label.clone().into_boxed_str(),
            child: Box::new(child),
        });
        fields.push((entry.label.clone().into_boxed_str(), child_space));
    }

    let resolution = PatternResolution::Record(resolved.into_boxed_slice());
    if impossible {
        (resolution, PatternSpace::Empty)
    } else {
        (
            resolution,
            PatternSpace::Record(crate::checker::pattern_space::RecordSpace {
                ty: subject.canonical,
                fields: fields.into_boxed_slice(),
            }),
        )
    }
}

fn resolve_map_pattern(
    ctx: &mut CheckingContext<'_>,
    entries: &[phalcom_ast::ast::MapPatternEntry],
    _range: SourceRange,
    subject: &CoverageSubject,
    bindings: &mut Vec<PatternBindingResolution>,
    binding_mode: BindingMode,
) -> (PatternResolution, PatternSpace) {
    let map_value_ty = ctx.store.applied_nominal_parts(subject.canonical).and_then(|(declaration, arguments)| {
        if declaration == ctx.core_ids.map && arguments.len() == 2 {
            arguments.get(1).copied()
        } else {
            None
        }
    });
    let mut resolved = Vec::with_capacity(entries.len());
    let mut spaces = Vec::with_capacity(entries.len());

    for entry in entries {
        let child_ty = map_value_ty.unwrap_or_else(|| conservative_pattern_type(ctx, subject.canonical));
        let child_subject = CoverageSubject::canonical(child_ty);
        let (child, child_space) = resolve_pattern_with_mode(ctx, &entry.pattern, &child_subject, bindings, binding_mode);
        resolved.push(crate::match_semantics::ResolvedMapEntryPattern {
            key: entry.key.clone(),
            child: Box::new(child),
        });
        spaces.push((entry.key.clone(), child_space));
    }

    (
        PatternResolution::Map(resolved.into_boxed_slice()),
        PatternSpace::Map(crate::checker::pattern_space::MapSpace {
            ty: subject.canonical,
            entries: spaces.into_boxed_slice(),
        }),
    )
}

fn conservative_pattern_type(ctx: &mut CheckingContext<'_>, fallback: TypeId) -> TypeId {
    // Record/map structure is refutable against an opaque receiver. For child
    // recursion we need a canonical type token, but must not claim the field's
    // actual type is known. Object is the least-specific structural fallback
    // available in this checker; its use remains confined to pattern knowledge.
    ctx.core_type(&ctx.core_ids.object.clone()).unwrap_or(fallback)
}

fn record_field_subject(subject: &CoverageSubject, label: &str, canonical: TypeId) -> CoverageSubject {
    let local = match &subject.local {
        LocalType::Record(fields) => fields.iter().find(|field| field.name.as_ref() == label).map(|field| field.ty.clone()),
        _ => None,
    };
    CoverageSubject::from_parts(canonical, local.unwrap_or(LocalType::Canonical(canonical)))
}

fn bind_name_pattern(
    ctx: &mut CheckingContext<'_>,
    name: &str,
    range: SourceRange,
    subject: &CoverageSubject,
    bindings: &mut Vec<PatternBindingResolution>,
    binding_mode: BindingMode,
) -> (PatternResolution, PatternSpace) {
    if let Some(existing) = bindings.iter().find(|binding| binding.name.as_ref() == name).cloned() {
        ctx.emit_diagnostic(
            crate::diagnostic::SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                crate::diagnostic::DiagnosticCode::MatchPatternDuplicateBinding,
                format!("pattern binds `{name}` more than once in the same alternative"),
                range,
            )
            .with_label(existing.source, "first binding in this pattern alternative"),
        );
        return (
            PatternResolution::Binding {
                binding: existing.binding,
                name: name.into(),
                knowledge: existing.knowledge,
            },
            PatternSpace::Opaque(subject.canonical),
        );
    }

    let knowledge = TypeKnowledge::established(subject.canonical, EvidenceOrigin::PatternDecomposition);
    let local_type = (!matches!(subject.local, LocalType::Canonical(_))).then(|| subject.local.clone());
    let binding_id = declare_pattern_binding(ctx, name, range, &knowledge, binding_mode);
    if let (BindingMode::Live, Some(lt)) = (binding_mode, local_type.clone()) {
        ctx.set_local_binding_type(binding_id, lt);
    }
    bindings.push(PatternBindingResolution {
        binding: binding_id,
        name: name.into(),
        knowledge: knowledge.clone(),
        local_type,
        source: range,
    });
    (
        PatternResolution::Binding {
            binding: binding_id,
            name: name.into(),
            knowledge,
        },
        PatternSpace::Opaque(subject.canonical),
    )
}

fn declare_pattern_binding(ctx: &mut CheckingContext<'_>, name: &str, range: SourceRange, knowledge: &TypeKnowledge, mode: BindingMode) -> BindingId {
    match mode {
        BindingMode::Detached => ctx.alloc_binding(),
        BindingMode::Live => {
            let result = ctx.bind_pattern_binding_with_causal(
                name.to_owned(),
                ValueSemanticFact::new(knowledge.clone()),
                range,
                crate::checker::causal::CausalInvalidity::Clean,
            );
            match result {
                crate::checker::binding::BindingDeclarationResult::Inserted(binding)
                | crate::checker::binding::BindingDeclarationResult::Redeclared(binding) => binding,
            }
        }
    }
}

fn commit_shared_bindings(
    ctx: &mut CheckingContext<'_>,
    alternatives: &[Vec<PatternBindingResolution>],
    output: &mut Vec<PatternBindingResolution>,
    range: SourceRange,
    mode: BindingMode,
    mismatch_code: crate::diagnostic::DiagnosticCode,
    mismatch_message: &str,
) -> BTreeMap<String, (BindingId, TypeKnowledge)> {
    let Some(first) = alternatives.first() else {
        return BTreeMap::new();
    };

    let first_names = first.iter().map(|binding| binding.name.to_string()).collect::<BTreeSet<_>>();
    let coherent = alternatives
        .iter()
        .all(|alternative| alternative.iter().map(|binding| binding.name.to_string()).collect::<BTreeSet<_>>() == first_names);
    if !coherent {
        ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            mismatch_code,
            mismatch_message,
            range,
        ));
        return BTreeMap::new();
    }

    let mut replacements = BTreeMap::new();
    for name in first_names {
        let matching = alternatives
            .iter()
            .filter_map(|alternative| alternative.iter().find(|binding| binding.name.as_ref() == name.as_str()))
            .collect::<Vec<_>>();
        if matching.len() != alternatives.len() {
            continue;
        }

        let knowledge = crate::types::evidence::join_type_knowledge(ctx.store, matching.iter().map(|binding| binding.knowledge.clone()).collect::<Vec<_>>());
        let source = matching.first().map(|binding| binding.source).unwrap_or(range);

        if let Some(existing) = output.iter().find(|binding| binding.name.as_ref() == name.as_str()).cloned() {
            ctx.emit_diagnostic(
                crate::diagnostic::SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    crate::diagnostic::DiagnosticCode::MatchPatternDuplicateBinding,
                    format!("pattern binds `{name}` more than once in the same alternative"),
                    source,
                )
                .with_label(existing.source, "first binding in this pattern alternative"),
            );
            replacements.insert(name, (existing.binding, existing.knowledge));
            continue;
        }

        let local_type = joined_local_type(&matching);
        let binding = declare_pattern_binding(ctx, &name, source, &knowledge, mode);
        if let Some(local_type) = local_type.clone() {
            ctx.set_local_binding_type(binding, local_type);
        }
        output.push(PatternBindingResolution {
            binding,
            name: name.clone().into_boxed_str(),
            knowledge: knowledge.clone(),
            local_type,
            source,
        });
        replacements.insert(name, (binding, knowledge));
    }
    replacements
}

fn joined_local_type(matching: &[&PatternBindingResolution]) -> Option<crate::types::rigid::LocalType> {
    let mut joined = None;
    for binding in matching {
        match (&joined, &binding.local_type) {
            (None, candidate) => joined = candidate.clone(),
            (Some(left), Some(right)) if left.alpha_equivalent(right) => {}
            (Some(_), _) => return None,
        }
    }
    joined
}

fn remap_pattern_bindings(resolution: &mut PatternResolution, replacements: &BTreeMap<String, (BindingId, TypeKnowledge)>) {
    match resolution {
        PatternResolution::Wildcard => {}
        PatternResolution::Binding { binding, name, knowledge } => {
            if let Some((replacement, joined)) = replacements.get(name.as_ref()) {
                *binding = *replacement;
                *knowledge = joined.clone();
            }
        }
        PatternResolution::Variant(variant) => {
            for candidate in variant.candidates.iter_mut() {
                for field in candidate.fields.iter_mut() {
                    remap_pattern_bindings(&mut field.child, replacements);
                }
            }
        }
        PatternResolution::Or(or_pattern) => {
            for alternative in or_pattern.alternatives.iter_mut() {
                remap_pattern_bindings(alternative, replacements);
            }
        }
        PatternResolution::Tuple(elements) => {
            for element in elements.iter_mut() {
                remap_pattern_bindings(element, replacements);
            }
        }
        PatternResolution::List(list) => {
            for element in list.prefix.iter_mut() {
                remap_pattern_bindings(element, replacements);
            }
            if let Some(rest) = list.rest.as_mut() {
                remap_pattern_bindings(rest, replacements);
            }
        }
        PatternResolution::Record(fields) => {
            for field in fields.iter_mut() {
                remap_pattern_bindings(&mut field.child, replacements);
            }
        }
        PatternResolution::Map(entries) => {
            for entry in entries.iter_mut() {
                remap_pattern_bindings(&mut entry.child, replacements);
            }
        }
    }
}

fn try_resolve_contextual_singleton(
    ctx: &mut CheckingContext<'_>,
    name: &str,
    range: SourceRange,
    subject: &CoverageSubject,
) -> Option<(ResolvedVariantPattern, PatternSpace)> {
    let enum_table = ctx.enum_table.cloned()?;
    let target_selector = Selector::getter(name).ok()?;

    // A union scrutinee has no single nominal origin. Keep each member's
    // declaration-backed type while looking up its variant; never construct an
    // owner from the contextual leaf spelling.
    let owner_types = nominal_owner_types(ctx.store, subject.canonical);
    let mut matches = Vec::new();
    for (owner, owner_ty) in owner_types {
        let Some(enum_info) = enum_table.enums.get(&owner) else {
            continue;
        };
        let Some(variant_id) = enum_info.variants.iter().find(|variant| variant.selector == target_selector) else {
            continue;
        };
        let Some(variant_info) = enum_table.variants.get(variant_id).cloned() else {
            continue;
        };
        if variant_info.shape != VariantShape::Singleton {
            continue;
        }

        ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::EnumDeclaration(owner.clone()));
        let owner_subject = contextual_owner_subject(ctx.store, subject, &owner, owner_ty);
        let Some(opened) =
            crate::checker::coverage::open_variant_case(ctx.declarations, ctx.store, &ctx.hierarchy, &mut ctx.rigids, &owner_subject, &variant_info)
        else {
            continue;
        };
        matches.push((
            owner,
            variant_id.clone(),
            variant_info,
            opened.exact_case,
            opened.proof,
            opened.case_instantiation,
        ));
    }

    if matches.is_empty() {
        return None;
    }
    matches.sort_by(|left, right| left.1.cmp(&right.1));

    let mut owner_candidates = Vec::new();
    let mut candidates = Vec::with_capacity(matches.len());
    let mut spaces = Vec::with_capacity(matches.len());
    for (owner, variant_id, _variant_info, exact_case, proof, case_instantiation) in matches.iter() {
        if !owner_candidates.contains(owner) {
            owner_candidates.push(owner.clone());
        }
        candidates.push(ResolvedVariantCandidate {
            variant: variant_id.clone(),
            exact_case: *exact_case,
            fields: Box::new([]),
            proof: proof.clone(),
            case_instantiation: Some(case_instantiation.clone()),
        });
        spaces.push(PatternSpace::Variant(VariantSpace {
            variant: variant_id.clone(),
            exact_case: *exact_case,
            fields: Box::new([]),
            proof: proof.clone(),
        }));
    }

    if owner_candidates.len() > 1 {
        let owners = owner_candidates.iter().map(|owner| owner.name.to_string()).collect::<Vec<_>>().join(", ");
        ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            crate::diagnostic::DiagnosticCode::MatchPatternUnresolved,
            format!("contextual variant `{name}` is ambiguous; candidate owners: {owners}"),
            range,
        ));
    }

    let family = if owner_candidates.len() == 1 {
        matches
            .first()
            .and_then(|(_, _, variant_info, _, _, _)| variant_info.family.clone())
            .or_else(|| Some(VariantFamilyId::new(owner_candidates[0].clone(), name)))
    } else {
        None
    };
    let space = match spaces.len() {
        1 => spaces.pop().expect("single contextual variant space exists"),
        _ => PatternSpace::Union(spaces.into_boxed_slice()).normalize(),
    };
    let resolution = ResolvedVariantPattern {
        owner: (owner_candidates.len() == 1).then(|| owner_candidates[0].clone()),
        family,
        owner_candidates: owner_candidates.into_boxed_slice(),
        selector: VariantSelectorConstraint::Exact(target_selector),
        candidates: candidates.into_boxed_slice(),
    };

    Some((resolution, space))
}

/// Preserve query-local terms when a contextual singleton belongs to that
/// nominal owner. Union members without a corresponding local term are
/// independent candidates and may use their declaration-backed canonical type.
fn contextual_owner_subject(store: &crate::types::store::TypeStore, subject: &CoverageSubject, owner: &DeclarationId, owner_ty: TypeId) -> CoverageSubject {
    match &subject.local {
        LocalType::Union(members) => members
            .iter()
            .find(|member| local_nominal_owner(store, member).as_ref() == Some(owner))
            .cloned()
            .map(|member| CoverageSubject::from_parts(owner_ty, member))
            .unwrap_or_else(|| CoverageSubject::canonical(owner_ty)),
        local if local_nominal_owner(store, local).as_ref() == Some(owner) => subject.clone(),
        _ => CoverageSubject::canonical(owner_ty),
    }
}

fn local_nominal_owner(store: &crate::types::store::TypeStore, local: &LocalType) -> Option<DeclarationId> {
    match local {
        LocalType::Canonical(ty) => store.nominal_origin_declaration(*ty).cloned(),
        LocalType::Applied { origin, .. } => local_nominal_owner(store, origin),
        LocalType::ExactCase { enum_type, .. } => local_nominal_owner(store, enum_type),
        _ => None,
    }
}

/// Returns nominal owners and the corresponding expected type for each union
/// member. The type is retained so GADT proof solving stays member-specific.
fn nominal_owner_types(store: &crate::types::store::TypeStore, ty: TypeId) -> Vec<(DeclarationId, TypeId)> {
    fn collect(store: &crate::types::store::TypeStore, ty: TypeId, seen: &mut BTreeSet<TypeId>, owners: &mut Vec<(DeclarationId, TypeId)>) {
        if !seen.insert(ty) {
            return;
        }
        match store.get(ty) {
            TypeData::Union(members) => {
                for member in members.iter().copied() {
                    collect(store, member, seen, owners);
                }
            }
            _ => {
                if let Some(owner) = store.nominal_origin_declaration(ty).cloned() {
                    if !owners.iter().any(|(known, known_ty)| known == &owner && known_ty == &ty) {
                        owners.push((owner, ty));
                    }
                }
            }
        }
    }

    let mut owners = Vec::new();
    collect(store, ty, &mut BTreeSet::new(), &mut owners);
    owners
}

fn variant_selector_constraint(variant_pat: &VariantPattern) -> VariantSelectorConstraint {
    match &variant_pat.mode {
        VariantPatternMode::WholeFamily { .. } => VariantSelectorConstraint::WholeFamily,
        VariantPatternMode::Singleton => {
            let selector =
                selector_from_exact_variant_pattern(variant_pat).unwrap_or_else(|_| Selector::getter(&variant_pat.base).expect("variant getter selector"));
            VariantSelectorConstraint::Exact(selector)
        }
        VariantPatternMode::ExactCall { .. } => {
            let selector = selector_from_exact_variant_pattern(variant_pat)
                .unwrap_or_else(|_| Selector::method(&variant_pat.base, vec![]).expect("variant method selector"));
            VariantSelectorConstraint::Exact(selector)
        }
        VariantPatternMode::CallablePattern { .. } => {
            let pattern = selector_pattern_from_variant_pattern(variant_pat).unwrap_or_else(|_| {
                phalcom_common::selector::SelectorPattern::named(
                    &variant_pat.base,
                    // Callable variant patterns select constructor methods.
                    // A same-named singleton getter is a distinct variant
                    // identity and must remain in the residual space.
                    phalcom_common::selector::SelectorKindPattern::Exact(SelectorKind::Method),
                    vec![],
                    vec![],
                    true,
                )
                .expect("fallback selector pattern")
            });
            VariantSelectorConstraint::Pattern(pattern)
        }
    }
}

fn format_variant_reference(variant_pat: &VariantPattern) -> String {
    let Some(owner) = variant_pat.owner.as_ref() else {
        return variant_pat.base.clone();
    };
    let mut reference = owner.root.clone();
    for member in &owner.members {
        reference.push('.');
        reference.push_str(&member.name);
    }
    reference.push_str("::");
    reference.push_str(&variant_pat.base);
    reference
}

fn resolve_variant_pattern(
    ctx: &mut CheckingContext<'_>,
    variant_pat: &VariantPattern,
    subject: &CoverageSubject,
    bindings: &mut Vec<PatternBindingResolution>,
    binding_mode: BindingMode,
) -> (ResolvedVariantPattern, PatternSpace) {
    let expected_nominal_decl = ctx.store.nominal_origin_declaration(subject.canonical).cloned();
    let expected_owners = nominal_owner_types(ctx.store, subject.canonical);
    let constraint = variant_selector_constraint(variant_pat);

    let owner_decl = if let Some(ref owner_ref) = variant_pat.owner {
        let members = owner_ref.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
        let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, &owner_ref.root, &members) else {
            ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                crate::diagnostic::DiagnosticCode::MatchPatternUnresolved,
                format!("cannot resolve explicit variant owner `{}`", format_variant_reference(variant_pat)),
                variant_pat.range,
            ));
            return (
                ResolvedVariantPattern {
                    owner: None,
                    family: None,
                    owner_candidates: Box::new([]),
                    selector: constraint,
                    candidates: Box::new([]),
                },
                PatternSpace::Empty,
            );
        };
        if !expected_owners.is_empty() && !expected_owners.iter().any(|(expected, _)| expected == &decl) {
            let expected_names = expected_owners.iter().map(|(owner, _)| owner.name.to_string()).collect::<Vec<_>>().join(", ");
            ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                crate::diagnostic::DiagnosticCode::MatchPatternContradictory,
                format!("pattern owner `{}` cannot match scrutinee owner(s) `{expected_names}`", decl.name),
                variant_pat.range,
            ));
        }
        decl
    } else if let Some(decl) = expected_nominal_decl {
        decl
    } else {
        ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            crate::diagnostic::DiagnosticCode::MatchPatternUnresolved,
            format!(
                "variant `{}` has no declaration-backed owner for scrutinee type",
                format_variant_reference(variant_pat)
            ),
            variant_pat.range,
        ));
        return (
            ResolvedVariantPattern {
                owner: None,
                family: None,
                owner_candidates: Box::new([]),
                selector: constraint,
                candidates: Box::new([]),
            },
            PatternSpace::Empty,
        );
    };

    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::EnumDeclaration(owner_decl.clone()));

    let enum_table = ctx.enum_table.cloned();
    let enum_info = enum_table.as_ref().and_then(|table| table.enums.get(&owner_decl).cloned());

    if enum_info.is_none() {
        ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            crate::diagnostic::DiagnosticCode::MatchPatternUnresolved,
            format!("type `{}` is not an enum or cannot be resolved", owner_decl.name),
            variant_pat.range,
        ));
        return (
            ResolvedVariantPattern {
                owner: Some(owner_decl.clone()),
                family: None,
                owner_candidates: Box::new([owner_decl]),
                selector: constraint,
                candidates: Box::new([]),
            },
            PatternSpace::Empty,
        );
    }

    let mut candidate_resolutions = Vec::new();
    let mut candidate_spaces = Vec::new();
    let mut candidate_bindings = Vec::new();

    if let (Some(table), Some(info)) = (&enum_table, &enum_info) {
        let matching_base_variants = info
            .variants
            .iter()
            .filter(|variant| match &variant.selector.base {
                SelectorBase::Named(name) => name == &variant_pat.base,
                _ => false,
            })
            .cloned()
            .collect::<Vec<_>>();

        let mut matched_any_variant = false;
        let mut had_shape_or_label_match = false;
        let mut had_arity_mismatch = false;
        let mut had_field_mismatch = false;

        for variant_id in &matching_base_variants {
            let Some(v_info) = table.variants.get(variant_id) else {
                continue;
            };
            if !matches_variant_info(v_info, &variant_pat.base, &constraint, variant_pat) {
                // Check what kind of mismatch occurred for diagnostic precision
                match &variant_pat.mode {
                    VariantPatternMode::ExactCall { arguments } => {
                        if arguments.len() != v_info.fields.len() {
                            had_arity_mismatch = true;
                        } else {
                            had_field_mismatch = true;
                        }
                    }
                    _ => {}
                }
                continue;
            }
            had_shape_or_label_match = true;

            let Some(opened) = crate::checker::coverage::open_variant_case(ctx.declarations, ctx.store, &ctx.hierarchy, &mut ctx.rigids, &subject, v_info)
            else {
                continue;
            };
            matched_any_variant = true;

            let proof = opened.proof;
            let exact_case = opened.exact_case;
            let case_instantiation = opened.case_instantiation;

            let mut resolved_fields = Vec::new();
            let mut field_spaces = Vec::new();
            let mut local_bindings = Vec::new();

            match &variant_pat.mode {
                VariantPatternMode::ExactCall { arguments } => {
                    for (i, argument) in arguments.iter().enumerate() {
                        let field_semantic = if let Some(ref label) = argument.label {
                            v_info
                                .fields
                                .iter()
                                .enumerate()
                                .find(|(_, field)| field.external_label.as_deref() == Some(label) || field.local_name.as_ref() == label.as_str())
                        } else {
                            v_info.fields.get(i).map(|field| (i, field))
                        };

                        let (field_id, field_type, local_type) = match field_semantic {
                            Some((field_index, field)) => match opened.fields.get(field_index) {
                                Some(field_subject) => (
                                    field.id.clone(),
                                    TypeKnowledge::established(field_subject.canonical, EvidenceOrigin::PatternDecomposition),
                                    Some(field_subject.local.clone()),
                                ),
                                None => (field.id.clone(), TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence), None),
                            },
                            None => (
                                crate::identity::VariantFieldId::new(variant_id.clone(), i as u32),
                                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                                None,
                            ),
                        };

                        let field_subject = field_semantic.and_then(|(field_index, _)| opened.fields.get(field_index).cloned());
                        let (child, field_space) = if let Some(field_subject) = field_subject.as_ref() {
                            resolve_pattern_with_mode(ctx, &argument.pattern, field_subject, &mut local_bindings, BindingMode::Detached)
                        } else {
                            // Invalid opened-field metadata must not invent a canonical
                            // child subject. Retain unknown field evidence and leave child
                            // resolution opaque until the opening invariant is repaired.
                            (PatternResolution::Wildcard, PatternSpace::Opaque(subject.canonical))
                        };
                        resolved_fields.push(ResolvedFieldPattern {
                            field: field_id,
                            field_type,
                            local_type,
                            child: Box::new(child),
                        });
                        field_spaces.push(field_space);
                    }
                }
                VariantPatternMode::CallablePattern { prefix, suffix, .. } => {
                    for field_subject in opened.fields.iter() {
                        field_spaces.push(PatternSpace::Opaque(field_subject.canonical));
                    }

                    for (i, argument) in prefix.iter().enumerate() {
                        let (field_index, field_semantic) = if let Some(ref label) = argument.label {
                            v_info
                                .fields
                                .iter()
                                .enumerate()
                                .find(|(_, field)| field.external_label.as_deref() == Some(label) || field.local_name.as_ref() == label.as_str())
                                .map(|(index, field)| (index, Some(field)))
                                .unwrap_or((i, None))
                        } else {
                            (i, v_info.fields.get(i))
                        };
                        let (field_id, field_type, local_type) = match field_semantic {
                            Some(field) => match opened.fields.get(field_index) {
                                Some(field_subject) => (
                                    field.id.clone(),
                                    TypeKnowledge::established(field_subject.canonical, EvidenceOrigin::PatternDecomposition),
                                    Some(field_subject.local.clone()),
                                ),
                                None => (field.id.clone(), TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence), None),
                            },
                            None => (
                                crate::identity::VariantFieldId::new(variant_id.clone(), field_index as u32),
                                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                                None,
                            ),
                        };
                        let field_subject = opened.fields.get(field_index).cloned();
                        let (child, field_space) = if let Some(field_subject) = field_subject.as_ref() {
                            resolve_pattern_with_mode(ctx, &argument.pattern, field_subject, &mut local_bindings, BindingMode::Detached)
                        } else {
                            // Invalid opened-field metadata must not invent a canonical
                            // child subject. Retain unknown field evidence and leave child
                            // resolution opaque until the opening invariant is repaired.
                            (PatternResolution::Wildcard, PatternSpace::Opaque(subject.canonical))
                        };
                        resolved_fields.push(ResolvedFieldPattern {
                            field: field_id,
                            field_type,
                            local_type,
                            child: Box::new(child),
                        });
                        if field_index < field_spaces.len() {
                            field_spaces[field_index] = field_space;
                        }
                    }

                    for (suffix_index, argument) in suffix.iter().enumerate() {
                        let (field_index, field_semantic) = if let Some(ref label) = argument.label {
                            v_info
                                .fields
                                .iter()
                                .enumerate()
                                .find(|(_, field)| field.external_label.as_deref() == Some(label) || field.local_name.as_ref() == label.as_str())
                                .map(|(index, field)| (index, Some(field)))
                                .unwrap_or((v_info.fields.len().saturating_sub(suffix.len() - suffix_index), None))
                        } else {
                            let index = v_info.fields.len().saturating_sub(suffix.len() - suffix_index);
                            (index, v_info.fields.get(index))
                        };
                        let (field_id, field_type, local_type) = match field_semantic {
                            Some(field) => match opened.fields.get(field_index) {
                                Some(field_subject) => (
                                    field.id.clone(),
                                    TypeKnowledge::established(field_subject.canonical, EvidenceOrigin::PatternDecomposition),
                                    Some(field_subject.local.clone()),
                                ),
                                None => (field.id.clone(), TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence), None),
                            },
                            None => (
                                crate::identity::VariantFieldId::new(variant_id.clone(), field_index as u32),
                                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                                None,
                            ),
                        };
                        let field_subject = opened.fields.get(field_index).cloned();
                        let (child, field_space) = if let Some(field_subject) = field_subject.as_ref() {
                            resolve_pattern_with_mode(ctx, &argument.pattern, field_subject, &mut local_bindings, BindingMode::Detached)
                        } else {
                            // Invalid opened-field metadata must not invent a canonical
                            // child subject. Retain unknown field evidence and leave child
                            // resolution opaque until the opening invariant is repaired.
                            (PatternResolution::Wildcard, PatternSpace::Opaque(subject.canonical))
                        };
                        resolved_fields.push(ResolvedFieldPattern {
                            field: field_id,
                            field_type,
                            local_type,
                            child: Box::new(child),
                        });
                        if field_index < field_spaces.len() {
                            field_spaces[field_index] = field_space;
                        }
                    }
                }
                VariantPatternMode::Singleton | VariantPatternMode::WholeFamily { .. } => {
                    for field_subject in opened.fields.iter() {
                        field_spaces.push(PatternSpace::Opaque(field_subject.canonical));
                    }
                }
            }

            let candidate_space = PatternSpace::Variant(VariantSpace {
                variant: variant_id.clone(),
                exact_case,
                fields: field_spaces.into_boxed_slice(),
                proof: proof.clone(),
            });

            candidate_resolutions.push(ResolvedVariantCandidate {
                variant: variant_id.clone(),
                exact_case,
                fields: resolved_fields.into_boxed_slice(),
                proof,
                case_instantiation: Some(case_instantiation),
            });
            candidate_spaces.push(candidate_space);
            candidate_bindings.push(local_bindings);
        }

        if !matched_any_variant {
            if matching_base_variants.is_empty() {
                ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    crate::diagnostic::DiagnosticCode::MatchPatternUnresolved,
                    format!("no variant `{}` exists on enum `{}`", variant_pat.base, owner_decl.name),
                    variant_pat.range,
                ));
            } else if had_shape_or_label_match {
                // Matching variant existed in shape/label, but was refuted by GADT typing
                ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    crate::diagnostic::DiagnosticCode::MatchPatternContradictory,
                    format!("variant `{}` cannot match the scrutinee type (refuted by GADT refinement)", variant_pat.base),
                    variant_pat.range,
                ));
            } else if had_arity_mismatch {
                ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    crate::diagnostic::DiagnosticCode::MatchPatternArityMismatch,
                    format!("pattern for variant `{}` has incorrect argument arity", variant_pat.base),
                    variant_pat.range,
                ));
            } else if had_field_mismatch {
                ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    crate::diagnostic::DiagnosticCode::MatchPatternFieldMismatch,
                    format!("pattern for variant `{}` has invalid field labels", variant_pat.base),
                    variant_pat.range,
                ));
            } else {
                ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    crate::diagnostic::DiagnosticCode::MatchPatternUnresolved,
                    format!("no variant `{}` matches the given pattern selector", variant_pat.base),
                    variant_pat.range,
                ));
            }
        }
    }

    let active_candidate_bindings: Vec<Vec<PatternBindingResolution>> = candidate_bindings.iter().filter(|alt| !alt.is_empty()).cloned().collect();
    let bindings_to_commit = if active_candidate_bindings.is_empty() {
        &candidate_bindings[..]
    } else {
        &active_candidate_bindings[..]
    };

    let replacements = commit_shared_bindings(
        ctx,
        bindings_to_commit,
        bindings,
        variant_pat.range,
        binding_mode,
        crate::diagnostic::DiagnosticCode::MatchPatternOrBindingMismatch,
        "variant candidate alternatives must introduce the same binding names",
    );
    for candidate in &mut candidate_resolutions {
        for field in candidate.fields.iter_mut() {
            remap_pattern_bindings(&mut field.child, &replacements);
        }
    }

    let space = match candidate_spaces.len() {
        0 => PatternSpace::Empty,
        1 => candidate_spaces.pop().expect("single candidate space exists").normalize(),
        _ => PatternSpace::Union(candidate_spaces.into_boxed_slice()).normalize(),
    };

    let family = if candidate_resolutions.len() == 1 {
        candidate_resolutions
            .first()
            .and_then(|cand| enum_table.as_ref().and_then(|table| table.variants.get(&cand.variant)))
            .and_then(|v_info| v_info.family.clone())
            .or_else(|| Some(VariantFamilyId::new(owner_decl.clone(), variant_pat.base.clone())))
    } else {
        None
    };

    (
        ResolvedVariantPattern {
            owner: Some(owner_decl.clone()),
            family,
            owner_candidates: Box::new([owner_decl]),
            selector: constraint,
            candidates: candidate_resolutions.into_boxed_slice(),
        },
        space,
    )
}

fn matches_variant_info(
    v_info: &crate::enum_semantics::VariantInfo,
    base_name: &str,
    constraint: &VariantSelectorConstraint,
    variant_pat: &VariantPattern,
) -> bool {
    let matches_base = match &v_info.id.selector.base {
        SelectorBase::Named(name) => name == base_name,
        _ => false,
    };
    if !matches_base {
        return false;
    }
    match constraint {
        VariantSelectorConstraint::WholeFamily => true,
        VariantSelectorConstraint::Exact(exact) => {
            if &v_info.id.selector == exact {
                return true;
            }

            // Exact calls may use local field names for binding selection, while
            // canonical constructor selectors retain only external labels. Keep
            // that compatibility for method-shaped variants, but never allow it
            // to bridge getter and constructor shapes.
            if exact.kind != SelectorKind::Method || v_info.id.selector.kind != SelectorKind::Method {
                return false;
            }

            let VariantPatternMode::ExactCall { arguments } = &variant_pat.mode else {
                return false;
            };
            if arguments.len() != v_info.fields.len() {
                return false;
            }

            arguments.iter().enumerate().all(|(i, arg)| {
                if let Some(ref label) = arg.label {
                    v_info
                        .fields
                        .iter()
                        .any(|field| field.external_label.as_deref() == Some(label) || field.local_name.as_ref() == label.as_str())
                } else {
                    i < v_info.fields.len()
                }
            })
        }
        VariantSelectorConstraint::Pattern(pattern) => pattern.matches(&v_info.id.selector),
    }
}

/// Lowers a resolved pattern into query-local coverage arena nodes.
pub(crate) fn coverage_pattern_for_resolution(
    arena: &mut crate::checker::coverage::CoveragePatternArena,
    resolution: &PatternResolution,
) -> crate::checker::coverage::CoveragePatternId {
    match resolution {
        PatternResolution::Wildcard | PatternResolution::Binding { .. } => arena.wildcard(),
        PatternResolution::Variant(variant) => {
            if variant.candidates.len() <= 1 {
                let (candidates, exact_cases, fields) = if let Some(cand) = variant.candidates.first() {
                    let arity = cand.variant.selector.parameter_count();
                    let mut fields = vec![arena.wildcard(); arity];
                    for f in cand.fields.iter() {
                        let idx = f.field.index as usize;
                        if idx < fields.len() {
                            fields[idx] = coverage_pattern_for_resolution(arena, &f.child);
                        }
                    }
                    (
                        vec![cand.variant.clone()].into_boxed_slice(),
                        vec![cand.exact_case].into_boxed_slice(),
                        fields.into_boxed_slice(),
                    )
                } else {
                    (Vec::new().into_boxed_slice(), Vec::new().into_boxed_slice(), Vec::new().into_boxed_slice())
                };
                arena.alloc(crate::checker::coverage::CoveragePattern::Variant {
                    candidates,
                    exact_cases,
                    fields,
                })
            } else {
                let alts: Box<[crate::checker::coverage::CoveragePatternId]> = variant
                    .candidates
                    .iter()
                    .map(|cand| {
                        let arity = cand.variant.selector.parameter_count();
                        let mut fields = vec![arena.wildcard(); arity];
                        for f in cand.fields.iter() {
                            let idx = f.field.index as usize;
                            if idx < fields.len() {
                                fields[idx] = coverage_pattern_for_resolution(arena, &f.child);
                            }
                        }
                        arena.alloc(crate::checker::coverage::CoveragePattern::Variant {
                            candidates: Box::new([cand.variant.clone()]),
                            exact_cases: Box::new([cand.exact_case]),
                            fields: fields.into_boxed_slice(),
                        })
                    })
                    .collect();
                arena.alloc(crate::checker::coverage::CoveragePattern::Or(alts))
            }
        }
        PatternResolution::Tuple(elements) => {
            let fields = elements.iter().map(|elem| coverage_pattern_for_resolution(arena, elem)).collect();
            arena.alloc(crate::checker::coverage::CoveragePattern::Tuple(fields))
        }
        PatternResolution::List(list) => {
            let prefix = list.prefix.iter().map(|elem| coverage_pattern_for_resolution(arena, elem)).collect();
            let rest = list.rest.as_ref().map(|r| coverage_pattern_for_resolution(arena, r));
            arena.alloc(crate::checker::coverage::CoveragePattern::List { prefix, rest })
        }
        PatternResolution::Or(or_pat) => {
            let alternatives = or_pat.alternatives.iter().map(|alt| coverage_pattern_for_resolution(arena, alt)).collect();
            arena.alloc(crate::checker::coverage::CoveragePattern::Or(alternatives))
        }
        PatternResolution::Record(_) => arena.alloc(crate::checker::coverage::CoveragePattern::RecordPredicate),
        PatternResolution::Map(_) => arena.alloc(crate::checker::coverage::CoveragePattern::MapPredicate),
    }
}
