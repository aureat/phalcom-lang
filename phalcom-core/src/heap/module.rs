//! Modules — top-level namespaces of global slots.
//!
//! A [`ModuleObject`] is a heap [`Object`](crate::heap::Object). Interior
//! mutability now lives in the [`Heap`](crate::heap::Heap): the globals table and
//! name index are plain fields, mutated through `heap.module_mut(id)` rather than
//! per-object `RefCell`s ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)).

use crate::error::{PhResult, RuntimeError};
use crate::heap::ObjRef;
use crate::interner::Symbol;
use crate::modules::{BindingRef, RuntimeLinkedRead};
use crate::value::Value;
pub use phalcom_modules::{ModuleId, ModuleKind};
use std::collections::HashMap;
use std::sync::Arc;

/// Hard limit on the number of globals a single module may declare.
pub const MAX_GLOBALS: usize = 1 << 16; // = 65,536

/// Logical name of the bootstrap core module.
pub const CORE_MODULE_NAME: &str = "core";
/// Logical name of the program entry module.
pub const MAIN_MODULE_NAME: &str = "main";

/// Runtime export reference stored in a module's public export table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExportRef {
    /// Live binding slot in an exporting module.
    Binding(BindingRef),
    /// Re-exported module or submodule object.
    Module(ObjRef),
}

/// A loaded module: its identity, source, top-level closure, and global slots.
#[derive(Debug)]
pub struct ModuleObject {
    /// Semantic module identity.
    pub id: ModuleId,
    /// Source kind: Module or Package.
    pub kind: ModuleKind,
    /// Interned symbol of the module's logical name.
    pub name_sym: Symbol,
    /// The module's display name.
    pub name: String,
    /// Absolute filesystem path (or an internal placeholder for the core module).
    pub path: String,
    /// Every source text compiled into this module, in compilation order and
    /// indexed by [`Chunk::source_id`](crate::chunk::Chunk::source_id).
    ///
    /// A file-backed module accumulates exactly one entry; the REPL feeds one
    /// module many cells and so accumulates one per cell. Diagnostics resolve a
    /// chunk's span against *its own* entry rather than against the module's
    /// latest text (U-REPL §D2, precondition 6).
    pub sources: Vec<Arc<String>>,
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
    /// Prior units' global bindings: name -> is_mutable (U-REPL §D4).
    pub global_bindings: HashMap<Symbol, bool>,
    /// Whether this module is a built-in module/package.
    pub builtin: bool,
    /// Nearest enclosing Package; None if standalone Module.
    pub package: Option<ObjRef>,
    /// Root package of owning project/tree; None if standalone.
    pub root_package: Option<ObjRef>,
    /// Whether the module's global namespace is frozen against modifications.
    pub namespace_frozen: bool,
    /// Runtime materialization of symbolic `GetLinked` entries.
    pub linked_reads: Vec<RuntimeLinkedRead>,
    /// Public export table.
    pub exports: HashMap<Symbol, RuntimeExportRef>,
    /// Module metadata retained from source.
    pub metadata: Option<Arc<phalcom_modules::ModuleMetadata>>,
}

impl ModuleObject {
    /// Creates an empty module. The caller must register it in the VM to keep it
    /// reachable.
    ///
    /// `source`, when `Some`, seeds [`Self::sources`] as entry `0`; modules that
    /// are compiled through [`VM::compile_closure`](crate::vm::VM::compile_closure)
    /// pass `None` and let that call append their text instead.
    pub fn new(id: ModuleId, kind: ModuleKind, name: String, name_sym: Symbol, path: String, source: Option<Arc<String>>, builtin: bool) -> Self {
        Self {
            id,
            kind,
            name,
            name_sym,
            path,
            closure: None,
            globals: Vec::new(),
            name_to_slot: HashMap::new(),
            globals_version: 0,
            sources: source.into_iter().collect(),
            global_bindings: HashMap::new(),
            builtin,
            package: None,
            root_package: None,
            namespace_frozen: false,
            linked_reads: Vec::new(),
            exports: HashMap::new(),
            metadata: None,
        }
    }

    /// Merges global binding definitions from a completed compilation unit into
    /// the module's prior global bindings map (U-REPL §D4).
    pub fn merge_global_bindings(&mut self, unit_bindings: &HashMap<Symbol, bool>) {
        for (&sym, &is_mut) in unit_bindings {
            self.global_bindings.insert(sym, is_mut);
        }
    }

    /// Appends `source` to [`Self::sources`] and returns its index, to be
    /// stamped into every [`Chunk`](crate::chunk::Chunk) compiled from that text
    /// (U-REPL §D2).
    pub fn push_source(&mut self, source: Arc<String>) -> u32 {
        self.sources.push(source);
        (self.sources.len() - 1) as u32
    }

    /// Returns the source text a [`Chunk::source_id`](crate::chunk::Chunk::source_id)
    /// refers to, or `None` if this module never recorded that entry.
    ///
    /// `None` is reachable for a chunk the compiler never stamped (a hand-built
    /// [`Chunk`](crate::chunk::Chunk) defaults to `0`) on a module with no
    /// recorded source at all, so callers must degrade rather than index.
    pub fn source_at(&self, source_id: u32) -> Option<&Arc<String>> {
        self.sources.get(source_id as usize)
    }

    /// Looks up a public export by name symbol.
    #[inline]
    pub fn export(&self, name: Symbol) -> Option<RuntimeExportRef> {
        self.exports.get(&name).copied()
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
    /// references declare with [`crate::value::NIL`]; the real definition later calls
    /// [`Self::set_global`].
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::Message`] if the module already holds
    /// [`MAX_GLOBALS`] globals.
    pub fn declare(&mut self, name: Symbol) -> PhResult<usize> {
        if self.namespace_frozen {
            return Err(RuntimeError::FrozenNamespace(self.name.clone()).into());
        }

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
        self.globals.push(Value::nil());
        Ok(cur)
    }

    /// Declares `name` (if needed) and initializes its slot to `value`.
    ///
    /// # Errors
    ///
    /// Propagates errors from [`Self::declare`] and [`Self::set_global`].
    pub fn define(&mut self, name: Symbol, value: Value) -> PhResult<usize> {
        if self.namespace_frozen {
            return Err(RuntimeError::FrozenNamespace(self.name.clone()).into());
        }
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
        if self.namespace_frozen {
            return Err(RuntimeError::FrozenNamespace(self.name.clone()).into());
        }
        if slot >= self.globals.len() {
            return Err(RuntimeError::Message("Global slot out of bounds".into()).into());
        }
        self.globals[slot] = value;
        Ok(())
    }
}
