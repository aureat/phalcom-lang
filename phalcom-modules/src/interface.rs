use crate::error::InterfaceError;
use crate::identity::{ModuleComponent, ModuleId};
use crate::metadata::ModuleMetadata;
use crate::source::ModuleKind;
use phalcom_ast::ast::{BindingKind, DependencyDecl, ImportDecl, ImportPath, ModuleImportDecl, Pattern, Program, ReExportDecl, SelectiveImportDecl, Statement};
use phalcom_common::range::SourceRange;
use std::collections::{BTreeMap, BTreeSet};

/// Surface declaration for a top-level binding defined in a module.
#[derive(Clone, Debug, PartialEq)]
pub struct DeclarationSurface {
    pub name: String,
    pub is_const: bool,
    pub range: SourceRange,
}

/// Surface representation of an exported binding.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportSurface {
    pub exported_name: String,
    pub internal_name: String,
    /// Source-local target before project linking.
    pub target: UnlinkedExportTarget,
    pub range: SourceRange,
}

/// Export target before import paths have been resolved to module identities.
#[derive(Clone, Debug, PartialEq)]
pub enum UnlinkedExportTarget {
    /// A declaration or imported local name in the current module.
    Local(String),
    /// A name selected from another module path.
    ReExport { path: ImportPath, remote: String },
}

/// Target of a linked export: either a live global declaration or a whole module identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkedExportTarget {
    Binding(crate::linker::SymbolId),
    Module(ModuleId),
}

/// Canonical export after linking.
#[derive(Clone, Debug, PartialEq)]
pub struct LinkedExport {
    /// Public name exposed by the exporting module.
    pub public_name: Box<str>,
    /// Export target (binding symbol or module).
    pub target: LinkedExportTarget,
    /// Source span of the export item.
    pub range: SourceRange,
}

impl LinkedExport {
    /// Helper returning reference to the binding symbol if target is `LinkedExportTarget::Binding`.
    pub fn symbol(&self) -> Option<&crate::linker::SymbolId> {
        match &self.target {
            LinkedExportTarget::Binding(sym) => Some(sym),
            LinkedExportTarget::Module(_) => None,
        }
    }
}

/// Module interface after all imports and exports are linked.
#[derive(Clone, Debug, PartialEq)]
pub struct LinkedModuleInterface {
    /// Module identity.
    pub module: ModuleId,
    /// Source kind retained for compile/materialization policy.
    pub kind: ModuleKind,
    /// Canonical public exports.
    pub exports: BTreeMap<Box<str>, LinkedExport>,
    /// Module metadata retained from source.
    pub metadata: ModuleMetadata,
}

/// Surface representation of an imported binding.
#[derive(Clone, Debug, PartialEq)]
pub enum ImportSurface {
    Module(ModuleImportDecl),
    Selective(SelectiveImportDecl),
    ReExport(ReExportDecl),
}

/// An unlinked module interface extracted from AST local declarations before path resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct UnlinkedModuleInterface {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub declarations: BTreeMap<String, DeclarationSurface>,
    pub exports: BTreeMap<String, ExportSurface>,
    pub imports: Vec<ImportSurface>,
    pub exposed_children: BTreeSet<ModuleComponent>,
    pub metadata: ModuleMetadata,
}

/// Path exposure surface of a package.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PackagePathSurface {
    pub exposed_children: BTreeSet<ModuleComponent>,
}

/// Unified namespace binding record used during source-local interface validation.
#[derive(Clone, Debug, PartialEq)]
pub enum ModuleNamespaceBinding {
    Declaration { is_const: bool, range: SourceRange },
    Import { range: SourceRange },
}

/// Module interface builder that performs source-local interface extraction and validation.
pub struct InterfaceBuilder;

impl InterfaceBuilder {
    /// Builds an `UnlinkedModuleInterface` from a parsed `Program`.
    pub fn build(id: ModuleId, kind: ModuleKind, program: &Program) -> Result<UnlinkedModuleInterface, InterfaceError> {
        let metadata = ModuleMetadata::from_ast(&program.preamble.metadata, kind)?;
        let mut declarations = BTreeMap::new();
        let mut exports = BTreeMap::new();
        let mut imports = Vec::new();
        let mut exposed_children = BTreeSet::new();
        let mut namespace = BTreeMap::new();

        // ─────────────────────────────────────────────────────────────────
        // Pass 1: Collect body declarations into namespace and declaration surface
        // ─────────────────────────────────────────────────────────────────
        for stmt in &program.statements {
            match stmt {
                Statement::Class(class_def) => {
                    let range = (class_def.range.start..class_def.name_range.end).into();
                    namespace.insert(class_def.name.clone(), ModuleNamespaceBinding::Declaration { is_const: true, range });
                    declarations.insert(
                        class_def.name.clone(),
                        DeclarationSurface {
                            name: class_def.name.clone(),
                            is_const: true,
                            range,
                        },
                    );
                }
                Statement::Let(let_binding) => {
                    let is_const = let_binding.kind == BindingKind::Const;
                    Self::collect_pattern_declarations(&let_binding.pattern, is_const, &mut namespace, &mut declarations)?;
                }
                _ => {}
            }
        }

        // ─────────────────────────────────────────────────────────────────
        // Pass 2: Collect preamble imports and check collisions with declarations
        // ─────────────────────────────────────────────────────────────────
        for dep in &program.preamble.dependencies {
            match dep {
                DependencyDecl::Import(import_decl) => {
                    match import_decl {
                        ImportDecl::Module(mod_decl) => {
                            let binding_name = if let Some(alias) = &mod_decl.alias {
                                alias.name.clone()
                            } else {
                                // Default binding is the final component
                                if mod_decl.path.segments.is_empty() {
                                    match &mod_decl.path.root {
                                        phalcom_ast::ast::ImportRoot::Absolute(seg) => seg.name.clone(),
                                        phalcom_ast::ast::ImportRoot::Relative { .. } => {
                                            return Err(InterfaceError::DuplicateImportBinding {
                                                name: "<root>".to_string(),
                                                range: mod_decl.range,
                                            });
                                        }
                                    }
                                } else {
                                    mod_decl.path.segments.last().unwrap().name.clone()
                                }
                            };

                            if namespace.contains_key(&binding_name) {
                                return Err(InterfaceError::DuplicateImportBinding {
                                    name: binding_name,
                                    range: mod_decl.range,
                                });
                            }
                            namespace.insert(binding_name, ModuleNamespaceBinding::Import { range: mod_decl.range });
                            imports.push(ImportSurface::Module(mod_decl.clone()));
                        }
                        ImportDecl::Selective(sel_decl) => {
                            for item in &sel_decl.items {
                                let binding_name = if let Some(alias) = &item.alias {
                                    alias.name.clone()
                                } else {
                                    item.name.clone()
                                };

                                if namespace.contains_key(&binding_name) {
                                    return Err(InterfaceError::DuplicateImportBinding {
                                        name: binding_name,
                                        range: item.range,
                                    });
                                }
                                namespace.insert(binding_name, ModuleNamespaceBinding::Import { range: item.range });
                            }
                            imports.push(ImportSurface::Selective(sel_decl.clone()));
                        }
                    }
                }
                DependencyDecl::ReExport(reexport_decl) => {
                    for item in &reexport_decl.items {
                        let exported_name = if let Some(alias) = &item.alias {
                            alias.name.clone()
                        } else {
                            item.local_or_remote_name.clone()
                        };

                        if exports.contains_key(&exported_name) {
                            return Err(InterfaceError::DuplicateExport {
                                name: exported_name,
                                range: item.range,
                            });
                        }

                        // Direct re-export creates an immutable local import binding as well
                        if namespace.contains_key(&item.local_or_remote_name) {
                            return Err(InterfaceError::DuplicateImportBinding {
                                name: item.local_or_remote_name.clone(),
                                range: item.range,
                            });
                        }
                        namespace.insert(item.local_or_remote_name.clone(), ModuleNamespaceBinding::Import { range: item.range });

                        exports.insert(
                            exported_name.clone(),
                            ExportSurface {
                                exported_name,
                                internal_name: item.local_or_remote_name.clone(),
                                target: UnlinkedExportTarget::ReExport {
                                    path: reexport_decl.path.clone(),
                                    remote: item.local_or_remote_name.clone(),
                                },
                                range: item.range,
                            },
                        );
                    }
                    imports.push(ImportSurface::ReExport(reexport_decl.clone()));
                }
                DependencyDecl::Expose(expose_decl) => {
                    if kind != ModuleKind::Package {
                        return Err(InterfaceError::ExposeOutsidePackage(expose_decl.range));
                    }
                    let comp = ModuleComponent::from_identifier(&expose_decl.child.name)
                        .map_err(|_| InterfaceError::InvalidExposeTarget(expose_decl.child.name.clone(), expose_decl.range))?;
                    exposed_children.insert(comp);
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────
        // Pass 3: Validate body export statements against unified namespace
        // ─────────────────────────────────────────────────────────────────
        for stmt in &program.statements {
            if let Statement::Export(export_decl) = stmt {
                for item in &export_decl.items {
                    let exported_name = if let Some(alias) = &item.alias {
                        alias.name.clone()
                    } else {
                        item.local_or_remote_name.clone()
                    };

                    if exports.contains_key(&exported_name) {
                        return Err(InterfaceError::DuplicateExport {
                            name: exported_name,
                            range: item.range,
                        });
                    }

                    // Validate that local_or_remote_name exists as a declaration or imported binding
                    let internal = &item.local_or_remote_name;
                    if !namespace.contains_key(internal) {
                        return Err(InterfaceError::UnknownExport {
                            name: internal.clone(),
                            range: item.range,
                        });
                    }

                    exports.insert(
                        exported_name.clone(),
                        ExportSurface {
                            exported_name,
                            internal_name: internal.clone(),
                            target: UnlinkedExportTarget::Local(internal.clone()),
                            range: item.range,
                        },
                    );
                }
            }
        }

        Ok(UnlinkedModuleInterface {
            id,
            kind,
            declarations,
            exports,
            imports,
            exposed_children,
            metadata,
        })
    }

    fn collect_pattern_declarations(
        pattern: &Pattern,
        is_const: bool,
        namespace: &mut BTreeMap<String, ModuleNamespaceBinding>,
        declarations: &mut BTreeMap<String, DeclarationSurface>,
    ) -> Result<(), InterfaceError> {
        match pattern {
            Pattern::Name { name, range } => {
                namespace.insert(name.clone(), ModuleNamespaceBinding::Declaration { is_const, range: *range });
                declarations.insert(
                    name.clone(),
                    DeclarationSurface {
                        name: name.clone(),
                        is_const,
                        range: *range,
                    },
                );
                Ok(())
            }
            Pattern::Tuple { elements, .. } => {
                for elem in elements {
                    Self::collect_pattern_declarations(elem, is_const, namespace, declarations)?;
                }
                Ok(())
            }
            Pattern::List { elements, rest, .. } => {
                for elem in elements {
                    Self::collect_pattern_declarations(elem, is_const, namespace, declarations)?;
                }
                if let Some(rest) = rest {
                    Self::collect_pattern_declarations(rest, is_const, namespace, declarations)?;
                }
                Ok(())
            }
        }
    }
}
