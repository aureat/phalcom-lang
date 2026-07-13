use crate::heap::ClassObject;
use crate::error::PhResult;
use crate::heap::{ClassId, Object, ObjRef};
use crate::interner::Symbol;
use crate::method::decode_selector;
use crate::heap::ModuleObject;
use crate::value::Value;
use std::sync::Arc;
use tracing::debug;

use super::VM;

impl VM {
    /// Interns `name`, returning its [`Symbol`].
    pub fn get_or_intern(&mut self, name: &str) -> Symbol {
        self.interner.intern(name)
    }

    /// Resolves `symbol` back to its interned string.
    pub fn resolve_symbol(&self, symbol: Symbol) -> &str {
        self.interner.lookup(symbol)
    }

    /// Allocates an immutable string on the heap and returns it as a [`Value`].
    ///
    /// Native primitives that produce strings (e.g. `toString`, string `+`) use
    /// this to move an owned [`String`] into an
    /// [`Object::Str`] and hand back a
    /// [`Value::Obj`] handle ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)).
    pub fn alloc_string_value(&mut self, text: String) -> Value {
        Value::Obj(self.heap.alloc_string(text))
    }

    /// Allocates a bare class named `name`, wired only to `superclass`.
    ///
    /// The metaclass link is left unset; callers such as [`Self::create_class`]
    /// patch it. Realizes the allocate-then-patch bootstrap
    /// ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)).
    pub fn create_single_class(&mut self, name: &str, superclass: Option<ClassId>) -> ClassId {
        let id = self.heap.alloc_class(ClassObject::bare(name));
        self.heap.class_mut(id).set_superclass(superclass);
        id
    }

    /// Follows the metaclass parallel rule
    /// ([ADR-0002](../../../docs/adr/0002-metaclass-tower-parallel-rule.md)):
    /// the metaclass `"{name}.class"` is an instance of `Metaclass` whose
    /// superclass is `superclass`'s own metaclass (`Class` if `superclass` is
    /// `None`), and the class itself is an instance of that metaclass with the
    /// requested `superclass`.
    pub fn create_class(&mut self, name: &str, superclass: Option<ClassId>) -> ClassId {
        let metaclass_class = self.universe.classes.metaclass_class;
        let metaclass_superclass = match superclass {
            Some(sc) => self.heap.class(sc).class,
            None => self.universe.classes.class_class,
        };

        let metaclass_name = name.to_owned() + ".class";
        let metaclass = self.create_single_class(&metaclass_name, Some(metaclass_superclass));
        self.heap.class_mut(metaclass).set_class(metaclass_class);

        let class = self.create_single_class(name, superclass);
        self.heap.class_mut(class).set_class(metaclass);

        let name_sym = self.interner.intern(name);
        let meta_sym = self.interner.intern(&metaclass_name);

        if let Some(layout) = self.field_layouts.get(&name_sym).cloned() {
            self.heap.class_mut(class).field_slots = layout.field_slots;
            self.heap.class_mut(class).field_count = layout.field_count;
            self.heap.class_mut(metaclass).field_slots = layout.static_field_slots;
            self.heap.class_mut(metaclass).field_count = layout.static_field_count;
            self.heap.class_mut(class).static_slots = vec![Value::Nil; layout.static_field_count as usize].into_boxed_slice();
        }

        self.classes.insert(name_sym, class);
        self.classes.insert(meta_sym, metaclass);

        class
    }

    /// Allocates a module with `logical_name`/`abs_path` and registers it.
    pub fn create_module(&mut self, logical_name: &str, abs_path: &str) -> ObjRef {
        let module_sym = self.interner.intern(logical_name);
        let module = ModuleObject::new(logical_name.to_string(), module_sym, abs_path.to_string(), None);
        let id = self.heap.alloc(Object::Module(module));
        self.modules.insert(module_sym, id);
        id
    }

    /// Updates the absolute filesystem path of the module named `module_sym`.
    pub fn register_path(&mut self, module_sym: Symbol, abs_path: &str) {
        if let Some(&module_id) = self.modules.get(&module_sym) {
            self.heap.module_mut(module_id).path = abs_path.to_string();
        } else {
            debug!("Module with symbol {:?} not found for path registration", module_sym);
        }
    }

    /// Registers `source` text for the module `logical_name` in the source map.
    pub fn register_source(&mut self, logical_name: &str, source: &str) {
        let source_ref = Arc::new(String::from(source));
        let module_sym = self.interner.intern(logical_name);
        crate::diagnostics::SOURCE_MAP.write().unwrap().insert(module_sym, source_ref.clone());

        let module_sym = self.interner.intern(logical_name);
        let src_ref = Arc::new(String::from(source));
        crate::diagnostics::SOURCE_MAP.write().unwrap().insert(module_sym, src_ref.clone());
    }

    /// Returns the module handle for `module_sym`, if loaded.
    pub fn get_module(&mut self, module_sym: Symbol) -> Option<ObjRef> {
        self.modules.get(&module_sym).copied()
    }

    /// Returns the module handle for the module named `name`, if loaded.
    pub fn get_module_from_str(&mut self, name: &str) -> Option<ObjRef> {
        let sym = self.interner.intern(name);
        self.modules.get(&sym).copied()
    }

    /// Defines global `name_sym = val` in the module `module_sym`.
    ///
    /// # Errors
    ///
    /// Propagates [`ModuleObject::define`](crate::heap::ModuleObject::define)
    /// errors (e.g. too many globals).
    pub fn define_global(&mut self, module_sym: Symbol, name_sym: Symbol, val: Value) -> PhResult<usize> {
        let module = self.get_module(module_sym).expect("correct module");
        self.heap.module_mut(module).define(name_sym, val)
    }

    /// Creates a user class `name` with its own metaclass and wires the tower.
    ///
    /// Rebuilds `class_id`'s [`base_names`](crate::heap::ClassObject::base_names)
    /// index from scratch (selectors.md §3.1, U16-Open): its own directly
    /// bound methods' base names, merged with its superclass's
    /// already-finalized (and thus already-flattened) index.
    ///
    /// Idempotent — safe to call repeatedly (once per kernel row at
    /// bootstrap, `VM::install_core`; again on every `.ph` class body or
    /// reopen, [`crate::bytecode::Bytecode::FinalizeClass`])
    /// since it always recomputes from the row's current
    /// [`methods`](crate::heap::ClassObject::methods) table rather than
    /// accumulating onto the prior index. Requires the superclass to already
    /// be finalized — every caller (the kernel bootstrap's dependency order,
    /// the compiler's single top-down class-compile pass) upholds that.
    pub fn finalize_class_base_names(&mut self, class_id: ClassId) {
        let (own_selectors, superclass): (Vec<Symbol>, Option<ClassId>) = {
            let class = self.heap.class(class_id);
            (class.methods.keys().copied().collect(), class.superclass)
        };
        let mut merged: std::collections::HashMap<Symbol, Vec<Symbol>> = match superclass {
            Some(sc) => self.heap.class(sc).base_names.clone(),
            None => std::collections::HashMap::new(),
        };
        for selector in own_selectors {
            let selector_str = self.resolve_symbol(selector).to_string();
            let (name, _labels, _kind) = decode_selector(&selector_str);
            let name_sym = self.interner.intern(&name);
            let bucket = merged.entry(name_sym).or_default();
            if !bucket.contains(&selector) {
                bucket.push(selector);
            }
        }
        self.heap.class_mut(class_id).base_names = merged;
    }
}
