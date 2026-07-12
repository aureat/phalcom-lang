use crate::vm::VM;
use std::collections::HashMap;
use std::mem;
use std::ops::Deref;

/// An interned string.
///
/// This is fast to move, clone and compare.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Symbol(pub(crate) u32);

impl Deref for Symbol {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Symbol {
    /// Renders this symbol the way `System.print` and the `toString` message
    /// present it: the interned text prefixed with `#` (U-CORE-4, BD-CORE4-2
    /// Option A). Kept in agreement with [`symbol_tostring`](crate::primitive::symbol::symbol_tostring)
    /// so the message and print paths never disagree (R-INV-4.1). For a
    /// label-free symbol this is byte-stable without the U-LEX `#…` literal;
    /// decoding an interned `move(to:duration:)` into the human `#move(_,to,duration)`
    /// form is U-LEX's job.
    pub fn to_string(&self, vm: &VM) -> String {
        format!("#{}", vm.resolve_symbol(*self))
    }

    /// Renders this symbol's debug form (used by error messages and
    /// diagnostics): the raw interned id, distinct from the display form.
    pub fn to_debug(&self) -> String {
        format!("<symbol {}>", self.0)
    }
}

/// A string interner.
///
/// This particular implementation comes from [matklad's "Fast and Simple Rust Interner" blog post](https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html).
#[derive(Debug)]
pub struct Interner {
    map: HashMap<&'static str, u32>,
    vec: Vec<&'static str>,
    buf: String,
    full: Vec<String>,
}

impl Interner {
    /// Initialize the interner with an initial capacity.
    pub fn with_capacity(cap: usize) -> Self {
        let cap = cap.next_power_of_two();
        Self {
            map: HashMap::default(),
            vec: Vec::new(),
            buf: String::with_capacity(cap),
            full: Vec::new(),
        }
    }

    /// Check if a string is already interned and return its Symbol.
    pub fn find(&self, name: &str) -> Option<Symbol> {
        self.map.get(name).map(|&id| Symbol(id))
    }

    /// Intern a given string.
    pub fn intern(&mut self, name: &str) -> Symbol {
        if let Some(&id) = self.map.get(name) {
            return Symbol(id);
        }

        let name = self.alloc(name);
        let id = self.map.len() as u32;
        self.map.insert(name, id);
        self.vec.push(name);

        let id = Symbol(id);

        debug_assert!(self.lookup(id) == name);
        debug_assert!(self.intern(name) == id);

        id
    }

    /// Get the string associated to a given interning ID.
    pub fn lookup(&self, id: Symbol) -> &str {
        self.vec[id.0 as usize]
    }

    fn alloc(&mut self, name: &str) -> &'static str {
        let cap = self.buf.capacity();
        if cap < self.buf.len() + name.len() {
            let new_cap = (cap.max(name.len()) + 1).next_power_of_two();
            let new_buf = String::with_capacity(new_cap);
            let old_buf = mem::replace(&mut self.buf, new_buf);
            self.full.push(old_buf);
        }

        let interned = {
            let start = self.buf.len();
            self.buf.push_str(name);
            &self.buf[start..]
        };

        unsafe { &*(interned as *const str) }
    }
}
