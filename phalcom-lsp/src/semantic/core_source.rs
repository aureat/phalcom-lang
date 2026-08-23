//! Live source and native declarations for the semantic core module.
//!
//! Canonical universe source is used as the bundled semantic fallback.
//! Workspace and open-buffer copies replace it through
//! [`super::SemanticDb::update_core`].
//! Native members use the same structured surface consumed by completion;
//! opaque native returns deliberately carry no guessed value shape.

use super::ids::{ClassId, DispatchSide, ModuleId, CORE_MODULE_URI};
use super::surface::{build_module_surface, ClassSurface, MemberKind, MemberSurface, MemberVisibility, ModuleSurface};
use phalcom_ast::ast::{Program, RestMode};
use phalcom_ast::parser::Parse;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower_lsp::lsp_types::Url;

/// Bundled source fallback for the semantic core module.
pub const BUNDLED_CORE_SOURCE: &str = include_str!("../../../phalcom-core/core/universe/src/package.ph");

pub use phalcom_native_surface::NativeReturnShape;

/// Source location and text representation for the semantic core module.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreSource {
    /// Explicitly configured sysroot/core source.
    Configured {
        /// Physical source URI.
        physical_uri: Url,
        /// Source text.
        text: Arc<str>,
    },
    /// Discovered canonical universe package source.
    Workspace {
        /// Physical source URI.
        physical_uri: Url,
        /// Source text.
        text: Arc<str>,
    },
    /// Bundled static core source fallback.
    Bundled {
        /// Provider-assembled source text.
        text: Arc<str>,
    },
}

impl CoreSource {
    /// Selects the best available core source based on precedence rules.
    ///
    /// 1. Explicitly configured sysroot/core path.
    /// 2. Workspace conventional `phalcom-core/core/universe/src/package.ph`.
    /// 3. Workspace conventional `core/universe/src/package.ph`.
    /// 4. Bundled core source.
    pub fn select(configured_path: Option<&Path>, workspace_roots: &[PathBuf]) -> Self {
        if let Some(path) = configured_path {
            if let Some(source) = Self::load_from_path(path, true) {
                return source;
            }
        }

        for root in workspace_roots {
            if let Some(source) = Self::load_from_path(&root.join("phalcom-core/core/universe/src/package.ph"), false) {
                return source;
            }
            if let Some(source) = Self::load_from_path(&root.join("core/universe/src/package.ph"), false) {
                return source;
            }
        }

        Self::Bundled {
            text: Arc::from(canonical_universe_source()),
        }
    }

    fn load_from_path(path: &Path, is_configured: bool) -> Option<Self> {
        let text = std::fs::read_to_string(path).ok()?;
        let uri = Url::from_file_path(path.canonicalize().unwrap_or_else(|_| path.to_path_buf())).ok()?;
        let text = Arc::from(text);

        if is_configured {
            Some(Self::Configured { physical_uri: uri, text })
        } else {
            Some(Self::Workspace { physical_uri: uri, text })
        }
    }

    /// Returns the source text of the core module.
    pub fn text(&self) -> &str {
        match self {
            Self::Configured { text, .. } => text,
            Self::Workspace { text, .. } => text,
            Self::Bundled { text } => text,
        }
    }

    /// Returns the physical URI of the on-disk core file if present.
    pub fn physical_uri(&self) -> Option<&Url> {
        match self {
            Self::Configured { physical_uri, .. } => Some(physical_uri),
            Self::Workspace { physical_uri, .. } => Some(physical_uri),
            Self::Bundled { .. } => None,
        }
    }
}

/// Parses bundled core source.
pub fn bundled_parse() -> Parse {
    phalcom_ast::parser::parse(&canonical_universe_source(), 0)
}

fn canonical_universe_source() -> String {
    let provider = phalcom_modules::builtin::BuiltinProjectSourceProvider::new(phalcom_modules::identity::BuiltinProject::Universe);
    let mut combined = String::new();
    for node in provider.nodes() {
        let path = phalcom_modules::identity::ModulePath::from_components(
            node.path
                .iter()
                .map(|component| phalcom_modules::ModuleComponent::from_identifier(component).expect("valid builtin component"))
                .collect(),
        );
        let module = phalcom_modules::identity::ModuleId::builtin(phalcom_modules::identity::BuiltinProject::Universe, path);
        let source = provider
            .source_text(&module)
            .unwrap_or_else(|error| panic!("failed to load canonical universe source {module}: {error}"));
        combined.push_str(&source);
        combined.push('\n');
    }
    combined
}

/// Builds the core surface from source and the canonical native declarations.
pub fn build_core_surface(program: &Program) -> ModuleSurface {
    use phalcom_native_surface::{NativeDispatch, NativeMemberKind, NativeVisibility, NATIVE_SURFACE_CATALOG};
    let module = ModuleId::new(CORE_MODULE_URI);
    let mut source = build_module_surface(module.clone(), program);

    // 1. Ensure canonical bootstrapped class entries exist from the shared
    // universe relation catalog when a configured source omits a module.
    for relation in phalcom_native_surface::UNIVERSE_CLASS_RELATIONS {
        let class_id = ClassId::new(module.clone(), relation.class.name());
        source.classes.entry(class_id.clone()).or_insert_with(|| ClassSurface {
            id: class_id.clone(),
            superclass: relation.superclass.map(|name| ClassId::new(module.clone(), name.name())),
            superclass_reference: None,
            members: Default::default(),
            fields: Default::default(),
            source_range: Default::default(),
            name_range: Default::default(),
        });
    }
    // 2. Ingest rich native metadata only when the canonical source did not
    //    already present the member. Source declarations own navigation and
    //    native metadata supplies implementation details.
    for native in NATIVE_SURFACE_CATALOG.iter() {
        let owner_name = native.owner().name();
        let class_id = ClassId::new(module.clone(), owner_name);
        let Some(class) = source.classes.get_mut(&class_id) else {
            continue;
        };
        let side = match native.side() {
            NativeDispatch::Instance => DispatchSide::Instance,
            NativeDispatch::Class => DispatchSide::Class,
        };
        if let Some(source_member) = class.members.get_mut(native.selector()).and_then(|members| members.get_mut(side)) {
            // Keep source origin/ranges for navigation while enriching the
            // same semantic member with native return metadata.
            source_member.native_return = Some(native.return_shape);
            continue;
        }
        let callable = super::ids::CallableId {
            owner: class_id.clone(),
            selector: native.selector().to_string(),
            side,
        };
        let selector_struct = phalcom_common::selector::Selector::decode(native.selector());
        let rest = super::surface::rest_surface_from_selector_str(native.selector());
        let member = MemberSurface {
            callable,
            selector: selector_struct,
            rest,
            kind: match native.kind {
                NativeMemberKind::Method => MemberKind::Method,
                NativeMemberKind::Getter => MemberKind::Getter,
                NativeMemberKind::Setter => MemberKind::Setter,
            },
            visibility: match native.visibility() {
                NativeVisibility::Public => MemberVisibility::Public,
                NativeVisibility::Internal => MemberVisibility::Internal,
            },
            side,
            is_constructor: native.selector().starts_with("new(") && native.side() == NativeDispatch::Class,
            native_return: Some(native.return_shape),
            source_range: Default::default(),
            name_range: Default::default(),
            params: native_params(native),
            // No AST sentinel: origin explicitly tracks that this is native.
            ast: None,
            origin: super::surface::MemberOrigin::Native(native.id()),
        };
        let members = class.members.entry(native.selector().to_string()).or_default();
        match member.side {
            DispatchSide::Instance => members.instance = Some(member),
            DispatchSide::Class => members.class = Some(member),
        }
    }

    source
}

fn native_params(native: &phalcom_native_surface::NativeSurfaceRecord) -> Vec<super::surface::ParamSurface> {
    let mut params = native
        .params()
        .positional
        .iter()
        .enumerate()
        .map(|(index, _)| super::surface::ParamSurface {
            name: format!("arg{index}"),
            label: None,
            rest_mode: RestMode::None,
            source_range: Default::default(),
            name_range: Default::default(),
            label_range: None,
        })
        .collect::<Vec<_>>();
    params.extend(native.params().labeled.iter().map(|parameter| super::surface::ParamSurface {
        name: parameter.label.to_owned(),
        label: Some(parameter.label.to_owned()),
        rest_mode: RestMode::None,
        source_range: Default::default(),
        name_range: Default::default(),
        label_range: None,
    }));
    if let Some(rest) = native.params().rest {
        params.push(super::surface::ParamSurface {
            name: "rest".to_string(),
            label: None,
            rest_mode: match rest.ty {
                Some(_) => RestMode::Positional,
                None => RestMode::Complete,
            },
            source_range: Default::default(),
            name_range: Default::default(),
            label_range: None,
        });
    }
    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_source_precedence_selection() {
        let root = std::env::temp_dir().join(format!("phalcom-core-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("phalcom-core/core/universe/src")).unwrap();
        std::fs::create_dir_all(root.join("core/universe/src")).unwrap();

        let workspace_core = root.join("phalcom-core/core/universe/src/package.ph");
        std::fs::write(&workspace_core, "class WorkspaceCore {}").unwrap();

        let root_core = root.join("core/universe/src/package.ph");
        std::fs::write(&root_core, "class RootCore {}").unwrap();

        // 1. Configured wins
        let configured_path = root.join("custom-core.ph");
        std::fs::write(&configured_path, "class CustomCore {}").unwrap();
        let selected = CoreSource::select(Some(&configured_path), std::slice::from_ref(&root));
        assert_eq!(selected.text(), "class CustomCore {}");
        assert!(matches!(selected, CoreSource::Configured { .. }));

        // 2. canonical workspace universe source wins over fallback
        let selected = CoreSource::select(None, std::slice::from_ref(&root));
        assert_eq!(selected.text(), "class WorkspaceCore {}");
        assert!(matches!(selected, CoreSource::Workspace { .. }));

        // 3. alternate canonical workspace source wins if primary is absent
        std::fs::remove_file(&workspace_core).unwrap();
        let selected = CoreSource::select(None, std::slice::from_ref(&root));
        assert_eq!(selected.text(), "class RootCore {}");
        assert!(matches!(selected, CoreSource::Workspace { .. }));

        // 4. Bundled fallback
        std::fs::remove_file(&root_core).unwrap();
        let selected = CoreSource::select(None, std::slice::from_ref(&root));
        assert!(selected.text().contains("class Object"));
        assert!(matches!(selected, CoreSource::Bundled { .. }));

        let _ = std::fs::remove_dir_all(root);
    }
}
