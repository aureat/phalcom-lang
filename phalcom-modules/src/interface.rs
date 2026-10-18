use crate::authority::{ModuleAuthorityBuildError, ModuleAuthorityMap, ModuleAuthoritySet};
use crate::dunder::{DunderPolicy, DunderRole};
use crate::error::InterfaceError;
use crate::identity::{ModuleComponent, ModuleId};
use crate::metadata::ModuleMetadata;
use crate::source::ModuleKind;
use phalcom_ast::ast::{BindingKind, DependencyDecl, ImportDecl, ImportPath, ModuleImportDecl, Pattern, Program, ReExportDecl, SelectiveImportDecl, Statement};
use phalcom_common::range::SourceRange;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Canonical semantics of a top-level module binding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ModuleBindingSemantics {
    Runtime { mutable: bool },
    StaticConstant,
    Declaration,
}

/// Surface declaration for a top-level binding defined in a module.
#[derive(Clone, Debug, PartialEq)]
pub struct DeclarationSurface {
    pub name: String,
    pub semantics: ModuleBindingSemantics,
    pub range: SourceRange,
}

impl DeclarationSurface {
    pub fn is_const(&self) -> bool {
        matches!(self.semantics, ModuleBindingSemantics::StaticConstant)
    }
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
    /// A convenience export that names a declaration owned by another canonical module.
    ///
    /// Unlike a source re-export, this target has no local import binding. It preserves
    /// the defining declaration identity for provider-owned interface policy such as
    /// Universe root convenience names.
    CanonicalDeclaration { module: ModuleId, name: String },
}

/// Target of a linked export: either a live global declaration or a whole module identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum LinkedExportTarget {
    Binding(crate::linker::SymbolId),
    Module(ModuleId),
}

/// Canonical export after linking.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
    pub exports: ModuleAuthorityMap<Box<str>, LinkedExport>,
    /// Module metadata retained from source.
    pub metadata: ModuleMetadata,
}

impl LinkedModuleInterface {
    /// Clones the interface with budgeted owned identities and metadata.
    /// Frozen exports share their immutable canonical backing; source exports
    /// are copied into a budgeted frozen map.
    pub fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        let exports = if let Some(frozen) = self.exports.as_frozen() {
            ModuleAuthorityMap::Frozen(Arc::clone(frozen))
        } else {
            let mut entries = allocations.vector(self.exports.len())?;
            for (name, export) in self.exports.iter() {
                entries.push((
                    allocations.boxed_str(name)?,
                    clone_linked_export_with_allocation_budget(export, allocations)?,
                ));
            }
            ModuleAuthorityMap::from_budgeted_entries(entries, allocations).map_err(|error| match error {
                ModuleAuthorityBuildError::Allocation(error) => error,
                ModuleAuthorityBuildError::Entries(_) | ModuleAuthorityBuildError::SetEntries(_) => AllocationError::Failed,
            })?
        };
        Ok(Self {
            module: self.module.clone_with_allocation_budget(allocations)?,
            kind: self.kind,
            exports,
            metadata: self.metadata.clone_with_allocation_budget(allocations)?,
        })
    }

    /// Computes the semantic product fingerprint for this linked module interface.
    pub fn fingerprint(&self) -> crate::fingerprint::LinkedInterfaceFingerprint {
        crate::fingerprint::linked_interface_fingerprint(self)
    }
}

fn clone_linked_export_with_allocation_budget(export: &LinkedExport, allocations: &mut AllocationBudget) -> Result<LinkedExport, AllocationError> {
    let target = match &export.target {
        LinkedExportTarget::Binding(symbol) => LinkedExportTarget::Binding(crate::linker::SymbolId {
            module: symbol.module.clone_with_allocation_budget(allocations)?,
            name: allocations.boxed_str(&symbol.name)?,
        }),
        LinkedExportTarget::Module(module) => LinkedExportTarget::Module(module.clone_with_allocation_budget(allocations)?),
    };
    Ok(LinkedExport {
        public_name: allocations.boxed_str(&export.public_name)?,
        target,
        range: export.range,
    })
}

/// Surface representation of an imported binding.
#[derive(Clone, Debug, PartialEq)]
pub enum ImportSurface {
    Module(ModuleImportDecl),
    Selective(SelectiveImportDecl),
    ReExport(ReExportDecl),
}

impl ImportSurface {
    /// Logical import path authored at this import site.
    pub fn path(&self) -> &ImportPath {
        match self {
            Self::Module(decl) => &decl.path,
            Self::Selective(decl) => &decl.path,
            Self::ReExport(decl) => &decl.path,
        }
    }

    /// Source span of this import declaration.
    pub fn range(&self) -> SourceRange {
        match self {
            Self::Module(decl) => decl.range,
            Self::Selective(decl) => decl.range,
            Self::ReExport(decl) => decl.range,
        }
    }
}

/// An unlinked module interface extracted from AST local declarations before path resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct UnlinkedModuleInterface {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub declarations: ModuleAuthorityMap<String, DeclarationSurface>,
    pub exports: ModuleAuthorityMap<String, ExportSurface>,
    pub imports: Vec<ImportSurface>,
    pub exposed_children: ModuleAuthoritySet<ModuleComponent>,
    pub metadata: ModuleMetadata,
}

impl UnlinkedModuleInterface {
    /// Computes the semantic product fingerprint for this unlinked module interface.
    pub fn fingerprint(&self) -> crate::fingerprint::InterfaceFingerprint {
        crate::fingerprint::interface_fingerprint(self)
    }

    /// Yields each import surface paired with its deterministic authored import site identity.
    pub fn import_sites(&self) -> impl Iterator<Item = (crate::identity::ImportSiteId, &ImportSurface)> {
        self.imports.iter().enumerate().map(|(idx, import)| {
            (
                crate::identity::ImportSiteId::new(self.id.clone(), crate::identity::ImportSiteLocalId::new(idx as u32)),
                import,
            )
        })
    }
}

/// Path exposure surface of a package.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PackagePathSurface {
    pub exposed_children: ModuleAuthoritySet<ModuleComponent>,
}

/// Unified namespace binding record used during source-local interface validation.
#[derive(Clone, Debug, PartialEq)]
pub enum ModuleNamespaceBinding {
    Declaration { semantics: ModuleBindingSemantics, range: SourceRange },
    Import { range: SourceRange },
}

impl ModuleNamespaceBinding {
    pub fn is_const(&self) -> bool {
        match self {
            Self::Declaration { semantics, .. } => matches!(semantics, ModuleBindingSemantics::StaticConstant),
            Self::Import { .. } => false,
        }
    }
}

/// Module interface builder that performs source-local interface extraction and validation.
pub struct InterfaceBuilder;

impl InterfaceBuilder {
    /// Builds an `UnlinkedModuleInterface` from a parsed `Program`.
    #[allow(deprecated)]
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
                    Self::validate_dunder(&class_def.name, DunderRole::Binding, range)?;
                    Self::collect_declaration(&class_def.name, ModuleBindingSemantics::Declaration, range, &mut namespace, &mut declarations)?;
                }
                Statement::Data(data_def) => {
                    let range = (data_def.range.start..data_def.name_range.end).into();
                    Self::validate_dunder(&data_def.name, DunderRole::Binding, range)?;
                    Self::collect_declaration(&data_def.name, ModuleBindingSemantics::Declaration, range, &mut namespace, &mut declarations)?;
                }
                Statement::Trait(trait_def) => {
                    let range = (trait_def.range.start..trait_def.name_range.end).into();
                    Self::validate_dunder(&trait_def.name, DunderRole::Binding, range)?;
                    Self::collect_declaration(&trait_def.name, ModuleBindingSemantics::Declaration, range, &mut namespace, &mut declarations)?;
                }
                Statement::TypeAlias(alias) | Statement::TypeDefinition(alias) => {
                    let range = (alias.range.start..alias.name_range.end).into();
                    Self::validate_dunder(&alias.name, DunderRole::Binding, range)?;
                    Self::collect_declaration(&alias.name, ModuleBindingSemantics::Declaration, range, &mut namespace, &mut declarations)?;
                }
                Statement::Let(let_binding) => {
                    let semantics = match let_binding.kind {
                        BindingKind::Let => ModuleBindingSemantics::Runtime { mutable: false },
                        BindingKind::LetMut => ModuleBindingSemantics::Runtime { mutable: true },
                        BindingKind::Const => ModuleBindingSemantics::StaticConstant,
                    };
                    Self::collect_pattern_declarations(&let_binding.pattern, semantics, &mut namespace, &mut declarations)?;
                }
                // Inherent impls contribute behavior to their canonical
                // target; they do not declare a module namespace binding.
                Statement::Impl(_) => {}
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

                            Self::validate_dunder(&binding_name, DunderRole::ImportAlias, mod_decl.range)?;
                            Self::declare_namespace(&mut namespace, binding_name, ModuleNamespaceBinding::Import { range: mod_decl.range })?;
                            imports.push(ImportSurface::Module(mod_decl.clone()));
                        }
                        ImportDecl::Selective(sel_decl) => {
                            for item in &sel_decl.items {
                                let binding_name = if let Some(alias) = &item.alias {
                                    alias.name.clone()
                                } else {
                                    item.name.clone()
                                };

                                Self::validate_dunder(&binding_name, DunderRole::ImportAlias, item.range)?;
                                Self::declare_namespace(&mut namespace, binding_name, ModuleNamespaceBinding::Import { range: item.range })?;
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

                        Self::validate_dunder(&exported_name, DunderRole::Export, item.range)?;
                        Self::validate_dunder(&item.local_or_remote_name, DunderRole::ImportAlias, item.range)?;

                        if exports.contains_key(&exported_name) {
                            return Err(InterfaceError::DuplicateExport {
                                name: exported_name,
                                range: item.range,
                            });
                        }

                        // Direct re-export creates an immutable local import binding as well.
                        Self::declare_namespace(
                            &mut namespace,
                            item.local_or_remote_name.clone(),
                            ModuleNamespaceBinding::Import { range: item.range },
                        )?;

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
                    if !kind.is_package_like() {
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

                    Self::validate_dunder(&exported_name, DunderRole::Export, item.range)?;

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
            declarations: declarations.into(),
            exports: exports.into(),
            imports,
            exposed_children: exposed_children.into(),
            metadata,
        })
    }

    fn validate_dunder(name: &str, role: DunderRole, range: SourceRange) -> Result<(), InterfaceError> {
        DunderPolicy::default()
            .validate_user_declaration(name, role)
            .map_err(|_| InterfaceError::ReservedDunder {
                name: name.to_string(),
                role: match role {
                    DunderRole::Binding => "binding",
                    DunderRole::ImportAlias => "import alias",
                    DunderRole::Export => "export",
                    DunderRole::Field => "field",
                    DunderRole::Method => "method",
                    DunderRole::Parameter => "parameter",
                },
                range,
            })
    }

    fn collect_declaration(
        name: &str,
        semantics: ModuleBindingSemantics,
        range: SourceRange,
        namespace: &mut BTreeMap<String, ModuleNamespaceBinding>,
        declarations: &mut BTreeMap<String, DeclarationSurface>,
    ) -> Result<(), InterfaceError> {
        if let Some(existing) = declarations.get(name) {
            return Err(InterfaceError::DuplicateDeclaration {
                name: name.to_string(),
                first_range: existing.range,
                range,
            });
        }

        namespace.insert(name.to_string(), ModuleNamespaceBinding::Declaration { semantics, range });
        declarations.insert(
            name.to_string(),
            DeclarationSurface {
                name: name.to_string(),
                semantics,
                range,
            },
        );
        Ok(())
    }

    fn declare_namespace(
        namespace: &mut BTreeMap<String, ModuleNamespaceBinding>,
        name: String,
        binding: ModuleNamespaceBinding,
    ) -> Result<(), InterfaceError> {
        if let Some(previous) = namespace.get(&name) {
            let previous_range = match previous {
                ModuleNamespaceBinding::Declaration { range, .. } | ModuleNamespaceBinding::Import { range } => *range,
            };
            let range = match &binding {
                ModuleNamespaceBinding::Declaration { range, .. } | ModuleNamespaceBinding::Import { range } => *range,
            };
            return Err(InterfaceError::DuplicateBinding { name, previous_range, range });
        }
        namespace.insert(name, binding);
        Ok(())
    }

    fn collect_pattern_declarations(
        pattern: &Pattern,
        semantics: ModuleBindingSemantics,
        namespace: &mut BTreeMap<String, ModuleNamespaceBinding>,
        declarations: &mut BTreeMap<String, DeclarationSurface>,
    ) -> Result<(), InterfaceError> {
        match pattern {
            Pattern::Name { name, range } => {
                Self::validate_dunder(name, DunderRole::Binding, *range)?;
                Self::collect_declaration(name, semantics, *range, namespace, declarations)
            }
            Pattern::Tuple { elements, .. } => {
                for elem in elements {
                    Self::collect_pattern_declarations(elem, semantics, namespace, declarations)?;
                }
                Ok(())
            }
            Pattern::List { elements, rest, .. } => {
                for elem in elements {
                    Self::collect_pattern_declarations(elem, semantics, namespace, declarations)?;
                }
                if let Some(rest) = rest {
                    Self::collect_pattern_declarations(rest, semantics, namespace, declarations)?;
                }
                Ok(())
            }
            Pattern::Wildcard { .. } | Pattern::Char { .. } => Ok(()),
            Pattern::Or { alternatives, .. } => {
                for alt in alternatives {
                    Self::collect_pattern_declarations(alt, semantics, namespace, declarations)?;
                }
                Ok(())
            }
            Pattern::Variant(variant) => {
                match &variant.mode {
                    phalcom_ast::ast::VariantPatternMode::ExactCall { arguments } => {
                        for argument in arguments {
                            Self::collect_pattern_declarations(&argument.pattern, semantics, namespace, declarations)?;
                        }
                    }
                    phalcom_ast::ast::VariantPatternMode::CallablePattern { prefix, suffix, .. } => {
                        for argument in prefix.iter().chain(suffix.iter()) {
                            Self::collect_pattern_declarations(&argument.pattern, semantics, namespace, declarations)?;
                        }
                    }
                    phalcom_ast::ast::VariantPatternMode::Singleton | phalcom_ast::ast::VariantPatternMode::WholeFamily { .. } => {}
                }
                Ok(())
            }
            Pattern::Record { entries, .. } => {
                for entry in entries {
                    Self::collect_pattern_declarations(&entry.pattern, semantics, namespace, declarations)?;
                }
                Ok(())
            }
            Pattern::Map { entries, .. } => {
                for entry in entries {
                    Self::collect_pattern_declarations(&entry.pattern, semantics, namespace, declarations)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::ModulePath;
    use phalcom_ast::parser::parse;

    #[test]
    fn module_surface_distinguishes_runtime_immutable_mutable_static() {
        let source = r#"
let immutable_val = 1
let mut mutable_val = 2
const static_const = 3
class Foo {}
"#;
        let parsed = parse(source, 0);
        assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
        let id = ModuleId::universe(ModulePath::root());
        let interface = InterfaceBuilder::build(id, ModuleKind::Module, &parsed.program).expect("built interface");

        assert_eq!(
            interface.declarations["immutable_val"].semantics,
            ModuleBindingSemantics::Runtime { mutable: false }
        );
        assert_eq!(
            interface.declarations["mutable_val"].semantics,
            ModuleBindingSemantics::Runtime { mutable: true }
        );
        assert_eq!(interface.declarations["static_const"].semantics, ModuleBindingSemantics::StaticConstant);
        assert_eq!(interface.declarations["Foo"].semantics, ModuleBindingSemantics::Declaration);
    }

    #[test]
    fn field_declaration_not_mislabeled_const() {
        let source = r#"
class Entity {
  _id: Int
  mut _count: Int
}
data Point(x: Int, y: Int)
trait Describable {}
type IntAlias = Int
"#;
        let parsed = parse(source, 0);
        assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
        let id = ModuleId::universe(ModulePath::root());
        let interface = InterfaceBuilder::build(id, ModuleKind::Module, &parsed.program).expect("built interface");

        // Top level declarations must have Declaration semantics, NOT StaticConstant (or is_const = true)
        assert_eq!(interface.declarations["Entity"].semantics, ModuleBindingSemantics::Declaration);
        assert_eq!(interface.declarations["Point"].semantics, ModuleBindingSemantics::Declaration);
        assert_eq!(interface.declarations["Describable"].semantics, ModuleBindingSemantics::Declaration);
        assert_eq!(interface.declarations["IntAlias"].semantics, ModuleBindingSemantics::Declaration);

        assert!(!interface.declarations["Entity"].is_const());
        assert!(!interface.declarations["Point"].is_const());
    }
}
