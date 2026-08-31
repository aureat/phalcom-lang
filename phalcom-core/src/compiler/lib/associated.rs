//! Associated expression lowering to direct bytecodes and family objects (Part 4).

use crate::bytecode::Bytecode;
use crate::compiler::lib::error::CompilerError;
use crate::compiler::lib::{Compiler, checked_send_arity};
use crate::modules::semantic_lowering::{AssociatedLoweringSpec, ExecutableFamilyCandidateSet, FamilyApplicationLoweringSpec, LoweringSiteKind};
use crate::value::Value;
use phalcom_ast::ast::{AssociatedInvokeExpr, AssociatedLookupExpr, AssociatedMemberSyntax, Expr, PackItem};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::DeclarationId;
use phalcom_semantic::checker::associated::BehavioralFamilySpec;
use phalcom_semantic::identity::VariantId;

impl<'vm> Compiler<'vm> {
    fn family_application_spec(&self, range: SourceRange) -> Option<FamilyApplicationLoweringSpec> {
        self.lowering().and_then(|l| {
            l.family_applications
                .iter()
                .find(|(site, _)| site.range == range && site.kind == LoweringSiteKind::FamilyApplication)
                .map(|(_, spec)| spec.clone())
        })
    }

    fn family_application_site_exists(&self, range: SourceRange) -> bool {
        self.lowering().is_some_and(|l| {
            l.family_application_sites
                .iter()
                .any(|site| site.range == range && site.kind == LoweringSiteKind::FamilyApplication)
        })
    }

    fn family_value_site_exists(&self, range: SourceRange) -> bool {
        self.lowering().is_some_and(|l| {
            l.family_values
                .iter()
                .any(|site| site.range == range && site.kind == LoweringSiteKind::FamilyApplication)
        })
    }

    /// Lowers an ordinary call on a first-class associated family, when formal
    /// semantics marked this source range as a family application.
    pub fn compile_family_application_call(&mut self, callee: Expr, args: Vec<PackItem>, range: SourceRange) -> Result<bool, CompilerError> {
        let Some(spec) = self.family_application_spec(range) else {
            if self.family_application_site_exists(range) || self.family_value_site_exists(callee.range()) {
                return Err(CompilerError::MissingFamilyApplicationResolution(range));
            }
            return Ok(false);
        };

        self.compile_expr(callee)?;
        match spec {
            FamilyApplicationLoweringSpec::Static { operation, arity, .. } => {
                for arg in args {
                    self.compile_pack_item(arg)?;
                }
                let operation_idx = self
                    .functions
                    .last_mut()
                    .unwrap()
                    .chunk
                    .executable_semantics
                    .add_family_operation(operation, range)?;
                self.emit(
                    Bytecode::InvokeAssociatedFamilyStatic {
                        operation: operation_idx,
                        arity,
                    },
                    range,
                );
            }
            FamilyApplicationLoweringSpec::DynamicPack { candidates } => {
                let builder_slot = self.reserve_pack_scratch("$family_pack_builder", range)?;
                self.emit(Bytecode::NewArgumentPack, range);
                self.emit(Bytecode::SetLocal(builder_slot), range);
                self.emit(Bytecode::Pop, range);
                self.compile_dynamic_pack_items(builder_slot, args)?;
                self.emit(Bytecode::GetLocal(builder_slot), range);
                let candidates_idx = self
                    .functions
                    .last_mut()
                    .unwrap()
                    .chunk
                    .executable_semantics
                    .add_family_candidate_set(ExecutableFamilyCandidateSet { candidates }, range)?;
                self.emit(Bytecode::InvokeAssociatedFamilyPack { candidates: candidates_idx }, range);
                self.release_pack_scratch_from(builder_slot, 1, range);
            }
        }

        Ok(true)
    }

    /// Lowers an AssociatedLookup expression (e.g. `Option::None`, `Option::Some::`, `Type::#method::*`).
    pub fn compile_associated_lookup(&mut self, expr: &AssociatedLookupExpr) -> Result<(), CompilerError> {
        let spec = self.lowering().and_then(|l| {
            l.associated
                .iter()
                .find(|(site, _)| site.range == expr.range && site.kind == LoweringSiteKind::AssociatedLookup)
                .map(|(_, spec)| spec.clone())
        });

        if let Some(spec) = spec {
            match spec {
                AssociatedLoweringSpec::MakeBehavioralFamily { spec } => {
                    self.compile_expr(expr.receiver.clone())?;
                    let (spec_idx, kind) = self.compile_behavioral_family_spec(&spec)?;
                    self.emit(Bytecode::MakeFamily { spec: spec_idx, kind }, expr.range);
                    return Ok(());
                }
                AssociatedLoweringSpec::SingletonLoad { variant } => {
                    let var_idx = self
                        .functions
                        .last_mut()
                        .unwrap()
                        .chunk
                        .executable_semantics
                        .add_variant_target(&variant, expr.range)?;
                    self.emit(Bytecode::LoadVariantSingleton(var_idx), expr.range);
                    return Ok(());
                }
                AssociatedLoweringSpec::MakeVariantConstructorThunk { variant, .. } => {
                    return self.compile_variant_constructor_thunk(&variant, expr.range);
                }
                AssociatedLoweringSpec::MakeResolvedBoundMethod { target } => {
                    let target_idx = self
                        .functions
                        .last_mut()
                        .unwrap()
                        .chunk
                        .executable_semantics
                        .add_associated_target(target, expr.range)?;
                    self.emit(Bytecode::MakeResolvedBoundMethod(target_idx), expr.range);
                    return Ok(());
                }
                AssociatedLoweringSpec::MakeAssociatedFamily { descriptor } => {
                    let desc_idx = self
                        .functions
                        .last_mut()
                        .unwrap()
                        .chunk
                        .executable_semantics
                        .add_family_descriptor(descriptor, expr.range)?;
                    self.emit(Bytecode::MakeAssociatedFamily(desc_idx), expr.range);
                    return Ok(());
                }
                _ => {}
            }
        }

        // Fallback for standalone/unlinked compilation
        if let Expr::Var { value: name, .. } = &expr.receiver {
            let module_id = self.vm.heap.module(self.module).id.clone();
            let owner = DeclarationId::new(module_id, name.clone().into_boxed_str());
            if let AssociatedMemberSyntax::Named(named) = &expr.member {
                let selector = match Selector::getter(named.base.clone()) {
                    Ok(sel) => sel,
                    Err(_) => return Err(CompilerError::AssociatedLookupNotLoweredYet(expr.range)),
                };
                let variant_id = VariantId::new(owner, selector);
                let var_idx = self
                    .functions
                    .last_mut()
                    .unwrap()
                    .chunk
                    .executable_semantics
                    .add_variant_target(&variant_id, expr.range)?;
                self.emit(Bytecode::LoadVariantSingleton(var_idx), expr.range);
                return Ok(());
            }
        }

        Err(CompilerError::AssociatedLookupNotLoweredYet(expr.range))
    }

    /// Lowers an AssociatedInvoke expression (e.g. `Option::Some(42)`, `Type::method(a, b)`).
    pub fn compile_associated_invoke(&mut self, expr: &AssociatedInvokeExpr) -> Result<(), CompilerError> {
        let spec = self.lowering().and_then(|l| {
            l.associated
                .iter()
                .find(|(site, _)| site.range == expr.range && site.kind == LoweringSiteKind::AssociatedInvoke)
                .map(|(_, spec)| spec.clone())
        });

        if let Some(spec) = spec {
            match spec {
                AssociatedLoweringSpec::InvokeBoundBehavioral { selector } => {
                    let arity = checked_send_arity("associated message send", expr.args.len(), expr.range)?;
                    self.compile_expr(expr.receiver.clone())?;
                    for arg in &expr.args {
                        self.compile_pack_item(arg.clone())?;
                    }
                    let selector_sym = self.vm.interner.intern(&selector.encode());
                    let selector_idx = self.add_constant(Value::symbol(selector_sym));
                    self.emit(Bytecode::Invoke(arity, selector_idx), expr.range);
                    return Ok(());
                }
                AssociatedLoweringSpec::SingletonLoad { variant } => {
                    let var_idx = self
                        .functions
                        .last_mut()
                        .unwrap()
                        .chunk
                        .executable_semantics
                        .add_variant_target(&variant, expr.range)?;
                    self.emit(Bytecode::LoadVariantSingleton(var_idx), expr.range);
                    return Ok(());
                }
                AssociatedLoweringSpec::ConstructVariant { variant, .. } => {
                    for arg in &expr.args {
                        self.compile_pack_item(arg.clone())?;
                    }
                    let arity = checked_send_arity("variant construction", expr.args.len(), expr.range)?;
                    let var_idx = self
                        .functions
                        .last_mut()
                        .unwrap()
                        .chunk
                        .executable_semantics
                        .add_variant_target(&variant, expr.range)?;
                    self.emit(Bytecode::ConstructVariant { variant: var_idx, arity }, expr.range);
                    return Ok(());
                }
                AssociatedLoweringSpec::InvokeResolvedAssociated { target, .. } => {
                    for arg in &expr.args {
                        self.compile_pack_item(arg.clone())?;
                    }
                    let arity = checked_send_arity("associated invocation", expr.args.len(), expr.range)?;
                    let target_idx = self
                        .functions
                        .last_mut()
                        .unwrap()
                        .chunk
                        .executable_semantics
                        .add_associated_target(target, expr.range)?;
                    self.emit(Bytecode::InvokeResolvedAssociated { target: target_idx, arity }, expr.range);
                    return Ok(());
                }
                AssociatedLoweringSpec::MakeAssociatedFamily { descriptor } => {
                    let desc_idx = self
                        .functions
                        .last_mut()
                        .unwrap()
                        .chunk
                        .executable_semantics
                        .add_family_descriptor(descriptor, expr.range)?;
                    self.emit(Bytecode::MakeAssociatedFamily(desc_idx), expr.range);
                    return Ok(());
                }
                _ => {}
            }
        }

        // Fallback for standalone/unlinked compilation
        if let Expr::Var { value: name, .. } = &expr.receiver {
            let module_id = self.vm.heap.module(self.module).id.clone();
            let owner = DeclarationId::new(module_id, name.clone().into_boxed_str());
            let mut slots = Vec::new();
            for item in &expr.args {
                match item {
                    PackItem::Positional { .. } => slots.push(phalcom_common::selector::SelectorSlot::Positional),
                    PackItem::Labeled { label, .. } => match label {
                        phalcom_ast::ast::PackLabel::Static { text, .. } => slots.push(phalcom_common::selector::SelectorSlot::Label(text.clone())),
                        phalcom_ast::ast::PackLabel::Computed { .. } => slots.push(phalcom_common::selector::SelectorSlot::Positional),
                    },
                    PackItem::Expand { .. } => slots.push(phalcom_common::selector::SelectorSlot::Positional),
                }
            }
            let selector = match Selector::new(
                phalcom_common::selector::SelectorBase::Named(expr.base.clone()),
                phalcom_common::selector::SelectorKind::Method,
                slots.into_boxed_slice(),
            ) {
                Ok(sel) => sel,
                Err(_) => return Err(CompilerError::AssociatedInvokeNotLoweredYet(expr.range)),
            };
            let variant_id = VariantId::new(owner, selector);
            for arg in &expr.args {
                self.compile_pack_item(arg.clone())?;
            }
            let arity = checked_send_arity("standalone variant construction", expr.args.len(), expr.range)?;
            let var_idx = self
                .functions
                .last_mut()
                .unwrap()
                .chunk
                .executable_semantics
                .add_variant_target(&variant_id, expr.range)?;
            self.emit(Bytecode::ConstructVariant { variant: var_idx, arity }, expr.range);
            return Ok(());
        }

        Err(CompilerError::AssociatedInvokeNotLoweredYet(expr.range))
    }

    fn compile_behavioral_family_spec(&mut self, spec: &BehavioralFamilySpec) -> Result<(u16, crate::bytecode::FamilySpecKind), CompilerError> {
        match spec {
            BehavioralFamilySpec::Exact(selector) => {
                let symbol = self.vm.interner.intern(&selector.encode());
                Ok((self.add_constant(Value::symbol(symbol)), crate::bytecode::FamilySpecKind::Exact))
            }
            BehavioralFamilySpec::Pattern(pattern) => {
                let pattern_object = crate::heap::SelectorPatternObject::compile(pattern.clone(), &mut self.vm.interner);
                let object = self.vm.heap.alloc(crate::heap::Object::SelectorPattern(Box::new(pattern_object)));
                Ok((self.add_constant(Value::obj(object)), crate::bytecode::FamilySpecKind::Pattern))
            }
        }
    }

    /// Emits a callable thunk that constructs a variant instance when called.
    fn compile_variant_constructor_thunk(&mut self, variant: &VariantId, range: SourceRange) -> Result<(), CompilerError> {
        let arity = checked_send_arity("variant constructor thunk", variant.selector.slots.len(), range)?;
        let name_sym = self.vm.interner.intern(&format!("thunk::{}", variant.selector));
        let param_names = (0..arity).map(|i| format!("_{i}")).collect();
        let closure = self.compile_block(
            Vec::new(),
            name_sym,
            phalcom_ast::ast::ClosureParameters::fixed(param_names),
            false,
            false,
            None,
        )?;

        let cls = self.vm.heap.closure_mut(closure);
        let mut callable = (*cls.callable).clone();
        callable.chunk.code.clear();
        callable.chunk.spans.clear();
        callable.chunk.caches.clear();
        callable.chunk.gcaches.clear();
        let var_idx = callable.chunk.executable_semantics.add_variant_target(variant, range)?;
        // Reserve slot 0 for receiver, arguments in slots 1..=arity
        for slot in 1..=u16::from(arity) {
            callable.chunk.add_instruction(Bytecode::GetLocal(slot), range);
        }
        callable.chunk.add_instruction(Bytecode::ConstructVariant { variant: var_idx, arity }, range);
        callable.chunk.add_instruction(Bytecode::Return, range);
        callable.max_slots = (arity as usize) + 2;
        cls.callable = std::rc::Rc::new(callable);

        let const_idx = self.add_constant(Value::obj(closure));
        self.emit(Bytecode::Closure(const_idx), range);

        Ok(())
    }
}
