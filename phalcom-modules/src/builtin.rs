//! Provider-backed builtin project interfaces and canonical virtual source identity.

use crate::error::{ModuleLoadError, ModuleResolutionError};
use crate::identity::{BuiltinProject, ModuleComponent, ModuleId, ModulePath, ProjectIdentity, SourceId};
use crate::interface::{DeclarationSurface, ExportSurface, UnlinkedExportTarget, UnlinkedModuleInterface};
use crate::metadata::ModuleMetadata;
use crate::source::ModuleKind;
use phalcom_common::range::SourceRange;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug)]
struct BuiltinNodeSpec {
    path: &'static [&'static str],
    kind: ModuleKind,
    children: &'static [&'static str],
}

const UNIVERSE_NODES: &[BuiltinNodeSpec] = &[
    BuiltinNodeSpec {
        path: &[],
        kind: ModuleKind::ProjectRoot,
        children: &["object", "scalar", "callable", "option", "collections", "errors", "reflection", "concurrency"],
    },
    BuiltinNodeSpec {
        path: &["object"],
        kind: ModuleKind::Package,
        children: &["object", "behavior", "class", "metaclass"],
    },
    BuiltinNodeSpec {
        path: &["object", "object"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["object", "behavior"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["object", "class"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["object", "metaclass"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["scalar"],
        kind: ModuleKind::Package,
        children: &["number", "string", "bool", "symbol"],
    },
    BuiltinNodeSpec {
        path: &["scalar", "number"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["scalar", "string"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["scalar", "bool"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["scalar", "symbol"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["callable"],
        kind: ModuleKind::Package,
        children: &["function", "closure", "method", "family"],
    },
    BuiltinNodeSpec {
        path: &["callable", "function"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["callable", "closure"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["callable", "method"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["callable", "family"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["option"],
        kind: ModuleKind::Package,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["collections"],
        kind: ModuleKind::Package,
        children: &["iterable", "list", "map", "set", "tuple", "record", "range", "bytes"],
    },
    BuiltinNodeSpec {
        path: &["collections", "iterable"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["collections", "list"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["collections", "map"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["collections", "set"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["collections", "tuple"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["collections", "record"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["collections", "range"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["collections", "bytes"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["errors"],
        kind: ModuleKind::Package,
        children: &["error", "argument", "indexing", "contracts"],
    },
    BuiltinNodeSpec {
        path: &["errors", "error"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["errors", "argument"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["errors", "indexing"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["errors", "contracts"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection"],
        kind: ModuleKind::Package,
        children: &["module", "package_object", "project", "selector", "message", "attribute"],
    },
    BuiltinNodeSpec {
        path: &["reflection", "module"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "package_object"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "project"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "selector"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "message"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "attribute"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["concurrency"],
        kind: ModuleKind::Package,
        children: &["fiber"],
    },
    BuiltinNodeSpec {
        path: &["concurrency", "fiber"],
        kind: ModuleKind::Module,
        children: &[],
    },
];

const STD_NODES: &[BuiltinNodeSpec] = &[
    BuiltinNodeSpec {
        path: &[],
        kind: ModuleKind::ProjectRoot,
        children: &["io", "fs", "path", "text", "regex", "json", "math", "random", "time", "process", "net", "concurrent", "testing"],
    },
    BuiltinNodeSpec {
        path: &["json"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["io"],
        kind: ModuleKind::Package,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["fs"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["path"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["text"],
        kind: ModuleKind::Package,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["regex"],
        kind: ModuleKind::Package,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["math"],
        kind: ModuleKind::Package,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["random"],
        kind: ModuleKind::Package,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["time"],
        kind: ModuleKind::Package,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["process"],
        kind: ModuleKind::Package,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["net"],
        kind: ModuleKind::Package,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["concurrent"],
        kind: ModuleKind::Package,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["testing"],
        kind: ModuleKind::Package,
        children: &[],
    },
];

/// Toolchain-owned source/interface authority for one builtin project.
#[derive(Clone, Copy, Debug)]
pub struct BuiltinProjectSourceProvider {
    builtin: BuiltinProject,
}

impl BuiltinProjectSourceProvider {
    pub const fn new(builtin: BuiltinProject) -> Self {
        Self { builtin }
    }

    /// Canonical virtual identity used by diagnostics/LSP for builtin documents.
    pub fn source_id(&self, id: &ModuleId) -> Result<SourceId, ModuleLoadError> {
        self.validate_id(id)?;
        let path = id.path.components().iter().map(|part| part.as_str()).collect::<Vec<_>>().join("/");
        if path.is_empty() {
            Ok(SourceId(format!("phalcom://{}/", self.builtin).into()))
        } else {
            Ok(SourceId(format!("phalcom://{}/{path}", self.builtin).into()))
        }
    }

    /// Loads the immutable public interface of a builtin node.
    pub fn load_interface(&self, id: &ModuleId) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
        self.validate_id(id)?;
        let node = self.node_spec(&id.path).ok_or_else(|| {
            ModuleResolutionError::ModuleNotFound(format!("builtin module {id} is not part of the {} project graph", self.builtin))
        })?;

        let mut declarations = BTreeMap::new();
        let mut exports = BTreeMap::new();
        if self.builtin == BuiltinProject::Universe && id.path.is_root() {
            for binding in phalcom_native_meta::UNIVERSE_BINDINGS.iter().filter(|binding| binding.exported) {
                let range = SourceRange::default();
                declarations.insert(
                    binding.name.to_string(),
                    DeclarationSurface {
                        name: binding.name.to_string(),
                        is_const: true,
                        range,
                    },
                );
                exports.insert(
                    binding.name.to_string(),
                    ExportSurface {
                        exported_name: binding.name.to_string(),
                        internal_name: binding.name.to_string(),
                        target: UnlinkedExportTarget::Local(binding.name.to_string()),
                        range,
                    },
                );
            }
        }

        let exposed_children = node
            .children
            .iter()
            .map(|name| ModuleComponent::from_identifier(name).expect("builtin graph components are canonical"))
            .collect::<BTreeSet<_>>();

        Ok(UnlinkedModuleInterface {
            id: id.clone(),
            kind: node.kind,
            declarations,
            exports,
            imports: Vec::new(),
            exposed_children,
            metadata: ModuleMetadata::default(),
        })
    }

    /// Returns whether a canonical module path exists in this builtin graph.
    pub fn contains(&self, path: &ModulePath) -> bool {
        self.node_spec(path).is_some()
    }

    /// Returns the source-unit kind for a canonical builtin path.
    pub fn kind(&self, path: &ModulePath) -> Option<ModuleKind> {
        self.node_spec(path).map(|node| node.kind)
    }

    fn node_spec(&self, path: &ModulePath) -> Option<&'static BuiltinNodeSpec> {
        let expected = path.components().iter().map(|component| component.as_str()).collect::<Vec<_>>();
        self.nodes().iter().find(|node| node.path == expected.as_slice())
    }

    fn nodes(&self) -> &'static [BuiltinNodeSpec] {
        match self.builtin {
            BuiltinProject::Universe => UNIVERSE_NODES,
            BuiltinProject::Std => STD_NODES,
        }
    }

    fn validate_id(&self, id: &ModuleId) -> Result<(), ModuleLoadError> {
        if id.project != ProjectIdentity::Builtin(self.builtin) {
            return Err(ModuleResolutionError::ModuleNotFound(format!("{id} does not belong to builtin project {}", self.builtin)).into());
        }
        Ok(())
    }
}
