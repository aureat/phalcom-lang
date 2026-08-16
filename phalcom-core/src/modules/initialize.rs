//! DAG Module Initializer with fail-fast startup and sticky failure propagation.

use crate::error::{ModuleInitializationError, PhError, PhResult, RuntimeError};
use crate::modules::compile::CompiledProgram;
use crate::modules::registry::{ModuleFailure, ModuleState};
use crate::vm::VM;
use phalcom_modules::ModuleId;

impl VM {
    /// Initializes all modules in `program` following its deterministic topological dependency-first order.
    pub fn initialize_program(&mut self, program: &CompiledProgram) -> PhResult<()> {
        for module_id in &program.initialization_order {
            self.initialize_single_module(program, module_id)?;
        }
        Ok(())
    }

    /// Initializes one module, ensuring all its declared runtime dependencies are Initialized first.
    pub fn initialize_single_module(&mut self, program: &CompiledProgram, id: &ModuleId) -> PhResult<()> {
        let record = self
            .module_registry
            .get(id)
            .ok_or_else(|| RuntimeError::Internal(format!("module {id} not found in registry")))?;

        match record.state {
            ModuleState::Initialized => return Ok(()),
            ModuleState::Initializing => {
                return Err(PhError::Runtime(RuntimeError::Internal(format!(
                    "InternalModuleOrderViolation: cyclic or re-entrant initialization of module {id}"
                ))));
            }
            ModuleState::Failed => {
                let failure = record.failure.clone().unwrap_or_else(|| ModuleFailure::Initializer {
                    cause: Box::new(PhError::Runtime(RuntimeError::Internal(format!("module {id} failed")))),
                });
                return Err(self.build_initialization_error(id, &failure));
            }
            ModuleState::Prepared => {}
        }

        // Verify all runtime dependencies in deterministic topological order
        if let Some(linked_mod) = program.linked.modules.get(id) {
            for dep_id in &linked_mod.runtime_dependencies {
                let dep_state = self.module_registry.get(dep_id).map(|r| r.state);
                match dep_state {
                    None => {
                        return Err(RuntimeError::Internal(format!("dependency {dep_id} not found in registry")).into());
                    }
                    Some(ModuleState::Initialized) => continue,
                    Some(ModuleState::Failed) => {
                        let dep_record = self.module_registry.get(dep_id).unwrap();
                        let dep_failure = dep_record.failure.clone().unwrap_or_else(|| ModuleFailure::Initializer {
                            cause: Box::new(PhError::Runtime(RuntimeError::Internal(format!("dependency {dep_id} failed")))),
                        });
                        let failure = ModuleFailure::Dependency {
                            dependency: dep_id.clone(),
                            cause: Box::new(dep_failure),
                        };
                        let rec = self.module_registry.get_mut(id).unwrap();
                        rec.state = ModuleState::Failed;
                        rec.failure = Some(failure.clone());
                        return Err(self.build_initialization_error(id, &failure));
                    }
                    Some(ModuleState::Initializing) | Some(ModuleState::Prepared) => {
                        return Err(PhError::Runtime(RuntimeError::Internal(format!(
                            "InternalModuleOrderViolation: dependency {dep_id} is not initialized before dependent module {id}"
                        ))));
                    }
                }
            }
        }

        // Transition -> Initializing
        self.module_registry.get_mut(id).unwrap().state = ModuleState::Initializing;

        let obj = self.module_registry.get(id).unwrap().object;
        let closure = self.heap.module(obj).closure;

        if let Some(closure) = closure {
            if let Err(err) = self.run_in_module(obj, closure) {
                let _ = self.runtime_error(err.clone());
                let failure = ModuleFailure::Initializer { cause: Box::new(err) };
                let rec = self.module_registry.get_mut(id).unwrap();
                rec.state = ModuleState::Failed;
                rec.failure = Some(failure.clone());
                return Err(self.build_initialization_error(id, &failure));
            }
        }

        // Transition -> Initialized
        self.module_registry.get_mut(id).unwrap().state = ModuleState::Initialized;
        Ok(())
    }

    fn build_initialization_error(&self, id: &ModuleId, failure: &ModuleFailure) -> PhError {
        let mut full_chain = vec![id.clone()];
        let mut current = failure;
        while let ModuleFailure::Dependency { dependency, cause } = current {
            full_chain.push(dependency.clone());
            current = cause;
        }

        let obj = self.module_registry.get(id).map(|r| r.object);
        let (display_name, source) = if let Some(obj) = obj {
            let m = self.heap.module(obj);
            (m.name.clone(), m.source_at(0).map(|s| s.as_ref().clone()))
        } else {
            (id.to_string(), None)
        };

        PhError::from(ModuleInitializationError {
            id: id.clone(),
            display_name,
            source,
            failure: failure.clone(),
            chain: full_chain,
        })
    }
}
