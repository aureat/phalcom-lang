//! Live source and native declarations for the semantic core module.
//!
//! `core.ph` is embedded only as an installation fallback. Workspace and
//! open-buffer copies replace it through [`super::SemanticDb::update_core`].
//! Native members use the same structured surface consumed by completion;
//! opaque native returns deliberately carry no guessed value shape.

use phalcom_ast::ast::Program;
use phalcom_ast::parser::Parse;

use super::ids::{CORE_MODULE_URI, ClassId, DispatchSide, ModuleId};
use super::surface::{MemberKind, MemberVisibility, ModuleSurface, build_module_surface};

/// Bundled source fallback for the semantic core module.
pub const BUNDLED_CORE_SOURCE: &str = include_str!("../../../phalcom-core/core/core.ph");

/// Describes one opaque native member in the VM-free semantic surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeMember {
    /// Core class receiving the member.
    pub class: &'static str,
    /// Canonical comma-form selector.
    pub selector: &'static str,
    /// Semantic member category.
    pub kind: MemberKind,
    /// Whether dispatch targets the class object.
    pub side: DispatchSide,
    /// Source/runtime visibility.
    pub visibility: MemberVisibility,
}

/// Return knowledge for an opaque native member.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeReturnKnowledge {
    /// Native implementation has no semantic return contract.
    Unknown,
    /// Native declaration supplies a known return shape.
    Declared,
}

/// Parses bundled core source.
pub fn bundled_parse() -> Parse {
    phalcom_ast::parser::parse(BUNDLED_CORE_SOURCE, 0)
}

/// Builds the core surface from source and the canonical native declarations.
pub fn build_core_surface(program: &Program) -> ModuleSurface {
    let module = ModuleId::new(CORE_MODULE_URI);
    let mut source = build_module_surface(module.clone(), program);
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
            side: native.side,
        };
        class.members.insert(
            native.selector.to_string(),
            super::surface::MemberSurface {
                callable,
                kind: native.kind,
                visibility: native.visibility,
                side: native.side,
                is_constructor: native.selector.starts_with("new(") && native.side == DispatchSide::Class,
                source_range: Default::default(),
                name_range: Default::default(),
                params: Vec::new(),
                body: Vec::new(),
            },
        );
    }
    source
}

/// Structured native member declarations shared by semantic queries.
///
/// This is intentionally source-shaped data, not a generated JSON bridge.
/// Native implementations without a source contract remain `Unknown`.
const NATIVE_MEMBERS: &[NativeMember] = &[
    NativeMember { class: "Object", selector: "!=(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Object", selector: "==(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Object", selector: "class" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Object", selector: "class=(put)" , kind: MemberKind::Setter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Object", selector: "doesNotUnderstand(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Object", selector: "hash" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Object", selector: "methodFor(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Object", selector: "name" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Object", selector: "perform(_,***)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Object", selector: "respondsTo(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Object", selector: "toString" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Behavior", selector: "methods" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Behavior", selector: "name" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Behavior", selector: "superclass" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Behavior", selector: "superclass=(put)" , kind: MemberKind::Setter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Class", selector: "+(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Message", selector: "args" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Message", selector: "labels" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Message", selector: "name" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Message", selector: "selector" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Error", selector: "message" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Error", selector: "raise()" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Number", selector: "%(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Number", selector: "*(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Number", selector: "**(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Number", selector: "+(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Number", selector: "-(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Number", selector: "/(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Number", selector: "<(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Number", selector: "<=(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Number", selector: ">(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Number", selector: ">=(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Number", selector: "hash" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Number", selector: "negated()" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Number", selector: "toString" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Number", selector: "~/(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Int", selector: "&(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Int", selector: "|(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Int", selector: "^(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Int", selector: "~()" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Int", selector: "<<(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Int", selector: ">>(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Int", selector: "bitAt(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Int", selector: "bitCount" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Int", selector: "bitLength" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Int", selector: "trailingZeros" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Float", selector: "abs" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Float", selector: "ceil" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Float", selector: "floor" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Float", selector: "isFinite" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Float", selector: "isInfinite" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Float", selector: "isInteger" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Float", selector: "isNaN" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Float", selector: "rounded" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Float", selector: "sign" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Float", selector: "toIntExact" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Float", selector: "truncated" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "String", selector: "+(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "String", selector: "hash" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Symbol", selector: "hash" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Symbol", selector: "toString" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Bool", selector: "and(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Bool", selector: "hash" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Bool", selector: "ifFalse(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Bool", selector: "ifTrue(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Bool", selector: "not()" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Bool", selector: "or(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Function", selector: "arity" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Function", selector: "call" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Function", selector: "callWith(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Function", selector: "name" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Block", selector: "arity" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Block", selector: "call" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Block", selector: "callWith(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Block", selector: "ensure(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Block", selector: "name" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Block", selector: "on(_,_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Block", selector: "whileTrue(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Method", selector: "bind(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Method", selector: "holder" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Method", selector: "invokeOn(_,***)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Method", selector: "selector" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Module", selector: "doesNotUnderstand(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "List", selector: "toString" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "System", selector: "gc" , kind: MemberKind::Getter, side: DispatchSide::Class, visibility: MemberVisibility::Public },
    NativeMember { class: "System", selector: "nextScheduled" , kind: MemberKind::Getter, side: DispatchSide::Class, visibility: MemberVisibility::Public },
    NativeMember { class: "System", selector: "print(_)" , kind: MemberKind::Method, side: DispatchSide::Class, visibility: MemberVisibility::Public },
    NativeMember { class: "System", selector: "schedule(_)" , kind: MemberKind::Method, side: DispatchSide::Class, visibility: MemberVisibility::Public },
    NativeMember { class: "Fiber", selector: "abort(_)" , kind: MemberKind::Method, side: DispatchSide::Class, visibility: MemberVisibility::Public },
    NativeMember { class: "Fiber", selector: "call()" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Fiber", selector: "call(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Fiber", selector: "current" , kind: MemberKind::Getter, side: DispatchSide::Class, visibility: MemberVisibility::Public },
    NativeMember { class: "Fiber", selector: "error" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Fiber", selector: "isDone" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Fiber", selector: "isRoot" , kind: MemberKind::Getter, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Fiber", selector: "try()" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Fiber", selector: "try(_)" , kind: MemberKind::Method, side: DispatchSide::Instance, visibility: MemberVisibility::Public },
    NativeMember { class: "Fiber", selector: "yield()" , kind: MemberKind::Method, side: DispatchSide::Class, visibility: MemberVisibility::Public },
    NativeMember { class: "Fiber", selector: "yield(_)" , kind: MemberKind::Method, side: DispatchSide::Class, visibility: MemberVisibility::Public },
];
