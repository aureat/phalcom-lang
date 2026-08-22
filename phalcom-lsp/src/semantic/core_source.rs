//! Live source and native declarations for the semantic core module.
//!
//! `core.ph` is embedded only as an installation fallback. Workspace and
//! open-buffer copies replace it through [`super::SemanticDb::update_core`].
//! Native members use the same structured surface consumed by completion;
//! opaque native returns deliberately carry no guessed value shape.

use phalcom_ast::ast::Program;
use phalcom_ast::parser::Parse;
use phalcom_native_surface::NATIVE_CLASSES;

use super::ids::{CORE_MODULE_URI, ClassId, DispatchSide, ModuleId};
use super::surface::{ClassSurface, MemberKind, MemberSurface, MemberVisibility, ModuleSurface, build_module_surface};

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower_lsp::lsp_types::Url;

/// Bundled source fallback for the semantic core module.
pub const BUNDLED_CORE_SOURCE: &str = include_str!("../../../phalcom-core/core/core.ph");

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
    /// Discovered workspace core source (`phalcom-core/core/core.ph` or `core/core.ph`).
    Workspace {
        /// Physical source URI.
        physical_uri: Url,
        /// Source text.
        text: Arc<str>,
    },
    /// Bundled static core source fallback.
    Bundled {
        /// Static source text.
        text: &'static str,
    },
}

impl CoreSource {
    /// Selects the best available core source based on precedence rules.
    ///
    /// 1. Explicitly configured sysroot/core path.
    /// 2. Workspace conventional `phalcom-core/core/core.ph`.
    /// 3. Workspace conventional `core/core.ph`.
    /// 4. Bundled core source.
    pub fn select(configured_path: Option<&Path>, workspace_roots: &[PathBuf]) -> Self {
        if let Some(path) = configured_path {
            if let Some(source) = Self::load_from_path(path, true) {
                return source;
            }
        }

        for root in workspace_roots {
            if let Some(source) = Self::load_from_path(&root.join("phalcom-core/core/core.ph"), false) {
                return source;
            }
            if let Some(source) = Self::load_from_path(&root.join("core/core.ph"), false) {
                return source;
            }
        }

        Self::Bundled { text: BUNDLED_CORE_SOURCE }
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
    phalcom_ast::parser::parse(BUNDLED_CORE_SOURCE, 0)
}

/// Builds the core surface from source and the canonical native declarations.
pub fn build_core_surface(program: &Program) -> ModuleSurface {
    use phalcom_native_surface::{NATIVE_MEMBERS, NATIVE_SURFACES, NativeDispatch, NativeMemberKind, NativeVisibility};
    let module = ModuleId::new(CORE_MODULE_URI);
    let mut source = build_module_surface(module.clone(), program);

    // 1. Ensure bootstrapped class entries exist from NATIVE_CLASSES
    for native_class in NATIVE_CLASSES {
        let class_id = ClassId::new(module.clone(), native_class.name);
        source.classes.entry(class_id.clone()).or_insert_with(|| ClassSurface {
            id: class_id.clone(),
            superclass: native_class.superclass.map(|name| ClassId::new(module.clone(), name)),
            superclass_reference: None,
            members: Default::default(),
            fields: Default::default(),
            source_range: Default::default(),
            name_range: Default::default(),
        });
    }

    // 2. Ingest rich native members from the generated canonical surface catalog (NATIVE_SURFACES).
    //    These records carry complete type/effect/doc metadata and are the preferred source.
    //    As #[primitive] annotations migrate to use rich specs, this set grows and NATIVE_MEMBERS shrinks.
    for native in NATIVE_SURFACES {
        let owner_name = native.owner().name();
        let class_id = ClassId::new(module.clone(), owner_name);
        let Some(class) = source.classes.get_mut(&class_id) else {
            continue;
        };
        let side = match native.side() {
            NativeDispatch::Instance => DispatchSide::Instance,
            NativeDispatch::Class => DispatchSide::Class,
        };
        if class.member(native.selector(), side).is_some() {
            // Source declaration wins; native surface is skipped.
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
            params: Vec::new(),
            // No AST sentinel: origin explicitly tracks that this is native.
            ast: super::surface::MemberAstRef::INVALID,
            origin: super::surface::MemberOrigin::Native,
        };
        let members = class.members.entry(native.selector().to_string()).or_default();
        match member.side {
            DispatchSide::Instance => members.instance = Some(member),
            DispatchSide::Class => members.class = Some(member),
        }
    }

    // 3. Augment with legacy NATIVE_MEMBERS for any member not yet migrated to NATIVE_SURFACES.
    //    This preserves full coverage during the migration period. Once all primitives carry
    //    rich #[primitive] metadata, this block can be removed.
    for native in NATIVE_MEMBERS {
        let class_id = ClassId::new(module.clone(), native.class);
        let Some(class) = source.classes.get_mut(&class_id) else {
            continue;
        };
        let side = match native.side {
            NativeDispatch::Instance => DispatchSide::Instance,
            NativeDispatch::Class => DispatchSide::Class,
        };
        if class.member(native.selector, side).is_some() {
            // Already registered from NATIVE_SURFACES or source; skip.
            continue;
        }
        let callable = super::ids::CallableId {
            owner: class_id.clone(),
            selector: native.selector.to_string(),
            side,
        };
        let selector_struct = phalcom_common::selector::Selector::decode(native.selector);
        let rest = super::surface::rest_surface_from_selector_str(native.selector);
        let member = MemberSurface {
            callable,
            selector: selector_struct,
            rest,
            kind: match native.kind {
                NativeMemberKind::Method => MemberKind::Method,
                NativeMemberKind::Getter => MemberKind::Getter,
                NativeMemberKind::Setter => MemberKind::Setter,
            },
            visibility: match native.visibility {
                NativeVisibility::Public => MemberVisibility::Public,
                NativeVisibility::Internal => MemberVisibility::Internal,
            },
            side,
            is_constructor: native.selector.starts_with("new(") && native.side == NativeDispatch::Class,
            native_return: Some(native.return_shape),
            source_range: Default::default(),
            name_range: Default::default(),
            params: Vec::new(),
            ast: super::surface::MemberAstRef::INVALID,
            origin: super::surface::MemberOrigin::Native,
        };
        let members = class.members.entry(native.selector.to_string()).or_default();
        match member.side {
            DispatchSide::Instance => members.instance = Some(member),
            DispatchSide::Class => members.class = Some(member),
        }
    }
    source
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_source_precedence_selection() {
        let root = std::env::temp_dir().join(format!("phalcom-core-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("phalcom-core/core")).unwrap();
        std::fs::create_dir_all(root.join("core")).unwrap();

        let workspace_core = root.join("phalcom-core/core/core.ph");
        std::fs::write(&workspace_core, "class WorkspaceCore {}").unwrap();

        let root_core = root.join("core/core.ph");
        std::fs::write(&root_core, "class RootCore {}").unwrap();

        // 1. Configured wins
        let configured_path = root.join("custom-core.ph");
        std::fs::write(&configured_path, "class CustomCore {}").unwrap();
        let selected = CoreSource::select(Some(&configured_path), std::slice::from_ref(&root));
        assert_eq!(selected.text(), "class CustomCore {}");
        assert!(matches!(selected, CoreSource::Configured { .. }));

        // 2. phalcom-core/core/core.ph wins over core/core.ph
        let selected = CoreSource::select(None, std::slice::from_ref(&root));
        assert_eq!(selected.text(), "class WorkspaceCore {}");
        assert!(matches!(selected, CoreSource::Workspace { .. }));

        // 3. core/core.ph wins if phalcom-core not present
        std::fs::remove_file(&workspace_core).unwrap();
        let selected = CoreSource::select(None, std::slice::from_ref(&root));
        assert_eq!(selected.text(), "class RootCore {}");
        assert!(matches!(selected, CoreSource::Workspace { .. }));

        // 4. Bundled fallback
        std::fs::remove_file(&root_core).unwrap();
        let selected = CoreSource::select(None, std::slice::from_ref(&root));
        assert_eq!(selected.text(), BUNDLED_CORE_SOURCE);
        assert!(matches!(selected, CoreSource::Bundled { .. }));

        let _ = std::fs::remove_dir_all(root);
    }
}
