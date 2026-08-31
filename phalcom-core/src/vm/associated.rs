//! VM execution and binding support for associated targets and families (Part 4).

use crate::error::RuntimeError;
use crate::heap::{ClassId, ObjRef};
use crate::modules::semantic_lowering::{ExecutableInvocationTarget, ExecutableRestMode};
use crate::value::Value;
use crate::vm::{ClassKey, VM};
use phalcom_modules::DeclarationId;

/// Statically bound exact behavioral associated target.
#[derive(Clone, Copy, Debug)]
pub struct ResolvedBehavioralAssociatedTarget {
    pub receiver: Value,
    pub method: ObjRef,
}

impl VM {
    /// Resolves a DeclarationId to its runtime ClassId.
    pub fn resolve_declaration_class(&self, decl: &DeclarationId) -> Result<ClassId, RuntimeError> {
        // 1. Check ADT enum registry
        if let Some(enum_id) = self.adt_registry.enum_by_declaration(decl) {
            if let Some(desc) = self.adt_registry.enum_descriptor(enum_id) {
                return Ok(desc.root_class);
            }
        }

        // 2. Check builtin classes
        if let Some(class_id) = self.resolve_builtin_class_name(&decl.name) {
            return Ok(class_id);
        }

        // 3. Check module registry
        if let Some(record) = self.module_registry.get(&decl.module) {
            let name_sym = self.interner.find(&decl.name);
            if let Some(sym) = name_sym {
                let key = ClassKey {
                    module: record.object,
                    name: sym,
                };
                if let Some(&class_id) = self.classes.get(&key) {
                    return Ok(class_id);
                }
            }
        }

        // 4. Fallback search across classes by name
        for (key, &class_id) in &self.classes {
            if self.interner.lookup(key.name) == decl.name.as_ref() {
                return Ok(class_id);
            }
        }

        Err(RuntimeError::Message(format!(
            "unable to resolve declaration class for `{}` in module `{}`",
            decl.name, decl.module
        )))
    }

    /// Binds an exact behavioral associated target to a live method handle and receiver.
    ///
    /// Per invariant I-RT-4:
    /// - Direct address lookup on defining owner class/metaclass.
    /// - Receiver bound to lookup_owner class object.
    /// - No hierarchy walk, no family-base search, no rest candidate ranking, no dNU, no visibility check.
    pub fn bind_behavioral_associated_target(&mut self, target: &ExecutableInvocationTarget) -> Result<ResolvedBehavioralAssociatedTarget, RuntimeError> {
        match target {
            ExecutableInvocationTarget::Behavioral {
                lookup_owner,
                callable,
                operation: _,
                rest_mode,
            } => {
                let receiver_class = self.resolve_declaration_class(lookup_owner)?;
                let defining_class = self.resolve_declaration_class(callable.owner.declaration())?;

                // Behavioral associated methods live on the defining class's metaclass (class-side)
                let defining_metaclass = self.heap.class(defining_class).class;

                let selector_sym = self.get_or_intern(&callable.selector.to_string());
                let method = match rest_mode {
                    ExecutableRestMode::None => {
                        self.heap.class(defining_metaclass).methods.get(&selector_sym).copied().or_else(|| {
                            // Also check directly on defining_class in case it was installed on instance side
                            self.heap.class(defining_class).methods.get(&selector_sym).copied()
                        })
                    }
                    ExecutableRestMode::Positional | ExecutableRestMode::Labeled | ExecutableRestMode::Complete => self
                        .heap
                        .class(defining_metaclass)
                        .rest_methods
                        .get(&selector_sym)
                        .copied()
                        .or_else(|| self.heap.class(defining_class).rest_methods.get(&selector_sym).copied()),
                };

                let method = method.ok_or_else(|| {
                    RuntimeError::Message(format!(
                        "associated method `{}` not found on defining class `{}`",
                        callable.selector,
                        callable.owner.declaration().name
                    ))
                })?;

                Ok(ResolvedBehavioralAssociatedTarget {
                    receiver: Value::obj(receiver_class),
                    method,
                })
            }
            ExecutableInvocationTarget::VariantConstructor { variant } => Err(RuntimeError::Message(format!(
                "bind_behavioral_associated_target called on variant constructor `{}`",
                variant.selector
            ))),
        }
    }
}
