//! Modules — top-level namespaces of global slots.
//!
//! A [`ModuleObject`] is a heap [`Object`](crate::heap::Object). Interior
//! mutability now lives in the [`Heap`](crate::heap::Heap): the globals table and
//! name index are plain fields, mutated through `heap.module_mut(id)` rather than
//! per-object `RefCell`s ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)).

use crate::error::{PhResult, RuntimeError};
use crate::heap::ObjRef;
use crate::interner::Symbol;
use crate::value::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Hard limit on the number of globals a single module may declare.
pub const MAX_GLOBALS: usize = 1 << 16; // = 65,536

/// Logical name of the bootstrap core module.
pub const CORE_MODULE_NAME: &str = "core";
/// Logical name of the program entry module.
pub const MAIN_MODULE_NAME: &str = "main";

/// A process-unique module identifier.
pub type ModuleId = u32;

/// Monotonic source of [`ModuleId`]s.
static MODULE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Returns a fresh, process-unique [`ModuleId`].
pub fn next_module_id() -> ModuleId {
    MODULE_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// A loaded module: its identity, source, top-level closure, and global slots.
#[derive(Debug)]
pub struct ModuleObject {
    /// Interned symbol of the module's logical name.
    pub name_sym: Symbol,
    /// The module's display name.
    pub name: String,
    /// Absolute filesystem path (or an internal placeholder for the core module).
    pub path: String,
    /// The module's source text, shared for diagnostics.
    pub source: Option<Arc<String>>,
    /// Handle to the module's top-level [`ClosureObject`](crate::heap::ClosureObject), once compiled.
    pub closure: Option<ObjRef>,
    /// Global variable slots, indexed by slot number.
    pub globals: Vec<Value>,
    /// Maps a global name [`Symbol`] to its slot index in [`Self::globals`].
    pub name_to_slot: HashMap<Symbol, usize>,
    /// Bumped **only** when [`Self::declare`] allocates a *new* slot; guards the
    /// per-callsite global caches in [`Chunk::gcaches`](crate::chunk::Chunk).
    ///
    /// A callsite that resolved through the core-module fallback must stop doing
    /// so the moment this module defines that same name (`var List = 42` after a
    /// site already read core's `List`). That is the *only* way a resolved
    /// `(module, slot)` pair can become wrong: slots are append-only
    /// ([`Self::declare`] returns the existing slot for a known name), and
    /// [`Self::set_global`] rewrites a slot's value without moving it — so
    /// neither redefinition nor assignment needs to invalidate anything.
    pub globals_version: u64,
    /// Attribute instances attached via `Object#__attach` (M-ATTR-ROOT,
    /// `attribute-classes.md`).
    pub attributes: Vec<Value>,
    /// Set by `Object#__freezeAttributes` — further `__attach` calls are
    /// rejected (`attr.frozen`).
    pub attributes_frozen: bool,
}

impl ModuleObject {
    /// Creates an empty module. The caller must register it in the VM to keep it
    /// reachable.
    pub fn new(name: String, name_sym: Symbol, path: String, source: Option<Arc<String>>) -> Self {
        Self {
            name,
            name_sym,
            path,
            closure: None,
            globals: Vec::new(),
            name_to_slot: HashMap::new(),
            globals_version: 0,
            source,
            attributes: Vec::new(),
            attributes_frozen: false,
        }
    }

    /// Appends `attr` to this module's attribute-retention store.
    ///
    /// # Errors
    ///
    /// Returns `Err(())` if [`Self::attributes_frozen`] is set.
    pub fn attach_attribute(&mut self, attr: Value) -> Result<(), ()> {
        if self.attributes_frozen {
            return Err(());
        }
        self.attributes.push(attr);
        Ok(())
    }

    /// Returns the module's name symbol.
    #[inline]
    pub fn symbol(&self) -> Symbol {
        self.name_sym
    }

    /// Attaches the module's top-level closure handle.
    pub fn add_closure(&mut self, closure: ObjRef) {
        self.closure = Some(closure);
    }

    /// Renders the module's debug form, `"<module Name>"`.
    pub fn to_debug(&self) -> String {
        format!("<module {}>", self.name)
    }

    /// Reserves a slot for a top-level variable, returning its index.
    ///
    /// Idempotent: an already-declared name returns its existing slot. Forward
    /// references declare with [`Value::Nil`]; the real definition later calls
    /// [`Self::set_global`].
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::Message`] if the module already holds
    /// [`MAX_GLOBALS`] globals.
    pub fn declare(&mut self, name: Symbol) -> PhResult<usize> {
        if let Some(&slot) = self.name_to_slot.get(&name) {
            return Ok(slot);
        }

        let cur = self.name_to_slot.len();
        if cur >= MAX_GLOBALS {
            return Err(RuntimeError::Message("Too many globals in module".into()).into());
        }

        self.name_to_slot.insert(name, cur);
        // A new name here can shadow one that callsites already resolved through
        // the core-module fallback, so their cached slots must be discarded.
        self.globals_version += 1;
        // Storage default only: a freshly-declared global slot backs its value
        // with the private sentinel until written. Never read raw — the
        // `GetGlobal` handler surfaces it to `None` (Invariant 4).
        self.globals.push(Value::Nil);
        Ok(cur)
    }

    /// Declares `name` (if needed) and initializes its slot to `value`.
    ///
    /// # Errors
    ///
    /// Propagates errors from [`Self::declare`] and [`Self::set_global`].
    pub fn define(&mut self, name: Symbol, value: Value) -> PhResult<usize> {
        let slot = self.declare(name)?;
        self.set_global(slot, value)?;
        Ok(slot)
    }

    /// Returns the value bound to `name`, or `None` if it is not declared yet.
    #[inline]
    pub fn get(&self, name: Symbol) -> Option<Value> {
        self.name_to_slot.get(&name).and_then(|&slot| self.globals.get(slot).copied())
    }

    /// Returns the slot `name` is declared in, or `None` if it is not declared yet.
    ///
    /// The resolution half of [`Self::get`], split out so a caller can cache the
    /// slot and skip the hash probe on later accesses
    /// ([`GlobalCache`](crate::chunk::GlobalCache)).
    #[inline]
    pub fn slot_of(&self, name: Symbol) -> Option<usize> {
        self.name_to_slot.get(&name).copied()
    }

    /// Returns the value in `slot`, or `None` if the slot is out of range.
    #[inline]
    pub fn get_by_slot(&self, slot: usize) -> Option<Value> {
        self.globals.get(slot).copied()
    }

    /// Writes `value` into an existing global `slot`.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::Message`] if `slot` is out of bounds.
    pub fn set_global(&mut self, slot: usize, value: Value) -> PhResult<()> {
        if slot >= self.globals.len() {
            return Err(RuntimeError::Message("Global slot out of bounds".into()).into());
        }
        self.globals[slot] = value;
        Ok(())
    }
}
