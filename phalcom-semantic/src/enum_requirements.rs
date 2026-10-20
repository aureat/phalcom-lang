//! Closed-enum method requirements and case-obligation checking.

use crate::diagnostic::SemanticDiagnostic;
use crate::identity::{CallableId, DeclarationId, SemanticSourceSpan, VariantId};
use crate::signature::CallableSemanticSignature;
use phalcom_common::selector::Selector;
use phalcom_type_meta::allocation::BudgetedSortedMap;
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::sync::Arc;

/// Unique identifier for a closed-enum method requirement.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EnumRequirementId {
    pub owner: DeclarationId,
    pub selector: Selector,
}

impl EnumRequirementId {
    pub fn new(owner: DeclarationId, selector: Selector) -> Self {
        Self { owner, selector }
    }

    pub fn clone_with_allocation_budget(
        &self,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, phalcom_type_meta::allocation::AllocationError> {
        Ok(Self {
            owner: self.owner.clone_with_allocation_budget(allocations)?,
            selector: crate::traits::clone_selector_with_allocation_budget(&self.selector, allocations)?,
        })
    }
}

/// A required behavioral method signature declared on an enum root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumRequirement {
    pub id: EnumRequirementId,
    pub signature: CallableSemanticSignature,
    pub source: Option<SemanticSourceSpan>,
}

/// Obligation status of a single variant against an enum requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaseRequirementStatus {
    Satisfied { implementation: CallableId },
    Missing,
    Incompatible { implementation: CallableId },
    Blocked,
}

/// Result of checking one variant against one enum requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseRequirementResult {
    pub variant: VariantId,
    pub requirement: EnumRequirementId,
    pub status: CaseRequirementStatus,
}

/// Shared immutable enum rows with a common slice read interface.
///
/// Source-built snapshots retain their ordinary `Arc<[T]>` backing. Artifact
/// loads retain a budgeted `Arc<Vec<T>>`, since the pinned standard library
/// does not expose a fallible `Arc<[T]>` slice constructor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnumRequirementRows<T> {
    Shared(Arc<[T]>),
    Budgeted(Arc<Vec<T>>),
}

impl<T> EnumRequirementRows<T> {
    pub fn from_shared(rows: Arc<[T]>) -> Self {
        Self::Shared(rows)
    }

    pub fn from_budgeted(rows: Arc<Vec<T>>) -> Self {
        Self::Budgeted(rows)
    }
}

impl<T> AsRef<[T]> for EnumRequirementRows<T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> Deref for EnumRequirementRows<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Shared(rows) => rows,
            Self::Budgeted(rows) => rows,
        }
    }
}

/// Source-built or frozen canonical owner map for enum requirement products.
/// Artifact loads use the frozen sorted-vector backing; source analysis keeps
/// its local ordered map and parent overlay behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumRequirementOwnerMap<V> {
    parent: Option<Arc<Self>>,
    frozen: Option<Arc<BudgetedSortedMap<DeclarationId, V>>>,
    local: BTreeMap<DeclarationId, V>,
}

impl<V> Default for EnumRequirementOwnerMap<V> {
    fn default() -> Self {
        Self {
            parent: None,
            frozen: None,
            local: BTreeMap::new(),
        }
    }
}

impl<V> EnumRequirementOwnerMap<V> {
    pub fn with_parent(parent: Arc<Self>) -> Self {
        Self {
            parent: Some(parent),
            frozen: None,
            local: BTreeMap::new(),
        }
    }

    /// Adopts an already budgeted immutable map without converting its storage.
    pub fn from_frozen(frozen: Arc<BudgetedSortedMap<DeclarationId, V>>) -> Self {
        Self {
            parent: None,
            frozen: Some(frozen),
            local: BTreeMap::new(),
        }
    }

    pub fn get(&self, owner: &DeclarationId) -> Option<&V> {
        self.local
            .get(owner)
            .or_else(|| self.frozen.as_ref().and_then(|frozen| frozen.get(owner)))
            .or_else(|| self.parent.as_ref().and_then(|parent| parent.get(owner)))
    }

    pub fn contains_key(&self, owner: &DeclarationId) -> bool {
        self.get(owner).is_some()
    }

    pub fn len(&self) -> usize {
        self.local.len()
            + self
                .frozen
                .as_ref()
                .map_or_else(|| self.parent.as_ref().map_or(0, |parent| parent.len()), |frozen| frozen.len())
    }

    pub fn insert(&mut self, owner: DeclarationId, value: V) -> Option<V> {
        assert!(
            !self.parent.as_ref().is_some_and(|parent| parent.contains_key(&owner)) && !self.frozen.as_ref().is_some_and(|frozen| frozen.contains_key(&owner)),
            "cannot overwrite a canonical enum-requirement owner"
        );
        self.local.insert(owner, value)
    }

    pub fn retain(&mut self, predicate: impl FnMut(&DeclarationId, &mut V) -> bool) {
        self.local.retain(predicate);
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = (&DeclarationId, &V)> + '_> {
        if let Some(parent) = &self.parent {
            Box::new(parent.iter().chain(self.local.iter()))
        } else if let Some(frozen) = &self.frozen {
            Box::new(frozen.iter())
        } else {
            Box::new(self.local.iter())
        }
    }

    pub fn is_empty(&self) -> bool {
        self.local.is_empty() && self.frozen.as_ref().is_none_or(|frozen| frozen.is_empty()) && self.parent.as_ref().is_none_or(|parent| parent.is_empty())
    }
}

/// Snapshot-level table of closed-enum requirements and case statuses.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnumRequirementTable {
    pub requirements: Arc<EnumRequirementOwnerMap<EnumRequirementRows<EnumRequirement>>>,
    pub case_statuses: Arc<EnumRequirementOwnerMap<EnumRequirementRows<CaseRequirementResult>>>,
}

impl EnumRequirementTable {
    /// Shares canonical entries and starts with empty local contributions.
    pub fn with_parent(parent: Arc<Self>) -> Self {
        Self {
            requirements: Arc::new(EnumRequirementOwnerMap::with_parent(parent.requirements.clone())),
            case_statuses: Arc::new(EnumRequirementOwnerMap::with_parent(parent.case_statuses.clone())),
        }
    }

    pub fn from_frozen(
        requirements: Arc<BudgetedSortedMap<DeclarationId, EnumRequirementRows<EnumRequirement>>>,
        case_statuses: Arc<BudgetedSortedMap<DeclarationId, EnumRequirementRows<CaseRequirementResult>>>,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, phalcom_type_meta::allocation::AllocationError> {
        Ok(Self {
            requirements: allocations.arc(EnumRequirementOwnerMap::from_frozen(requirements))?,
            case_statuses: allocations.arc(EnumRequirementOwnerMap::from_frozen(case_statuses))?,
        })
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, owner: DeclarationId, requirements: Arc<[EnumRequirement]>, statuses: Arc<[CaseRequirementResult]>) {
        Arc::make_mut(&mut self.requirements).insert(owner.clone(), EnumRequirementRows::from_shared(requirements));
        Arc::make_mut(&mut self.case_statuses).insert(owner, EnumRequirementRows::from_shared(statuses));
    }

    pub fn remove_module(&mut self, module: &phalcom_modules::identity::ModuleId) {
        Arc::make_mut(&mut self.requirements).retain(|owner, _| &owner.module != module);
        Arc::make_mut(&mut self.case_statuses).retain(|owner, _| &owner.module != module);
    }
}

/// Checks all concrete variants of an enum against its closed-enum requirements.
#[allow(clippy::too_many_arguments)]
pub fn check_enum_requirements(
    owner: &DeclarationId,
    _enum_info: &crate::enum_semantics::EnumInfo,
    variants: &[crate::enum_semantics::VariantInfo],
    root_requirements: &[EnumRequirement],
    case_methods: &HashMap<VariantId, Vec<CallableSemanticSignature>>,
    store: &mut crate::types::store::TypeStore,
    hierarchy: &dyn crate::types::relation::TypeHierarchy,
    module_id: &crate::identity::ModuleId,
) -> (Arc<[CaseRequirementResult]>, Arc<[SemanticDiagnostic]>) {
    let mut diagnostics = Vec::new();
    let mut statuses = Vec::new();

    for req in root_requirements {
        // Check if root requirement signature is complete
        let req_ret_incomplete = req.signature.declared_return.is_unknown();
        let req_params_incomplete = req.signature.parameters.iter().any(|p| match &p.contract {
            crate::signature::ParameterContract::Fixed(fact) => fact.is_unknown(),
            crate::signature::ParameterContract::Rest { contract, .. } => match contract {
                crate::signature::RestValueContract::Unconstrained | crate::signature::RestValueContract::Lane(_) => false,
                crate::signature::RestValueContract::Element(fact) => fact.is_unknown(),
                crate::signature::RestValueContract::Invalid(_) => true,
            },
        });

        if req_ret_incomplete || req_params_incomplete {
            diagnostics.push(SemanticDiagnostic::error_in(
                module_id.clone(),
                crate::diagnostic::DiagnosticCode::EnumRequirementIncomplete,
                format!(
                    "closed-enum requirement `{}` on `{}` has unresolved types",
                    req.id.selector.encode(),
                    owner.name
                ),
                req.source.as_ref().map(|s| s.range).unwrap_or_default(),
            ));
            for v in variants {
                statuses.push(CaseRequirementResult {
                    variant: v.id.clone(),
                    requirement: req.id.clone(),
                    status: CaseRequirementStatus::Blocked,
                });
            }
            continue;
        }

        let req_ret_ty = req.signature.declared_return.canonical_type();

        for v in variants {
            let methods = case_methods.get(&v.id);
            let case_method = methods.and_then(|ms| ms.iter().find(|m| m.selector == req.signature.selector));

            match case_method {
                None => {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        module_id.clone(),
                        crate::diagnostic::DiagnosticCode::EnumRequirementMissing,
                        format!(
                            "enum variant `{}` does not implement required method `{}`",
                            v.id.selector.encode(),
                            req.id.selector.encode()
                        ),
                        v.source.as_ref().map(|s| s.range).unwrap_or_default(),
                    ));
                    statuses.push(CaseRequirementResult {
                        variant: v.id.clone(),
                        requirement: req.id.clone(),
                        status: CaseRequirementStatus::Missing,
                    });
                }
                Some(case_sig) => {
                    let case_subst = v.case_environment.to_substitution();

                    // Check param length and rest mode
                    let param_count_matches = case_sig.parameters.len() == req.signature.parameters.len();
                    let rest_mode_matches = case_sig
                        .parameters
                        .iter()
                        .zip(req.signature.parameters.iter())
                        .all(|(cp, rp)| cp.rest == rp.rest);

                    if !param_count_matches || !rest_mode_matches {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            module_id.clone(),
                            crate::diagnostic::DiagnosticCode::EnumRequirementIncompatible,
                            format!(
                                "method `{}` on variant `{}` has incompatible parameter shape for requirement",
                                case_sig.selector.encode(),
                                v.id.selector.encode()
                            ),
                            case_sig.source.as_ref().map(|s| s.range).unwrap_or_default(),
                        ));
                        statuses.push(CaseRequirementResult {
                            variant: v.id.clone(),
                            requirement: req.id.clone(),
                            status: CaseRequirementStatus::Incompatible {
                                implementation: case_sig.callable.clone(),
                            },
                        });
                        continue;
                    }

                    // Check parameter equivalence under specialized environment
                    let mut params_compatible = true;
                    for (cp, rp) in case_sig.parameters.iter().zip(req.signature.parameters.iter()) {
                        match (&rp.contract, &cp.contract) {
                            (
                                crate::signature::ParameterContract::Rest {
                                    mode: required_mode,
                                    contract: required,
                                },
                                crate::signature::ParameterContract::Rest {
                                    mode: case_mode,
                                    contract: case,
                                },
                            ) if required_mode == case_mode => match (required, case) {
                                (crate::signature::RestValueContract::Unconstrained, crate::signature::RestValueContract::Unconstrained) => {}
                                (crate::signature::RestValueContract::Element(required), crate::signature::RestValueContract::Element(case)) => {
                                    if required.canonical_type().map(|ty| case_subst.apply(store, ty)) != case.canonical_type() {
                                        params_compatible = false;
                                        break;
                                    }
                                }
                                (crate::signature::RestValueContract::Lane(required), crate::signature::RestValueContract::Lane(case)) => {
                                    if case_subst.apply(store, *required) != *case {
                                        params_compatible = false;
                                        break;
                                    }
                                }
                                _ => {
                                    params_compatible = false;
                                    break;
                                }
                            },
                            (crate::signature::ParameterContract::Fixed(required), crate::signature::ParameterContract::Fixed(case)) => {
                                if required.canonical_type().map(|ty| case_subst.apply(store, ty)) != case.canonical_type() {
                                    params_compatible = false;
                                    break;
                                }
                            }
                            _ => {
                                params_compatible = false;
                                break;
                            }
                        }
                    }

                    if !params_compatible {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            module_id.clone(),
                            crate::diagnostic::DiagnosticCode::EnumRequirementIncompatible,
                            format!(
                                "parameter types of `{}` on variant `{}` are incompatible with requirement",
                                case_sig.selector.encode(),
                                v.id.selector.encode()
                            ),
                            case_sig.source.as_ref().map(|s| s.range).unwrap_or_default(),
                        ));
                        statuses.push(CaseRequirementResult {
                            variant: v.id.clone(),
                            requirement: req.id.clone(),
                            status: CaseRequirementStatus::Incompatible {
                                implementation: case_sig.callable.clone(),
                            },
                        });
                        continue;
                    }

                    // Check return subtyping under specialized environment
                    let spec_req_ret = req_ret_ty.map(|t| case_subst.apply(store, t));
                    let case_ret = case_sig.declared_return.canonical_type();

                    let ret_compatible = match (spec_req_ret, case_ret) {
                        (Some(spec_ret), Some(case_ret_ty)) => crate::types::relation::is_subtype(store, hierarchy, case_ret_ty, spec_ret),
                        _ => false,
                    };

                    if ret_compatible {
                        statuses.push(CaseRequirementResult {
                            variant: v.id.clone(),
                            requirement: req.id.clone(),
                            status: CaseRequirementStatus::Satisfied {
                                implementation: case_sig.callable.clone(),
                            },
                        });
                    } else {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            module_id.clone(),
                            crate::diagnostic::DiagnosticCode::EnumRequirementIncompatible,
                            format!(
                                "return type of `{}` on variant `{}` is incompatible with requirement",
                                case_sig.selector.encode(),
                                v.id.selector.encode()
                            ),
                            case_sig.source.as_ref().map(|s| s.range).unwrap_or_default(),
                        ));
                        statuses.push(CaseRequirementResult {
                            variant: v.id.clone(),
                            requirement: req.id.clone(),
                            status: CaseRequirementStatus::Incompatible {
                                implementation: case_sig.callable.clone(),
                            },
                        });
                    }
                }
            }
        }
    }

    (Arc::from(statuses.into_boxed_slice()), Arc::from(diagnostics.into_boxed_slice()))
}
