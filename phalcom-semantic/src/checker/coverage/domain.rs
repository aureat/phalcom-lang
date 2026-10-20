//! Domain decomposition and shared variant constructor observation.

use crate::declarations::DeclarationTypeTable;
use crate::enum_semantics::{EnumSemanticTable, VariantInfo};
use crate::identity::VariantId;
use crate::match_semantics::BranchProofEnvironment;
use crate::types::case_instantiation::CaseInstantiation;
use crate::types::id::TypeId;
use crate::types::outcome::BlockReason;
use crate::types::relation::TypeHierarchy;
use crate::types::rigid::{LocalType, RigidArena};
use crate::types::store::{TypeData, TypeStore};
use std::collections::HashMap;

use super::subject::CoverageSubject;

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DomainDecomposition {
    Empty,
    Closed(Box<[ConstructorCase]>),
    /// The closed Unicode scalar domain owned by the canonical Char type.
    Char,
    /// A closed union containing both constructor cases and the Char scalar domain.
    ClosedWithChar(Box<[ConstructorCase]>),
    Open,
    Blocked(BlockReason),
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ConstructorHead {
    Variant(VariantId),
    Tuple { arity: usize },
    ListNil,
    ListCons,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConstructorCase {
    pub head: ConstructorHead,
    pub fields: Box<[CoverageSubject]>,
    pub proof: BranchProofEnvironment,
    pub exact_case: Option<TypeId>,
    pub case_instantiation: Option<CaseInstantiation>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OpenedVariantCase {
    pub variant: VariantId,
    pub exact_case: TypeId,
    pub proof: BranchProofEnvironment,
    pub case_instantiation: CaseInstantiation,
    pub fields: Box<[CoverageSubject]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VariantOpenOutcome {
    Reachable(OpenedVariantCase),
    Refuted,
    Blocked(BlockReason),
}

/// Centralized semantic authority for observing a variant case against an expected subject.
///
/// Refutes incompatible cases, opens constructor-local existential generics into fresh rigids,
/// refines branch proofs, and specializes payloads canonically and locally.
pub(crate) fn open_variant_case(
    declarations: &DeclarationTypeTable,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    rigids: &mut RigidArena,
    subject: &CoverageSubject,
    variant_info: &VariantInfo,
    control: &crate::checker::context::CheckerControl,
) -> VariantOpenOutcome {
    if let Err(reason) = crate::checker::gadt_proof::local_proof_control(control) {
        return VariantOpenOutcome::Blocked(reason);
    }
    let (mut proof, exact_case) =
        match crate::checker::gadt_proof::solve_gadt_branch_proof(store, hierarchy, &variant_info.id.owner, variant_info, subject.canonical) {
            crate::checker::gadt_proof::GadtProofResult::Reachable { proof, exact_case } => (proof, exact_case),
            crate::checker::gadt_proof::GadtProofResult::Refuted => return VariantOpenOutcome::Refuted,
            crate::checker::gadt_proof::GadtProofResult::Blocked(reason) => return VariantOpenOutcome::Blocked(reason),
        };

    let case_instantiation = CaseInstantiation::open(store, rigids, variant_info, None);
    if case_instantiation.is_local() || subject.contains_local_rigids() {
        for ty in [variant_info.result_type_template, subject.canonical] {
            if let Err(reason) = check_local_representation(store, ty, control) {
                return VariantOpenOutcome::Blocked(reason);
            }
        }
    }
    match crate::checker::gadt_proof::solve_local_case_proof_against_local(store, &proof, &subject.local, &case_instantiation, control) {
        crate::checker::gadt_proof::LocalCaseProof::Reachable { bindings, equalities } => {
            proof.local_bindings = bindings.into();
            proof.local_equalities = equalities;
        }
        crate::checker::gadt_proof::LocalCaseProof::Refuted => return VariantOpenOutcome::Refuted,
        crate::checker::gadt_proof::LocalCaseProof::Blocked(reason) => return VariantOpenOutcome::Blocked(reason),
    }

    let substitution = crate::types::substitution::substitution_for_applied(declarations, store, subject.canonical);
    let mut replacements = local_subject_replacements(declarations, &variant_info.id.owner, &subject.local);
    replacements.extend(case_instantiation.replacements());
    for (k, v) in &proof.local_bindings {
        // Constructor-local rigids describe payload binders and must remain
        // authoritative. Local proof equalities may refine enclosing
        // parameters, but cannot erase a freshly opened constructor rigid.
        if !case_instantiation.local_rigids.contains_key(k) {
            replacements.insert(*k, v.clone());
        }
    }

    let mut fields = Vec::with_capacity(variant_info.fields.len());
    for field in &variant_info.fields {
        let raw = field.declared_type.canonical_type().unwrap_or(subject.canonical);
        let declaration_specialized = substitution.as_ref().map(|sub| sub.apply(store, raw)).unwrap_or(raw);
        let mut canonical_field_ty = crate::checker::gadt_proof::apply_branch_proof(store, &proof, declaration_specialized);
        // Local constructor equalities may solve a constructor parameter to a
        // canonical enclosing term. Apply only those canonical bindings to the
        // canonical view; local rigid payload identity remains in `local_field_ty`.
        let mut local_canonical_substitution = crate::types::substitution::TypeSubstitution::new();
        for (&parameter, replacement) in &proof.local_bindings {
            if let LocalType::Canonical(ty) = replacement {
                local_canonical_substitution.bind(parameter, *ty);
            }
        }
        canonical_field_ty = local_canonical_substitution.apply(store, canonical_field_ty);
        // Canonical declaration terms retain proof IDs when no local rigid is
        // present. Once a constructor/local rigid is in scope, localize raw
        // field terms before canonical substitution so nested recursion keeps it.
        let has_local_rigid = replacements.values().any(|term| !term.free_rigids().is_empty());
        let localized = if has_local_rigid {
            LocalType::from_canonical(store, raw, &replacements)
        } else {
            LocalType::from_canonical(store, canonical_field_ty, &replacements)
        };
        let local_field_ty = crate::checker::gadt_proof::apply_branch_proof_to_local(store, &proof, &localized);
        fields.push(CoverageSubject::from_parts(canonical_field_ty, local_field_ty));
    }

    VariantOpenOutcome::Reachable(OpenedVariantCase {
        variant: variant_info.id.clone(),
        exact_case,
        proof,
        case_instantiation,
        fields: fields.into_boxed_slice(),
    })
}

/// LocalType currently erases open row/tuple tails and cannot localize
/// heterogeneous static applications. Such inputs cannot support a local
/// contradiction proof; leave their representation unchanged and fail closed.
fn check_local_representation(store: &TypeStore, ty: TypeId, control: &crate::checker::context::CheckerControl) -> Result<(), BlockReason> {
    let mut pending = vec![ty];
    let mut seen = std::collections::BTreeSet::new();
    while let Some(ty) = pending.pop() {
        crate::checker::gadt_proof::local_proof_control(control)?;
        if !seen.insert(ty) {
            continue;
        }
        match store.get(ty) {
            TypeData::Record(row) => {
                let row = store.record_row(*row);
                if row.tail != crate::types::row::RecordRowTail::Closed {
                    return Err(BlockReason::ReflectionBoundary);
                }
                pending.extend(row.fields.iter().map(|field| field.ty));
            }
            TypeData::Tuple(tuple) => {
                if tuple.positional_remainder != crate::types::store::AnonymousTupleRemainder::None
                    || tuple.labeled_remainder != crate::types::store::AnonymousTupleRemainder::None
                {
                    return Err(BlockReason::ReflectionBoundary);
                }
                pending.extend(tuple.elements.iter().map(|element| element.ty));
            }
            TypeData::Applied { origin, arguments } => {
                pending.push(*origin);
                for argument in arguments.iter() {
                    let Some(ty) = argument.as_type_id() else {
                        return Err(BlockReason::ReflectionBoundary);
                    };
                    pending.push(ty);
                }
            }
            TypeData::ExactCase { enum_type, .. } => pending.push(*enum_type),
            TypeData::Union(members) => pending.extend(members.iter().copied()),
            TypeData::Callable(callable) => {
                pending.extend(callable.parameters.iter().map(|parameter| parameter.ty));
                pending.push(callable.return_type);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Maps declaration-owned generic parameters to the local arguments carried by
/// this query's subject. Canonical substitutions cannot retain parent rigids;
/// this query-local map keeps them in nested constructor payloads.
fn local_subject_replacements(
    declarations: &DeclarationTypeTable,
    owner: &crate::identity::DeclarationId,
    local: &LocalType,
) -> HashMap<crate::types::id::TypeParameterId, LocalType> {
    let LocalType::Applied { arguments, .. } = local else {
        return HashMap::new();
    };
    let Some(signature) = declarations.generic_signature(owner) else {
        return HashMap::new();
    };
    signature.parameters.iter().copied().zip(arguments.iter().cloned()).collect()
}

/// Decomposes a coverage subject into its top-level constructors one layer deep.
///
/// Closed enums/GADTs, tuples, lists, and closed exact cases return a finite set of ConstructorCases.
/// Char returns its closed scalar domain without enumerating every scalar.
/// Open domains (Object, unknown, etc.) return Open.
/// Uninhabited types return Empty.
pub(crate) fn decompose_domain(
    declarations: &DeclarationTypeTable,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    rigids: &mut RigidArena,
    enum_table: Option<&EnumSemanticTable>,
    subject: &CoverageSubject,
    control: &crate::checker::context::CheckerControl,
) -> DomainDecomposition {
    if matches!(store.get(subject.canonical), TypeData::Never) {
        return DomainDecomposition::Empty;
    }

    let char_declaration = crate::core_surface::CoreDeclarationIds::default().char_;
    if store.nominal_origin_declaration(subject.canonical) == Some(&char_declaration) {
        return DomainDecomposition::Char;
    }

    if let TypeData::ExactCase { variant, .. } = store.get(subject.canonical).clone() {
        let variant_id = store.variant_identity(variant).clone();
        let Some(var_info) = enum_table.and_then(|table| table.variants.get(&variant_id)).cloned() else {
            return DomainDecomposition::Open;
        };
        let opened = match open_variant_case(declarations, store, hierarchy, rigids, subject, &var_info, control) {
            VariantOpenOutcome::Reachable(opened) => opened,
            VariantOpenOutcome::Refuted => return DomainDecomposition::Empty,
            VariantOpenOutcome::Blocked(reason) => return DomainDecomposition::Blocked(reason),
        };
        return DomainDecomposition::Closed(Box::new([ConstructorCase {
            head: ConstructorHead::Variant(opened.variant),
            fields: opened.fields,
            proof: opened.proof,
            exact_case: Some(opened.exact_case),
            case_instantiation: Some(opened.case_instantiation),
        }]));
    }

    if let TypeData::Tuple(elements) = store.get(subject.canonical).clone() {
        let fields: Vec<CoverageSubject> = match &subject.local {
            LocalType::Tuple(local_elems) if local_elems.len() == elements.elements.len() => elements
                .elements
                .iter()
                .zip(local_elems.iter())
                .map(|(elem, local)| CoverageSubject::from_parts(elem.ty, local.ty.clone()))
                .collect(),
            _ => elements.elements.iter().map(|elem| CoverageSubject::canonical(elem.ty)).collect(),
        };
        return DomainDecomposition::Closed(Box::new([ConstructorCase {
            head: ConstructorHead::Tuple {
                arity: elements.elements.len(),
            },
            fields: fields.into_boxed_slice(),
            proof: BranchProofEnvironment::default(),
            exact_case: None,
            case_instantiation: None,
        }]));
    }

    if let Some((origin_decl, args)) = store.applied_nominal_parts(subject.canonical) {
        if origin_decl.name.as_ref() == "List" && args.len() == 1 {
            let elem_ty = args[0];
            let list_ty = subject.canonical;
            let nil_case = ConstructorCase {
                head: ConstructorHead::ListNil,
                fields: Box::new([]),
                proof: BranchProofEnvironment::default(),
                exact_case: None,
                case_instantiation: None,
            };
            let (local_elem_ty, local_list_ty) = match &subject.local {
                LocalType::Applied { arguments, .. } if arguments.len() == 1 => (arguments[0].clone(), subject.local.clone()),
                _ => (LocalType::Canonical(elem_ty), LocalType::Canonical(list_ty)),
            };
            let cons_case = ConstructorCase {
                head: ConstructorHead::ListCons,
                fields: Box::new([
                    CoverageSubject::from_parts(elem_ty, local_elem_ty),
                    CoverageSubject::from_parts(list_ty, local_list_ty),
                ]),
                proof: BranchProofEnvironment::default(),
                exact_case: None,
                case_instantiation: None,
            };
            return DomainDecomposition::Closed(Box::new([nil_case, cons_case]));
        }
    }

    let owner = store.nominal_origin_declaration(subject.canonical).cloned();
    if let Some(owner) = owner {
        if let Some(enum_info) = enum_table.and_then(|table| table.enums.get(&owner)).cloned() {
            let mut cases = Vec::with_capacity(enum_info.variants.len());
            for variant_id in &enum_info.variants {
                let Some(var_info) = enum_table.and_then(|table| table.variants.get(variant_id)).cloned() else {
                    return DomainDecomposition::Open;
                };
                match open_variant_case(declarations, store, hierarchy, rigids, subject, &var_info, control) {
                    VariantOpenOutcome::Refuted => continue,
                    VariantOpenOutcome::Blocked(reason) => return DomainDecomposition::Blocked(reason),
                    VariantOpenOutcome::Reachable(opened) => cases.push(ConstructorCase {
                        head: ConstructorHead::Variant(opened.variant),
                        fields: opened.fields,
                        proof: opened.proof,
                        exact_case: Some(opened.exact_case),
                        case_instantiation: Some(opened.case_instantiation),
                    }),
                }
            }
            if cases.is_empty() {
                return DomainDecomposition::Empty;
            }
            return DomainDecomposition::Closed(cases.into_boxed_slice());
        }
    }

    if let TypeData::Union(members) = store.get(subject.canonical).clone() {
        let mut all_cases = Vec::new();
        let mut has_char_domain = false;
        let mut has_open_member = false;
        for (index, &member) in members.iter().enumerate() {
            let member_sub = match &subject.local {
                LocalType::Union(local_members) if local_members.len() == members.len() => CoverageSubject::from_parts(member, local_members[index].clone()),
                LocalType::Canonical(_) => CoverageSubject::canonical(member),
                _ => return DomainDecomposition::Blocked(BlockReason::ReflectionBoundary),
            };
            match decompose_domain(declarations, store, hierarchy, rigids, enum_table, &member_sub, control) {
                DomainDecomposition::Closed(member_cases) => {
                    all_cases.extend(member_cases.into_vec());
                }
                DomainDecomposition::Char => has_char_domain = true,
                DomainDecomposition::ClosedWithChar(member_cases) => {
                    all_cases.extend(member_cases.into_vec());
                    has_char_domain = true;
                }
                DomainDecomposition::Empty => {}
                DomainDecomposition::Open => has_open_member = true,
                DomainDecomposition::Blocked(b) => return DomainDecomposition::Blocked(b),
            }
        }
        // An open member cannot hide uncertainty in a later union member.
        if has_open_member {
            return DomainDecomposition::Open;
        }
        if all_cases.is_empty() {
            return if has_char_domain {
                DomainDecomposition::Char
            } else {
                DomainDecomposition::Empty
            };
        }
        if has_char_domain {
            return DomainDecomposition::ClosedWithChar(all_cases.into_boxed_slice());
        }
        return DomainDecomposition::Closed(all_cases.into_boxed_slice());
    }

    DomainDecomposition::Open
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::context::CheckerControl;
    use crate::identity::{DeclarationId, ModuleId};
    use crate::types::id::KindId;
    use crate::types::outcome::{CancellationToken, QueryBudget};
    use crate::types::relation::MapTypeHierarchy;
    use crate::types::rigid::RigidOrigin;
    use phalcom_common::selector::Selector;
    use std::sync::Arc;

    #[test]
    fn mixed_reachable_and_blocked_constructors_do_not_form_an_empty_or_closed_domain() {
        let module = ModuleId::universe_root();
        let source: Arc<str> = Arc::from("data Case<T> { Any(_ value: T) -> Case<T> Int(_ value: Int) -> Case<Int> }\n");
        let parsed = phalcom_ast::parse_source(&source, 0).unwrap();
        let analysis = crate::analyze_single_module(module.clone(), source, Arc::new(parsed)).expect("semantic fixture should analyze");
        let mut store = (*analysis.snapshot.store).clone();
        let owner = DeclarationId::new(module, "Case".into());
        let origin = analysis.snapshot.declarations.get(&owner).unwrap().form;
        let parameter = analysis.snapshot.declarations.generic_signature(&owner).unwrap().parameters[0];
        let parameter_type = store.parameter_form(parameter);
        let canonical = store.apply_type_form(origin, &[parameter_type]).unwrap();
        let table = &analysis.snapshot.enum_semantics;
        let any = &table.variants[&VariantId::new(
            owner.clone(),
            Selector::method("Any", [phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        )];
        let int = &table.variants[&VariantId::new(owner, Selector::method("Int", [phalcom_common::selector::SelectorSlot::Positional]).unwrap())];
        let mut rigids = RigidArena::new();
        let scope = rigids.fresh_scope(None);
        let rigid = rigids.fresh(scope, KindId::TYPE, RigidOrigin::Synthetic);
        let subject = CoverageSubject::from_parts(
            canonical,
            LocalType::Applied {
                origin: Box::new(LocalType::Canonical(origin)),
                arguments: Box::new([LocalType::Rigid(rigid)]),
            },
        );
        let hierarchy = MapTypeHierarchy::new();
        let control = CheckerControl::default();
        let opened_any = open_variant_case(&analysis.snapshot.declarations, &mut store, &hierarchy, &mut rigids, &subject, any, &control);
        assert!(matches!(opened_any, VariantOpenOutcome::Reachable(_)), "{opened_any:?}");
        assert!(matches!(
            open_variant_case(&analysis.snapshot.declarations, &mut store, &hierarchy, &mut rigids, &subject, int, &control),
            VariantOpenOutcome::Blocked(_)
        ));
        assert!(matches!(
            decompose_domain(
                &analysis.snapshot.declarations,
                &mut store,
                &hierarchy,
                &mut rigids,
                Some(table),
                &subject,
                &control
            ),
            DomainDecomposition::Blocked(_)
        ));
        // Exact-case and union decomposition must preserve the same uncertainty.
        let projection_owner = DeclarationId::new(ModuleId::universe_root(), "Projection".into());
        let projection = store.associated_projection(
            origin,
            crate::traits::TraitRef::new(projection_owner.clone(), Vec::new()),
            crate::traits::AssociatedTypeRequirementId::new(projection_owner, 0),
        );
        let exact = store.exact_case_type(&int.id, canonical).unwrap();
        let exact_subject = CoverageSubject::from_parts(
            exact,
            LocalType::ExactCase {
                variant: int.id.clone(),
                enum_type: Box::new(LocalType::Applied {
                    origin: Box::new(LocalType::Canonical(origin)),
                    arguments: Box::new([LocalType::Canonical(projection)]),
                }),
            },
        );
        assert!(matches!(
            decompose_domain(
                &analysis.snapshot.declarations,
                &mut store,
                &hierarchy,
                &mut rigids,
                Some(table),
                &exact_subject,
                &control
            ),
            DomainDecomposition::Blocked(_)
        ));
        let unit = store.unit();
        let union = store.union(&[exact, unit]);
        let local_members = match store.get(union) {
            TypeData::Union(members) => members
                .iter()
                .map(|member| {
                    if *member == exact {
                        exact_subject.local.clone()
                    } else {
                        LocalType::Canonical(*member)
                    }
                })
                .collect(),
            _ => panic!("union should retain both members"),
        };
        let union_subject = CoverageSubject::from_parts(union, LocalType::Union(local_members));
        assert!(matches!(
            decompose_domain(
                &analysis.snapshot.declarations,
                &mut store,
                &hierarchy,
                &mut rigids,
                Some(table),
                &union_subject,
                &control
            ),
            DomainDecomposition::Blocked(_)
        ));
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        assert!(matches!(
            open_variant_case(
                &analysis.snapshot.declarations,
                &mut store,
                &hierarchy,
                &mut rigids,
                &subject,
                any,
                &CheckerControl::new(QueryBudget::default(), &cancellation)
            ),
            VariantOpenOutcome::Blocked(_)
        ));
        assert!(matches!(
            open_variant_case(
                &analysis.snapshot.declarations,
                &mut store,
                &hierarchy,
                &mut rigids,
                &subject,
                any,
                &CheckerControl::new(QueryBudget::new(0), &CancellationToken::new())
            ),
            VariantOpenOutcome::Blocked(_)
        ));
    }
}
