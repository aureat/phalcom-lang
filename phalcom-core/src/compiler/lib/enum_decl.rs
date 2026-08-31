//! Enum declaration lowering to Bytecode::Enum, VariantMethod, and FinalizeEnum (Part 4).

use crate::bytecode::Bytecode;
use crate::compiler::lib::Compiler;
use crate::compiler::lib::error::CompilerError;
use crate::method::MethodObject;
use crate::modules::semantic_lowering::{EnumLoweringSpec, VariantFieldLoweringSpec, VariantLoweringSpec};
use phalcom_ast::ast::{EnumBehaviorMember, EnumDef, EnumMember};
use phalcom_common::selector::Selector;
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
            lowering
                .enums
                .iter()
                .find(|e| e.owner.name.as_ref() == enum_def.name)
                .cloned()
                .ok_or(CompilerError::MissingEnumLoweringSemantics(enum_def.range))?
        } else {
            // Synthesize lowering spec for standalone/unlinked compiles
            let module_id = self.vm.heap.module(self.module).id.clone();
            let owner = DeclarationId::new(module_id, enum_def.name.clone().into_boxed_str());
            let mut variants = Vec::new();
            for m in &enum_def.members {
                if let EnumMember::Variant(v) = m {
                    let selector = if let Some(payload) = &v.payload {
                        let mut slots = Vec::new();
                        for p in &payload.parameters {
                            slots.push(phalcom_common::selector::SelectorSlot::Label(p.name.clone()));
                        }
                        match Selector::new(
                            phalcom_common::selector::SelectorBase::Named(v.name.clone()),
                            phalcom_common::selector::SelectorKind::Method,
                            slots.into_boxed_slice(),
                        ) {
                            Ok(s) => s,
                            Err(_) => return Err(CompilerError::MissingEnumLoweringSemantics(enum_def.range)),
                        }
                    } else {
                        match Selector::getter(v.name.clone()) {
                            Ok(s) => s,
                            Err(_) => return Err(CompilerError::MissingEnumLoweringSemantics(enum_def.range)),
                        }
                    };
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
                            fields.push(VariantFieldLoweringSpec {
                                id: VariantFieldId::new(vid.clone(), idx as u32),
                                local_name: field_name.into_boxed_str(),
                                slot: idx as u16,
                            });
                        }
                    }
                    variants.push(VariantLoweringSpec {
                        id: vid,
                        shape,
                        payload_fields: fields.into_boxed_slice(),
                    });
                }
            }
            EnumLoweringSpec {
                owner,
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

        // 2. Emit Enum root allocation
        self.emit(Bytecode::Enum(spec_idx), enum_def.range);

        // 3. Compile case-specific variant methods
        for m in &enum_def.members {
            if let EnumMember::Variant(v) = m {
                if let Some(body) = &v.body {
                    let v_spec = spec
                        .variants
                        .iter()
                        .find(|vs| vs.id.selector.base == phalcom_common::selector::SelectorBase::Named(v.name.clone()));
                    if let Some(v_spec) = v_spec {
                        let var_idx = self
                            .functions
                            .last_mut()
                            .unwrap()
                            .chunk
                            .executable_semantics
                            .add_variant_target(&v_spec.id, v.range)?;

                        for b_member in &body.members {
                            match b_member {
                                EnumBehaviorMember::Method(method_def) => {
                                    let sig_kind = crate::method::SignatureKind::Method(method_def.params.len() as u8);
                                    let sig_str = crate::method::make_signature(&method_def.name, sig_kind);
                                    let selector_sym = self.vm.interner.intern(&sig_str);
                                    let selector_const = self.add_constant(crate::value::Value::symbol(selector_sym));
                                    let param_names = method_def.params.iter().map(|p| p.name.clone()).collect();
                                    let body_stmts = method_def.body.statements().map(|s| s.to_vec()).unwrap_or_default();
                                    let closure = self.compile_block(
                                        body_stmts,
                                        selector_sym,
                                        phalcom_ast::ast::ClosureParameters::fixed(param_names),
                                        true,
                                        false,
                                        None,
                                    )?;
                                    let method_obj = self.vm.heap.alloc(crate::heap::Object::Method(Box::new(MethodObject::new_single(
                                        selector_sym,
                                        sig_kind,
                                        crate::method::MethodKind::Closure(closure),
                                    ))));
                                    let method_const = self.add_constant(crate::value::Value::obj(method_obj));
                                    self.emit(Bytecode::Constant(method_const), method_def.range);
                                    self.emit(
                                        Bytecode::VariantMethod {
                                            variant: var_idx,
                                            selector: selector_const,
                                        },
                                        method_def.range,
                                    );
                                }
                                EnumBehaviorMember::Getter(getter_def) => {
                                    let sig_kind = crate::method::SignatureKind::Getter;
                                    let sig_str = crate::method::make_signature(&getter_def.name, sig_kind);
                                    let selector_sym = self.vm.interner.intern(&sig_str);
                                    let selector_const = self.add_constant(crate::value::Value::symbol(selector_sym));
                                    let body_stmts = getter_def.body.statements().map(|s| s.to_vec()).unwrap_or_default();
                                    let closure = self.compile_block(
                                        body_stmts,
                                        selector_sym,
                                        phalcom_ast::ast::ClosureParameters::fixed(Vec::new()),
                                        true,
                                        false,
                                        None,
                                    )?;
                                    let method_obj = self.vm.heap.alloc(crate::heap::Object::Method(Box::new(MethodObject::new_single(
                                        selector_sym,
                                        sig_kind,
                                        crate::method::MethodKind::Closure(closure),
                                    ))));
                                    let method_const = self.add_constant(crate::value::Value::obj(method_obj));
                                    self.emit(Bytecode::Constant(method_const), getter_def.range);
                                    self.emit(
                                        Bytecode::VariantMethod {
                                            variant: var_idx,
                                            selector: selector_const,
                                        },
                                        getter_def.range,
                                    );
                                }
                                EnumBehaviorMember::Setter(_) | EnumBehaviorMember::Index(_) => {}
                            }
                        }
                    }
                }
            }
        }

        // 4. Finalize enum root & case classes
        self.emit(Bytecode::FinalizeEnum(spec_idx), enum_def.range);

        // 5. Define global slot for the enum root class
        self.declare_global(name_sym, false)?;
        let name_idx = self.add_constant(crate::value::Value::symbol(name_sym));
        self.emit(Bytecode::DefineGlobal(name_idx), enum_def.range);

        Ok(())
    }
}
