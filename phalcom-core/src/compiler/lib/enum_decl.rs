//! Enum declaration lowering to Bytecode::Enum, VariantMethod, and FinalizeEnum.

use crate::bytecode::Bytecode;
use crate::compiler::lib::Compiler;
use crate::compiler::lib::error::CompilerError;
use crate::modules::semantic_lowering::{EnumLoweringSpec, VariantFieldLoweringSpec, VariantLoweringSpec};
use crate::value::Value;
use phalcom_ast::ast::EnumDef;
use phalcom_modules::DeclarationId;
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::{VariantFieldId, VariantId};
use std::sync::Arc;

impl<'vm> Compiler<'vm> {
    /// Compiles an enum declaration into runtime enum root and variant behavior classes.
    pub fn compile_enum(&mut self, enum_def: &EnumDef) -> Result<(), CompilerError> {
        let name_sym = self.vm.interner.intern(&enum_def.name);
        self.known_globals.insert(name_sym);

        // 1. Locate or synthesize EnumLoweringSpec
        let spec = if let Some(lowering) = self.lowering() {
            let module_id = self.vm.heap.module(self.module).id.clone();
            lowering
                .enums
                .iter()
                .find(|e| e.owner.module == module_id && e.owner.name.as_ref() == enum_def.name)
                .cloned()
                .ok_or(CompilerError::MissingEnumLoweringSemantics(enum_def.range))?
        } else {
            // Synthesize lowering spec for standalone/unlinked compiles
            let module_id = self.vm.heap.module(self.module).id.clone();
            let owner = DeclarationId::new(module_id, enum_def.name.clone().into_boxed_str());
            let mut variants = Vec::new();
            for v in &enum_def.variants {
                let selector = phalcom_ast::selector::selector_from_variant(v);
                let vid = VariantId::new(owner.clone(), selector);
                let shape = if v.payload.is_some() {
                    VariantShape::Constructor
                } else {
                    VariantShape::Singleton
                };
                let mut fields = Vec::new();
                if let Some(payload) = &v.payload {
                    for (idx, p) in payload.parameters.iter().enumerate() {
                        let field_name = p.name.clone();
                        let field_index =
                            u32::try_from(idx).map_err(|_| CompilerError::Message(format!("variant `{}` has too many payload fields", v.name)))?;
                        let slot = u16::try_from(idx).map_err(|_| CompilerError::Message(format!("variant `{}` has too many payload slots", v.name)))?;
                        fields.push(VariantFieldLoweringSpec {
                            id: VariantFieldId::new(vid.clone(), field_index),
                            local_name: field_name.into_boxed_str(),
                            slot,
                        });
                    }
                }
                variants.push(VariantLoweringSpec {
                    id: vid,
                    shape,
                    // Standalone compilation has no canonical type facts: keep universal slots.
                    layout: (!phalcom_semantic::core_surface::CoreDeclarationIds::default().is_option(&owner)).then(|| {
                        crate::product::ProductLayoutSpec::new(
                            fields
                                .iter()
                                .map(|field| crate::product::ProductComponentSpec {
                                    logical_index: u32::from(field.slot),
                                    repr: crate::product::ProductSlotRepr::Value,
                                })
                                .collect(),
                        )
                    }),
                    payload_fields: fields.into_boxed_slice(),
                });
            }
            let core_ids = phalcom_semantic::core_surface::CoreDeclarationIds::default();
            let representation = if core_ids.is_option(&owner) {
                crate::adt::RuntimeAdtRepresentation::NativeOption
            } else {
                crate::adt::RuntimeAdtRepresentation::General
            };
            EnumLoweringSpec {
                owner,
                representation,
                variants: variants.into_boxed_slice(),
            }
        };

        let spec_idx = self
            .functions
            .last_mut()
            .unwrap()
            .chunk
            .executable_semantics
            .add_enum_spec(Arc::new(spec.clone()), enum_def.range)?;

        // 2. Emit Enum root allocation (pushes root class on stack)
        self.emit(Bytecode::Enum(spec_idx), enum_def.range);

        // 3. Install accepted root inherent behavior while the enum root class
        // remains on the stack.
        self.install_accepted_inherent_impl_members(&spec.owner)?;

        // 4. Install accepted exact-case inherent behavior for each variant.
        for v_spec in spec.variants.iter() {
            self.install_accepted_exact_case_impl_members(&v_spec.id)?;
        }

        // 5. Finalize enum root & case classes
        self.emit(Bytecode::FinalizeEnum(spec_idx), enum_def.range);

        // 6. Define global slot for the enum root class
        self.declare_global(name_sym, false)?;
        let name_idx = self.add_constant(Value::symbol(name_sym));
        self.emit(Bytecode::DefineGlobal(name_idx), enum_def.range);

        Ok(())
    }
}
