//! Immutable snapshot of VM runtime state captured at cell boundaries.
//!
//! Provides lock-free access to globals, class structures, and member inheritance
//! chains for auto-completion, hints, and live-layer syntax highlighting without
//! executing user code on keystrokes.

use phalcom_core::heap::{ClassId, ObjRef, Object};
use phalcom_core::interner::Symbol;
use phalcom_core::method::{SignatureKind, decode_selector};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::collections::{HashMap, HashSet};

/// Classification of a class member for completion rendering and arity insertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberKind {
    /// Zero-argument getter (`object.x`).
    Getter,
    /// Single-argument setter (`object.x = val`).
    Setter,
    /// Positional or keyword method call.
    Method,
    /// Subscript getter or setter (`object[index]`).
    Subscript,
    /// Variadic method call (`object.sum(*args)`).
    Variadic,
}

/// Description of a single class member in an inheritance chain.
#[derive(Debug, Clone)]
pub struct Member {
    /// Method or member base name (e.g. `"add"`, `"[]"`).
    pub name: String,
    /// Full interned selector representation (e.g. `"add(_)"`).
    pub selector: String,
    /// Member signature kind.
    pub kind: MemberKind,
    /// Fixed/positional arity of the member.
    pub arity: u8,
    /// Inheritance depth (0 for own members, 1 for immediate parent, etc.).
    pub own_depth: usize,
}

/// An immutable snapshot of the REPL VM state.
#[derive(Debug, Clone, Default)]
pub struct ReplSnapshot {
    /// Globals bound in the session module -> class of their current value.
    pub globals: HashMap<Symbol, ClassId>,
    /// Global name strings -> class of their current value.
    pub global_names: HashMap<String, ClassId>,
    /// Full inheritance chain per class, tagged with own_depth.
    pub members: HashMap<ClassId, Vec<Member>>,
    /// Class-valued globals bound in the session module.
    pub classes: HashMap<Symbol, ClassId>,
    /// Class name strings -> ClassId handle.
    pub class_names: HashMap<String, ClassId>,
}

impl ReplSnapshot {
    /// Creates an empty snapshot.
    pub fn new() -> Self {
        Self::default()
    }

    /// Captures a snapshot of the current VM state at a cell boundary.
    pub fn capture(vm: &VM, module_ref: ObjRef) -> Self {
        let mut globals = HashMap::new();
        let mut global_names = HashMap::new();
        let mut classes = HashMap::new();
        let mut class_names = HashMap::new();
        let mut members = HashMap::new();

        let module = vm.heap.module(module_ref);

        for (&sym, &slot) in &module.name_to_slot {
            let name_str = vm.resolve_symbol(sym).to_string();
            if let Some(val) = module.globals.get(slot) {
                let class_id = val.class(vm);
                globals.insert(sym, class_id);
                global_names.insert(name_str.clone(), class_id);

                if let Value::Obj(obj_ref) = val {
                    if matches!(vm.heap.get(*obj_ref), Object::Class(_)) {
                        classes.insert(sym, *obj_ref);
                        class_names.insert(name_str, *obj_ref);
                    }
                }
            }
        }

        let mut visited_classes = HashSet::new();
        for &class_id in globals.values().chain(classes.values()) {
            visited_classes.insert(class_id);
        }

        let core_classes = [
            vm.universe.classes.object_class,
            vm.universe.classes.class_class,
            vm.universe.classes.string_class,
            vm.universe.classes.list_class,
            vm.universe.classes.map_class,
            vm.universe.classes.number_class,
            vm.universe.classes.bool_class,
            vm.universe.classes.true_class,
            vm.universe.classes.false_class,
            vm.universe.classes.nil_class,
            vm.universe.classes.symbol_class,
        ];

        for class_id in core_classes {
            if class_id != ClassId::default() {
                visited_classes.insert(class_id);
            }
        }

        for class_id in visited_classes {
            let chain_members = Self::walk_class_chain(vm, class_id);
            members.insert(class_id, chain_members);
        }

        Self {
            globals,
            global_names,
            members,
            classes,
            class_names,
        }
    }

    /// Returns whether `name` is a bound global.
    pub fn is_global_name_bound(&self, name: &str) -> bool {
        self.global_names.contains_key(name)
    }

    fn walk_class_chain(vm: &VM, start_class: ClassId) -> Vec<Member> {
        let mut result = Vec::new();
        let mut curr = Some(start_class);
        let mut depth = 0;
        let mut seen_selectors = HashSet::new();

        while let Some(class_id) = curr {
            let class_obj = vm.heap.class(class_id);
            for &sel_sym in class_obj.methods.keys() {
                let sel_str = vm.resolve_symbol(sel_sym);
                if seen_selectors.contains(sel_str) {
                    continue;
                }
                seen_selectors.insert(sel_str.to_string());

                let (name, _labels, sig_kind) = decode_selector(sel_str);
                let (kind, arity) = match sig_kind {
                    SignatureKind::Getter => (MemberKind::Getter, 0),
                    SignatureKind::Setter => (MemberKind::Setter, 1),
                    SignatureKind::Method(n) => (MemberKind::Method, n),
                    SignatureKind::SubscriptGet(n) => (MemberKind::Subscript, n),
                    SignatureKind::SubscriptSet(n) => (MemberKind::Subscript, n + 1),
                    SignatureKind::Variadic(n) => (MemberKind::Variadic, n),
                };

                result.push(Member {
                    name,
                    selector: sel_str.to_string(),
                    kind,
                    arity,
                    own_depth: depth,
                });
            }

            curr = class_obj.superclass;
            depth += 1;
        }

        result
    }
}
