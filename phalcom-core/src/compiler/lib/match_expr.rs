use crate::bytecode::Bytecode;
use crate::compiler::lib::Compiler;
use crate::compiler::lib::error::CompilerError;
use crate::compiler::lib::scope::BareNameResolution;
use crate::modules::semantic_lowering::{ExecutableBindingSpec, ExecutableMatchArm, ExecutablePattern, LoweringSiteKind, MatchLoweringSpec};
use phalcom_ast::ast::{Expr, MatchExpr, Pattern};
use phalcom_common::range::SourceRange;

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
            // Standalone/unlinked compilation may synthesize only structural
            // patterns whose runtime meaning is syntax-complete. Variant
            // identity is semantic and therefore cannot be guessed here.
            let mut arms = Vec::new();
            for (arm_idx, arm) in node.arms.iter().enumerate() {
                let mut binding_counter = 0;
                let mut bindings = Vec::new();
                let pattern = synthesize_fallback_pattern(&arm.pattern, &mut binding_counter, &mut bindings)?;
                arms.push(ExecutableMatchArm {
                    arm_index: arm_idx as u32,
                    pattern,
                    bindings: bindings.into_boxed_slice(),
                });
            }
            MatchLoweringSpec { arms: arms.into_boxed_slice() }
        };

        self.compile_match_with_spec(node, spec)
    }

    fn compile_match_with_spec(&mut self, node: MatchExpr, spec: MatchLoweringSpec) -> Result<(), CompilerError> {
        let range = node.range;
        self.begin_scope();

        let active_variant = if let Expr::Var { value, .. } = &*node.value {
            let sym = self.vm.interner.intern(value);
            if let BareNameResolution::Local(slot) = self.resolve_bare_name(sym) {
                self.active_virtual_product(slot).cloned()
            } else {
                None
            }
        } else {
            None
        };

        if let Some(active_variant) = active_variant {
            if let super::product_opt::VirtualProductKind::Variant { variant: known_variant } = &active_variant.shape.kind {
                let result_slot = self.reserve_pack_scratch("$match_result", range)?;
                let mut end_jumps = Vec::new();

                for (arm_idx, arm) in node.arms.into_iter().enumerate() {
                    let arm_spec = spec.arms.get(arm_idx).ok_or(CompilerError::InvalidExecutablePattern(arm.range))?;

                    self.begin_scope();
                    let frame = self.setup_pattern_frame(&arm_spec.bindings, arm.range)?;

                    let mut failure_jumps = Vec::new();
                    self.emit_virtual_variant_pattern(&arm_spec.pattern, &active_variant, known_variant, &frame, &mut failure_jumps, arm.range)?;

                    let arm_scratch_count = self.functions.last().unwrap().num_locals - frame.scope_start as usize;

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

                self.emit_release_scratch_range(result_slot, 1, range);
                self.end_scope(range);
                return Ok(());
            }
        }

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

            let arm_scratch_count = self.functions.last().unwrap().num_locals - frame.scope_start as usize;

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

    fn emit_virtual_variant_pattern(
        &mut self,
        pattern: &ExecutablePattern,
        active_variant: &super::product_opt::ActiveVirtualProduct,
        known_variant: &phalcom_semantic::identity::VariantId,
        frame: &super::patterns::PatternExecutionFrame,
        failures: &mut Vec<usize>,
        range: SourceRange,
    ) -> Result<(), CompilerError> {
        match pattern {
            ExecutablePattern::Wildcard => Ok(()),
            ExecutablePattern::Binding { .. } => Err(CompilerError::Message("cannot bind whole unmaterialized variant in pattern".to_string())),
            ExecutablePattern::Variant { candidates } => {
                let matching_candidate = candidates.iter().find(|c| c.variant == *known_variant);
                let Some(candidate) = matching_candidate else {
                    let fail_jump = self.emit_forward_jump(Bytecode::Jump, range);
                    failures.push(fail_jump);
                    return Ok(());
                };

                for field_proj in candidate.fields.iter() {
                    if matches!(field_proj.child, ExecutablePattern::Wildcard) {
                        continue;
                    }
                    if let ExecutablePattern::Binding { binding_index, .. } = &field_proj.child {
                        if let Some(&stage_slot) = frame.staging_slots.get(*binding_index as usize) {
                            let leaf_slot = active_variant.head_slot + field_proj.slot;
                            self.emit(Bytecode::GetLocal(leaf_slot), range);
                            self.emit(Bytecode::SetLocal(stage_slot), range);
                            self.emit(Bytecode::Pop, range);
                        }
                        continue;
                    }

                    // Structured or refutable child
                    let child_temp = self.reserve_pack_scratch("$variant_child", range)?;
                    let leaf_slot = active_variant.head_slot + field_proj.slot;
                    self.emit(Bytecode::GetLocal(leaf_slot), range);
                    self.emit(Bytecode::SetLocal(child_temp), range);
                    self.emit(Bytecode::Pop, range);

                    self.emit_executable_pattern(&field_proj.child, child_temp, frame, failures, range)?;
                }
                Ok(())
            }
            ExecutablePattern::Or { alternatives } => {
                let mut alt_success_jumps = Vec::new();
                let alt_count = alternatives.len();

                for (idx, alt) in alternatives.iter().enumerate() {
                    let is_last = idx == alt_count - 1;
                    let mut alt_failures = Vec::new();

                    self.emit_virtual_variant_pattern(alt, active_variant, known_variant, frame, &mut alt_failures, range)?;

                    if !is_last {
                        let succ_jump = self.emit_forward_jump(Bytecode::Jump, range);
                        alt_success_jumps.push(succ_jump);

                        let next_alt_label = self.chunk_len();
                        for j in alt_failures {
                            self.patch_forward_jump_to(j, next_alt_label);
                        }
                    } else {
                        failures.extend(alt_failures);
                    }
                }

                let success_label = self.chunk_len();
                for j in alt_success_jumps {
                    self.patch_forward_jump_to(j, success_label);
                }

                Ok(())
            }
            _ => Err(CompilerError::InvalidExecutablePattern(range)),
        }
    }
}

fn synthesize_fallback_pattern(
    pat: &Pattern,
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
        Pattern::Variant(v) => Err(CompilerError::MissingMatchLoweringSemantics(v.range)),
        Pattern::Tuple { elements, .. } => {
            let mut el_pats = Vec::new();
            for el in elements {
                el_pats.push(synthesize_fallback_pattern(el, binding_counter, bindings)?);
            }
            Ok(ExecutablePattern::Tuple {
                elements: el_pats.into_boxed_slice(),
            })
        }
        Pattern::Or { alternatives, .. } => {
            let mut alt_pats = Vec::new();
            for alt in alternatives {
                alt_pats.push(synthesize_fallback_pattern(alt, binding_counter, bindings)?);
            }
            Ok(ExecutablePattern::Or {
                alternatives: alt_pats.into_boxed_slice(),
            })
        }
        Pattern::List { elements, rest, .. } => {
            let mut el_pats = Vec::new();
            for el in elements {
                el_pats.push(synthesize_fallback_pattern(el, binding_counter, bindings)?);
            }
            let rest_pat = if let Some(r) = rest {
                Some(Box::new(synthesize_fallback_pattern(r, binding_counter, bindings)?))
            } else {
                None
            };
            Ok(ExecutablePattern::List {
                elements: el_pats.into_boxed_slice(),
                rest: rest_pat,
            })
        }
        Pattern::Record { .. } | Pattern::Map { .. } => Err(CompilerError::InvalidExecutablePattern(pat.range())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::ast::{VariantPattern, VariantPatternMode};

    #[test]
    fn fallback_variant_pattern_requires_semantic_identity() {
        let range = SourceRange::new(0, 4);
        let pattern = Pattern::Variant(VariantPattern {
            owner: None,
            base: "None".into(),
            base_range: range,
            mode: VariantPatternMode::Singleton,
            range,
        });
        let mut binding_counter = 0;
        let mut bindings = Vec::new();

        assert!(matches!(
            synthesize_fallback_pattern(&pattern, &mut binding_counter, &mut bindings),
            Err(CompilerError::MissingMatchLoweringSemantics(error_range)) if error_range == range
        ));
    }
}
