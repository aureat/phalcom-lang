//! Canonical enum behavior and closed-requirement semantic builder.
//!
//! This module analyzes enum-root behavior (defaults and closed requirements)
//! and variant-local case behavior, building canonical semantic signatures
//! and requirement objects before session publication and body checking.

use super::context::CheckingContext;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::enum_requirements::{EnumRequirement, EnumRequirementId};
use crate::identity::{CallableOwnerId, DeclarationId, DispatchSide, VariantId};
use crate::signature::CallableSemanticSignature;
use phalcom_ast::ast::EnumDef;
use std::collections::{BTreeMap, HashSet};

/// Canonical semantic product containing all behavior declared on an enum root and its cases.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumBehaviorProduct {
    pub owner: DeclarationId,
    pub root_defaults: Box<[CallableSemanticSignature]>,
    pub root_requirements: Box<[EnumRequirement]>,
    pub case_implementations: BTreeMap<VariantId, Box<[CallableSemanticSignature]>>,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}

/// Builds the canonical [`EnumBehaviorProduct`] from [`InherentImplContribution`]s.
pub fn build_enum_behavior(
    ctx: &mut CheckingContext<'_>,
    owner: &DeclarationId,
    _enum_def: Option<&EnumDef>,
    contributions: &[crate::impls::InherentImplContribution],
) -> EnumBehaviorProduct {
    let mut diagnostics = Vec::new();
    let mut root_defaults = Vec::new();
    let mut root_requirements = Vec::new();
    let mut case_implementations: BTreeMap<VariantId, Vec<CallableSemanticSignature>> = BTreeMap::new();
    let mut seen_root_selectors = HashSet::new();

    let _root_owner = CallableOwnerId::Declaration(owner.clone());

    // Process normalized InherentImplContributions
    for contribution in contributions {
        diagnostics.extend(contribution.diagnostics.iter().cloned());

        match &contribution.target {
            crate::impls::InherentImplTarget::Declaration(target_decl) if target_decl == owner => {
                for member in contribution.members.iter() {
                    let side = member.signature.side;
                    let selector = &member.signature.selector;

                    if member.is_requirement {
                        if side == DispatchSide::Class {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                ctx.current_module.clone(),
                                DiagnosticCode::EnumCaseStaticBehaviorUnsupported,
                                format!("class-side requirement is not supported on enum `{}`", owner.name),
                                member.signature.source.as_ref().map(|s| s.range).unwrap_or_default(),
                            ));
                            continue;
                        }
                        if !seen_root_selectors.insert((selector.clone(), side)) {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                ctx.current_module.clone(),
                                DiagnosticCode::ImplMemberConflict,
                                format!("duplicate root requirement `{}` on enum `{}`", selector, owner.name),
                                member.signature.source.as_ref().map(|s| s.range).unwrap_or_default(),
                            ));
                            continue;
                        }
                        let req_id = EnumRequirementId::new(owner.clone(), selector.clone());
                        let req_source = member.signature.source.clone();
                        root_requirements.push(EnumRequirement {
                            id: req_id,
                            signature: member.signature.clone(),
                            source: req_source,
                        });
                    } else {
                        // Instance-side bodyful member -> root default
                        if side == DispatchSide::Instance {
                            if !seen_root_selectors.insert((selector.clone(), side)) {
                                diagnostics.push(SemanticDiagnostic::error_in(
                                    ctx.current_module.clone(),
                                    DiagnosticCode::ImplMemberConflict,
                                    format!("duplicate root member `{}` on enum `{}`", selector, owner.name),
                                    member.signature.source.as_ref().map(|s| s.range).unwrap_or_default(),
                                ));
                                continue;
                            }
                            root_defaults.push(member.signature.clone());
                        }
                    }
                }
            }
            crate::impls::InherentImplTarget::ExactEnumCase(variant_id) if &variant_id.owner == owner => {
                for member in contribution.members.iter() {
                    let side = member.signature.side;
                    let selector = &member.signature.selector;

                    if side == DispatchSide::Class {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::EnumCaseStaticBehaviorUnsupported,
                            format!("case-local behavior on variant `{}` cannot be class-side", variant_id.selector.encode()),
                            member.signature.source.as_ref().map(|s| s.range).unwrap_or_default(),
                        ));
                        continue;
                    }

                    let case_list = case_implementations.entry(variant_id.clone()).or_default();
                    if case_list.iter().any(|existing| existing.selector == *selector) {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::ImplMemberConflict,
                            format!("duplicate case implementation `{}` on variant `{}`", selector, variant_id.selector.encode()),
                            member.signature.source.as_ref().map(|s| s.range).unwrap_or_default(),
                        ));
                        continue;
                    }
                    case_list.push(member.signature.clone());
                }
            }
            _ => {}
        }
    }

    let case_boxed = case_implementations.into_iter().map(|(k, v)| (k, v.into_boxed_slice())).collect();

    EnumBehaviorProduct {
        owner: owner.clone(),
        root_defaults: root_defaults.into_boxed_slice(),
        root_requirements: root_requirements.into_boxed_slice(),
        case_implementations: case_boxed,
        diagnostics: diagnostics.into_boxed_slice(),
    }
}
