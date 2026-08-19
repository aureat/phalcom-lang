//! Centralized reflection object materialization, caching, and lifecycle management.

use crate::heap::reflection::*;
use crate::heap::{ObjRef, Object, RuntimeExportRef, TupleObject};
use crate::interner::Symbol;
use crate::value::Value;
use crate::vm::VM;
use phalcom_modules::identity::ModuleId;
use phalcom_modules::package_info::{PackageArtifactIdentity, PackageAuthorDescriptor, PackageInfoDescriptor, PackageRequirementDescriptor};
use std::collections::HashMap;

/// Cache for canonical, immutable reflection descriptors.
#[derive(Debug, Default)]
pub struct ReflectionCache {
    uris: HashMap<String, ObjRef>,
    module_identities: HashMap<String, ObjRef>,
    package_identities: HashMap<String, ObjRef>,
    project_identities: HashMap<String, ObjRef>,
    package_infos: HashMap<String, ObjRef>,
    project_manifests: HashMap<String, ObjRef>,
    export_tables: HashMap<ObjRef, ObjRef>,
    child_module_tables: HashMap<ObjRef, ObjRef>,
    projects: HashMap<String, ObjRef>,
}

impl ReflectionCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Traces all cached object references during garbage collection.
    pub fn trace(&self, push: &mut impl FnMut(ObjRef)) {
        for obj in self.uris.values() {
            push(*obj);
        }
        for obj in self.module_identities.values() {
            push(*obj);
        }
        for obj in self.package_identities.values() {
            push(*obj);
        }
        for obj in self.project_identities.values() {
            push(*obj);
        }
        for obj in self.package_infos.values() {
            push(*obj);
        }
        for obj in self.project_manifests.values() {
            push(*obj);
        }
        for obj in self.export_tables.values() {
            push(*obj);
        }
        for obj in self.child_module_tables.values() {
            push(*obj);
        }
        for obj in self.projects.values() {
            push(*obj);
        }
    }

    /// Gets or creates a canonical [`UriObject`].
    pub fn get_or_create_uri(vm: &mut VM, uri_str: &str) -> ObjRef {
        if let Some(obj) = vm.reflection_cache.uris.get(uri_str) {
            return *obj;
        }
        let obj = vm.heap.alloc(Object::Uri(Box::new(UriObject { uri_str: uri_str.to_string() })));
        vm.reflection_cache.uris.insert(uri_str.to_string(), obj);
        obj
    }

    /// Gets or creates a canonical [`ModuleIdentityObject`].
    pub fn get_or_create_module_identity(vm: &mut VM, id: &ModuleId) -> ObjRef {
        let key = format!("mod:{id}");
        if let Some(obj) = vm.reflection_cache.module_identities.get(&key) {
            return *obj;
        }
        let uri = Self::get_or_create_uri(vm, &key);
        let obj = vm
            .heap
            .alloc(Object::ModuleIdentity(Box::new(ModuleIdentityObject { id_str: key.clone(), uri })));
        vm.reflection_cache.module_identities.insert(key, obj);
        obj
    }

    /// Gets or creates a canonical [`PackageIdentityObject`].
    pub fn get_or_create_package_identity(vm: &mut VM, identity: &PackageArtifactIdentity) -> ObjRef {
        let key = identity.to_string();
        if let Some(obj) = vm.reflection_cache.package_identities.get(&key) {
            return *obj;
        }
        let obj = vm
            .heap
            .alloc(Object::PackageIdentity(Box::new(PackageIdentityObject { identity_str: key.clone() })));
        vm.reflection_cache.package_identities.insert(key, obj);
        obj
    }

    /// Gets or creates a canonical [`ProjectIdentityObject`].
    pub fn get_or_create_project_identity(vm: &mut VM, identity_str: &str) -> ObjRef {
        if let Some(obj) = vm.reflection_cache.project_identities.get(identity_str) {
            return *obj;
        }
        let obj = vm.heap.alloc(Object::ProjectIdentity(Box::new(ProjectIdentityObject {
            identity_str: identity_str.to_string(),
        })));
        vm.reflection_cache.project_identities.insert(identity_str.to_string(), obj);
        obj
    }

    /// Gets or creates a [`PackageAuthorObject`].
    pub fn get_or_create_package_author(vm: &mut VM, author: &PackageAuthorDescriptor) -> ObjRef {
        let url = author.url.as_ref().map(|u| Self::get_or_create_uri(vm, u));
        vm.heap.alloc(Object::PackageAuthor(Box::new(PackageAuthorObject {
            name: author.name.clone(),
            email: author.email.clone(),
            url,
        })))
    }

    /// Gets or creates a [`PackageRequirementObject`].
    pub fn get_or_create_package_requirement(vm: &mut VM, req: &PackageRequirementDescriptor) -> ObjRef {
        let alias = vm.interner.intern(&req.alias);
        vm.heap.alloc(Object::PackageRequirement(Box::new(PackageRequirementObject {
            alias,
            package: req.package.clone(),
            version_requirement: req.version_requirement.clone(),
            optional: req.optional,
        })))
    }

    /// Gets or creates a canonical [`PackageInfoObject`].
    pub fn get_or_create_package_info(vm: &mut VM, desc: &PackageInfoDescriptor) -> ObjRef {
        let key = desc.identity.to_string();
        if let Some(obj) = vm.reflection_cache.package_infos.get(&key) {
            return *obj;
        }

        let namespace = vm.interner.intern(&desc.namespace);
        let author_refs: Vec<Value> = desc.authors.iter().map(|a| Value::obj(Self::get_or_create_package_author(vm, a))).collect();
        let authors = vm.heap.alloc(Object::Tuple(TupleObject::positional(author_refs)));

        let homepage = desc.homepage.as_ref().map(|u| Self::get_or_create_uri(vm, u));
        let repository = desc.repository.as_ref().map(|u| Self::get_or_create_uri(vm, u));

        let req_refs: Vec<Value> = desc
            .requirements
            .iter()
            .map(|r| Value::obj(Self::get_or_create_package_requirement(vm, r)))
            .collect();
        let requirements = vm.heap.alloc(Object::Tuple(TupleObject::positional(req_refs)));

        let identity_obj = Self::get_or_create_package_identity(vm, &desc.identity);

        let info_obj = vm.heap.alloc(Object::PackageInfo(Box::new(PackageInfoObject {
            name: desc.name.to_string(),
            namespace,
            version: desc.version.as_ref().map(|v| v.to_string()),
            authors,
            description: desc.description.clone(),
            license: desc.license.clone(),
            homepage,
            repository,
            requirements,
            default_entry: desc.default_entry.clone(),
            identity: identity_obj,
        })));

        vm.reflection_cache.package_infos.insert(key, info_obj);
        info_obj
    }

    /// Gets or creates a canonical `ProjectManifest` object.
    pub fn get_or_create_project_manifest(vm: &mut VM, manifest: &phalcom_modules::manifest::ValidatedProjectManifest) -> ObjRef {
        let key = manifest.name.to_string();
        if let Some(obj) = vm.reflection_cache.project_manifests.get(&key) {
            return *obj;
        }

        let namespace = vm.interner.intern(manifest.namespace.as_str());
        let homepage = manifest.homepage.as_ref().map(|hp| Self::get_or_create_uri(vm, hp));
        let repository = manifest.repository.as_ref().map(|repo| Self::get_or_create_uri(vm, repo));

        let mut author_refs: Vec<Value> = Vec::new();
        for author_str in &manifest.authors {
            let author_desc = phalcom_modules::package_info::PackageAuthorDescriptor::parse(author_str);
            let author_obj = Self::get_or_create_package_author(vm, &author_desc);
            author_refs.push(Value::obj(author_obj));
        }
        let authors = vm.heap.alloc(Object::Tuple(TupleObject::positional(author_refs)));

        let mut dep_decl_refs: Vec<Value> = Vec::new();
        for (comp, (alias, spec)) in &manifest.dependencies {
            let req_desc = match spec {
                phalcom_modules::manifest::DependencySpec::Package { package, version } => phalcom_modules::package_info::PackageRequirementDescriptor {
                    alias: comp.as_str().to_string().into_boxed_str(),
                    package: package.clone(),
                    version_requirement: version.clone(),
                    optional: false,
                },
                phalcom_modules::manifest::DependencySpec::Path { .. } => phalcom_modules::package_info::PackageRequirementDescriptor {
                    alias: comp.as_str().to_string().into_boxed_str(),
                    package: alias.clone(),
                    version_requirement: "*".to_string(),
                    optional: false,
                },
            };
            let req_obj = Self::get_or_create_package_requirement(vm, &req_desc);
            dep_decl_refs.push(Value::obj(req_obj));
        }
        let dependency_declarations = vm.heap.alloc(Object::Tuple(TupleObject::positional(dep_decl_refs)));

        let manifest_obj = vm.heap.alloc(Object::ProjectManifest(Box::new(ProjectManifestObject {
            name: manifest.name.to_string(),
            namespace,
            version: manifest.version.clone(),
            authors,
            description: manifest.description.clone(),
            license: manifest.license.clone(),
            homepage,
            repository,
            source: manifest.source.display().to_string(),
            entry: manifest.entry.clone(),
            default_entry: manifest.default_entry.clone(),
            dependency_declarations,
        })));

        vm.reflection_cache.project_manifests.insert(key, manifest_obj);
        manifest_obj
    }

    /// Gets or creates a canonical `ExportTable` object for the given module/package.
    pub fn get_or_create_export_table(vm: &mut VM, module_ref: ObjRef) -> ObjRef {
        if let Some(obj) = vm.reflection_cache.export_tables.get(&module_ref) {
            return *obj;
        }

        let module_obj = vm.heap.module(module_ref);
        let mut names: Vec<Symbol> = module_obj.exports.keys().copied().collect();
        // Deterministic sort by string representation
        names.sort_by_cached_key(|s| vm.interner.lookup(*s).to_string());

        let binding_sym = vm.interner.intern("#binding");
        let module_sym = vm.interner.intern("#module");

        let mut export_kinds: Vec<(Symbol, Symbol)> = Vec::new();
        for name in &names {
            let export_ref = module_obj.exports.get(name).expect("export exists");
            let kind_sym = match export_ref {
                RuntimeExportRef::Binding(_) => binding_sym,
                RuntimeExportRef::Module(_) => module_sym,
            };
            export_kinds.push((*name, kind_sym));
        }

        let mut descriptors = HashMap::new();
        for (name, kind_sym) in export_kinds {
            let desc_obj = vm.heap.alloc(Object::Export(Box::new(ExportObject {
                name,
                kind_sym,
                module: module_ref,
            })));
            descriptors.insert(name, desc_obj);
        }

        let names_values: Vec<Value> = names.iter().map(|s| Value::symbol(*s)).collect();
        let names_tuple = vm.heap.alloc(Object::Tuple(TupleObject::positional(names_values)));

        let table_obj = vm.heap.alloc(Object::ExportTable(Box::new(ExportTableObject {
            module: module_ref,
            names,
            names_tuple,
            descriptors,
        })));

        vm.reflection_cache.export_tables.insert(module_ref, table_obj);
        table_obj
    }

    /// Gets or creates a canonical `ChildModuleTable` object for the given package.
    pub fn get_or_create_child_module_table(vm: &mut VM, package_ref: ObjRef) -> ObjRef {
        if let Some(obj) = vm.reflection_cache.child_module_tables.get(&package_ref) {
            return *obj;
        }

        let pkg_id = vm.heap.module(package_ref).id.clone();
        let mut child_entries: Vec<(Symbol, ObjRef)> = Vec::new();

        // Query loaded modules in registry that are direct children of this package
        for (mod_id, record) in vm.module_registry.iter() {
            if mod_id.project == pkg_id.project {
                if let Some(parent_path) = mod_id.path.parent() {
                    if parent_path == pkg_id.path {
                        if let Some(child_comp) = mod_id.path.components().last() {
                            let sym = vm.interner.intern(child_comp.as_str());
                            child_entries.push((sym, record.object));
                        }
                    }
                }
            }
        }

        // Deterministic sort by name
        child_entries.sort_by_cached_key(|(s, _)| vm.interner.lookup(*s).to_string());

        let names: Vec<Symbol> = child_entries.iter().map(|(s, _)| *s).collect();
        let mut children = HashMap::new();
        for (s, obj) in &child_entries {
            children.insert(*s, *obj);
        }

        let names_values: Vec<Value> = names.iter().map(|s| Value::symbol(*s)).collect();
        let names_tuple = vm.heap.alloc(Object::Tuple(TupleObject::positional(names_values)));

        let table_obj = vm.heap.alloc(Object::ChildModuleTable(Box::new(ChildModuleTableObject {
            package: package_ref,
            names,
            names_tuple,
            children,
        })));

        vm.reflection_cache.child_module_tables.insert(package_ref, table_obj);
        table_obj
    }

    /// Gets or creates a canonical [`ProjectObject`].
    #[allow(clippy::too_many_arguments)]
    pub fn get_or_create_project(
        vm: &mut VM,
        name: &str,
        namespace_sym: Symbol,
        manifest_ref: ObjRef,
        root_package_ref: ObjRef,
        dependencies_tuple_ref: ObjRef,
        development_entry: Option<ObjRef>,
        identity_ref: ObjRef,
    ) -> ObjRef {
        if let Some(obj) = vm.reflection_cache.projects.get(name) {
            return *obj;
        }

        let proj_obj = vm.heap.alloc(Object::Project(Box::new(ProjectObject {
            name: name.to_string(),
            namespace: namespace_sym,
            manifest: manifest_ref,
            root_package: root_package_ref,
            dependencies: dependencies_tuple_ref,
            development_entry,
            identity: identity_ref,
        })));

        vm.reflection_cache.projects.insert(name.to_string(), proj_obj);
        proj_obj
    }
}
