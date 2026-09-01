//! Enum declaration lowering to Bytecode::Enum, VariantMethod, and FinalizeEnum (Part 4).

use crate::bytecode::Bytecode;
use crate::compiler::lib::Compiler;
use crate::compiler::lib::checked_send_arity;
use crate::compiler::lib::error::CompilerError;
use crate::heap::Object;
use crate::method::{MemberVisibility, MethodKind, MethodObject, SignatureKind, encode_selector, make_signature};
use crate::modules::semantic_lowering::{EnumLoweringSpec, VariantFieldLoweringSpec, VariantLoweringSpec};
use crate::value::Value;
use phalcom_ast::ast::{AttrKind, Attribute, BuiltinAttr, ClosureParameters, EnumBehaviorMember, EnumDef, EnumMember, IndexAccessor, MemberBody};
use phalcom_modules::DeclarationId;
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::{VariantFieldId, VariantId};
use std::sync::Arc;

fn member_visibility(name: Option<&str>, attributes: &[Attribute]) -> MemberVisibility {
    if name.is_some_and(|name| name.starts_with("_$")) {
        MemberVisibility::Internal
    } else if attributes.iter().any(|attr| matches!(attr.kind, AttrKind::Builtin(BuiltinAttr::Private))) {
        MemberVisibility::Private
    } else if attributes.iter().any(|attr| matches!(attr.kind, AttrKind::Builtin(BuiltinAttr::Protected))) {
        MemberVisibility::Protected
    } else {
        MemberVisibility::Public
    }
}
fn has_class_attr(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|a| a.name == "class")
}

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
                        payload_fields: fields.into_boxed_slice(),
                    });
                }
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

        // 3. Compile root bodyful behavior onto the root class
        for m in &enum_def.members {
            if let EnumMember::Behavior(b) = m {
                match b {
                    EnumBehaviorMember::Method(method_def) => {
                        let body_stmts = match &method_def.body {
                            MemberBody::Block(stmts) => stmts.clone(),
                            MemberBody::Declaration => continue, // Static requirement: no body
                        };
                        let arity = checked_send_arity("method declaration", method_def.params.len(), method_def.range)?;
                        let labels: Vec<Option<String>> = method_def.params.iter().map(|p| p.label.clone()).collect();
                        let sig_kind = SignatureKind::Method(arity);
                        let selector = encode_selector(&method_def.name, &labels, sig_kind);
                        let selector_sym = self.vm.interner.intern(&selector);

                        let param_names = method_def.params.iter().map(|p| p.name.clone()).collect();
                        let is_static = method_def.is_static || has_class_attr(&method_def.attributes);
                        self.is_static_context = is_static;
                        let closure = self.compile_block(body_stmts, selector_sym, ClosureParameters::fixed(param_names), true, false, None)?;
                        let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                            selector_sym,
                            sig_kind,
                            MethodKind::Closure(closure),
                        ))));
                        self.vm.heap.method_mut(method_obj).visibility = member_visibility(Some(&method_def.name), &method_def.attributes);
                        let method_const = self.add_constant(Value::obj(method_obj));
                        self.emit(Bytecode::Constant(method_const), method_def.range);
                        let selector_idx = self.add_constant(Value::symbol(selector_sym));
                        self.emit(Bytecode::Method(selector_idx, is_static), method_def.range);
                    }
                    EnumBehaviorMember::Getter(getter_def) => {
                        let body_stmts = match &getter_def.body {
                            MemberBody::Block(stmts) => stmts.clone(),
                            MemberBody::Declaration => continue,
                        };
                        let sig_kind = SignatureKind::Getter;
                        let selector = make_signature(&getter_def.name, sig_kind);
                        let selector_sym = self.vm.interner.intern(&selector);
                        let is_static = getter_def.is_static || has_class_attr(&getter_def.attributes);
                        self.is_static_context = is_static;
                        let closure = self.compile_block(body_stmts, selector_sym, ClosureParameters::default(), true, false, None)?;
                        let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                            selector_sym,
                            sig_kind,
                            MethodKind::Closure(closure),
                        ))));
                        self.vm.heap.method_mut(method_obj).visibility = member_visibility(Some(&getter_def.name), &getter_def.attributes);
                        let method_const = self.add_constant(Value::obj(method_obj));
                        self.emit(Bytecode::Constant(method_const), getter_def.range);
                        let selector_idx = self.add_constant(Value::symbol(selector_sym));
                        self.emit(Bytecode::Method(selector_idx, is_static), getter_def.range);
                    }
                    EnumBehaviorMember::Setter(setter_def) => {
                        let body_stmts = match &setter_def.body {
                            MemberBody::Block(stmts) => stmts.clone(),
                            MemberBody::Declaration => continue,
                        };
                        let sig_kind = SignatureKind::Setter;
                        let selector = make_signature(&setter_def.name, sig_kind);
                        let selector_sym = self.vm.interner.intern(&selector);
                        let is_static = setter_def.is_static || has_class_attr(&setter_def.attributes);
                        self.is_static_context = is_static;
                        let closure = self.compile_block(
                            body_stmts,
                            selector_sym,
                            ClosureParameters::fixed(vec![setter_def.param.name.clone()]),
                            true,
                            false,
                            None,
                        )?;
                        let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                            selector_sym,
                            sig_kind,
                            MethodKind::Closure(closure),
                        ))));
                        self.vm.heap.method_mut(method_obj).visibility = member_visibility(Some(&setter_def.name), &setter_def.attributes);
                        let method_const = self.add_constant(Value::obj(method_obj));
                        self.emit(Bytecode::Constant(method_const), setter_def.range);
                        let selector_idx = self.add_constant(Value::symbol(selector_sym));
                        self.emit(Bytecode::Method(selector_idx, is_static), setter_def.range);
                    }
                    EnumBehaviorMember::Index(index_def) => {
                        let arity = checked_send_arity("subscript declaration", index_def.params.len(), index_def.range)?;
                        let labels: Vec<Option<String>> = index_def.params.iter().map(|p| p.label.clone()).collect();
                        let mut param_names: Vec<String> = index_def.params.iter().map(|p| p.name.clone()).collect();
                        let sig_kind = match &index_def.accessor {
                            IndexAccessor::Get => SignatureKind::SubscriptGet(arity),
                            IndexAccessor::Set { put } => {
                                checked_send_arity("subscript declaration", index_def.params.len() + 1, index_def.range)?;
                                param_names.push(put.name.clone());
                                SignatureKind::SubscriptSet(arity)
                            }
                        };
                        let selector = encode_selector("", &labels, sig_kind);
                        let selector_sym = self.vm.interner.intern(&selector);
                        self.is_static_context = false;
                        let closure = self.compile_block(index_def.body.clone(), selector_sym, ClosureParameters::fixed(param_names), true, false, None)?;
                        let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                            selector_sym,
                            sig_kind,
                            MethodKind::Closure(closure),
                        ))));
                        self.vm.heap.method_mut(method_obj).visibility = member_visibility(None, &index_def.attributes);
                        let method_const = self.add_constant(Value::obj(method_obj));
                        self.emit(Bytecode::Constant(method_const), index_def.range);
                        let selector_idx = self.add_constant(Value::symbol(selector_sym));
                        self.emit(Bytecode::Method(selector_idx, false), index_def.range);
                    }
                }
            }
        }

        // 4. Compile case-specific variant methods
        for m in &enum_def.members {
            if let EnumMember::Variant(v) = m {
                if let Some(body) = &v.body {
                    let expected_selector = phalcom_ast::selector::selector_from_variant(v);
                    let v_spec = spec.variants.iter().find(|vs| vs.id.selector == expected_selector);
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
                                    if method_def.is_static || has_class_attr(&method_def.attributes) {
                                        return Err(CompilerError::IllegalStaticOnVariantMember(method_def.name.clone(), method_def.range));
                                    }
                                    let body_stmts = match &method_def.body {
                                        MemberBody::Block(stmts) => stmts.clone(),
                                        MemberBody::Declaration => {
                                            return Err(CompilerError::DeclarationBodyRequiresImplementation(method_def.name.clone(), method_def.range));
                                        }
                                    };
                                    let arity = checked_send_arity("method declaration", method_def.params.len(), method_def.range)?;
                                    let labels: Vec<Option<String>> = method_def.params.iter().map(|p| p.label.clone()).collect();
                                    let sig_kind = SignatureKind::Method(arity);
                                    let selector = encode_selector(&method_def.name, &labels, sig_kind);
                                    let selector_sym = self.vm.interner.intern(&selector);
                                    let selector_const = self.add_constant(Value::symbol(selector_sym));
                                    let param_names = method_def.params.iter().map(|p| p.name.clone()).collect();
                                    self.is_static_context = false;
                                    let closure = self.compile_block(body_stmts, selector_sym, ClosureParameters::fixed(param_names), true, false, None)?;
                                    let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                                        selector_sym,
                                        sig_kind,
                                        MethodKind::Closure(closure),
                                    ))));
                                    self.vm.heap.method_mut(method_obj).visibility = member_visibility(Some(&method_def.name), &method_def.attributes);
                                    let method_const = self.add_constant(Value::obj(method_obj));
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
                                    if getter_def.is_static || has_class_attr(&getter_def.attributes) {
                                        return Err(CompilerError::IllegalStaticOnVariantMember(getter_def.name.clone(), getter_def.range));
                                    }
                                    let body_stmts = match &getter_def.body {
                                        MemberBody::Block(stmts) => stmts.clone(),
                                        MemberBody::Declaration => {
                                            return Err(CompilerError::DeclarationBodyRequiresImplementation(getter_def.name.clone(), getter_def.range));
                                        }
                                    };
                                    let sig_kind = SignatureKind::Getter;
                                    let sig_str = make_signature(&getter_def.name, sig_kind);
                                    let selector_sym = self.vm.interner.intern(&sig_str);
                                    let selector_const = self.add_constant(Value::symbol(selector_sym));
                                    self.is_static_context = false;
                                    let closure = self.compile_block(body_stmts, selector_sym, ClosureParameters::default(), true, false, None)?;
                                    let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                                        selector_sym,
                                        sig_kind,
                                        MethodKind::Closure(closure),
                                    ))));
                                    self.vm.heap.method_mut(method_obj).visibility = member_visibility(Some(&getter_def.name), &getter_def.attributes);
                                    let method_const = self.add_constant(Value::obj(method_obj));
                                    self.emit(Bytecode::Constant(method_const), getter_def.range);
                                    self.emit(
                                        Bytecode::VariantMethod {
                                            variant: var_idx,
                                            selector: selector_const,
                                        },
                                        getter_def.range,
                                    );
                                }
                                EnumBehaviorMember::Setter(setter_def) => {
                                    if setter_def.is_static || has_class_attr(&setter_def.attributes) {
                                        return Err(CompilerError::IllegalStaticOnVariantMember(setter_def.name.clone(), setter_def.range));
                                    }
                                    let body_stmts = match &setter_def.body {
                                        MemberBody::Block(stmts) => stmts.clone(),
                                        MemberBody::Declaration => {
                                            return Err(CompilerError::DeclarationBodyRequiresImplementation(setter_def.name.clone(), setter_def.range));
                                        }
                                    };
                                    let sig_kind = SignatureKind::Setter;
                                    let sig_str = make_signature(&setter_def.name, sig_kind);
                                    let selector_sym = self.vm.interner.intern(&sig_str);
                                    let selector_const = self.add_constant(Value::symbol(selector_sym));
                                    self.is_static_context = false;
                                    let closure = self.compile_block(
                                        body_stmts,
                                        selector_sym,
                                        ClosureParameters::fixed(vec![setter_def.param.name.clone()]),
                                        true,
                                        false,
                                        None,
                                    )?;
                                    let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                                        selector_sym,
                                        sig_kind,
                                        MethodKind::Closure(closure),
                                    ))));
                                    self.vm.heap.method_mut(method_obj).visibility = member_visibility(Some(&setter_def.name), &setter_def.attributes);
                                    let method_const = self.add_constant(Value::obj(method_obj));
                                    self.emit(Bytecode::Constant(method_const), setter_def.range);
                                    self.emit(
                                        Bytecode::VariantMethod {
                                            variant: var_idx,
                                            selector: selector_const,
                                        },
                                        setter_def.range,
                                    );
                                }
                                EnumBehaviorMember::Index(index_def) => {
                                    let arity = checked_send_arity("subscript declaration", index_def.params.len(), index_def.range)?;
                                    let labels: Vec<Option<String>> = index_def.params.iter().map(|p| p.label.clone()).collect();
                                    let mut param_names: Vec<String> = index_def.params.iter().map(|p| p.name.clone()).collect();
                                    let sig_kind = match &index_def.accessor {
                                        IndexAccessor::Get => SignatureKind::SubscriptGet(arity),
                                        IndexAccessor::Set { put } => {
                                            checked_send_arity("subscript declaration", index_def.params.len() + 1, index_def.range)?;
                                            param_names.push(put.name.clone());
                                            SignatureKind::SubscriptSet(arity)
                                        }
                                    };
                                    let selector = encode_selector("", &labels, sig_kind);
                                    let selector_sym = self.vm.interner.intern(&selector);
                                    let selector_const = self.add_constant(Value::symbol(selector_sym));
                                    self.is_static_context = false;
                                    let closure =
                                        self.compile_block(index_def.body.clone(), selector_sym, ClosureParameters::fixed(param_names), true, false, None)?;
                                    let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                                        selector_sym,
                                        sig_kind,
                                        MethodKind::Closure(closure),
                                    ))));
                                    self.vm.heap.method_mut(method_obj).visibility = member_visibility(None, &index_def.attributes);
                                    let method_const = self.add_constant(Value::obj(method_obj));
                                    self.emit(Bytecode::Constant(method_const), index_def.range);
                                    self.emit(
                                        Bytecode::VariantMethod {
                                            variant: var_idx,
                                            selector: selector_const,
                                        },
                                        index_def.range,
                                    );
                                }
                            }
                        }
                    }
                }
            }
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
