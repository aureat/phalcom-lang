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
) -> Option<OpenedVariantCase> {
    let (mut proof, exact_case) =
        match crate::checker::gadt_proof::solve_gadt_branch_proof(store, hierarchy, &variant_info.id.owner, variant_info, subject.canonical) {
            crate::checker::gadt_proof::GadtProofResult::Reachable { proof, exact_case } => (proof, exact_case),
            crate::checker::gadt_proof::GadtProofResult::Refuted => return None,
        };

    let case_instantiation = CaseInstantiation::open(store, rigids, variant_info, None);
    let (local_proof_bindings, local_proof_equalities) =
        crate::checker::gadt_proof::solve_local_case_proof_against_local(store, &proof, &subject.local, &case_instantiation)?;
    proof.local_bindings = local_proof_bindings;
    proof.local_equalities = local_proof_equalities;

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

    Some(OpenedVariantCase {
        variant: variant_info.id.clone(),
        exact_case,
        proof,
        case_instantiation,
        fields: fields.into_boxed_slice(),
    })
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
/// Open domains (Object, unknown, etc.) return Open.
/// Uninhabited types return Empty.
pub(crate) fn decompose_domain(
    declarations: &DeclarationTypeTable,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    rigids: &mut RigidArena,
    enum_table: Option<&EnumSemanticTable>,
    subject: &CoverageSubject,
) -> DomainDecomposition {
    if matches!(store.get(subject.canonical), TypeData::Never) {
        return DomainDecomposition::Empty;
    }

    if let TypeData::ExactCase { variant, .. } = store.get(subject.canonical).clone() {
        let variant_id = store.variant_identity(variant).clone();
        let Some(var_info) = enum_table.and_then(|table| table.variants.get(&variant_id)).cloned() else {
            return DomainDecomposition::Open;
        };
        let Some(opened) = open_variant_case(declarations, store, hierarchy, rigids, subject, &var_info) else {
            return DomainDecomposition::Empty;
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
            LocalType::Tuple(local_elems) if local_elems.len() == elements.len() => elements
                .iter()
                .zip(local_elems.iter())
                .map(|(elem, local)| CoverageSubject::from_parts(elem.ty, local.ty.clone()))
                .collect(),
            _ => elements.iter().map(|elem| CoverageSubject::canonical(elem.ty)).collect(),
        };
        return DomainDecomposition::Closed(Box::new([ConstructorCase {
            head: ConstructorHead::Tuple { arity: elements.len() },
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
                if let Some(opened) = open_variant_case(declarations, store, hierarchy, rigids, subject, &var_info) {
                    cases.push(ConstructorCase {
                        head: ConstructorHead::Variant(opened.variant),
                        fields: opened.fields,
                        proof: opened.proof,
                        exact_case: Some(opened.exact_case),
                        case_instantiation: Some(opened.case_instantiation),
                    });
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
        for &member in members.iter() {
            let member_sub = CoverageSubject::canonical(member);
            match decompose_domain(declarations, store, hierarchy, rigids, enum_table, &member_sub) {
                DomainDecomposition::Closed(member_cases) => {
                    all_cases.extend(member_cases.into_vec());
                }
                DomainDecomposition::Empty => {}
                DomainDecomposition::Open => return DomainDecomposition::Open,
                DomainDecomposition::Blocked(b) => return DomainDecomposition::Blocked(b),
            }
        }
        if all_cases.is_empty() {
            return DomainDecomposition::Empty;
        }
        return DomainDecomposition::Closed(all_cases.into_boxed_slice());
    }

    DomainDecomposition::Open
}
