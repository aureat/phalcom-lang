use crate::bytecode::Bytecode;
use crate::compiler::lib::Compiler;
use crate::compiler::lib::error::CompilerError;
use crate::modules::semantic_lowering::{LoweringSiteKind, MatchLoweringSpec};
use phalcom_ast::ast::{Expr, MatchExpr};
use phalcom_common::range::SourceRange;

impl<'vm> Compiler<'vm> {
    pub(crate) fn compile_match_expr(&mut self, node: MatchExpr) -> Result<(), CompilerError> {
        let range = node.range;
        let spec = self.lowering().and_then(|l| {
            l.matches
                .iter()
                .find(|(site, _)| site.range == range && site.kind == LoweringSiteKind::Match)
                .map(|(_, spec)| spec.clone())
        });

        let Some(spec) = spec else {
            return Err(CompilerError::MissingMatchLoweringSemantics(range));
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
