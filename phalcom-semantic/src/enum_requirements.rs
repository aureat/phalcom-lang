//! Closed-enum method requirements and case-obligation checking.

use crate::diagnostic::SemanticDiagnostic;
use crate::identity::{CallableId, DeclarationId, SemanticSourceSpan, VariantId};
use crate::signature::CallableSemanticSignature;
use phalcom_common::selector::Selector;
use std::collections::HashMap;
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

/// Snapshot-level table of closed-enum requirements and case statuses.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnumRequirementTable {
    pub requirements: HashMap<DeclarationId, Arc<[EnumRequirement]>>,
    pub case_statuses: HashMap<DeclarationId, Arc<[CaseRequirementResult]>>,
}

impl EnumRequirementTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, owner: DeclarationId, requirements: Arc<[EnumRequirement]>, statuses: Arc<[CaseRequirementResult]>) {
        self.requirements.insert(owner.clone(), requirements);
        self.case_statuses.insert(owner, statuses);
    }

    pub fn remove_module(&mut self, module: &phalcom_modules::identity::ModuleId) {
        self.requirements.retain(|owner, _| &owner.module != module);
        self.case_statuses.retain(|owner, _| &owner.module != module);
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
        let req_params_incomplete = req.signature.parameters.iter().any(|p| p.declared_type.is_unknown());

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
                        let req_param_ty = rp.declared_type.canonical_type().map(|t| case_subst.apply(store, t));
                        let case_param_ty = cp.declared_type.canonical_type();

                        match (req_param_ty, case_param_ty) {
                            (Some(rt), Some(ct)) => {
                                if rt != ct {
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
