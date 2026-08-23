//! Provider-backed builtin project interfaces and canonical virtual source identity.

use crate::error::{ModuleLoadError, ModuleResolutionError};
use crate::identity::{BuiltinProject, ModuleId, ModulePath, ProjectIdentity, SourceId};
use crate::interface::UnlinkedModuleInterface;
use crate::source::ModuleKind;

#[derive(Clone, Copy, Debug)]
pub struct BuiltinNodeSpec {
    pub path: &'static [&'static str],
    pub kind: ModuleKind,
    #[allow(dead_code)]
    pub children: &'static [&'static str],
}

pub const UNIVERSE_NODES: &[BuiltinNodeSpec] = &[
    BuiltinNodeSpec {
        path: &[],
        kind: ModuleKind::Package,
        children: &["object", "scalar", "errors", "callable", "option", "concurrency", "collections", "reflection"],
    },
    BuiltinNodeSpec {
        path: &["object"],
        kind: ModuleKind::Package,
        children: &["object", "behavior", "class", "metaclass", "ellipsis", "ordering"],
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
        path: &["object", "ellipsis"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["object", "ordering"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["scalar"],
        kind: ModuleKind::Package,
        children: &["number", "nil", "string", "bool", "symbol"],
    },
    BuiltinNodeSpec {
        path: &["scalar", "number"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["scalar", "nil"],
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
        path: &["errors"],
        kind: ModuleKind::Package,
        children: &["error", "argument", "indexing", "contracts", "unsupported", "unimplemented"],
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
        path: &["errors", "unsupported"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["errors", "unimplemented"],
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
        children: &["option"],
    },
    BuiltinNodeSpec {
        path: &["option", "option"],
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
        path: &["reflection"],
        kind: ModuleKind::Package,
        children: &[
            "module",
            "package_object",
            "project",
            "project_manifest",
            "package_info",
            "package_author",
            "package_requirement",
            "resolved_project_dependency",
            "module_dependency",
            "export_table",
            "export",
            "export_kind",
            "child_module_table",
            "module_identity",
            "package_identity",
            "project_identity",
            "uri",
            "selector",
            "message",
            "attribute",
            "implementation",
            "typing",
        ],
    },
    BuiltinNodeSpec {
        path: &["reflection", "implementation"],
        kind: ModuleKind::Module,
        children: &[],
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
        path: &["reflection", "project_manifest"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "package_info"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "package_author"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "package_requirement"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "resolved_project_dependency"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "module_dependency"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "export_table"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "export"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "export_kind"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "child_module_table"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "module_identity"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "package_identity"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "project_identity"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "uri"],
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
        path: &["reflection", "typing"],
        kind: ModuleKind::Package,
        children: &[
            "kind",
            "type_descriptor",
            "type_parameter",
            "generic_signature",
            "signature",
            "type_use",
            "result",
            "evidence",
            "context",
        ],
    },
    BuiltinNodeSpec {
        path: &["reflection", "typing", "kind"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "typing", "type_descriptor"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "typing", "type_parameter"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "typing", "generic_signature"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "typing", "signature"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "typing", "type_use"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "typing", "result"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "typing", "evidence"],
        kind: ModuleKind::Module,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["reflection", "typing", "context"],
        kind: ModuleKind::Module,
        children: &[],
    },
];

const STD_NODES: &[BuiltinNodeSpec] = &[
    BuiltinNodeSpec {
        path: &[],
        kind: ModuleKind::Package,
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
        kind: ModuleKind::Package,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["io"],
        kind: ModuleKind::Package,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["fs"],
        kind: ModuleKind::Package,
        children: &[],
    },
    BuiltinNodeSpec {
        path: &["path"],
        kind: ModuleKind::Package,
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

    /// Loads the immutable public interface of a builtin node derived from source + native overlay.
    pub fn load_interface(&self, id: &ModuleId) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
        self.validate_id(id)?;
        crate::builtin_interface::BuiltinInterfaceBuilder::build(self, id)
    }

    /// Loads the parsed module unit for a builtin node.
    pub fn load_parsed(&self, id: &ModuleId) -> Result<std::sync::Arc<crate::source::ParsedModuleUnit>, ModuleLoadError> {
        self.validate_id(id)?;
        crate::builtin_interface::BuiltinInterfaceBuilder::load_parsed(self, id)
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
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "object" && m.as_str() == "ellipsis" => {
                include_str!("../../phalcom-core/core/universe/src/object/ellipsis.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "object" && m.as_str() == "ordering" => {
                include_str!("../../phalcom-core/core/universe/src/object/ordering.ph")
            }
            (BuiltinProject::Universe, [c]) if c.as_str() == "scalar" => include_str!("../../phalcom-core/core/universe/src/scalar/package.ph"),
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "scalar" && m.as_str() == "number" => {
                include_str!("../../phalcom-core/core/universe/src/scalar/number.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "scalar" && m.as_str() == "nil" => {
                include_str!("../../phalcom-core/core/universe/src/scalar/nil.ph")
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
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "option" && m.as_str() == "option" => {
                include_str!("../../phalcom-core/core/universe/src/option/option.ph")
            }
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
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "errors" && m.as_str() == "unsupported" => {
                include_str!("../../phalcom-core/core/universe/src/errors/unsupported.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "errors" && m.as_str() == "unimplemented" => {
                include_str!("../../phalcom-core/core/universe/src/errors/unimplemented.ph")
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
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "project_manifest" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/project-manifest.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "package_info" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/package-info.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "package_author" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/package-author.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "package_requirement" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/package-requirement.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "resolved_project_dependency" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/resolved-project-dependency.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "module_dependency" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/module-dependency.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "export_table" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/export-table.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "export" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/export.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "export_kind" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/export-kind.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "child_module_table" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/child-module-table.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "module_identity" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/module-identity.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "package_identity" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/package-identity.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "project_identity" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/project-identity.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "uri" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/uri.ph")
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
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "implementation" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/implementation.ph")
            }
            (BuiltinProject::Universe, [c, m]) if c.as_str() == "reflection" && m.as_str() == "typing" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/typing/package.ph")
            }
            (BuiltinProject::Universe, [c, m, child]) if c.as_str() == "reflection" && m.as_str() == "typing" => match child.as_str() {
                "kind" => include_str!("../../phalcom-core/core/universe/src/reflection/typing/kind.ph"),
                "type_descriptor" => include_str!("../../phalcom-core/core/universe/src/reflection/typing/type-descriptor.ph"),
                "type_parameter" => include_str!("../../phalcom-core/core/universe/src/reflection/typing/type-parameter.ph"),
                "generic_signature" => include_str!("../../phalcom-core/core/universe/src/reflection/typing/generic-signature.ph"),
                "signature" => include_str!("../../phalcom-core/core/universe/src/reflection/typing/signature.ph"),
                "type_use" => include_str!("../../phalcom-core/core/universe/src/reflection/typing/type-use.ph"),
                "result" => include_str!("../../phalcom-core/core/universe/src/reflection/typing/result.ph"),
                "evidence" => include_str!("../../phalcom-core/core/universe/src/reflection/typing/evidence.ph"),
                "context" => include_str!("../../phalcom-core/core/universe/src/reflection/typing/context.ph"),
                _ => return Err(ModuleResolutionError::ModuleNotFound(format!("builtin source for {id} not found")).into()),
            },
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

    pub fn nodes(&self) -> &'static [BuiltinNodeSpec] {
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
