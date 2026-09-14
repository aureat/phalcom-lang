//! Inherent impl behavior member compilation and target installation.

use crate::bytecode::Bytecode;
use crate::compiler::attributes::CompileMode;
use crate::compiler::lib::class_decl::{member_visibility, rest_layout, rest_selector};
use crate::compiler::lib::error::CompilerError;
use crate::compiler::lib::{Compiler, checked_send_arity};
use crate::heap::{ObjRef, Object};
use crate::method::{MethodKind, MethodObject, SignatureKind, encode_selector, make_signature};
use crate::value::Value;
use phalcom_ast::ast::{AttrKind, Attribute, BehaviorMember, BuiltinAttr, ClosureParameters, IndexAccessor, MemberBody};
use phalcom_common::range::SourceRange;
use phalcom_modules::DeclarationId;
use phalcom_semantic::identity::{CallableId, CallableOwnerId, DispatchSide, VariantId};

pub(crate) struct CompiledBehaviorMember {
    pub method_obj: ObjRef,
    pub method_obj_idx: u16,
    pub selector_const: u16,
    pub is_class_side: bool,
    pub range: SourceRange,
    pub attributes: Box<[Attribute]>,
}

impl<'vm> Compiler<'vm> {
    /// Compiles a single [`BehaviorMember`] into a method object constant and selector constant.
    ///
    /// Validates that the generated selector and side strictly match the supplied
    /// target-owned [`CallableId`]; any mismatch fails closed.
    pub(crate) fn compile_behavior_member(&mut self, member: &BehaviorMember, expected_callable: &CallableId) -> Result<CompiledBehaviorMember, CompilerError> {
        let strip_metadata = match self.vm.compile_mode {
            CompileMode::Debug => false,
            CompileMode::Release => self.vm.strip_contract_metadata,
            CompileMode::Unchecked => true,
        };

        match member {
            BehaviorMember::Method(method_def) => {
                let range = method_def.range;
                let arity = method_def.params.len();
                let encoded_arity = checked_send_arity("method declaration", arity, method_def.range)?;
                let sig_kind = SignatureKind::Method(encoded_arity);
                let rest = rest_layout(&method_def.params, &mut self.vm.interner);
                let selector = if rest.is_some() {
                    rest_selector(&method_def.name, &method_def.params)
                } else {
                    let labels: Vec<Option<String>> = method_def.params.iter().map(|p| p.label.clone()).collect();
                    encode_selector(&method_def.name, &labels, sig_kind)
                };
                let selector_sym = self.vm.interner.intern(&selector);

                let is_class_side = method_def.is_static || method_def.attributes.iter().any(|a| a.name == "class");
                let side = if is_class_side { DispatchSide::Class } else { DispatchSide::Instance };

                // Validate that generated selector and side match the target-owned CallableId
                if expected_callable.selector.encode() != selector || expected_callable.side != side {
                    return Err(CompilerError::ImplCallableMismatch(range));
                }

                let param_names: Vec<String> = method_def.params.iter().map(|p| p.name.clone()).collect();
                self.is_static_context = is_class_side;
                let prior_compiler_internal = self.compiler_internal;
                self.compiler_internal = method_def
                    .attributes
                    .iter()
                    .any(|attr| matches!(attr.kind, AttrKind::Builtin(BuiltinAttr::Constructor)) || attr.name == "__synthetic");
                let body_stmts = match &method_def.body {
                    MemberBody::Block(stmts) => stmts.clone(),
                    MemberBody::Declaration => {
                        return Err(CompilerError::DeclarationBodyRequiresImplementation(method_def.name.clone(), method_def.range));
                    }
                };
                let closure_result = self.compile_block(body_stmts, selector_sym, ClosureParameters::fixed(param_names), true, false, None);
                self.compiler_internal = prior_compiler_internal;
                let closure = closure_result?;

                let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                    selector_sym,
                    sig_kind,
                    MethodKind::Closure(closure),
                ))));
                {
                    let method = self.vm.heap.method_mut(method_obj);
                    method.visibility = member_visibility(Some(&method_def.name), &method_def.attributes);
                    if let Some(rest) = rest {
                        method.signature = crate::method::Signature::new_with_arity(selector_sym, sig_kind, rest.fixed_positionals(), Some(rest));
                    }
                }

                if !strip_metadata {
                    let contracts = self.build_contracts_metadata(&method_def.attributes)?;
                    if !contracts.is_empty() {
                        self.vm.heap.method_mut(method_obj).contracts = Some(contracts);
                    }
                }

                let method_obj_idx = self.add_constant(Value::obj(method_obj));
                let selector_const = self.add_constant(Value::symbol(selector_sym));

                Ok(CompiledBehaviorMember {
                    method_obj,
                    method_obj_idx,
                    selector_const,
                    is_class_side,
                    range,
                    attributes: method_def.attributes.clone().into_boxed_slice(),
                })
            }
            BehaviorMember::Getter(getter_def) => {
                let range = getter_def.range;
                let selector = make_signature(&getter_def.name, SignatureKind::Getter);
                let selector_sym = self.vm.interner.intern(&selector);

                let is_class_side = getter_def.is_static || getter_def.attributes.iter().any(|a| a.name == "class");
                let side = if is_class_side { DispatchSide::Class } else { DispatchSide::Instance };

                if expected_callable.selector.encode() != selector || expected_callable.side != side {
                    return Err(CompilerError::ImplCallableMismatch(range));
                }

                self.is_static_context = is_class_side;
                let body_stmts = match &getter_def.body {
                    MemberBody::Block(stmts) => stmts.clone(),
                    MemberBody::Declaration => {
                        return Err(CompilerError::DeclarationBodyRequiresImplementation(getter_def.name.clone(), getter_def.range));
                    }
                };
                let closure = self.compile_block(body_stmts, selector_sym, ClosureParameters::fixed(Vec::new()), true, false, None)?;

                let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                    selector_sym,
                    SignatureKind::Getter,
                    MethodKind::Closure(closure),
                ))));
                self.vm.heap.method_mut(method_obj).visibility = member_visibility(Some(&getter_def.name), &getter_def.attributes);

                if !strip_metadata {
                    let contracts = self.build_contracts_metadata(&getter_def.attributes)?;
                    if !contracts.is_empty() {
                        self.vm.heap.method_mut(method_obj).contracts = Some(contracts);
                    }
                }

                let method_obj_idx = self.add_constant(Value::obj(method_obj));
                let selector_const = self.add_constant(Value::symbol(selector_sym));

                Ok(CompiledBehaviorMember {
                    method_obj,
                    method_obj_idx,
                    selector_const,
                    is_class_side,
                    range,
                    attributes: getter_def.attributes.clone().into_boxed_slice(),
                })
            }
            BehaviorMember::Setter(setter_def) => {
                let range = setter_def.range;
                let selector = make_signature(&setter_def.name, SignatureKind::Setter);
                let selector_sym = self.vm.interner.intern(&selector);

                let is_class_side = setter_def.is_static || setter_def.attributes.iter().any(|a| a.name == "class");
                let side = if is_class_side { DispatchSide::Class } else { DispatchSide::Instance };

                if expected_callable.selector.encode() != selector || expected_callable.side != side {
                    return Err(CompilerError::ImplCallableMismatch(range));
                }

                self.is_static_context = is_class_side;
                let body_stmts = match &setter_def.body {
                    MemberBody::Block(stmts) => stmts.clone(),
                    MemberBody::Declaration => {
                        return Err(CompilerError::DeclarationBodyRequiresImplementation(setter_def.name.clone(), setter_def.range));
                    }
                };
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
                    SignatureKind::Setter,
                    MethodKind::Closure(closure),
                ))));
                self.vm.heap.method_mut(method_obj).visibility = member_visibility(Some(&setter_def.name), &setter_def.attributes);

                if !strip_metadata {
                    let contracts = self.build_contracts_metadata(&setter_def.attributes)?;
                    if !contracts.is_empty() {
                        self.vm.heap.method_mut(method_obj).contracts = Some(contracts);
                    }
                }

                let method_obj_idx = self.add_constant(Value::obj(method_obj));
                let selector_const = self.add_constant(Value::symbol(selector_sym));

                Ok(CompiledBehaviorMember {
                    method_obj,
                    method_obj_idx,
                    selector_const,
                    is_class_side,
                    range,
                    attributes: setter_def.attributes.clone().into_boxed_slice(),
                })
            }
            BehaviorMember::Index(index_def) => {
                let range = index_def.range;
                let arity = checked_send_arity("subscript declaration", index_def.params.len(), index_def.range)?;
                let labels: Vec<Option<String>> = index_def.params.iter().map(|p| p.label.clone()).collect();

                let mut param_names: Vec<String> = index_def.params.iter().map(|p| p.name.clone()).collect();
                let sig_kind = match &index_def.accessor {
                    IndexAccessor::Get => SignatureKind::SubscriptGet(arity),
                    IndexAccessor::Set { value } => {
                        checked_send_arity("subscript declaration", index_def.params.len() + 1, index_def.range)?;
                        param_names.push(value.name.clone());
                        SignatureKind::SubscriptSet(arity)
                    }
                };

                let selector = encode_selector("", &labels, sig_kind);
                let selector_sym = self.vm.interner.intern(&selector);

                if expected_callable.selector.encode() != selector || expected_callable.side != DispatchSide::Instance {
                    return Err(CompilerError::ImplCallableMismatch(range));
                }

                self.is_static_context = false;
                let closure = self.compile_block(index_def.body.clone(), selector_sym, ClosureParameters::fixed(param_names), true, false, None)?;

                let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
                    selector_sym,
                    sig_kind,
                    MethodKind::Closure(closure),
                ))));
                self.vm.heap.method_mut(method_obj).visibility = member_visibility(None, &index_def.attributes);

                if !strip_metadata {
                    let contracts = self.build_contracts_metadata(&index_def.attributes)?;
                    if !contracts.is_empty() {
                        self.vm.heap.method_mut(method_obj).contracts = Some(contracts);
                    }
                }

                let method_obj_idx = self.add_constant(Value::obj(method_obj));
                let selector_const = self.add_constant(Value::symbol(selector_sym));

                Ok(CompiledBehaviorMember {
                    method_obj,
                    method_obj_idx,
                    selector_const,
                    is_class_side: false,
                    range,
                    attributes: index_def.attributes.clone().into_boxed_slice(),
                })
            }
        }
    }

    /// Looks up or compiles the compiled method object for a conditional inherent callable.
    pub(crate) fn get_or_compile_conditional_method(&mut self, callable: &CallableId) -> Result<ObjRef, CompilerError> {
        if let Some(&obj_ref) = self.conditional_method_objects.get(callable) {
            return Ok(obj_ref);
        }
        for specs in self.inherent_impl_specs.values() {
            for spec in specs {
                for member_lowering in spec.members.iter() {
                    if &member_lowering.callable == callable {
                        if let Some(impl_def) = self.inherent_impl_defs.get(&spec.id).cloned() {
                            if let Some(member) = impl_def.members.get(member_lowering.source_member_index) {
                                let compiled = self.compile_behavior_member(member, callable)?;
                                self.conditional_method_objects.insert(callable.clone(), compiled.method_obj);
                                return Ok(compiled.method_obj);
                            }
                        }
                    }
                }
            }
        }
        for specs in self.inherent_impl_specs_by_variant.values() {
            for spec in specs {
                for member_lowering in spec.members.iter() {
                    if &member_lowering.callable == callable {
                        if let Some(impl_def) = self.inherent_impl_defs.get(&spec.id).cloned() {
                            if let Some(member) = impl_def.members.get(member_lowering.source_member_index) {
                                let compiled = self.compile_behavior_member(member, callable)?;
                                self.conditional_method_objects.insert(callable.clone(), compiled.method_obj);
                                return Ok(compiled.method_obj);
                            }
                        }
                    }
                }
            }
        }
        Err(CompilerError::ImplCallableMismatch(SourceRange::new(0, 0)))
    }

    /// Installs all semantically accepted inherent `impl` members for nominal `target`
    /// onto the target behavior class currently sitting on top of the stack.
    pub(crate) fn install_accepted_inherent_impl_members(&mut self, target: &DeclarationId) -> Result<(), CompilerError> {
        let Some(specs) = self.inherent_impl_specs.get(target).cloned() else {
            return Ok(());
        };

        for spec in specs {
            let Some(impl_def) = self.inherent_impl_defs.get(&spec.id).cloned() else {
                return Err(CompilerError::MissingImplLoweringSemantics(SourceRange::new(0, 0)));
            };

            for member_lowering in spec.members.iter() {
                if member_lowering.callable.owner != CallableOwnerId::Declaration(target.clone()) {
                    return Err(CompilerError::ImplCallableMismatch(impl_def.range));
                }

                let Some(member) = impl_def.members.get(member_lowering.source_member_index) else {
                    return Err(CompilerError::ImplCallableMismatch(impl_def.range));
                };

                let compiled = self.compile_behavior_member(member, &member_lowering.callable)?;
                self.conditional_method_objects.insert(member_lowering.callable.clone(), compiled.method_obj);
                if !spec.is_conditional {
                    self.emit(Bytecode::Constant(compiled.method_obj_idx), compiled.range);
                    self.emit(Bytecode::Method(compiled.selector_const, compiled.is_class_side), compiled.range);
                    self.emit_member_attribute_attaches(&compiled.attributes, compiled.method_obj_idx, compiled.range)?;
                }
            }
        }

        Ok(())
    }

    /// Installs all semantically accepted exact-case inherent `impl` members for `variant`
    /// via `Bytecode::VariantMethod`.
    pub(crate) fn install_accepted_exact_case_impl_members(&mut self, variant: &VariantId) -> Result<(), CompilerError> {
        let Some(specs) = self.inherent_impl_specs_by_variant.get(variant).cloned() else {
            return Ok(());
        };

        for spec in specs {
            let Some(impl_def) = self.inherent_impl_defs.get(&spec.id).cloned() else {
                return Err(CompilerError::MissingImplLoweringSemantics(SourceRange::new(0, 0)));
            };

            for member_lowering in spec.members.iter() {
                if member_lowering.callable.owner != CallableOwnerId::Variant(variant.clone()) || member_lowering.callable.side != DispatchSide::Instance {
                    return Err(CompilerError::ImplCallableMismatch(impl_def.range));
                }

                let Some(member) = impl_def.members.get(member_lowering.source_member_index) else {
                    return Err(CompilerError::ImplCallableMismatch(impl_def.range));
                };

                let compiled = self.compile_behavior_member(member, &member_lowering.callable)?;
                if compiled.is_class_side {
                    return Err(CompilerError::ImplCallableMismatch(compiled.range));
                }

                let var_idx = self
                    .functions
                    .last_mut()
                    .unwrap()
                    .chunk
                    .executable_semantics
                    .add_variant_target(variant, compiled.range)?;

                self.conditional_method_objects.insert(member_lowering.callable.clone(), compiled.method_obj);
                if !spec.is_conditional {
                    self.emit(Bytecode::Constant(compiled.method_obj_idx), compiled.range);
                    self.emit(
                        Bytecode::VariantMethod {
                            variant: var_idx,
                            selector: compiled.selector_const,
                        },
                        compiled.range,
                    );
                    self.emit_member_attribute_attaches(&compiled.attributes, compiled.method_obj_idx, compiled.range)?;
                }
            }
        }

        Ok(())
    }
}
