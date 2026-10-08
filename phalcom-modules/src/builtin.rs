//! Provider-backed builtin project interfaces and canonical virtual source identity.

use crate::error::{ModuleLoadError, ModuleResolutionError};
use crate::identity::{ModuleId, ModulePath, SourceId, universe_module_uri};
use crate::interface::UnlinkedModuleInterface;
use crate::source::ModuleKind;
#[cfg(feature = "test-support")]
use std::collections::BTreeMap;
#[cfg(feature = "test-support")]
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct UniverseNodeSpec {
    pub path: &'static [&'static str],
    pub kind: ModuleKind,
    #[allow(dead_code)]
    pub children: &'static [&'static str],
}

pub const UNIVERSE_NODES: &[UniverseNodeSpec] = &[
    UniverseNodeSpec {
        path: &[],
        kind: ModuleKind::Package,
        children: &[
            "object",
            "scalar",
            "errors",
            "callable",
            "option",
            "concurrency",
            "collections",
            "reflection",
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
    UniverseNodeSpec {
        path: &["io"],
        kind: ModuleKind::Package,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["fs"],
        kind: ModuleKind::Package,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["path"],
        kind: ModuleKind::Package,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["text"],
        kind: ModuleKind::Package,
        children: &["substring", "traits", "patterns", "split", "lines", "writer", "builder", "cursor", "line_map", "errors"],
    },
    UniverseNodeSpec {
        path: &["text", "substring"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["text", "traits"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["text", "patterns"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["text", "split"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["text", "lines"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["text", "writer"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["text", "builder"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["text", "cursor"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["text", "line_map"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["text", "errors"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["regex"],
        kind: ModuleKind::Package,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["json"],
        kind: ModuleKind::Package,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["math"],
        kind: ModuleKind::Package,
        children: &["float_kernel", "integer", "reduction", "numerical_errors", "special"],
    },
    UniverseNodeSpec {
        path: &["math", "float_kernel"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["math", "integer"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["math", "reduction"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["math", "numerical_errors"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["math", "special"],
        kind: ModuleKind::Package,
        children: &["kernel"],
    },
    UniverseNodeSpec {
        path: &["math", "special", "kernel"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["random"],
        kind: ModuleKind::Package,
        children: &["generator", "xoshiro256pp", "distribution", "uniform_int", "bernoulli", "secure"],
    },
    UniverseNodeSpec {
        path: &["random", "generator"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["random", "xoshiro256pp"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["random", "distribution"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["random", "uniform_int"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["random", "bernoulli"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["random", "secure"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time"],
        kind: ModuleKind::Package,
        children: &[
            "clock",
            "duration",
            "timestamp",
            "offset",
            "calendar",
            "timezone",
            "zone_transition",
            "zoned",
            "resolution",
            "testing",
        ],
    },
    UniverseNodeSpec {
        path: &["time", "clock"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "duration"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "timestamp"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "offset"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "timezone"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "zone_transition"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "zoned"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "resolution"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "testing"],
        kind: ModuleKind::Package,
        children: &["clocks"],
    },
    UniverseNodeSpec {
        path: &["time", "testing", "clocks"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "calendar"],
        kind: ModuleKind::Package,
        children: &["month", "weekday", "period", "date", "time", "datetime", "partial"],
    },
    UniverseNodeSpec {
        path: &["time", "calendar", "month"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "calendar", "weekday"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "calendar", "period"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "calendar", "date"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "calendar", "time"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "calendar", "datetime"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["time", "calendar", "partial"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["process"],
        kind: ModuleKind::Package,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["net"],
        kind: ModuleKind::Package,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["concurrent"],
        kind: ModuleKind::Package,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["testing"],
        kind: ModuleKind::Package,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["object"],
        kind: ModuleKind::Package,
        children: &["object", "behavior", "class", "metaclass", "ellipsis", "ordering"],
    },
    UniverseNodeSpec {
        path: &["object", "object"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["object", "behavior"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["object", "class"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["object", "metaclass"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["object", "ellipsis"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["object", "ordering"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["scalar"],
        kind: ModuleKind::Package,
        children: &["number", "nil", "string", "string_index", "char", "bool", "symbol"],
    },
    UniverseNodeSpec {
        path: &["scalar", "number"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["scalar", "nil"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["scalar", "string"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["scalar", "string_index"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["scalar", "char"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["scalar", "bool"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["scalar", "symbol"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["errors"],
        kind: ModuleKind::Package,
        children: &[
            "error",
            "result",
            "argument",
            "iteration",
            "indexing",
            "contracts",
            "unsupported",
            "unimplemented",
        ],
    },
    UniverseNodeSpec {
        path: &["errors", "error"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["errors", "result"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["errors", "argument"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["errors", "iteration"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["errors", "indexing"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["errors", "contracts"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["errors", "unsupported"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["errors", "unimplemented"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["callable"],
        kind: ModuleKind::Package,
        children: &["function", "closure", "method", "family"],
    },
    UniverseNodeSpec {
        path: &["callable", "function"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["callable", "closure"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["callable", "method"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["callable", "family"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["option"],
        kind: ModuleKind::Package,
        children: &["option", "unit"],
    },
    UniverseNodeSpec {
        path: &["option", "option"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["option", "unit"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["concurrency"],
        kind: ModuleKind::Package,
        children: &["fiber"],
    },
    UniverseNodeSpec {
        path: &["concurrency", "fiber"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["collections"],
        kind: ModuleKind::Package,
        children: &["collection", "iterator_laws", "list", "map", "set", "tuple", "record", "pack", "range", "bytes"],
    },
    UniverseNodeSpec {
        path: &["collections", "collection"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["collections", "iterator_laws"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["collections", "list"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["collections", "map"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["collections", "set"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["collections", "tuple"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["collections", "record"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["collections", "pack"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["collections", "range"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["collections", "bytes"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
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
    UniverseNodeSpec {
        path: &["reflection", "implementation"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "module"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "package_object"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "project"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "project_manifest"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "package_info"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "package_author"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "package_requirement"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "resolved_project_dependency"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "module_dependency"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "export_table"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "export"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "export_kind"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "child_module_table"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "module_identity"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "package_identity"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "project_identity"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "uri"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "selector"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "message"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "attribute"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
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
    UniverseNodeSpec {
        path: &["reflection", "typing", "kind"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "typing", "type_descriptor"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "typing", "type_parameter"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "typing", "generic_signature"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "typing", "signature"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "typing", "type_use"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "typing", "result"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "typing", "evidence"],
        kind: ModuleKind::Module,
        children: &[],
    },
    UniverseNodeSpec {
        path: &["reflection", "typing", "context"],
        kind: ModuleKind::Module,
        children: &[],
    },
];

/// Toolchain-owned source/interface authority for the Universe package.
#[derive(Clone, Debug, Default)]
pub struct UniverseSourceProvider {
    #[cfg(feature = "test-support")]
    test_source_overrides: Arc<BTreeMap<ModuleId, Arc<str>>>,
}

impl UniverseSourceProvider {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a test-only provider that overrides source text for one canonical
    /// Universe module while preserving its existing module/source identity.
    ///
    /// This is a fixture seam, not a production source customization API.
    #[cfg(feature = "test-support")]
    pub fn with_test_source_override(module: ModuleId, text: Arc<str>) -> Result<Self, ModuleLoadError> {
        let provider = Self {
            test_source_overrides: Arc::new(BTreeMap::from([(module.clone(), text)])),
        };
        provider.validate_id(&module)?;
        Ok(provider)
    }

    pub fn is_canonical(&self) -> bool {
        !self.has_test_source_overrides()
    }

    pub(crate) fn has_test_source_overrides(&self) -> bool {
        #[cfg(feature = "test-support")]
        {
            !self.test_source_overrides.is_empty()
        }
        #[cfg(not(feature = "test-support"))]
        {
            false
        }
    }

    /// Canonical virtual identity used by diagnostics/LSP for builtin documents.
    pub fn source_id(&self, id: &ModuleId) -> Result<SourceId, ModuleLoadError> {
        self.validate_id(id)?;
        let uri = universe_module_uri(id).ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("{id} is not a Universe module")))?;
        Ok(SourceId(uri.into()))
    }

    /// Loads immutable public interface of builtin node derived from canonical source.
    pub fn load_interface(&self, id: &ModuleId) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
        self.validate_id(id)?;
        crate::builtin_interface::BuiltinInterfaceBuilder::build(self, id)
    }

    /// Loads the parsed module unit for a builtin node.
    pub fn load_parsed(&self, id: &ModuleId) -> Result<std::sync::Arc<crate::source::ParsedModuleUnit>, ModuleLoadError> {
        self.validate_id(id)?;
        crate::builtin_interface::BuiltinInterfaceBuilder::load_parsed(self, id)
    }

    /// Returns the embedded source text for a canonical Universe module.
    pub fn source_text(&self, id: &ModuleId) -> Result<std::sync::Arc<str>, ModuleLoadError> {
        use std::sync::Arc;
        self.validate_id(id)?;
        let _node = self
            .node_spec(&id.path)
            .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("Universe module {id} is not part of the Universe graph")))?;

        #[cfg(feature = "test-support")]
        if let Some(text) = self.test_source_overrides.get(id) {
            return Ok(text.clone());
        }

        let components = id.path.components();
        let content = match components {
            [] => include_str!("../../phalcom-core/core/universe/src/package.ph"),
            [c] if c.as_str() == "io" => include_str!("../../phalcom-core/core/universe/src/io/package.ph"),
            [c] if c.as_str() == "fs" => include_str!("../../phalcom-core/core/universe/src/fs/package.ph"),
            [c] if c.as_str() == "path" => include_str!("../../phalcom-core/core/universe/src/path/package.ph"),
            [c] if c.as_str() == "text" => include_str!("../../phalcom-core/core/universe/src/text/package.ph"),
            [c, m] if c.as_str() == "text" && m.as_str() == "substring" => {
                include_str!("../../phalcom-core/core/universe/src/text/substring.ph")
            }
            [c, m] if c.as_str() == "text" && m.as_str() == "traits" => {
                include_str!("../../phalcom-core/core/universe/src/text/traits.ph")
            }
            [c, m] if c.as_str() == "text" && m.as_str() == "patterns" => {
                include_str!("../../phalcom-core/core/universe/src/text/patterns.ph")
            }
            [c, m] if c.as_str() == "text" && m.as_str() == "split" => {
                include_str!("../../phalcom-core/core/universe/src/text/split.ph")
            }
            [c, m] if c.as_str() == "text" && m.as_str() == "lines" => {
                include_str!("../../phalcom-core/core/universe/src/text/lines.ph")
            }
            [c, m] if c.as_str() == "text" && m.as_str() == "writer" => {
                include_str!("../../phalcom-core/core/universe/src/text/writer.ph")
            }
            [c, m] if c.as_str() == "text" && m.as_str() == "builder" => {
                include_str!("../../phalcom-core/core/universe/src/text/builder.ph")
            }
            [c, m] if c.as_str() == "text" && m.as_str() == "cursor" => {
                include_str!("../../phalcom-core/core/universe/src/text/cursor.ph")
            }
            [c, m] if c.as_str() == "text" && m.as_str() == "line_map" => {
                include_str!("../../phalcom-core/core/universe/src/text/line_map.ph")
            }
            [c, m] if c.as_str() == "text" && m.as_str() == "errors" => {
                include_str!("../../phalcom-core/core/universe/src/text/errors.ph")
            }
            [c] if c.as_str() == "regex" => include_str!("../../phalcom-core/core/universe/src/regex/package.ph"),
            [c] if c.as_str() == "json" => include_str!("../../phalcom-core/core/universe/src/json/package.ph"),
            [c] if c.as_str() == "math" => include_str!("../../phalcom-core/core/universe/src/math/package.ph"),
            [c, m] if c.as_str() == "math" && m.as_str() == "float_kernel" => {
                include_str!("../../phalcom-core/core/universe/src/math/float_kernel.ph")
            }
            [c, m] if c.as_str() == "math" && m.as_str() == "integer" => {
                include_str!("../../phalcom-core/core/universe/src/math/integer.ph")
            }
            [c, m] if c.as_str() == "math" && m.as_str() == "reduction" => {
                include_str!("../../phalcom-core/core/universe/src/math/reduction.ph")
            }
            [c, m] if c.as_str() == "math" && m.as_str() == "numerical_errors" => {
                include_str!("../../phalcom-core/core/universe/src/math/numerical_errors.ph")
            }
            [c, m] if c.as_str() == "math" && m.as_str() == "special" => {
                include_str!("../../phalcom-core/core/universe/src/math/special/package.ph")
            }
            [c, m, k] if c.as_str() == "math" && m.as_str() == "special" && k.as_str() == "kernel" => {
                include_str!("../../phalcom-core/core/universe/src/math/special/kernel.ph")
            }
            [c] if c.as_str() == "random" => include_str!("../../phalcom-core/core/universe/src/random/package.ph"),
            [c, m] if c.as_str() == "random" && m.as_str() == "generator" => include_str!("../../phalcom-core/core/universe/src/random/generator.ph"),
            [c, m] if c.as_str() == "random" && m.as_str() == "xoshiro256pp" => include_str!("../../phalcom-core/core/universe/src/random/xoshiro256pp.ph"),
            [c, m] if c.as_str() == "random" && m.as_str() == "distribution" => include_str!("../../phalcom-core/core/universe/src/random/distribution.ph"),
            [c, m] if c.as_str() == "random" && m.as_str() == "uniform_int" => include_str!("../../phalcom-core/core/universe/src/random/uniform_int.ph"),
            [c, m] if c.as_str() == "random" && m.as_str() == "bernoulli" => include_str!("../../phalcom-core/core/universe/src/random/bernoulli.ph"),
            [c, m] if c.as_str() == "random" && m.as_str() == "secure" => include_str!("../../phalcom-core/core/universe/src/random/secure.ph"),
            [c] if c.as_str() == "time" => include_str!("../../phalcom-core/core/universe/src/time/package.ph"),
            [c, m] if c.as_str() == "time" && m.as_str() == "clock" => {
                include_str!("../../phalcom-core/core/universe/src/time/clock.ph")
            }
            [c, m] if c.as_str() == "time" && m.as_str() == "duration" => {
                include_str!("../../phalcom-core/core/universe/src/time/duration.ph")
            }
            [c, m] if c.as_str() == "time" && m.as_str() == "timestamp" => {
                include_str!("../../phalcom-core/core/universe/src/time/timestamp.ph")
            }
            [c, m] if c.as_str() == "time" && m.as_str() == "offset" => {
                include_str!("../../phalcom-core/core/universe/src/time/offset.ph")
            }
            [c, m] if c.as_str() == "time" && m.as_str() == "timezone" => {
                include_str!("../../phalcom-core/core/universe/src/time/timezone.ph")
            }
            [c, m] if c.as_str() == "time" && m.as_str() == "zone_transition" => {
                include_str!("../../phalcom-core/core/universe/src/time/zone_transition.ph")
            }
            [c, m] if c.as_str() == "time" && m.as_str() == "zoned" => {
                include_str!("../../phalcom-core/core/universe/src/time/zoned.ph")
            }
            [c, m] if c.as_str() == "time" && m.as_str() == "resolution" => {
                include_str!("../../phalcom-core/core/universe/src/time/resolution.ph")
            }
            [c, p, m] if c.as_str() == "time" && p.as_str() == "testing" && m.as_str() == "clocks" => {
                include_str!("../../phalcom-core/core/universe/src/time/testing/clocks.ph")
            }
            [c, p] if c.as_str() == "time" && p.as_str() == "testing" => {
                include_str!("../../phalcom-core/core/universe/src/time/testing/package.ph")
            }
            [c, p] if c.as_str() == "time" && p.as_str() == "calendar" => {
                include_str!("../../phalcom-core/core/universe/src/time/calendar/package.ph")
            }
            [c, p, m] if c.as_str() == "time" && p.as_str() == "calendar" && m.as_str() == "month" => {
                include_str!("../../phalcom-core/core/universe/src/time/calendar/month.ph")
            }
            [c, p, m] if c.as_str() == "time" && p.as_str() == "calendar" && m.as_str() == "weekday" => {
                include_str!("../../phalcom-core/core/universe/src/time/calendar/weekday.ph")
            }
            [c, p, m] if c.as_str() == "time" && p.as_str() == "calendar" && m.as_str() == "period" => {
                include_str!("../../phalcom-core/core/universe/src/time/calendar/period.ph")
            }
            [c, p, m] if c.as_str() == "time" && p.as_str() == "calendar" && m.as_str() == "date" => {
                include_str!("../../phalcom-core/core/universe/src/time/calendar/date.ph")
            }
            [c, p, m] if c.as_str() == "time" && p.as_str() == "calendar" && m.as_str() == "time" => {
                include_str!("../../phalcom-core/core/universe/src/time/calendar/time.ph")
            }
            [c, p, m] if c.as_str() == "time" && p.as_str() == "calendar" && m.as_str() == "datetime" => {
                include_str!("../../phalcom-core/core/universe/src/time/calendar/datetime.ph")
            }
            [c, p, m] if c.as_str() == "time" && p.as_str() == "calendar" && m.as_str() == "partial" => {
                include_str!("../../phalcom-core/core/universe/src/time/calendar/partial.ph")
            }
            [c] if c.as_str() == "process" => include_str!("../../phalcom-core/core/universe/src/process/package.ph"),
            [c] if c.as_str() == "net" => include_str!("../../phalcom-core/core/universe/src/net/package.ph"),
            [c] if c.as_str() == "concurrent" => include_str!("../../phalcom-core/core/universe/src/concurrent/package.ph"),
            [c] if c.as_str() == "testing" => include_str!("../../phalcom-core/core/universe/src/testing/package.ph"),
            [c] if c.as_str() == "object" => include_str!("../../phalcom-core/core/universe/src/object/package.ph"),
            [c, m] if c.as_str() == "object" && m.as_str() == "object" => {
                include_str!("../../phalcom-core/core/universe/src/object/object.ph")
            }
            [c, m] if c.as_str() == "object" && m.as_str() == "behavior" => {
                include_str!("../../phalcom-core/core/universe/src/object/behavior.ph")
            }
            [c, m] if c.as_str() == "object" && m.as_str() == "class" => {
                include_str!("../../phalcom-core/core/universe/src/object/class.ph")
            }
            [c, m] if c.as_str() == "object" && m.as_str() == "metaclass" => {
                include_str!("../../phalcom-core/core/universe/src/object/metaclass.ph")
            }
            [c, m] if c.as_str() == "object" && m.as_str() == "ellipsis" => {
                include_str!("../../phalcom-core/core/universe/src/object/ellipsis.ph")
            }
            [c, m] if c.as_str() == "object" && m.as_str() == "ordering" => {
                include_str!("../../phalcom-core/core/universe/src/object/ordering.ph")
            }
            [c] if c.as_str() == "scalar" => include_str!("../../phalcom-core/core/universe/src/scalar/package.ph"),
            [c, m] if c.as_str() == "scalar" && m.as_str() == "number" => {
                include_str!("../../phalcom-core/core/universe/src/scalar/number.ph")
            }
            [c, m] if c.as_str() == "scalar" && m.as_str() == "nil" => {
                include_str!("../../phalcom-core/core/universe/src/scalar/nil.ph")
            }
            [c, m] if c.as_str() == "scalar" && m.as_str() == "string" => {
                include_str!("../../phalcom-core/core/universe/src/scalar/string.ph")
            }
            [c, m] if c.as_str() == "scalar" && m.as_str() == "string_index" => {
                include_str!("../../phalcom-core/core/universe/src/scalar/string_index.ph")
            }
            [c, m] if c.as_str() == "scalar" && m.as_str() == "char" => {
                include_str!("../../phalcom-core/core/universe/src/scalar/char.ph")
            }
            [c, m] if c.as_str() == "scalar" && m.as_str() == "bool" => {
                include_str!("../../phalcom-core/core/universe/src/scalar/bool.ph")
            }
            [c, m] if c.as_str() == "scalar" && m.as_str() == "symbol" => {
                include_str!("../../phalcom-core/core/universe/src/scalar/symbol.ph")
            }
            [c] if c.as_str() == "callable" => include_str!("../../phalcom-core/core/universe/src/callable/package.ph"),
            [c, m] if c.as_str() == "callable" && m.as_str() == "function" => {
                include_str!("../../phalcom-core/core/universe/src/callable/function.ph")
            }
            [c, m] if c.as_str() == "callable" && m.as_str() == "closure" => {
                include_str!("../../phalcom-core/core/universe/src/callable/closure.ph")
            }
            [c, m] if c.as_str() == "callable" && m.as_str() == "method" => {
                include_str!("../../phalcom-core/core/universe/src/callable/method.ph")
            }
            [c, m] if c.as_str() == "callable" && m.as_str() == "family" => {
                include_str!("../../phalcom-core/core/universe/src/callable/family.ph")
            }
            [c] if c.as_str() == "option" => include_str!("../../phalcom-core/core/universe/src/option/package.ph"),
            [c, m] if c.as_str() == "option" && m.as_str() == "option" => {
                include_str!("../../phalcom-core/core/universe/src/option/option.ph")
            }
            [c, m] if c.as_str() == "option" && m.as_str() == "unit" => {
                include_str!("../../phalcom-core/core/universe/src/option/unit.ph")
            }
            [c] if c.as_str() == "collections" => include_str!("../../phalcom-core/core/universe/src/collections/package.ph"),
            [c, m] if c.as_str() == "collections" && m.as_str() == "collection" => {
                include_str!("../../phalcom-core/core/universe/src/collections/collection.ph")
            }
            [c, m] if c.as_str() == "collections" && m.as_str() == "iterator_laws" => {
                include_str!("../../phalcom-core/core/universe/src/collections/iterator_laws.ph")
            }
            [c, m] if c.as_str() == "collections" && m.as_str() == "list" => {
                include_str!("../../phalcom-core/core/universe/src/collections/list.ph")
            }
            [c, m] if c.as_str() == "collections" && m.as_str() == "map" => {
                include_str!("../../phalcom-core/core/universe/src/collections/map.ph")
            }
            [c, m] if c.as_str() == "collections" && m.as_str() == "set" => {
                include_str!("../../phalcom-core/core/universe/src/collections/set.ph")
            }
            [c, m] if c.as_str() == "collections" && m.as_str() == "tuple" => {
                include_str!("../../phalcom-core/core/universe/src/collections/tuple.ph")
            }
            [c, m] if c.as_str() == "collections" && m.as_str() == "record" => {
                include_str!("../../phalcom-core/core/universe/src/collections/record.ph")
            }
            [c, m] if c.as_str() == "collections" && m.as_str() == "pack" => {
                include_str!("../../phalcom-core/core/universe/src/collections/pack.ph")
            }
            [c, m] if c.as_str() == "collections" && m.as_str() == "range" => {
                include_str!("../../phalcom-core/core/universe/src/collections/range.ph")
            }
            [c, m] if c.as_str() == "collections" && m.as_str() == "bytes" => {
                include_str!("../../phalcom-core/core/universe/src/collections/bytes.ph")
            }
            [c] if c.as_str() == "errors" => include_str!("../../phalcom-core/core/universe/src/errors/package.ph"),
            [c, m] if c.as_str() == "errors" && m.as_str() == "error" => {
                include_str!("../../phalcom-core/core/universe/src/errors/error.ph")
            }
            [c, m] if c.as_str() == "errors" && m.as_str() == "result" => {
                include_str!("../../phalcom-core/core/universe/src/errors/result.ph")
            }
            [c, m] if c.as_str() == "errors" && m.as_str() == "argument" => {
                include_str!("../../phalcom-core/core/universe/src/errors/argument.ph")
            }
            [c, m] if c.as_str() == "errors" && m.as_str() == "iteration" => {
                include_str!("../../phalcom-core/core/universe/src/errors/iteration.ph")
            }
            [c, m] if c.as_str() == "errors" && m.as_str() == "indexing" => {
                include_str!("../../phalcom-core/core/universe/src/errors/indexing.ph")
            }
            [c, m] if c.as_str() == "errors" && m.as_str() == "contracts" => {
                include_str!("../../phalcom-core/core/universe/src/errors/contracts.ph")
            }
            [c, m] if c.as_str() == "errors" && m.as_str() == "unsupported" => {
                include_str!("../../phalcom-core/core/universe/src/errors/unsupported.ph")
            }
            [c, m] if c.as_str() == "errors" && m.as_str() == "unimplemented" => {
                include_str!("../../phalcom-core/core/universe/src/errors/unimplemented.ph")
            }
            [c] if c.as_str() == "reflection" => include_str!("../../phalcom-core/core/universe/src/reflection/package.ph"),
            [c, m] if c.as_str() == "reflection" && m.as_str() == "module" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/module.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "package_object" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/package-object.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "project" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/project.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "project_manifest" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/project-manifest.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "package_info" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/package-info.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "package_author" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/package-author.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "package_requirement" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/package-requirement.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "resolved_project_dependency" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/resolved-project-dependency.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "module_dependency" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/module-dependency.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "export_table" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/export-table.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "export" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/export.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "export_kind" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/export-kind.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "child_module_table" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/child-module-table.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "module_identity" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/module-identity.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "package_identity" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/package-identity.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "project_identity" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/project-identity.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "uri" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/uri.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "selector" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/selector.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "message" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/message.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "attribute" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/attribute.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "implementation" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/implementation.ph")
            }
            [c, m] if c.as_str() == "reflection" && m.as_str() == "typing" => {
                include_str!("../../phalcom-core/core/universe/src/reflection/typing/package.ph")
            }
            [c, m, child] if c.as_str() == "reflection" && m.as_str() == "typing" => match child.as_str() {
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
            [c] if c.as_str() == "concurrency" => include_str!("../../phalcom-core/core/universe/src/concurrency/package.ph"),
            [c, m] if c.as_str() == "concurrency" && m.as_str() == "fiber" => {
                include_str!("../../phalcom-core/core/universe/src/concurrency/fiber.ph")
            }
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

    fn node_spec(&self, path: &ModulePath) -> Option<&'static UniverseNodeSpec> {
        let expected = path.components().iter().map(|component| component.as_str()).collect::<Vec<_>>();
        self.nodes().iter().find(|node| node.path == expected.as_slice())
    }

    pub fn nodes(&self) -> &'static [UniverseNodeSpec] {
        UNIVERSE_NODES
    }

    fn validate_id(&self, id: &ModuleId) -> Result<(), ModuleLoadError> {
        if !id.project.is_universe() {
            return Err(ModuleResolutionError::ModuleNotFound(format!("{id} does not belong to Universe package")).into());
        }
        Ok(())
    }
}
