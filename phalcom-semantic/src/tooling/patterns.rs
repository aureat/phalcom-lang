//! Protocol-neutral tooling products for pattern completion, missing-case plans, and match generation (Part 06).

use crate::enum_semantics::{EnumInfo, VariantInfo, VariantShape};
use crate::identity::{VariantFamilyId, VariantId};
use crate::match_semantics::{CoverageWitness, PatternSpaceSummary};
use crate::types::evidence::TypeKnowledge;
use phalcom_common::selector::Selector;

/// Candidate item for pattern position completion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatternCompletionCandidate {
    pub variant: VariantId,
    pub selector: Selector,
    pub family: Option<VariantFamilyId>,
    pub shape: VariantShape,
    pub label: Box<str>,
    pub insert_text: Box<str>,
    pub is_exact: bool,
    pub covers_residual: bool,
}

/// Protocol-neutral completion context for pattern positions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatternCompletionContext {
    pub expected: TypeKnowledge,
    pub residual: PatternSpaceSummary,
    pub candidates: Box<[PatternCompletionCandidate]>,
    pub wildcard_recommended: bool,
}

/// One planned missing arm for "Add Missing Cases".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissingCaseArmPlan {
    pub pattern_syntax: Box<str>,
    pub variant: Option<VariantId>,
}

/// Protocol-neutral edit plan for "Add Missing Cases".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissingCaseEditPlan {
    pub missing_arms: Box<[MissingCaseArmPlan]>,
}

fn format_variant_pattern(variant: &VariantInfo) -> String {
    let base_name = match &variant.id.selector.base {
        phalcom_common::selector::SelectorBase::Named(n) => n.as_str(),
        phalcom_common::selector::SelectorBase::Subscript => "[]",
    };
    match variant.shape {
        VariantShape::Singleton => variant.id.selector.to_string(),
        VariantShape::Constructor => {
            if variant.fields.is_empty() {
                format!("{}()", base_name)
            } else {
                let params: Vec<String> = variant
                    .fields
                    .iter()
                    .map(|f| {
                        if let Some(ref l) = f.external_label {
                            format!("{}: {}", l, f.local_name)
                        } else {
                            f.local_name.to_string()
                        }
                    })
                    .collect();
                format!("{}({})", base_name, params.join(", "))
            }
        }
    }
}

impl MissingCaseEditPlan {
    pub fn from_witnesses(witnesses: &[CoverageWitness], variants: &[&VariantInfo]) -> Self {
        let mut arms = Vec::new();
        for witness in witnesses {
            match witness {
                CoverageWitness::Variant { variant, .. } => {
                    let vinfo = variants.iter().find(|v| v.id == *variant);
                    let syntax = if let Some(v) = vinfo {
                        format_variant_pattern(v)
                    } else {
                        variant.selector.to_string()
                    };
                    arms.push(MissingCaseArmPlan {
                        pattern_syntax: syntax.into_boxed_str(),
                        variant: Some(variant.clone()),
                    });
                }
                CoverageWitness::Wildcard | CoverageWitness::Opaque(_) => {
                    arms.push(MissingCaseArmPlan {
                        pattern_syntax: "_".into(),
                        variant: None,
                    });
                }
                CoverageWitness::Tuple(_) => {
                    arms.push(MissingCaseArmPlan {
                        pattern_syntax: "(_, _)".into(),
                        variant: None,
                    });
                }
                CoverageWitness::List(_) => {
                    arms.push(MissingCaseArmPlan {
                        pattern_syntax: "[...]".into(),
                        variant: None,
                    });
                }
            }
        }
        Self {
            missing_arms: arms.into_boxed_slice(),
        }
    }
}

/// One planned arm for "Generate Match".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedMatchArmPlan {
    pub pattern_syntax: Box<str>,
    pub body_template: Box<str>,
}

/// Protocol-neutral plan for "Generate Match".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedMatchPlan {
    pub expression_syntax: Box<str>,
    pub arms: Box<[GeneratedMatchArmPlan]>,
}

impl GeneratedMatchPlan {
    /// Generates a match skeleton for all variants in an enum.
    pub fn from_enum_info(_enum_info: &EnumInfo, variants: &[&VariantInfo], expr_syntax: &str) -> Self {
        let mut arms = Vec::new();
        for variant in variants {
            let pat_str = format_variant_pattern(variant);
            arms.push(GeneratedMatchArmPlan {
                pattern_syntax: pat_str.into_boxed_str(),
                body_template: "".into(),
            });
        }
        Self {
            expression_syntax: expr_syntax.into(),
            arms: arms.into_boxed_slice(),
        }
    }
}
