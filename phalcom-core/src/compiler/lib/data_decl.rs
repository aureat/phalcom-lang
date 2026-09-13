//! Data declaration lowering to Bytecode::Data, FinalizeData (Part 4).

use crate::bytecode::Bytecode;
use crate::compiler::lib::Compiler;
use crate::compiler::lib::error::CompilerError;
use crate::modules::semantic_lowering::{DataComponentLoweringSpec, DataDeclarationLoweringSpec};
use crate::value::Value;
use phalcom_ast::ast::DataDef;
use phalcom_modules::DeclarationId;
use phalcom_semantic::identity::DataComponentId;
use std::sync::Arc;

impl<'vm> Compiler<'vm> {
    /// Compiles a data declaration into runtime data behavior class and descriptor.
    pub fn compile_data(&mut self, data_def: &DataDef) -> Result<(), CompilerError> {
        let name_sym = self.vm.interner.intern(&data_def.name);
        self.known_globals.insert(name_sym);

        // 1. Locate or synthesize DataDeclarationLoweringSpec
        let spec = if let Some(lowering) = self.lowering() {
            let module_id = self.vm.heap.module(self.module).id.clone();
            lowering
                .data_decls
                .iter()
                .find(|d| d.owner.module == module_id && d.owner.name.as_ref() == data_def.name)
                .cloned()
                .ok_or(CompilerError::MissingDataLoweringSemantics(data_def.range))?
        } else {
            // Synthesize lowering spec for standalone/unlinked compiles
            let module_id = self.vm.heap.module(self.module).id.clone();
            let owner = DeclarationId::new(module_id, data_def.name.clone().into_boxed_str());
            let mut components = Vec::new();
            let mut component_specs = Vec::new();
            for (idx, comp) in data_def.shape.components().iter().enumerate() {
                let logical_index = u32::try_from(idx).map_err(|_| CompilerError::Message(format!("data `{}` has too many components", data_def.name)))?;
                components.push(DataComponentLoweringSpec {
                    id: DataComponentId::new(owner.clone(), logical_index),
                    local_name: comp.local_name.clone().into_boxed_str(),
                    logical_index,
                });
                component_specs.push(crate::product::ProductComponentSpec {
                    logical_index,
                    repr: crate::product::ProductSlotRepr::Value,
                });
            }
            let layout = crate::product::ProductLayoutSpec::new(component_specs);
            DataDeclarationLoweringSpec {
                owner,
                layout,
                components: components.into_boxed_slice(),
            }
        };

        let spec_idx = self
            .functions
            .last_mut()
            .unwrap()
            .chunk
            .executable_semantics
            .add_data_spec(Arc::new(spec.clone()), data_def.range)?;

        // 2. Emit Data class allocation (pushes data class on stack)
        self.emit(Bytecode::Data(spec_idx), data_def.range);

        // 3. Finalize data class
        self.emit(Bytecode::FinalizeData(spec_idx), data_def.range);

        // 4. Define global slot for the data class
        self.declare_global(name_sym, false)?;
        let name_idx = self.add_constant(Value::symbol(name_sym));
        self.emit(Bytecode::DefineGlobal(name_idx), data_def.range);

        Ok(())
    }
}
