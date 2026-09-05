//! Canonical materialization of builtin packages and primordial native bindings.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{ModuleObject, ObjRef, Object, RuntimeExportRef};
use crate::modules::linkage::BindingRef;
use crate::modules::registry::{ModulePlanFingerprint, ModuleRecord, RuntimeProgramId};
use crate::value::Value;
use crate::vm::VM;
use phalcom_modules::builtin::UniverseSourceProvider;
use phalcom_modules::identity::{ModuleId, ModulePath};
use phalcom_modules::source::ModuleKind;
use std::collections::HashMap;
use std::sync::Arc;

fn canonical_universe_path(components: &[&str]) -> Result<ModulePath, RuntimeError> {
    components
        .iter()
        .map(|component| {
            phalcom_modules::ModuleComponent::from_identifier(component)
                .map_err(|error| RuntimeError::Internal(format!("invalid canonical Universe component `{component}`: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(ModulePath::from_components)
}

/// Initializes the canonical builtin `universe` package in the module registry
/// and populates its native bindings and export table.
pub fn initialize_canonical_universe(vm: &mut VM) -> PhResult<ObjRef> {
    let provider = UniverseSourceProvider::new();
    let root_id = ModuleId::universe(ModulePath::root());
    let mut modules = HashMap::new();

    // Allocate every canonical module before wiring parents. This gives source
    // execution stable identities and prevents package imports from depending
    // on provider enumeration order.
    for node in provider.nodes() {
        let path = canonical_universe_path(node.path)?;
        let id = ModuleId::universe(path);
        let iface = provider
            .load_interface(&id)
            .map_err(|e| RuntimeError::Internal(format!("failed to load builtin universe interface {id}: {e}")))?;
        let source_id = provider
            .source_id(&id)
            .map_err(|e| RuntimeError::Internal(format!("failed to identify builtin universe source {id}: {e}")))?;
        let name = id.path.components().last().map(|component| component.as_str()).unwrap_or("universe");
        let name_sym = vm.interner.intern(name);
        let mut module_obj = ModuleObject::new(id.clone(), node.kind, name.to_owned(), name_sym, source_id.to_string(), None, true);
        module_obj.metadata = Some(Arc::new(iface.metadata));
        let object = vm.heap.alloc(Object::Module(Box::new(module_obj)));
        vm.privileged_modules.insert(object);
        vm.module_registry
            .register_new(id.clone(), ModuleRecord::prepared(object, RuntimeProgramId(0), ModulePlanFingerprint(0)))
            .map_err(|e| RuntimeError::Internal(e.to_string()))?;
        modules.insert(id, object);
    }

    let root = *modules
        .get(&root_id)
        .ok_or_else(|| RuntimeError::Internal("Universe root must be materialized".into()))?;
    for node in provider.nodes() {
        let path = canonical_universe_path(node.path)?;
        let id = ModuleId::universe(path);
        let object = *modules
            .get(&id)
            .ok_or_else(|| RuntimeError::Internal(format!("Universe module {id} was not allocated")))?;
        if id.path.is_root() {
            vm.heap.module_mut(object).package = Some(object);
            vm.heap.module_mut(object).root_package = Some(object);
            continue;
        }
        let parent_path = id
            .path
            .parent()
            .ok_or_else(|| RuntimeError::Internal(format!("non-root Universe module {id} has no parent")))?;
        let parent_id = ModuleId::universe(parent_path);
        let parent = *modules
            .get(&parent_id)
            .ok_or_else(|| RuntimeError::Internal(format!("Universe parent module {parent_id} was not allocated")))?;
        vm.heap.module_mut(object).package = Some(if node.kind == ModuleKind::Package { object } else { parent });
        vm.heap.module_mut(object).root_package = Some(root);
    }

    let root_iface = provider
        .load_interface(&root_id)
        .map_err(|e| RuntimeError::Internal(format!("failed to load builtin universe interface: {e}")))?;
    install_universe_native_bindings(vm, &modules, root, &root_iface)?;
    Ok(root)
}

/// Installs native bindings in their canonical source-owning modules and
/// exposes direct child packages from the Universe root.
pub fn install_universe_native_bindings(
    vm: &mut VM,
    modules: &HashMap<ModuleId, ObjRef>,
    universe_root: ObjRef,
    _iface: &phalcom_modules::UnlinkedModuleInterface,
) -> PhResult<()> {
    for binding in phalcom_native_meta::UNIVERSE_BINDINGS {
        let class_id = vm.universe.classes.resolve(binding.key);
        let name_sym = vm.interner.intern(binding.name);
        let owner_id = VM::canonical_universe_module_id(binding.key).map_err(|error| RuntimeError::Internal(error.to_string()))?;
        let owner = *modules
            .get(&owner_id)
            .ok_or_else(|| RuntimeError::Internal(format!("Universe owner module {owner_id} is not materialized")))?;
        let slot = vm.heap.module_mut(owner).declare(name_sym)?;
        let value = if binding.key == phalcom_native_meta::UniverseKey::None {
            Value::none()
        } else {
            Value::obj(class_id)
        };
        vm.heap.module_mut(owner).set_global(slot, value)?;
        if binding.exported {
            vm.heap.module_mut(owner).exports.insert(
                name_sym,
                RuntimeExportRef::Binding(BindingRef {
                    module: owner,
                    slot: u16::try_from(slot).map_err(|_| RuntimeError::Internal(format!("Universe binding slot for `{}` does not fit u16", binding.name)))?,
                }),
            );
        }
    }

    let provider = UniverseSourceProvider::new();
    for node in provider.nodes().iter().filter(|node| node.path.len() == 1) {
        let component_name = node
            .path
            .first()
            .ok_or_else(|| RuntimeError::Internal("canonical Universe child has no path component".into()))?;
        let component = phalcom_modules::ModuleComponent::from_identifier(component_name)
            .map_err(|error| RuntimeError::Internal(format!("invalid canonical Universe component `{component_name}`: {error}")))?;
        let child_id = ModuleId::universe(ModulePath::from_components(vec![component.clone()]));
        let child = *modules
            .get(&child_id)
            .ok_or_else(|| RuntimeError::Internal(format!("Universe child module {child_id} was not allocated")))?;
        let name_sym = vm.interner.intern(component.as_str());
        let slot = vm.heap.module_mut(universe_root).declare(name_sym)?;
        vm.heap.module_mut(universe_root).set_global(slot, Value::obj(child))?;
        vm.heap.module_mut(universe_root).exports.insert(name_sym, RuntimeExportRef::Module(child));
    }

    // Context intrinsics
    let module_ids = modules.keys().cloned().collect::<Vec<_>>();
    for id in module_ids {
        let module = *modules
            .get(&id)
            .ok_or_else(|| RuntimeError::Internal(format!("Universe module {id} was not allocated")))?;
        let mod_sym = vm.interner.intern("__module__");
        let slot = vm.heap.module_mut(module).declare(mod_sym)?;
        vm.heap.module_mut(module).set_global(slot, Value::obj(module))?;

        let pkg_sym = vm.interner.intern("__package__");
        let pkg_val = vm
            .heap
            .module(module)
            .package
            .map(Value::obj)
            .map(|v| v.wrap_some())
            .transpose()?
            .unwrap_or(Value::none());
        let slot = vm.heap.module_mut(module).declare(pkg_sym)?;
        vm.heap.module_mut(module).set_global(slot, pkg_val)?;

        let root_sym = vm.interner.intern("__root__");
        let slot = vm.heap.module_mut(module).declare(root_sym)?;
        let root_val = Value::obj(universe_root).wrap_some()?;
        vm.heap.module_mut(module).set_global(slot, root_val)?;

        let proj_sym = vm.interner.intern("__project__");
        let slot = vm.heap.module_mut(module).declare(proj_sym)?;
        vm.heap.module_mut(module).set_global(slot, Value::none())?;
    }

    Ok(())
}
