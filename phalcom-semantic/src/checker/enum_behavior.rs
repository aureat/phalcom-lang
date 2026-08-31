//! Canonical enum behavior and closed-requirement semantic builder.
//!
//! This module analyzes enum-root behavior (defaults and closed requirements)
//! and variant-local case behavior, building canonical semantic signatures
//! and requirement objects before session publication and body checking.

use super::context::CheckingContext;
use super::declaration_signature::{CallableSyntaxRef, semantic_signature_for_syntax};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic, SemanticSourceSpan};
use crate::enum_requirements::{EnumRequirement, EnumRequirementId};
use crate::identity::{CallableOwnerId, DeclarationId, DispatchSide, VariantId};
use crate::signature::CallableSemanticSignature;
use phalcom_ast::ast::{EnumBehaviorMember, EnumDef, EnumMember};
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

/// Builds the canonical [`EnumBehaviorProduct`] from an [`EnumDef`].
pub fn build_enum_behavior(ctx: &mut CheckingContext<'_>, owner: &DeclarationId, enum_def: &EnumDef) -> EnumBehaviorProduct {
    let mut diagnostics = Vec::new();
    let mut root_defaults = Vec::new();
    let mut root_requirements = Vec::new();
    let mut case_implementations: BTreeMap<VariantId, Vec<CallableSemanticSignature>> = BTreeMap::new();
    let mut seen_root_selectors = HashSet::new();

    let root_owner = CallableOwnerId::Declaration(owner.clone());

    for member in &enum_def.members {
        match member {
            EnumMember::Behavior(behavior) => {
                let syntax = CallableSyntaxRef::from(behavior);
                let is_class_side = syntax.attributes().iter().any(|attr| attr.name == "class")
                    || match behavior {
                        EnumBehaviorMember::Method(m) => m.is_static,
                        EnumBehaviorMember::Getter(g) => g.is_static,
                        EnumBehaviorMember::Setter(s) => s.is_static,
                        EnumBehaviorMember::Index(_) => false,
                    };
                let side = if is_class_side { DispatchSide::Class } else { DispatchSide::Instance };

                let has_body = syntax.has_body();

                if has_body {
                    if let Some(sig) = semantic_signature_for_syntax(ctx, &root_owner, syntax, side) {
                        if seen_root_selectors.insert((sig.selector.clone(), side)) {
                            root_defaults.push(sig);
                        }
                    }
                } else {
                    // Signature-only root member -> closed-enum requirement
                    if side == DispatchSide::Class {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::EnumCaseStaticBehaviorUnsupported,
                            format!("class-side requirement is not supported on enum `{}`", owner.name),
                            syntax.range(),
                        ));
                    } else if let Some(sig) = semantic_signature_for_syntax(ctx, &root_owner, syntax, DispatchSide::Instance) {
                        if seen_root_selectors.insert((sig.selector.clone(), DispatchSide::Instance)) {
                            let req_id = EnumRequirementId::new(owner.clone(), sig.selector.clone());
                            let req_source = Some(SemanticSourceSpan::new(ctx.current_module.clone(), syntax.range()));
                            root_requirements.push(EnumRequirement {
                                id: req_id,
                                signature: sig,
                                source: req_source,
                            });
                        }
                    }
                }
            }
            EnumMember::Variant(variant_decl) => {
                let selector = phalcom_ast::selector::selector_from_variant(variant_decl);
                let variant_id = VariantId::new(owner.clone(), selector);
                let variant_owner = CallableOwnerId::Variant(variant_id.clone());

                if let Some(ref body) = variant_decl.body {
                    let mut seen_case_selectors = HashSet::new();
                    for case_member in &body.members {
                        let syntax = CallableSyntaxRef::from(case_member);
                        let is_class_side = syntax.attributes().iter().any(|attr| attr.name == "class")
                            || match case_member {
                                EnumBehaviorMember::Method(m) => m.is_static,
                                EnumBehaviorMember::Getter(g) => g.is_static,
                                EnumBehaviorMember::Setter(s) => s.is_static,
                                EnumBehaviorMember::Index(_) => false,
                            };

                        if is_class_side {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                ctx.current_module.clone(),
                                DiagnosticCode::EnumCaseStaticBehaviorUnsupported,
                                format!("case-local behavior on variant `{}` cannot be class-side", variant_decl.name),
                                syntax.range(),
                            ));
                            continue;
                        }

                        if !syntax.has_body() {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                ctx.current_module.clone(),
                                DiagnosticCode::EnumCaseDeclarationOnlyBehavior,
                                format!("case-local behavior on variant `{}` must have an executable body", variant_decl.name),
                                syntax.range(),
                            ));
                            continue;
                        }

                        if let Some(sig) = semantic_signature_for_syntax(ctx, &variant_owner, syntax, DispatchSide::Instance) {
                            if seen_case_selectors.insert(sig.selector.clone()) {
                                case_implementations.entry(variant_id.clone()).or_default().push(sig);
                            }
                        }
                    }
                }
            }
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
