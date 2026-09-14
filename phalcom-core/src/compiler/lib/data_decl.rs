//! Data declaration lowering to Bytecode::Data, FinalizeData (Part 4).

use crate::bytecode::Bytecode;
use crate::compiler::lib::Compiler;
use crate::compiler::lib::error::CompilerError;
use crate::value::Value;
use phalcom_ast::ast::DataDef;
use std::sync::Arc;

impl<'vm> Compiler<'vm> {
    /// Compiles a data declaration into runtime data behavior class and descriptor.
    pub fn compile_data(&mut self, data_def: &DataDef) -> Result<(), CompilerError> {
        let name_sym = self.vm.interner.intern(&data_def.name);
        self.known_globals.insert(name_sym);

        // 1. Locate DataDeclarationLoweringSpec from semantic lowering
        let module_id = self.vm.heap.module(self.module).id.clone();
        let spec = self
            .lowering()
            .and_then(|lowering| {
                lowering
                    .data_decls
                    .iter()
                    .find(|d| d.owner.module == module_id && d.owner.name.as_ref() == data_def.name)
                    .cloned()
            })
            .ok_or(CompilerError::MissingDataLoweringSemantics(data_def.range))?;

        let spec_idx = self
            .functions
            .last_mut()
            .unwrap()
            .chunk
            .executable_semantics
            .add_data_spec(Arc::new(spec.clone()), data_def.range)?;

        // 2. Emit Data class allocation (pushes data class on stack)
        self.emit(Bytecode::Data(spec_idx), data_def.range);

        // 3. Install accepted inherent behavior while the data behavior class
        // remains on the stack. The helper uses target-owned semantic
        // provenance and does not alter data layout or representation.
        self.install_accepted_inherent_impl_members(&spec.owner)?;

        // 4. Finalize data class
        self.emit(Bytecode::FinalizeData(spec_idx), data_def.range);

        // 5. Define global slot for the data class
        self.declare_global(name_sym, false)?;
        let name_idx = self.add_constant(Value::symbol(name_sym));
        self.emit(Bytecode::DefineGlobal(name_idx), data_def.range);

        Ok(())
    }
}
