//! Native primitives for reflection and modularity classes.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{Object, RuntimeExportRef, TupleObject};
use crate::modules::reflection_cache::ReflectionCache;
use crate::value::Value;
use crate::vm::VM;
use phalcom_modules::package_info::PackageInfoDescriptor;

// ==========================================
// Module Primitives
// ==========================================

pub fn module_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Module",
        found: receiver.type_name(),
    })?;
    let module = vm.heap.module(id);
    Ok(Value::symbol(module.name_sym))
}

pub fn module_namespace(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let info = module_package_info(vm, receiver, args)?;
    if let Some(info_obj) = info.without_some_wrappers().as_obj() {
        let ns = vm.heap.package_info(info_obj).namespace;
        Ok(Value::symbol(ns).wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn module_package(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Module",
        found: receiver.type_name(),
    })?;
    let module = vm.heap.module(id);
    if let Some(pkg_ref) = module.package {
        Ok(Value::obj(pkg_ref).wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn module_root_package(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Module",
        found: receiver.type_name(),
    })?;
    let module = vm.heap.module(id);
    if let Some(root_ref) = module.root_package {
        Ok(Value::obj(root_ref).wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn module_package_info(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Module",
        found: receiver.type_name(),
    })?;
    let pkg_ref = {
        let module = vm.heap.module(id);
        module.root_package.or(module.package)
    };

    let desc = if let Some(pkg_ref) = pkg_ref {
        let pkg = vm.heap.module(pkg_ref);
        let project_sym = vm.interner.intern("__project__");
        if let Some(proj_val) = pkg.get(project_sym) {
            if let Some(proj_obj) = proj_val.without_some_wrappers().as_obj() {
                let manifest_obj = vm.heap.project(proj_obj).manifest;
                let m = vm.heap.project_manifest(manifest_obj);
                let name = m.name.clone();
                let ns = vm.interner.lookup(m.namespace).to_string();
                let desc = PackageInfoDescriptor {
                    name: name.clone(),
                    namespace: ns.into_boxed_str(),
                    version: m.version.clone(),
                    authors: Vec::new(),
                    description: m.description.clone(),
                    license: m.license.clone(),
                    homepage: None,
                    repository: None,
                    requirements: Vec::new(),
                    default_entry: m.default_entry.clone(),
                    identity: phalcom_modules::package_info::PackageArtifactIdentity::Resolved {
                        name: name.clone(),
                        version: m.version.clone(),
                    },
                };
                let info_obj = ReflectionCache::get_or_create_package_info(vm, &desc);
                return Ok(Value::obj(info_obj).wrap_some()?);
            }
        }

        match &pkg.id.project {
            phalcom_modules::ProjectIdentity::Builtin(b) => match b {
                phalcom_modules::BuiltinProject::Universe => PackageInfoDescriptor::builtin_universe(None),
                phalcom_modules::BuiltinProject::Std => PackageInfoDescriptor::builtin_std(None),
            },
            phalcom_modules::ProjectIdentity::Resolved(_) => {
                let name = pkg.name.clone();
                PackageInfoDescriptor::standalone(&name)
            }
            phalcom_modules::ProjectIdentity::Synthetic(_) => {
                let name = pkg.name.clone();
                PackageInfoDescriptor::standalone(&name)
            }
        }
    } else {
        let name = vm.heap.module(id).name.clone();
        PackageInfoDescriptor::standalone(&name)
    };

    let info_obj = ReflectionCache::get_or_create_package_info(vm, &desc);
    Ok(Value::obj(info_obj).wrap_some()?)
}

pub fn module_exports(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Module",
        found: receiver.type_name(),
    })?;
    let table = ReflectionCache::get_or_create_export_table(vm, id);
    Ok(Value::obj(table))
}

pub fn module_export_by_name(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Module",
        found: receiver.type_name(),
    })?;
    let table_ref = ReflectionCache::get_or_create_export_table(vm, id);
    let table_val = Value::obj(table_ref);
    export_table_get(vm, &table_val, args)
}

pub fn module_understands(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Module",
        found: receiver.type_name(),
    })?;
    if args.is_empty() {
        return Err(RuntimeError::Arity {
            signature: "__understands__(_)",
            expected: 1,
            found: args.len(),
        }
        .into());
    }
    let name_sym = match args[0].symbol_value() {
        Some(s) => s,
        None => {
            if let Some(str_id) = args[0].as_obj() {
                if let Some(s) = vm.heap.as_string(str_id) {
                    vm.interner.intern(&s.value())
                } else {
                    return Ok(Value::bool(false));
                }
            } else {
                return Ok(Value::bool(false));
            }
        }
    };

    let (has_export, module_kind) = {
        let module = vm.heap.module(id);
        (module.exports.contains_key(&name_sym), module.kind)
    };

    if has_export {
        return Ok(Value::bool(true));
    }

    let module_cls = match module_kind {
        crate::heap::ModuleKind::Module => vm.universe.classes.module_class,
        crate::heap::ModuleKind::Package => vm.universe.classes.package_class,
    };
    let understands_method = crate::heap::lookup_method_in_hierarchy(&vm.heap, module_cls, name_sym).is_some();

    Ok(Value::bool(understands_method))
}

pub fn module_metadata(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Module",
        found: receiver.type_name(),
    })?;
    let doc_opt = if let Some(meta) = &vm.heap.module(id).metadata {
        meta.attributes.iter().find(|a| a.name == "documentation").and_then(|a| {
            if let Some(phalcom_ast::ast::MetadataLiteral::String(s)) = a.arguments.first() {
                Some(s.clone())
            } else {
                None
            }
        })
    } else {
        None
    };

    if let Some(doc) = doc_opt {
        let s = vm.alloc_string_value(doc);
        Ok(s.wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn module_dependencies(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Module",
        found: receiver.type_name(),
    })?;
    let reads = vm.heap.module(id).linked_reads.clone();
    let phase_sym = vm.interner.intern("#runtime");
    let reason_sym = vm.interner.intern("#selectiveValueImport");

    let mut dep_values: Vec<Value> = Vec::new();
    for read in reads {
        let dep_mod_ref = match read {
            crate::modules::RuntimeLinkedRead::Binding(b) => b.module,
            crate::modules::RuntimeLinkedRead::Module(obj) => obj,
        };
        let dep_obj = vm
            .heap
            .alloc(Object::ModuleDependency(Box::new(crate::heap::reflection::ModuleDependencyObject {
                module: dep_mod_ref,
                phase_sym,
                reason_sym,
            })));
        dep_values.push(Value::obj(dep_obj));
    }
    let tuple_obj = vm.heap.alloc(Object::Tuple(TupleObject::positional(dep_values)));
    Ok(Value::obj(tuple_obj))
}

pub fn module_uri(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Module",
        found: receiver.type_name(),
    })?;
    let path = vm.heap.module(id).path.clone();
    let uri_str = if path.starts_with('/') {
        format!("file://{path}")
    } else if path.starts_with("file:") || path.contains("://") {
        path
    } else {
        format!("file:///{path}")
    };
    let uri_obj = ReflectionCache::get_or_create_uri(vm, &uri_str);
    Ok(Value::obj(uri_obj))
}

pub fn module_identity(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Module",
        found: receiver.type_name(),
    })?;
    let mod_id = vm.heap.module(id).id.clone();
    let id_obj = ReflectionCache::get_or_create_module_identity(vm, &mod_id);
    Ok(Value::obj(id_obj))
}

pub fn module_path(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Module",
        found: receiver.type_name(),
    })?;
    let path = vm.heap.module(id).id.path.to_string();
    Ok(vm.alloc_string_value(path))
}

pub fn module_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Module",
        found: receiver.type_name(),
    })?;
    let display_name = vm.heap.module(id).name.clone();
    let str_val = format!("<Module {}>", display_name);
    Ok(vm.alloc_string_value(str_val))
}

// ==========================================
// Package Primitives
// ==========================================

pub fn package_package(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(*receiver)
}

pub fn package_parent_package(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Package",
        found: receiver.type_name(),
    })?;
    let module = vm.heap.module(id);
    if let Some(pkg_ref) = module.package {
        if pkg_ref == id {
            // Root package has no parent
            Ok(Value::none())
        } else {
            Ok(Value::obj(pkg_ref).wrap_some()?)
        }
    } else {
        Ok(Value::none())
    }
}

pub fn package_root_package(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Package",
        found: receiver.type_name(),
    })?;
    let module = vm.heap.module(id);
    let root_ref = module.root_package.unwrap_or(id);
    Ok(Value::obj(root_ref))
}

pub fn package_package_info(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Package",
        found: receiver.type_name(),
    })?;
    let root_ref = {
        let module = vm.heap.module(id);
        module.root_package.unwrap_or(id)
    };
    let root = vm.heap.module(root_ref);

    let project_sym = vm.interner.intern("__project__");
    if let Some(proj_val) = root.get(project_sym) {
        if let Some(proj_obj) = proj_val.without_some_wrappers().as_obj() {
            let manifest_obj = vm.heap.project(proj_obj).manifest;
            let m = vm.heap.project_manifest(manifest_obj);
            let name = m.name.clone();
            let ns = vm.interner.lookup(m.namespace).to_string();
            let desc = PackageInfoDescriptor {
                name: name.clone(),
                namespace: ns.into_boxed_str(),
                version: m.version.clone(),
                authors: Vec::new(),
                description: m.description.clone(),
                license: m.license.clone(),
                homepage: None,
                repository: None,
                requirements: Vec::new(),
                default_entry: m.default_entry.clone(),
                identity: phalcom_modules::package_info::PackageArtifactIdentity::Resolved {
                    name: name.clone(),
                    version: m.version.clone(),
                },
            };
            let info_obj = ReflectionCache::get_or_create_package_info(vm, &desc);
            return Ok(Value::obj(info_obj));
        }
    }

    let desc = match &root.id.project {
        phalcom_modules::ProjectIdentity::Builtin(b) => match b {
            phalcom_modules::BuiltinProject::Universe => PackageInfoDescriptor::builtin_universe(None),
            phalcom_modules::BuiltinProject::Std => PackageInfoDescriptor::builtin_std(None),
        },
        phalcom_modules::ProjectIdentity::Resolved(_) => {
            let name = root.name.clone();
            PackageInfoDescriptor::standalone(&name)
        }
        phalcom_modules::ProjectIdentity::Synthetic(_) => {
            let name = root.name.clone();
            PackageInfoDescriptor::standalone(&name)
        }
    };

    let info_obj = ReflectionCache::get_or_create_package_info(vm, &desc);
    Ok(Value::obj(info_obj))
}

pub fn package_children(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Package",
        found: receiver.type_name(),
    })?;
    let table = ReflectionCache::get_or_create_child_module_table(vm, id);
    Ok(Value::obj(table))
}

pub fn package_is_root(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Package",
        found: receiver.type_name(),
    })?;
    let module = vm.heap.module(id);
    let is_root = module.root_package.is_none_or(|r| r == id);
    Ok(Value::bool(is_root))
}

pub fn package_version(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let info = package_package_info(vm, receiver, args)?;
    let info_ref = info.as_obj().unwrap();
    let version_opt = vm.heap.package_info(info_ref).version.clone();
    if let Some(v) = version_opt {
        let s = vm.alloc_string_value(v);
        Ok(s.wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn package_namespace(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let info = package_package_info(vm, receiver, args)?;
    let info_ref = info.as_obj().unwrap();
    let ns = vm.heap.package_info(info_ref).namespace;
    Ok(Value::symbol(ns))
}

pub fn package_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Package",
        found: receiver.type_name(),
    })?;
    let display_name = vm.heap.module(id).name.clone();
    let str_val = format!("<Package {}>", display_name);
    Ok(vm.alloc_string_value(str_val))
}

// ==========================================
// Project Primitives
// ==========================================

pub fn project_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Project",
        found: receiver.type_name(),
    })?;
    let name = vm.heap.project(id).name.clone();
    Ok(vm.alloc_string_value(name))
}

pub fn project_namespace(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Project",
        found: receiver.type_name(),
    })?;
    let ns = vm.heap.project(id).namespace;
    Ok(Value::symbol(ns))
}

pub fn project_manifest(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Project",
        found: receiver.type_name(),
    })?;
    let manifest = vm.heap.project(id).manifest;
    Ok(Value::obj(manifest))
}

pub fn project_root_package(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Project",
        found: receiver.type_name(),
    })?;
    let root = vm.heap.project(id).root_package;
    Ok(Value::obj(root))
}

pub fn project_dependencies(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Project",
        found: receiver.type_name(),
    })?;
    let deps = vm.heap.project(id).dependencies;
    Ok(Value::obj(deps))
}

pub fn project_development_entry(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Project",
        found: receiver.type_name(),
    })?;
    if let Some(entry) = vm.heap.project(id).development_entry {
        Ok(Value::obj(entry).wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn project_identity(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Project",
        found: receiver.type_name(),
    })?;
    let identity = vm.heap.project(id).identity;
    Ok(Value::obj(identity))
}

pub fn project_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Project",
        found: receiver.type_name(),
    })?;
    let name = vm.heap.project(id).name.clone();
    let str_val = format!("<Project {}>", name);
    Ok(vm.alloc_string_value(str_val))
}

// ==========================================
// ProjectManifest Primitives
// ==========================================

pub fn project_manifest_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ProjectManifest",
        found: receiver.type_name(),
    })?;
    let name = vm.heap.project_manifest(id).name.clone();
    Ok(vm.alloc_string_value(name))
}

pub fn project_manifest_namespace(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ProjectManifest",
        found: receiver.type_name(),
    })?;
    let ns = vm.heap.project_manifest(id).namespace;
    Ok(Value::symbol(ns))
}

pub fn project_manifest_version(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ProjectManifest",
        found: receiver.type_name(),
    })?;
    let v_opt = vm.heap.project_manifest(id).version.clone();
    if let Some(v) = v_opt {
        let s = vm.alloc_string_value(v);
        Ok(s.wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn project_manifest_authors(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ProjectManifest",
        found: receiver.type_name(),
    })?;
    let authors = vm.heap.project_manifest(id).authors;
    Ok(Value::obj(authors))
}

pub fn project_manifest_description(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ProjectManifest",
        found: receiver.type_name(),
    })?;
    let d_opt = vm.heap.project_manifest(id).description.clone();
    if let Some(d) = d_opt {
        let s = vm.alloc_string_value(d);
        Ok(s.wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn project_manifest_license(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ProjectManifest",
        found: receiver.type_name(),
    })?;
    let l_opt = vm.heap.project_manifest(id).license.clone();
    if let Some(l) = l_opt {
        let s = vm.alloc_string_value(l);
        Ok(s.wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn project_manifest_homepage(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ProjectManifest",
        found: receiver.type_name(),
    })?;
    if let Some(hp) = vm.heap.project_manifest(id).homepage {
        Ok(Value::obj(hp).wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn project_manifest_repository(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ProjectManifest",
        found: receiver.type_name(),
    })?;
    if let Some(repo) = vm.heap.project_manifest(id).repository {
        Ok(Value::obj(repo).wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn project_manifest_source(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ProjectManifest",
        found: receiver.type_name(),
    })?;
    let src = vm.heap.project_manifest(id).source.clone();
    Ok(vm.alloc_string_value(src))
}

pub fn project_manifest_entry(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ProjectManifest",
        found: receiver.type_name(),
    })?;
    let e_opt = vm.heap.project_manifest(id).entry.clone();
    if let Some(e) = e_opt {
        let s = vm.alloc_string_value(e);
        Ok(s.wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn project_manifest_default_entry(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ProjectManifest",
        found: receiver.type_name(),
    })?;
    let de_opt = vm.heap.project_manifest(id).default_entry.clone();
    if let Some(de) = de_opt {
        let s = vm.alloc_string_value(de);
        Ok(s.wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn project_manifest_dependencies(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ProjectManifest",
        found: receiver.type_name(),
    })?;
    let deps = vm.heap.project_manifest(id).dependency_declarations;
    Ok(Value::obj(deps))
}

pub fn project_manifest_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ProjectManifest",
        found: receiver.type_name(),
    })?;
    let name = vm.heap.project_manifest(id).name.clone();
    let str_val = format!("<ProjectManifest {}>", name);
    Ok(vm.alloc_string_value(str_val))
}

// ==========================================
// PackageInfo Primitives
// ==========================================

pub fn package_info_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageInfo",
        found: receiver.type_name(),
    })?;
    let name = vm.heap.package_info(id).name.clone();
    Ok(vm.alloc_string_value(name))
}

pub fn package_info_namespace(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageInfo",
        found: receiver.type_name(),
    })?;
    let ns = vm.heap.package_info(id).namespace;
    Ok(Value::symbol(ns))
}

pub fn package_info_version(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageInfo",
        found: receiver.type_name(),
    })?;
    let v_opt = vm.heap.package_info(id).version.clone();
    if let Some(v) = v_opt {
        let s = vm.alloc_string_value(v);
        Ok(s.wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn package_info_authors(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageInfo",
        found: receiver.type_name(),
    })?;
    let authors = vm.heap.package_info(id).authors;
    Ok(Value::obj(authors))
}

pub fn package_info_description(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageInfo",
        found: receiver.type_name(),
    })?;
    let d_opt = vm.heap.package_info(id).description.clone();
    if let Some(d) = d_opt {
        let s = vm.alloc_string_value(d);
        Ok(s.wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn package_info_license(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageInfo",
        found: receiver.type_name(),
    })?;
    let l_opt = vm.heap.package_info(id).license.clone();
    if let Some(l) = l_opt {
        let s = vm.alloc_string_value(l);
        Ok(s.wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn package_info_homepage(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageInfo",
        found: receiver.type_name(),
    })?;
    if let Some(hp) = vm.heap.package_info(id).homepage {
        Ok(Value::obj(hp).wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn package_info_repository(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageInfo",
        found: receiver.type_name(),
    })?;
    if let Some(repo) = vm.heap.package_info(id).repository {
        Ok(Value::obj(repo).wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn package_info_requirements(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageInfo",
        found: receiver.type_name(),
    })?;
    let reqs = vm.heap.package_info(id).requirements;
    Ok(Value::obj(reqs))
}

pub fn package_info_default_entry(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageInfo",
        found: receiver.type_name(),
    })?;
    let de_opt = vm.heap.package_info(id).default_entry.clone();
    if let Some(de) = de_opt {
        let s = vm.alloc_string_value(de);
        Ok(s.wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn package_info_identity(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageInfo",
        found: receiver.type_name(),
    })?;
    let identity = vm.heap.package_info(id).identity;
    Ok(Value::obj(identity))
}

pub fn package_info_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageInfo",
        found: receiver.type_name(),
    })?;
    let name = vm.heap.package_info(id).name.clone();
    let str_val = format!("<PackageInfo {}>", name);
    Ok(vm.alloc_string_value(str_val))
}

// ==========================================
// ExportTable & Export Primitives
// ==========================================

pub fn export_table_names(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ExportTable",
        found: receiver.type_name(),
    })?;
    let tuple_ref = vm.heap.export_table(id).names_tuple;
    Ok(Value::obj(tuple_ref))
}

pub fn export_table_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ExportTable",
        found: receiver.type_name(),
    })?;
    let len = vm.heap.export_table(id).names.len() as i64;
    Ok(Value::int(len))
}

pub fn export_table_contains(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ExportTable",
        found: receiver.type_name(),
    })?;
    if args.is_empty() {
        return Err(RuntimeError::Arity {
            signature: "contains(_)",
            expected: 1,
            found: args.len(),
        }
        .into());
    }
    let name_sym = match args[0].symbol_value() {
        Some(s) => s,
        None => {
            if let Some(str_id) = args[0].as_obj() {
                if let Some(s) = vm.heap.as_string(str_id) {
                    vm.interner.intern(&s.value())
                } else {
                    return Ok(Value::bool(false));
                }
            } else {
                return Ok(Value::bool(false));
            }
        }
    };
    let has = vm.heap.export_table(id).descriptors.contains_key(&name_sym);
    Ok(Value::bool(has))
}

pub fn export_table_descriptor(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ExportTable",
        found: receiver.type_name(),
    })?;
    if args.is_empty() {
        return Err(RuntimeError::Arity {
            signature: "descriptor(_)",
            expected: 1,
            found: args.len(),
        }
        .into());
    }
    let name_sym = match args[0].symbol_value() {
        Some(s) => s,
        None => {
            if let Some(str_id) = args[0].as_obj() {
                if let Some(s) = vm.heap.as_string(str_id) {
                    vm.interner.intern(&s.value())
                } else {
                    return Ok(Value::none());
                }
            } else {
                return Ok(Value::none());
            }
        }
    };
    let desc_opt = vm.heap.export_table(id).descriptors.get(&name_sym).copied();
    if let Some(desc_ref) = desc_opt {
        Ok(Value::obj(desc_ref).wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn export_table_get(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ExportTable",
        found: receiver.type_name(),
    })?;
    if args.is_empty() {
        return Err(RuntimeError::Arity {
            signature: "get(_)",
            expected: 1,
            found: args.len(),
        }
        .into());
    }
    let name_sym = match args[0].symbol_value() {
        Some(s) => s,
        None => {
            if let Some(str_id) = args[0].as_obj() {
                if let Some(s) = vm.heap.as_string(str_id) {
                    vm.interner.intern(&s.value())
                } else {
                    return Ok(Value::none());
                }
            } else {
                return Ok(Value::none());
            }
        }
    };

    let module_ref = vm.heap.export_table(id).module;
    let export_ref_opt = vm.heap.module(module_ref).exports.get(&name_sym).cloned();

    match export_ref_opt {
        Some(RuntimeExportRef::Binding(b)) => {
            let val = vm.heap.module(b.module).globals.get(b.slot as usize).copied().unwrap_or(Value::nil());
            Ok(val.wrap_some()?)
        }
        Some(RuntimeExportRef::Module(mod_obj)) => Ok(Value::obj(mod_obj).wrap_some()?),
        None => Ok(Value::none()),
    }
}

pub fn export_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Export",
        found: receiver.type_name(),
    })?;
    let name = vm.heap.export(id).name;
    Ok(Value::symbol(name))
}

pub fn export_kind(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Export",
        found: receiver.type_name(),
    })?;
    let kind = vm.heap.export(id).kind_sym;
    Ok(Value::symbol(kind))
}

pub fn export_module(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Export",
        found: receiver.type_name(),
    })?;
    let module = vm.heap.export(id).module;
    Ok(Value::obj(module))
}

pub fn export_value(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Export",
        found: receiver.type_name(),
    })?;
    let (module_ref, name) = {
        let export = vm.heap.export(id);
        (export.module, export.name)
    };

    let export_ref_opt = vm.heap.module(module_ref).exports.get(&name).cloned();
    match export_ref_opt {
        Some(RuntimeExportRef::Binding(b)) => {
            let val = vm.heap.module(b.module).globals.get(b.slot as usize).copied().unwrap_or(Value::nil());
            Ok(val)
        }
        Some(RuntimeExportRef::Module(mod_obj)) => Ok(Value::obj(mod_obj)),
        None => Err(RuntimeError::Internal(format!("Export '{}' not found in module exports", vm.interner.lookup(name))).into()),
    }
}

pub fn export_is_module(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Export",
        found: receiver.type_name(),
    })?;
    let module_sym = vm.interner.intern("#module");
    let is_mod = vm.heap.export(id).kind_sym == module_sym;
    Ok(Value::bool(is_mod))
}

pub fn export_is_binding(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Export",
        found: receiver.type_name(),
    })?;
    let binding_sym = vm.interner.intern("#binding");
    let is_b = vm.heap.export(id).kind_sym == binding_sym;
    Ok(Value::bool(is_b))
}

// ==========================================
// ChildModuleTable Primitives
// ==========================================

pub fn child_module_table_names(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ChildModuleTable",
        found: receiver.type_name(),
    })?;
    let tuple_ref = vm.heap.child_module_table(id).names_tuple;
    Ok(Value::obj(tuple_ref))
}

pub fn child_module_table_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ChildModuleTable",
        found: receiver.type_name(),
    })?;
    let len = vm.heap.child_module_table(id).names.len() as i64;
    Ok(Value::int(len))
}

pub fn child_module_table_contains(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ChildModuleTable",
        found: receiver.type_name(),
    })?;
    if args.is_empty() {
        return Err(RuntimeError::Arity {
            signature: "contains(_)",
            expected: 1,
            found: args.len(),
        }
        .into());
    }
    let name_sym = match args[0].symbol_value() {
        Some(s) => s,
        None => {
            if let Some(str_id) = args[0].as_obj() {
                if let Some(s) = vm.heap.as_string(str_id) {
                    vm.interner.intern(&s.value())
                } else {
                    return Ok(Value::bool(false));
                }
            } else {
                return Ok(Value::bool(false));
            }
        }
    };
    let has = vm.heap.child_module_table(id).children.contains_key(&name_sym);
    Ok(Value::bool(has))
}

pub fn child_module_table_get(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ChildModuleTable",
        found: receiver.type_name(),
    })?;
    if args.is_empty() {
        return Err(RuntimeError::Arity {
            signature: "get(_)",
            expected: 1,
            found: args.len(),
        }
        .into());
    }
    let name_sym = match args[0].symbol_value() {
        Some(s) => s,
        None => {
            if let Some(str_id) = args[0].as_obj() {
                if let Some(s) = vm.heap.as_string(str_id) {
                    vm.interner.intern(&s.value())
                } else {
                    return Ok(Value::none());
                }
            } else {
                return Ok(Value::none());
            }
        }
    };
    let child_opt = vm.heap.child_module_table(id).children.get(&name_sym).copied();
    if let Some(child_ref) = child_opt {
        Ok(Value::obj(child_ref).wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

// ==========================================
// Uri, Identity, Author, Requirement, Dependency
// ==========================================

pub fn uri_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Uri",
        found: receiver.type_name(),
    })?;
    let uri_str = vm.heap.uri(id).uri_str.clone();
    Ok(vm.alloc_string_value(uri_str))
}

pub fn uri_eq(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Uri",
        found: receiver.type_name(),
    })?;
    if args.is_empty() {
        return Ok(Value::bool(false));
    }
    if let Some(other_id) = args[0].as_obj() {
        if matches!(vm.heap.get(other_id), Object::Uri(_)) {
            let a = vm.heap.uri(id).uri_str.clone();
            let b = vm.heap.uri(other_id).uri_str.clone();
            return Ok(Value::bool(a == b));
        }
    }
    Ok(Value::bool(false))
}

pub fn module_identity_uri(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ModuleIdentity",
        found: receiver.type_name(),
    })?;
    let uri = vm.heap.module_identity(id).uri;
    Ok(Value::obj(uri))
}

pub fn module_identity_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ModuleIdentity",
        found: receiver.type_name(),
    })?;
    let id_str = vm.heap.module_identity(id).id_str.clone();
    Ok(vm.alloc_string_value(id_str))
}

pub fn package_identity_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageIdentity",
        found: receiver.type_name(),
    })?;
    let id_str = vm.heap.package_identity(id).identity_str.clone();
    Ok(vm.alloc_string_value(id_str))
}

pub fn project_identity_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ProjectIdentity",
        found: receiver.type_name(),
    })?;
    let id_str = vm.heap.project_identity(id).identity_str.clone();
    Ok(vm.alloc_string_value(id_str))
}

pub fn package_author_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageAuthor",
        found: receiver.type_name(),
    })?;
    let name = vm.heap.package_author(id).name.clone();
    Ok(vm.alloc_string_value(name))
}

pub fn package_author_email(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageAuthor",
        found: receiver.type_name(),
    })?;
    let email_opt = vm.heap.package_author(id).email.clone();
    if let Some(email) = email_opt {
        let s = vm.alloc_string_value(email);
        Ok(s.wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn package_author_url(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageAuthor",
        found: receiver.type_name(),
    })?;
    if let Some(url) = vm.heap.package_author(id).url {
        Ok(Value::obj(url).wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn package_requirement_alias(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageRequirement",
        found: receiver.type_name(),
    })?;
    let alias = vm.heap.package_requirement(id).alias;
    Ok(Value::symbol(alias))
}

pub fn package_requirement_package(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageRequirement",
        found: receiver.type_name(),
    })?;
    let pkg = vm.heap.package_requirement(id).package.clone();
    Ok(vm.alloc_string_value(pkg))
}

pub fn package_requirement_version_requirement(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageRequirement",
        found: receiver.type_name(),
    })?;
    let vreq = vm.heap.package_requirement(id).version_requirement.clone();
    Ok(vm.alloc_string_value(vreq))
}

pub fn package_requirement_optional(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "PackageRequirement",
        found: receiver.type_name(),
    })?;
    let opt = vm.heap.package_requirement(id).optional;
    Ok(Value::bool(opt))
}

pub fn resolved_project_dependency_alias(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ResolvedProjectDependency",
        found: receiver.type_name(),
    })?;
    let alias = vm.heap.resolved_project_dependency(id).alias;
    Ok(Value::symbol(alias))
}

pub fn resolved_project_dependency_requirement(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ResolvedProjectDependency",
        found: receiver.type_name(),
    })?;
    if let Some(req) = vm.heap.resolved_project_dependency(id).requirement {
        Ok(Value::obj(req).wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

pub fn resolved_project_dependency_package_info(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ResolvedProjectDependency",
        found: receiver.type_name(),
    })?;
    let info = vm.heap.resolved_project_dependency(id).package_info;
    Ok(Value::obj(info))
}

pub fn resolved_project_dependency_root_package(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ResolvedProjectDependency",
        found: receiver.type_name(),
    })?;
    let root = vm.heap.resolved_project_dependency(id).root_package;
    Ok(Value::obj(root))
}

pub fn resolved_project_dependency_origin(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ResolvedProjectDependency",
        found: receiver.type_name(),
    })?;
    let origin = vm.heap.resolved_project_dependency(id).origin_sym;
    Ok(Value::symbol(origin))
}

pub fn module_dependency_module(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ModuleDependency",
        found: receiver.type_name(),
    })?;
    let module = vm.heap.module_dependency(id).module;
    Ok(Value::obj(module))
}

pub fn module_dependency_phase(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ModuleDependency",
        found: receiver.type_name(),
    })?;
    let phase = vm.heap.module_dependency(id).phase_sym;
    Ok(Value::symbol(phase))
}

pub fn module_dependency_reason(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "ModuleDependency",
        found: receiver.type_name(),
    })?;
    let reason = vm.heap.module_dependency(id).reason_sym;
    Ok(Value::symbol(reason))
}
