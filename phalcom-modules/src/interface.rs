use crate::error::InterfaceError;
use crate::identity::{ModuleComponent, ModuleId};
use crate::metadata::ModuleMetadata;
use crate::source::ModuleKind;
use phalcom_ast::ast::{
    BindingKind, DependencyDecl, ImportDecl, ImportPath, ModuleImportDecl, Pattern, Program, ReExportDecl, SelectiveImportDecl, Statement,
};
use phalcom_common::range::SourceRange;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
pub struct DeclarationSurface {
    pub name: String,
    pub is_const: bool,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExportSurface {
    pub exported_name: String,
    pub internal_name: String,
    pub target: UnlinkedExportTarget,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnlinkedExportTarget {
    Local(String),
    ReExport { path: ImportPath, remote: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkedExportTarget {
    Binding(crate::linker::SymbolId),
    Module(ModuleId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinkedExport {
    pub public_name: Box<str>,
    pub target: LinkedExportTarget,
    pub range: SourceRange,
}

impl LinkedExport {
    pub fn symbol(&self) -> Option<&crate::linker::SymbolId> {
        match &self.target {
            LinkedExportTarget::Binding(sym) => Some(sym),
            LinkedExportTarget::Module(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinkedModuleInterface {
    pub module: ModuleId,
    pub kind: ModuleKind,
    pub exports: BTreeMap<Box<str>, LinkedExport>,
    pub metadata: ModuleMetadata,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportSurface {
    Module(ModuleImportDecl),
    Selective(SelectiveImportDecl),
    ReExport(ReExportDecl),
}

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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PackagePathSurface {
    pub exposed_children: BTreeSet<ModuleComponent>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModuleNamespaceBinding {
    Declaration { is_const: bool, range: SourceRange },
    Import { range: SourceRange },
}

impl ModuleNamespaceBinding {
    fn range(&self) -> SourceRange {
        match self {
            Self::Declaration { range, .. } | Self::Import { range } => *range,
        }
    }
}

pub struct InterfaceBuilder;

impl InterfaceBuilder {
    /// Three-pass declarative interface construction. Every explicit local name
    /// enters through one checked namespace insertion path.
    pub fn build(id: ModuleId, kind: ModuleKind, program: &Program) -> Result<UnlinkedModuleInterface, InterfaceError> {
        let metadata = ModuleMetadata::from_ast(&program.preamble.metadata, kind)?;
        let mut declarations = BTreeMap::new();
        let mut exports = BTreeMap::new();
        let mut imports = Vec::new();
        let mut exposed_children = BTreeSet::new();
        let mut namespace = BTreeMap::new();

        // Pass 1: body declarations. Export statements are intentionally not
        // processed here, so export-before-declaration remains valid.
        for stmt in &program.statements {
            match stmt {
                Statement::Class(class_def) => {
                    let range = (class_def.range.start..class_def.name_range.end).into();
                    Self::collect_declaration(&class_def.name, true, range, &mut namespace, &mut declarations)?;
                }
                Statement::Let(let_binding) => {
                    let is_const = let_binding.kind == BindingKind::Const;
                    Self::collect_pattern_declarations(&let_binding.pattern, is_const, &mut namespace, &mut declarations)?;
                }
                _ => {}
            }
        }

        // Pass 2: imports/re-exports/exposure, using the same namespace.
        for dep in &program.preamble.dependencies {
            match dep {
                DependencyDecl::Import(import_decl) => match import_decl {
                    ImportDecl::Module(mod_decl) => {
                        let binding_name = if let Some(alias) = &mod_decl.alias {
                            alias.name.clone()
                        } else if let Some(last) = mod_decl.path.segments.last() {
                            last.name.clone()
                        } else {
                            match &mod_decl.path.root {
                                phalcom_ast::ast::ImportRoot::Absolute(seg) => seg.name.clone(),
                                phalcom_ast::ast::ImportRoot::Relative { .. } => {
                                    return Err(InterfaceError::DuplicateImportBinding {
                                        name: "<relative-root>".to_string(),
                                        previous_range: mod_decl.range,
                                        range: mod_decl.range,
                                    });
                                }
                            }
                        };
                        Self::collect_import_binding(&binding_name, mod_decl.range, &mut namespace)?;
                        imports.push(ImportSurface::Module(mod_decl.clone()));
                    }
                    ImportDecl::Selective(sel_decl) => {
                        for item in &sel_decl.items {
                            let binding_name = item.alias.as_ref().map(|alias| alias.name.clone()).unwrap_or_else(|| item.name.clone());
                            Self::collect_import_binding(&binding_name, item.range, &mut namespace)?;
                        }
                        imports.push(ImportSurface::Selective(sel_decl.clone()));
                    }
                },
                DependencyDecl::ReExport(reexport_decl) => {
                    for item in &reexport_decl.items {
                        let exported_name = item
                            .alias
                            .as_ref()
                            .map(|alias| alias.name.clone())
                            .unwrap_or_else(|| item.local_or_remote_name.clone());
                        if exports.contains_key(&exported_name) {
                            return Err(InterfaceError::DuplicateExport {
                                name: exported_name,
                                range: item.range,
                            });
                        }
                        Self::collect_import_binding(&item.local_or_remote_name, item.range, &mut namespace)?;
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

        // Pass 3: declarative exports against the completed namespace.
        for stmt in &program.statements {
            if let Statement::Export(export_decl) = stmt {
                for item in &export_decl.items {
                    let exported_name = item
                        .alias
                        .as_ref()
                        .map(|alias| alias.name.clone())
                        .unwrap_or_else(|| item.local_or_remote_name.clone());
                    if exports.contains_key(&exported_name) {
                        return Err(InterfaceError::DuplicateExport {
                            name: exported_name,
                            range: item.range,
                        });
                    }
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

    fn collect_declaration(
        name: &str,
        is_const: bool,
        range: SourceRange,
        namespace: &mut BTreeMap<String, ModuleNamespaceBinding>,
        declarations: &mut BTreeMap<String, DeclarationSurface>,
    ) -> Result<(), InterfaceError> {
        if let Some(previous) = namespace.get(name) {
            return match previous {
                ModuleNamespaceBinding::Declaration { range: first_range, .. } => Err(InterfaceError::DuplicateDeclaration {
                    name: name.to_string(),
                    first_range: *first_range,
                    range,
                }),
                ModuleNamespaceBinding::Import { .. } => Err(InterfaceError::DuplicateImportBinding {
                    name: name.to_string(),
                    previous_range: previous.range(),
                    range,
                }),
            };
        }
        namespace.insert(
            name.to_string(),
            ModuleNamespaceBinding::Declaration {
                is_const,
                range,
            },
        );
        declarations.insert(
            name.to_string(),
            DeclarationSurface {
                name: name.to_string(),
                is_const,
                range,
            },
        );
        Ok(())
    }

    fn collect_import_binding(
        name: &str,
        range: SourceRange,
        namespace: &mut BTreeMap<String, ModuleNamespaceBinding>,
    ) -> Result<(), InterfaceError> {
        if let Some(previous) = namespace.get(name) {
            return Err(InterfaceError::DuplicateImportBinding {
                name: name.to_string(),
                previous_range: previous.range(),
                range,
            });
        }
        namespace.insert(name.to_string(), ModuleNamespaceBinding::Import { range });
        Ok(())
    }

    fn collect_pattern_declarations(
        pattern: &Pattern,
        is_const: bool,
        namespace: &mut BTreeMap<String, ModuleNamespaceBinding>,
        declarations: &mut BTreeMap<String, DeclarationSurface>,
    ) -> Result<(), InterfaceError> {
        match pattern {
            Pattern::Name { name, range } => Self::collect_declaration(name, is_const, *range, namespace, declarations),
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
