//! Live source and native declarations for the semantic core module.
//!
//! `core.ph` is embedded only as an installation fallback. Workspace and
//! open-buffer copies replace it through [`super::SemanticDb::update_core`].
//! Native members use the same structured surface consumed by completion;
//! opaque native returns deliberately carry no guessed value shape.

use phalcom_ast::ast::Program;
use phalcom_ast::parser::Parse;
use phalcom_native_surface::{NATIVE_CLASSES, NATIVE_MEMBERS, NativeDispatch, NativeMemberKind, NativeVisibility};

use super::ids::{CORE_MODULE_URI, ClassId, DispatchSide, ModuleId};
use super::surface::{ClassSurface, MemberKind, MemberSurface, MemberVisibility, ModuleSurface, build_module_surface};

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
    let module = ModuleId::new(CORE_MODULE_URI);
    let mut source = build_module_surface(module.clone(), program);
    for native_class in NATIVE_CLASSES {
        let class_id = ClassId::new(module.clone(), native_class.name);
        source.classes.entry(class_id.clone()).or_insert_with(|| ClassSurface {
            id: class_id.clone(),
            superclass: native_class.superclass.map(|name| ClassId::new(module.clone(), name)),
            members: Default::default(),
            members_by_side: Default::default(),
            fields: Default::default(),
            source_range: Default::default(),
            name_range: Default::default(),
        });
    }
    for native in NATIVE_MEMBERS {
        let class_id = ClassId::new(module.clone(), native.class);
        let Some(class) = source.classes.get_mut(&class_id) else {
            continue;
        };
        let side = match native.side {
            NativeDispatch::Instance => DispatchSide::Instance,
            NativeDispatch::Class => DispatchSide::Class,
        };
        if class.members_by_side.contains_key(&(native.selector.to_string(), side)) {
            continue;
        }
        let callable = super::ids::CallableId {
            owner: class_id.clone(),
            selector: native.selector.to_string(),
            side,
        };
        let member = MemberSurface {
            callable,
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
            body: Vec::new(),
        };
        class.members_by_side.insert((native.selector.to_string(), member.side), member.clone());
        class.members.entry(native.selector.to_string()).or_insert(member);
    }
    source
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_source_precedence_and_uri_accessors() {
        let uri = Url::parse("file:///custom/core.ph").unwrap();
        let configured = CoreSource::Configured {
            physical_uri: uri.clone(),
            text: Arc::from("class Core {}"),
        };
        assert_eq!(configured.text(), "class Core {}");
        assert_eq!(configured.physical_uri(), Some(&uri));

        let bundled = CoreSource::Bundled {
            text: "class BundledCore {}",
        };
        assert_eq!(bundled.text(), "class BundledCore {}");
        assert_eq!(bundled.physical_uri(), None);
    }
}
