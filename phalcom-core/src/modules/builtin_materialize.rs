//! Canonical materialization of builtin packages and primordial native bindings.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{ModuleObject, ObjRef, Object, RuntimeExportRef};
use crate::modules::linkage::BindingRef;
use crate::modules::registry::{ModulePlanFingerprint, ModuleRecord, RuntimeProgramId};
use crate::value::Value;
use crate::vm::VM;
use phalcom_modules::builtin::BuiltinProjectSourceProvider;
use phalcom_modules::identity::{BuiltinPackage, ModuleId, ModulePath};
use phalcom_modules::source::ModuleKind;
use std::collections::HashMap;
use std::sync::Arc;

/// Initializes the canonical builtin `universe` package in the module registry
/// and populates its native bindings and export table.
pub fn initialize_canonical_universe(vm: &mut VM) -> PhResult<ObjRef> {
    let universe_root_id = ModuleId::builtin(BuiltinPackage::Universe, ModulePath::root());
    let provider = BuiltinProjectSourceProvider::new(BuiltinPackage::Universe);
    let iface = provider
        .load_interface(&universe_root_id)
        .map_err(|e| RuntimeError::Internal(format!("failed to load builtin universe interface: {e}")))?;

    let display_name = universe_root_id.to_string();
    let name_sym = vm.interner.intern("universe");
    let path = "<builtin:universe>".to_string();

    let mut module_obj = ModuleObject::new(universe_root_id.clone(), ModuleKind::Package, display_name, name_sym, path, None, true);
    module_obj.metadata = Some(Arc::new(iface.metadata.clone()));

    let obj_ref = vm.heap.alloc(Object::Module(Box::new(module_obj)));
    vm.heap.module_mut(obj_ref).package = Some(obj_ref);
    vm.heap.module_mut(obj_ref).root_package = Some(obj_ref);

    vm.module_registry
        .register_new(
            universe_root_id.clone(),
            ModuleRecord::prepared(obj_ref, RuntimeProgramId(0), ModulePlanFingerprint(0)),
        )
        .map_err(|e| RuntimeError::Internal(e.to_string()))?;

    // Install native bindings & export table
    install_universe_native_bindings(vm, obj_ref, &iface)?;

    Ok(obj_ref)
}

/// Installs all UNIVERSE_BINDINGS into slot storage and populates exports on `universe_root`.
pub fn install_universe_native_bindings(vm: &mut VM, universe_root: ObjRef, _iface: &phalcom_modules::UnlinkedModuleInterface) -> PhResult<()> {
    let mut exports = HashMap::new();

    // Install all UNIVERSE_BINDINGS into slot storage
    for binding in phalcom_native_meta::UNIVERSE_BINDINGS {
        let class_id = vm.universe.classes.resolve(binding.key);
        let name_sym = vm.interner.intern(binding.name);

        let slot = vm.heap.module_mut(universe_root).declare(name_sym)?;
        vm.heap.module_mut(universe_root).set_global(slot, Value::obj(class_id))?;

        if binding.exported {
            exports.insert(
                name_sym,
                RuntimeExportRef::Binding(BindingRef {
                    module: universe_root,
                    slot: slot as u16,
                }),
            );
        }
    }

    // Context intrinsics
    let mod_sym = vm.interner.intern("__module__");
    let slot = vm.heap.module_mut(universe_root).declare(mod_sym)?;
    vm.heap.module_mut(universe_root).set_global(slot, Value::obj(universe_root))?;

    let pkg_sym = vm.interner.intern("__package__");
    let slot = vm.heap.module_mut(universe_root).declare(pkg_sym)?;
    let pkg_val = Value::obj(universe_root).wrap_some()?;
    vm.heap.module_mut(universe_root).set_global(slot, pkg_val)?;

    let root_sym = vm.interner.intern("__root__");
    let slot = vm.heap.module_mut(universe_root).declare(root_sym)?;
    let root_val = Value::obj(universe_root).wrap_some()?;
    vm.heap.module_mut(universe_root).set_global(slot, root_val)?;

    let proj_sym = vm.interner.intern("__project__");
    let slot = vm.heap.module_mut(universe_root).declare(proj_sym)?;
    vm.heap.module_mut(universe_root).set_global(slot, Value::none())?;

    // Freeze universe namespace
    vm.heap.module_mut(universe_root).exports = exports;
    vm.heap.module_mut(universe_root).namespace_frozen = true;

    Ok(())
}
