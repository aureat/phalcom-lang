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
        children: &[
            "io",
            "fs",
            "path",
            "text",
            "regex",
            "json",
            "math",
            "random",
            "time",
            "process",
            "net",
            "concurrent",
            "testing",
        ],
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
        let node = self
            .node_spec(&id.path)
            .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("builtin module {id} is not part of the {} project graph", self.builtin)))?;

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
        } else if self.builtin == BuiltinProject::Universe
            && id.path.components().len() == 2
            && id.path.components()[0].as_str() == "reflection"
            && id.path.components()[1].as_str() == "selector"
        {
            let range = SourceRange::default();
            declarations.insert(
                "Selector".to_string(),
                DeclarationSurface {
                    name: "Selector".to_string(),
                    is_const: true,
                    range,
                },
            );
            exports.insert(
                "Selector".to_string(),
                ExportSurface {
                    exported_name: "Selector".to_string(),
                    internal_name: "Selector".to_string(),
                    target: UnlinkedExportTarget::Local("Selector".to_string()),
                    range,
                },
            );
        } else if self.builtin == BuiltinProject::Std && id.path.components().len() == 1 && id.path.components()[0].as_str() == "json" {
            for name in ["parse", "stringify"] {
                let range = SourceRange::default();
                declarations.insert(
                    name.to_string(),
                    DeclarationSurface {
                        name: name.to_string(),
                        is_const: true,
                        range,
                    },
                );
                exports.insert(
                    name.to_string(),
                    ExportSurface {
                        exported_name: name.to_string(),
                        internal_name: name.to_string(),
                        target: UnlinkedExportTarget::Local(name.to_string()),
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

    /// Returns the embedded source text for a canonical builtin module.
    pub fn source_text(&self, id: &ModuleId) -> Result<std::sync::Arc<str>, ModuleLoadError> {
        use std::sync::Arc;
        self.validate_id(id)?;
        let _node = self
            .node_spec(&id.path)
            .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("builtin module {id} is not part of the {} project graph", self.builtin)))?;

        let components = id.path.components();
        let content = match (self.builtin, components) {
            (BuiltinProject::Universe, []) => include_str!("../../phalcom-core/core/universe/src/package.ph"),
            (BuiltinProject::Universe, [c]) if c.as_str() == "object" => include_str!("../../phalcom-core/core/universe/src/object/package.ph"),
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "object" && m.as_str() == "object" => {
                include_str!("../../phalcom-core/core/universe/src/object/object.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "object" && m.as_str() == "behavior" => {
                include_str!("../../phalcom-core/core/universe/src/object/behavior.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "object" && m.as_str() == "class" => {
                include_str!("../../phalcom-core/core/universe/src/object/class.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "object" && m.as_str() == "metaclass" => {
                include_str!("../../phalcom-core/core/universe/src/object/metaclass.ph")
            }
            (BuiltinProject::Universe, [c]) if c.as_str() == "scalar" => include_str!("../../phalcom-core/core/universe/src/scalar/package.ph"),
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "scalar" && m.as_str() == "number" => {
                include_str!("../../phalcom-core/core/universe/src/scalar/number.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "scalar" && m.as_str() == "string" => {
                include_str!("../../phalcom-core/core/universe/src/scalar/string.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "scalar" && m.as_str() == "bool" => {
                include_str!("../../phalcom-core/core/universe/src/scalar/bool.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "scalar" && m.as_str() == "symbol" => {
                include_str!("../../phalcom-core/core/universe/src/scalar/symbol.ph")
            }
            (BuiltinProject::Universe, [c]) if c.as_str() == "callable" => include_str!("../../phalcom-core/core/universe/src/callable/package.ph"),
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "callable" && m.as_str() == "function" => {
                include_str!("../../phalcom-core/core/universe/src/callable/function.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "callable" && m.as_str() == "closure" => {
                include_str!("../../phalcom-core/core/universe/src/callable/closure.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "callable" && m.as_str() == "method" => {
                include_str!("../../phalcom-core/core/universe/src/callable/method.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "callable" && m.as_str() == "family" => {
                include_str!("../../phalcom-core/core/universe/src/callable/family.ph")
            }
            (BuiltinProject::Universe, [c]) if c.as_str() == "option" => include_str!("../../phalcom-core/core/universe/src/option/package.ph"),
            (BuiltinProject::Universe, [c]) if c.as_str() == "collections" => include_str!("../../phalcom-core/core/universe/src/collections/package.ph"),
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "collections" && m.as_str() == "iterable" => {
                include_str!("../../phalcom-core/core/universe/src/collections/iterable.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "collections" && m.as_str() == "list" => {
                include_str!("../../phalcom-core/core/universe/src/collections/list.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "collections" && m.as_str() == "map" => {
                include_str!("../../phalcom-core/core/universe/src/collections/map.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "collections" && m.as_str() == "set" => {
                include_str!("../../phalcom-core/core/universe/src/collections/set.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "collections" && m.as_str() == "tuple" => {
                include_str!("../../phalcom-core/core/universe/src/collections/tuple.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "collections" && m.as_str() == "record" => {
                include_str!("../../phalcom-core/core/universe/src/collections/record.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "collections" && m.as_str() == "range" => {
                include_str!("../../phalcom-core/core/universe/src/collections/range.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "collections" && m.as_str() == "bytes" => {
                include_str!("../../phalcom-core/core/universe/src/collections/bytes.ph")
            }
            (BuiltinProject::Universe, [c]) if c.as_str() == "errors" => include_str!("../../phalcom-core/core/universe/src/errors/package.ph"),
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "errors" && m.as_str() == "error" => {
                include_str!("../../phalcom-core/core/universe/src/errors/error.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "errors" && m.as_str() == "argument" => {
                include_str!("../../phalcom-core/core/universe/src/errors/argument.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "errors" && m.as_str() == "indexing" => {
                include_str!("../../phalcom-core/core/universe/src/errors/indexing.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "errors" && m.as_str() == "contracts" => {
                include_str!("../../phalcom-core/core/universe/src/errors/contracts.ph")
            }
            (BuiltinProject::Universe, [c]) if c.as_str() == "reflection" => include_str!("../../phalcom-core/core/universe/src/reflection/package.ph"),
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "module" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/module.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "package_object" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/package-object.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "project" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/project.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "selector" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/selector.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "message" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/message.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "attribute" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/attribute.ph")
            }
            (BuiltinProject::Universe, [c]) if c.as_str() == "concurrency" => include_str!("../../phalcom-core/core/universe/src/concurrency/package.ph"),
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "concurrency" && m.as_str() == "fiber" => {
                include_str!("../../phalcom-core/core/universe/src/concurrency/fiber.ph")
            }
            (BuiltinProject::Std, []) => include_str!("../../phalcom-core/core/std/src/package.ph"),
            (BuiltinProject::Std, [c]) if c.as_str() == "json" => include_str!("../../phalcom-core/core/std/src/json/package.ph"),
            (BuiltinProject::Std, [c]) if c.as_str() == "io" => include_str!("../../phalcom-core/core/std/src/io/package.ph"),
            (BuiltinProject::Std, [c]) if c.as_str() == "fs" => include_str!("../../phalcom-core/core/std/src/fs/package.ph"),
            (BuiltinProject::Std, [c]) if c.as_str() == "path" => include_str!("../../phalcom-core/core/std/src/path/package.ph"),
            (BuiltinProject::Std, [c]) if c.as_str() == "text" => include_str!("../../phalcom-core/core/std/src/text/package.ph"),
            (BuiltinProject::Std, [c]) if c.as_str() == "regex" => include_str!("../../phalcom-core/core/std/src/regex/package.ph"),
            (BuiltinProject::Std, [c]) if c.as_str() == "math" => include_str!("../../phalcom-core/core/std/src/math/package.ph"),
            (BuiltinProject::Std, [c]) if c.as_str() == "random" => include_str!("../../phalcom-core/core/std/src/random/package.ph"),
            (BuiltinProject::Std, [c]) if c.as_str() == "time" => include_str!("../../phalcom-core/core/std/src/time/package.ph"),
            (BuiltinProject::Std, [c]) if c.as_str() == "process" => include_str!("../../phalcom-core/core/std/src/process/package.ph"),
            (BuiltinProject::Std, [c]) if c.as_str() == "net" => include_str!("../../phalcom-core/core/std/src/net/package.ph"),
            (BuiltinProject::Std, [c]) if c.as_str() == "concurrent" => include_str!("../../phalcom-core/core/std/src/concurrent/package.ph"),
            (BuiltinProject::Std, [c]) if c.as_str() == "testing" => include_str!("../../phalcom-core/core/std/src/testing/package.ph"),
            _ => return Err(ModuleResolutionError::ModuleNotFound(format!("builtin source for {id} not found")).into()),
        };
        Ok(Arc::from(content))
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
