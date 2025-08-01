use crate::error::{PhResult, RuntimeError};
use crate::interner::Symbol;
use crate::string::{phstring_new, PhString, StringObject};
use crate::value::Value;
use crate::vm::VM;
use phalcom_common::PhRef;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Hard limit on globals per module
pub const MAX_GLOBALS: usize = 1 << 16; // = 65,536

pub const CORE_MODULE_NAME: &str = "core";

pub type ModuleId = u32;

static MODULE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

pub fn next_module_id() -> ModuleId {
    MODULE_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug)]
pub struct ModuleObject {
    pub name: PhRef<StringObject>,
    pub name_sym: Symbol,
    pub globals: RefCell<Vec<Value>>,
    pub name_to_slot: RefCell<HashMap<Symbol, usize>>,
    pub source: Option<Arc<String>>,
}

impl ModuleObject {
    /// Creates an *empty* module.  The caller must register it in
    /// `vm.modules` to keep it alive.
    pub fn new(vm: &mut VM, name: Symbol, source: Option<Arc<String>>) -> Self {
        let name_str = vm.resolve_symbol(name).to_string();
        Self {
            name: phstring_new(name_str),
            name_sym: name,
            globals: RefCell::new(Vec::new()),
            name_to_slot: RefCell::new(HashMap::new()),
            source,
        }
    }

    pub fn name(&self) -> PhRef<StringObject> {
        self.name.clone()
    }

    /// Returns the *symbol* of the module's name.
    #[inline]
    pub fn symbol(&self) -> Symbol {
        self.name_sym
    }

    pub fn to_phalcom_string(&self) -> PhString {
        phstring_new(format!("<module {}>", self.name.borrow().as_str()))
    }

    // ---------------------------------------------------------------------
    //  Declaration / definition
    // ---------------------------------------------------------------------

    /// Reserves a slot for a top‑level variable (may already exist).
    ///
    /// Forward references call this with `Value::Nil`, the real definition
    /// later calls [`set_global`].
    pub fn declare(&self, name: Symbol) -> PhResult<usize> {
        // Fast path: already declared.
        if let Some(&slot) = self.name_to_slot.borrow().get(&name) {
            return Ok(slot);
        }

        // Bounds check.
        let cur = self.name_to_slot.borrow().len();
        if cur >= MAX_GLOBALS {
            return Err(RuntimeError::Message("Too many globals in module".into()).into());
        }

        // Insert.
        self.name_to_slot.borrow_mut().insert(name, cur);
        self.globals.borrow_mut().push(Value::Nil);
        Ok(cur)
    }

    /// Same as [`declare`] but also initialises the slot immediately.
    pub fn define(&self, name: Symbol, value: Value) -> PhResult<usize> {
        let slot = self.declare(name)?;
        self.set_global(slot, value)?;
        Ok(slot)
    }

    // ---------------------------------------------------------------------
    //  Access
    // ---------------------------------------------------------------------

    /// `None` if the name does not exist *yet*.
    #[inline]
    pub fn get(&self, name: Symbol) -> Option<Value> {
        let map = self.name_to_slot.borrow();
        map.get(&name).and_then(|&slot| self.globals.borrow().get(slot).cloned())
    }

    #[inline]
    pub fn get_by_slot(&self, slot: usize) -> Option<Value> {
        self.globals.borrow().get(slot).cloned()
    }

    pub fn set_global(&self, slot: usize, value: Value) -> PhResult<()> {
        let mut globals = self.globals.borrow_mut();
        if slot >= globals.len() {
            return Err(RuntimeError::Message("Global slot out of bounds".into()).into());
        }
        globals[slot] = value;
        Ok(())
    }
}
