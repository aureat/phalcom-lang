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

/// Canonical export after linking.
#[derive(Clone, Debug, PartialEq)]
pub struct LinkedExport {
    /// Public name exposed by the exporting module.
    pub public_name: Box<str>,
    /// Original declaration identity, preserved through aliases/re-exports.
    pub symbol: crate::linker::SymbolId,
    /// Source span of the export item.
    pub range: SourceRange,
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
        let mut local_import_bindings = BTreeMap::new(); // name -> range

        // 1. Process preamble dependencies
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

                            if let Some(_prev_range) = local_import_bindings.get(&binding_name) {
                                return Err(InterfaceError::DuplicateImportBinding {
                                    name: binding_name,
                                    range: mod_decl.range,
                                });
                            }
                            local_import_bindings.insert(binding_name, mod_decl.range);
                            imports.push(ImportSurface::Module(mod_decl.clone()));
                        }
                        ImportDecl::Selective(sel_decl) => {
                            for item in &sel_decl.items {
                                let binding_name = if let Some(alias) = &item.alias {
                                    alias.name.clone()
                                } else {
                                    item.name.clone()
                                };

                                if let Some(_prev_range) = local_import_bindings.get(&binding_name) {
                                    return Err(InterfaceError::DuplicateImportBinding {
                                        name: binding_name,
                                        range: item.range,
                                    });
                                }
                                local_import_bindings.insert(binding_name, item.range);
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
                        if local_import_bindings.contains_key(&item.local_or_remote_name) {
                            return Err(InterfaceError::DuplicateImportBinding {
                                name: item.local_or_remote_name.clone(),
                                range: item.range,
                            });
                        }
                        local_import_bindings.insert(item.local_or_remote_name.clone(), item.range);

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

        // 2. Process body statements: top-level declarations and local exports
        for stmt in &program.statements {
            match stmt {
                Statement::Class(class_def) => {
                    declarations.insert(
                        class_def.name.clone(),
                        DeclarationSurface {
                            name: class_def.name.clone(),
                            is_const: true,
                            range: (class_def.range.start..class_def.name_range.end).into(),
                        },
                    );
                }
                Statement::Let(let_binding) => {
                    let is_const = let_binding.kind == BindingKind::Const;
                    Self::collect_pattern_bindings(&let_binding.pattern, is_const, &mut declarations);
                }
                Statement::Export(export_decl) => {
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
                        if !declarations.contains_key(internal) && !local_import_bindings.contains_key(internal) {
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
                _ => {}
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

    fn collect_pattern_bindings(pattern: &Pattern, is_const: bool, declarations: &mut BTreeMap<String, DeclarationSurface>) {
        match pattern {
            Pattern::Name { name, range } => {
                declarations.insert(
                    name.clone(),
                    DeclarationSurface {
                        name: name.clone(),
                        is_const,
                        range: *range,
                    },
                );
            }
            Pattern::Tuple { elements, .. } => {
                for elem in elements {
                    Self::collect_pattern_bindings(elem, is_const, declarations);
                }
            }
            Pattern::List { elements, rest, .. } => {
                for elem in elements {
                    Self::collect_pattern_bindings(elem, is_const, declarations);
                }
                if let Some(rest) = rest {
                    Self::collect_pattern_bindings(rest, is_const, declarations);
                }
            }
        }
    }
}
