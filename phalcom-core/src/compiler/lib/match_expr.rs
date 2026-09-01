use crate::bytecode::Bytecode;
use crate::compiler::lib::Compiler;
use crate::compiler::lib::error::CompilerError;
use crate::modules::semantic_lowering::{
    ExecutableBindingSpec, ExecutableFieldProjection, ExecutableMatchArm, ExecutablePattern, ExecutableVariantCandidate,
    LoweringSiteKind, MatchLoweringSpec,
};
use phalcom_ast::ast::{Expr, MatchExpr, Pattern, VariantPatternMode};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::{DeclarationId, ModuleId};
use phalcom_semantic::identity::{VariantFieldId, VariantId};

impl<'vm> Compiler<'vm> {
    pub(crate) fn compile_match_expr(&mut self, node: MatchExpr) -> Result<(), CompilerError> {
        let range = node.range;
        let spec = if let Some(spec) = self.lowering().and_then(|l| {
            l.matches
                .iter()
                .find(|(site, _)| site.range == range && site.kind == LoweringSiteKind::Match)
                .map(|(_, spec)| spec.clone())
        }) {
            spec
        } else {
            // Synthesize fallback match lowering spec for standalone/unlinked compiles
            let module_id = self.vm.heap.module(self.module).id.clone();
            let mut arms = Vec::new();
            for (arm_idx, arm) in node.arms.iter().enumerate() {
                let mut binding_counter = 0;
                let mut bindings = Vec::new();
                let pattern = synthesize_fallback_pattern(&arm.pattern, &module_id, &mut binding_counter, &mut bindings)?;
                arms.push(ExecutableMatchArm {
                    arm_index: arm_idx as u32,
                    pattern,
                    bindings: bindings.into_boxed_slice(),
                });
            }
            MatchLoweringSpec {
                arms: arms.into_boxed_slice(),
            }
        };

        self.compile_match_with_spec(node, spec)
    }

    fn compile_match_with_spec(&mut self, node: MatchExpr, spec: MatchLoweringSpec) -> Result<(), CompilerError> {
        let range = node.range;
        self.begin_scope();

        // 1. Evaluate scrutinee exactly once and store in $match_scrutinee
        self.compile_expr(*node.value)?;
        let scrutinee_slot = self.reserve_pack_scratch("$match_scrutinee", range)?;
        self.emit(Bytecode::SetLocal(scrutinee_slot), range);
        self.emit(Bytecode::Pop, range);

        // 2. Reserve $match_result local for branch value join
        let result_slot = self.reserve_pack_scratch("$match_result", range)?;

        let mut end_jumps = Vec::new();

        for (arm_idx, arm) in node.arms.into_iter().enumerate() {
            let arm_spec = spec.arms.get(arm_idx).ok_or(CompilerError::InvalidExecutablePattern(arm.range))?;

            self.begin_scope();
            let frame = self.setup_pattern_frame(&arm_spec.bindings, arm.range)?;

            let mut failure_jumps = Vec::new();
            self.emit_executable_pattern(&arm_spec.pattern, scrutinee_slot, &frame, &mut failure_jumps, arm.range)?;

            let arm_scratch_count = (self.functions.last().unwrap().num_locals as usize) - (frame.scope_start as usize);

            // Success path: commit bindings to visible locals
            self.commit_pattern_frame(&frame, arm.range)?;

            // Compile arm branch (inline for block, expr otherwise)
            self.compile_match_branch(*arm.branch, arm.range)?;
            self.emit(Bytecode::SetLocal(result_slot), arm.range);
            self.emit(Bytecode::Pop, arm.range);

            self.end_scope(arm.range);
            self.emit_release_scratch_range(frame.scope_start, arm_scratch_count, arm.range);

            let jump_end = self.emit_forward_jump(Bytecode::Jump, arm.range);
            end_jumps.push(jump_end);

            // Failure path: next arm
            let next_arm_label = self.chunk_len();
            for jump in failure_jumps {
                self.patch_forward_jump_to(jump, next_arm_label);
            }
            self.emit_release_scratch_range(frame.scope_start, arm_scratch_count, arm.range);
        }

        // Exhaustive match internal fallthrough trap
        self.emit(Bytecode::MatchInvariantFailure, range);

        let end_label = self.chunk_len();
        for jump in end_jumps {
            self.patch_forward_jump_to(jump, end_label);
        }

        // Put result on top of operand stack
        self.emit(Bytecode::GetLocal(result_slot), range);

        self.emit_release_scratch_range(scrutinee_slot, 2, range);
        self.end_scope(range);
        Ok(())
    }

    fn compile_match_branch(&mut self, branch: Expr, range: SourceRange) -> Result<(), CompilerError> {
        match branch {
            Expr::Block(block) => self.compile_inline_block_body(*block),
            other => self.compile_expr(other),
        }
    }
}

fn synthesize_fallback_pattern(
    pat: &Pattern,
    module_id: &ModuleId,
    binding_counter: &mut u32,
    bindings: &mut Vec<ExecutableBindingSpec>,
) -> Result<ExecutablePattern, CompilerError> {
    match pat {
        Pattern::Wildcard { .. } => Ok(ExecutablePattern::Wildcard),
        Pattern::Name { name, range } => {
            let idx = *binding_counter;
            *binding_counter += 1;
            bindings.push(ExecutableBindingSpec {
                binding: phalcom_semantic::identity::BindingId(idx),
                name: name.clone().into_boxed_str(),
                range: *range,
            });
            Ok(ExecutablePattern::Binding {
                binding_index: idx,
                name: name.clone().into_boxed_str(),
            })
        }
        Pattern::Variant(v) => {
            let owner_name = v.owner.as_ref().map(|o| o.leaf_name()).unwrap_or_else(|| {
                if v.base == "Some" || v.base == "None" {
                    "Option"
                } else if v.base == "Ok" || v.base == "Error" || v.base == "Err" {
                    "Result"
                } else if v.base == "Less" || v.base == "Equal" || v.base == "Greater" || v.base == "Unordered" {
                    "Ordering"
                } else {
                    ""
                }
            });
            let is_core = owner_name == "Option" || owner_name == "Result" || owner_name == "Ordering";
            let owner_decl = DeclarationId::new(
                if is_core {
                    ModuleId::core()
                } else {
                    module_id.clone()
                },
                owner_name.into(),
            );

            let (selector, field_patterns) = match &v.mode {
                VariantPatternMode::Singleton => {
                    let sel = Selector::getter(v.base.clone()).map_err(|e| CompilerError::Message(e.to_string()))?;
                    (sel, Vec::new())
                }
                VariantPatternMode::ExactCall { arguments } => {
                    let mut slots = Vec::new();
                    let mut pats = Vec::new();
                    for arg in arguments {
                        if let Some(label) = &arg.label {
                            slots.push(phalcom_common::selector::SelectorSlot::Label(label.clone()));
                        } else {
                            slots.push(phalcom_common::selector::SelectorSlot::Positional);
                        }
                        pats.push(&arg.pattern);
                    }
                    let sel = Selector::method(v.base.clone(), slots).map_err(|e| CompilerError::Message(e.to_string()))?;
                    (sel, pats)
                }
                _ => return Err(CompilerError::InvalidExecutablePattern(v.range)),
            };

            let variant_id = VariantId::new(owner_decl, selector);
            let mut field_projections = Vec::new();
            for (idx, field_pat) in field_patterns.iter().enumerate() {
                let child = synthesize_fallback_pattern(field_pat, module_id, binding_counter, bindings)?;
                field_projections.push(ExecutableFieldProjection {
                    field_id: VariantFieldId::new(variant_id.clone(), idx as u32),
                    slot: idx as u16,
                    child,
                });
            }

            Ok(ExecutablePattern::Variant {
                candidates: Box::new([ExecutableVariantCandidate {
                    variant: variant_id,
                    fields: field_projections.into_boxed_slice(),
                }]),
            })
        }
        Pattern::Tuple { elements, .. } => {
            let mut el_pats = Vec::new();
            for el in elements {
                el_pats.push(synthesize_fallback_pattern(el, module_id, binding_counter, bindings)?);
            }
            Ok(ExecutablePattern::Tuple {
                elements: el_pats.into_boxed_slice(),
            })
        }
        Pattern::Or { alternatives, .. } => {
            let mut alt_pats = Vec::new();
            for alt in alternatives {
                alt_pats.push(synthesize_fallback_pattern(alt, module_id, binding_counter, bindings)?);
            }
            Ok(ExecutablePattern::Or {
                alternatives: alt_pats.into_boxed_slice(),
            })
        }
        Pattern::List { elements, rest, .. } => {
            let mut el_pats = Vec::new();
            for el in elements {
                el_pats.push(synthesize_fallback_pattern(el, module_id, binding_counter, bindings)?);
            }
            let rest_pat = if let Some(r) = rest {
                Some(Box::new(synthesize_fallback_pattern(r, module_id, binding_counter, bindings)?))
            } else {
                None
            };
            Ok(ExecutablePattern::List {
                elements: el_pats.into_boxed_slice(),
                rest: rest_pat,
            })
        }
        Pattern::Record { .. } | Pattern::Map { .. } => {
            Err(CompilerError::InvalidExecutablePattern(pat.range()))
        }
    }
}
