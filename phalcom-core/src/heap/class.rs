//! Classes and metaclasses — the rows of the object-model tower.
//!
//! A [`ClassObject`] is a heap [`Object`](crate::heap::Object) referenced by a
//! [`ClassId`]. Its links to its metaclass and superclass are plain [`ClassId`]
//! handles ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)), so the
//! kernel's cyclic wiring (e.g. `Metaclass.class == Metaclass`) is just a handle
//! that points at itself — no `Rc`, no `Weak`, no `RefCell` (`object-model.md`
//! §5–6). Method lookup walks the superclass chain through the heap.

use crate::heap::{ClassId, Heap, ObjRef};
use crate::interner::Symbol;
use crate::value::Value;
use indexmap::IndexMap;
use std::collections::HashMap;

/// Selector → [`MethodObject`](crate::method::MethodObject) handle table.
type MethodsMap = IndexMap<Symbol, ObjRef>;

/// A class or metaclass row in the object-model tower.
///
/// Both `class` (the metaclass) and `superclass` are [`ClassId`] handles into the
/// [`Heap`], patched during bootstrap ([`crate::universe`]). See
/// [ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md).
#[derive(Debug, Clone)]
pub struct ClassObject {
    /// The class's display name (e.g. `"Number"`, `"Number.class"`).
    pub name: String,
    /// Handle to this class's metaclass (its `class`). For `Metaclass` this is a
    /// self-cycle.
    pub class: ClassId,
    /// Handle to this class's superclass, or `None` at the tower's apex
    /// (`Object`).
    pub superclass: Option<ClassId>,
    /// Methods defined directly on this class, keyed by selector [`Symbol`].
    pub methods: MethodsMap,
    /// One rest-capable method per base family, defined directly on this class.
    /// Exact methods remain in `methods`; fallback never parses wildcard text.
    pub rest_methods: HashMap<Symbol, ObjRef>,
    /// Instance fields, keyed by name [`Symbol`] to their slot offset (ADR-0011).
    pub field_slots: IndexMap<Symbol, u16>,
    /// Number of instance slots (ADR-0011).
    pub field_count: u16,
    /// Class-side stored fields (static fields), stored as a fixed-size slot vector (ADR-0017).
    pub static_slots: Box<[Value]>,
    /// The per-class **base-name index** (selectors.md §3.1, U16-Open):
    /// every base method name reachable on this class — flattened through
    /// inheritance — mapped to the full selector [`Symbol`]s that share it.
    ///
    /// Rebuilt from scratch at class-finalization time
    /// ([`Bytecode::FinalizeClass`](crate::bytecode::Bytecode::FinalizeClass),
    /// `VM::finalize_class_base_names`) by merging this row's own directly
    /// bound [`methods`](Self::methods) with its already-finalized
    /// superclass's index. Backs the `::` reference-time empty-family check
    /// (a non-empty entry means at least one method answers to that base
    /// name) and, incidentally, a future candidate-list/reflection surface.
    /// Empty by default (`bare`) — a bootstrapped kernel row is finalized
    /// once its native primitives are installed (`VM::install_core`), and a
    /// `.ph` class body is finalized at its own compile tail.
    pub base_names: HashMap<Symbol, Vec<Symbol>>,
    /// Attribute instances attached via `Object#__attach`
    /// (M-ATTR-ROOT, `attribute-classes.md`) — the retention store behind
    /// `Behavior#attributes`/`attributesOfType(_)`. Empty until a class-level
    /// `@AttrName(...)` attaches one.
    pub attributes: Vec<Value>,
    /// Set by `Object#__freezeAttributes` once class-definition codegen has
    /// finished attaching every class-level attribute — further `__attach`
    /// calls are rejected (`attr.frozen`).
    pub attributes_frozen: bool,
    /// Refusal flag for native representation classes that cannot be allocated
    /// via standard `InstanceObject::new` generic allocator (`new_`).
    pub native_repr: bool,
    /// Refusal flag for abstract classes (e.g. Number, Bool, Function, Option, Behavior)
    /// that cannot be allocated via generic InstanceObject::new (new_).
    pub is_abstract: bool,
}

/// Walks `class` and its superclasses for a method bound to `selector`.
///
/// Returns the first matching [`MethodObject`](crate::method::MethodObject)
/// handle, or `None` if no class in the chain defines it. Traversal follows
/// [`ClassId`] handles through `heap`, so it neither borrows nor clones any
/// object across steps.
pub fn lookup_method_in_hierarchy(heap: &Heap, mut class: ClassId, selector: Symbol) -> Option<ObjRef> {
    loop {
        let current = heap.class(class);
        if let Some(&method) = current.methods.get(&selector) {
            return Some(method);
        }
        class = current.superclass?;
    }
}

impl ClassObject {
    /// Creates an unwired class row: `name` set, links defaulted.
    ///
    /// The `class` handle is the null [`ObjRef`] and `superclass` is `None`; the
    /// bootstrap (allocate-then-patch, [`crate::universe`]) fills them in before
    /// the class is used. See [ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md).
    pub fn bare(name: &str) -> Self {
        Self {
            name: name.to_string(),
            class: ClassId::default(),
            superclass: None,
            methods: MethodsMap::new(),
            rest_methods: HashMap::new(),
            field_slots: IndexMap::new(),
            field_count: 0,
            static_slots: Box::default(),
            base_names: HashMap::new(),
            attributes: Vec::new(),
            attributes_frozen: false,
            native_repr: false,
            is_abstract: false,
        }
    }

    /// Appends `attr` to this class's attribute-retention store.
    ///
    /// # Errors
    ///
    /// Returns `false` if [`Self::attributes_frozen`] is set — the caller
    /// (`primitive::attribute::attribute_attach`) renders the `attr.frozen`
    /// `RuntimeError`.
    pub fn attach_attribute(&mut self, attr: Value) -> bool {
        if self.attributes_frozen {
            return false;
        }
        self.attributes.push(attr);
        true
    }

    /// Resolves a field name to its slot index (own table only, non-inherited).
    pub fn resolve_field(&self, name: Symbol) -> Option<u16> {
        self.field_slots.get(&name).copied()
    }

    /// Returns a copy of this class's display name.
    pub fn name_copy(&self) -> String {
        self.name.clone()
    }

    /// Renders this class's debug form, `"<class Name>"`.
    #[inline]
    pub fn to_debug(&self) -> String {
        format!("<class {}>", self.name)
    }

    /// Sets this class's superclass handle.
    pub fn set_superclass(&mut self, class: Option<ClassId>) {
        self.superclass = class;
    }

    /// Sets this class's metaclass (`class`) handle.
    pub fn set_class(&mut self, class: ClassId) {
        self.class = class;
    }

    /// Binds `method` under `selector`, replacing any prior binding.
    pub fn add_method(&mut self, selector: Symbol, method: ObjRef) {
        self.methods.insert(selector, method);
    }

    pub fn add_rest_method(&mut self, base: Symbol, method: ObjRef) -> Option<ObjRef> {
        self.rest_methods.insert(base, method)
    }

    pub fn get_rest_method(&self, base: Symbol) -> Option<ObjRef> {
        self.rest_methods.get(&base).copied()
    }

    /// Returns the method handle bound directly to `selector` on this class, if
    /// any (no superclass walk).
    pub fn get_method(&self, selector: Symbol) -> Option<ObjRef> {
        self.methods.get(&selector).copied()
    }

    /// Returns whether this class's flattened [`base_names`](Self::base_names)
    /// index answers to `name` (selectors.md §3.1, U16-Open) — the first half
    /// of the `::` reference-time empty-family check. The second half (a
    /// `doesNotUnderstand` override) needs a hierarchy walk and is checked
    /// separately (`VM`'s `Bytecode::MakeFamily` handler).
    pub fn responds_to_base_name(&self, name: Symbol) -> bool {
        self.base_names.get(&name).is_some_and(|selectors| !selectors.is_empty())
    }
}
