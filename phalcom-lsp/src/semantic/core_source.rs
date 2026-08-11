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

/// Bundled source fallback for the semantic core module.
pub const BUNDLED_CORE_SOURCE: &str = include_str!("../../../phalcom-core/core/core.ph");

pub use phalcom_native_surface::NativeReturnKnowledge;

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
            fields: Default::default(),
            source_range: Default::default(),
        });
    }
    for native in NATIVE_MEMBERS {
        let class_id = ClassId::new(module.clone(), native.class);
        let Some(class) = source.classes.get_mut(&class_id) else {
            continue;
        };
        if class.members.contains_key(native.selector) {
            continue;
        }
        let callable = super::ids::CallableId {
            owner: class_id.clone(),
            selector: native.selector.to_string(),
            side: match native.side {
                NativeDispatch::Instance => DispatchSide::Instance,
                NativeDispatch::Class => DispatchSide::Class,
            },
        };
        class.members.insert(
            native.selector.to_string(),
            MemberSurface {
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
                side: match native.side {
                    NativeDispatch::Instance => DispatchSide::Instance,
                    NativeDispatch::Class => DispatchSide::Class,
                },
                is_constructor: native.selector.starts_with("new(") && native.side == NativeDispatch::Class,
                source_range: Default::default(),
                name_range: Default::default(),
                params: Vec::new(),
                body: Vec::new(),
            },
        );
    }
    source
}
