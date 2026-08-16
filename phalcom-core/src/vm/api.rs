use crate::error::{PhResult, RuntimeError};
use crate::heap::ClassObject;
use crate::heap::ModuleObject;
use crate::heap::{ClassId, ObjRef, Object};
use crate::interner::Symbol;
use crate::method::decode_selector;
use crate::value::Value;

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
    /// [`Value::Obj`] handle ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)).
    pub fn alloc_string_value(&mut self, text: String) -> Value {
        Value::Obj(self.heap.alloc_string(text))
    }

    /// Allocates a bare class named `name`, wired only to `superclass`.
    ///
    /// The metaclass link is left unset; callers such as [`Self::create_class`]
    /// patch it. Realizes the allocate-then-patch bootstrap
    /// ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)).
    pub fn create_single_class(&mut self, name: &str, superclass: Option<ClassId>) -> ClassId {
        let id = self.heap.alloc_class(ClassObject::bare(name));
        self.heap.class_mut(id).set_superclass(superclass);
        id
    }

    /// Follows the metaclass parallel rule
    /// ([ADR-0002](../../../docs/adr/accepted/0002-metaclass-tower-parallel-rule.md)):
    /// the metaclass `"{name}.class"` is an instance of `Metaclass` whose
    /// superclass is `superclass`'s own metaclass (`Class` if `superclass` is
    /// `None`), and the class itself is an instance of that metaclass with the
    /// requested `superclass`.
    /// requested `superclass`.
    pub fn create_class(&mut self, module: ObjRef, name: &str, superclass: Option<ClassId>) -> ClassId {
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

        let class_key = super::ClassKey { module, name: name_sym };
        if let Some(layout) = self.field_layouts.get(&class_key).cloned() {
            self.heap.class_mut(class).field_slots = layout.field_slots;
            self.heap.class_mut(class).field_count = layout.field_count;
            self.heap.class_mut(metaclass).field_slots = layout.static_field_slots;
            self.heap.class_mut(metaclass).field_count = layout.static_field_count;
            self.heap.class_mut(class).static_slots = vec![Value::Nil; layout.static_field_count as usize].into_boxed_slice();
        }

        let meta_key = super::ClassKey { module, name: meta_sym };
        self.classes.insert(class_key, class);
        self.classes.insert(meta_key, metaclass);

        class
    }

    /// Allocates an ad-hoc module with a synthetic identity and registers it.
    pub fn create_module(&mut self, logical_name: &str, abs_path: &str) -> ObjRef {
        let id = phalcom_modules::ModuleId::synthetic(logical_name);
        self.create_module_with_id(id, crate::heap::ModuleKind::Module, logical_name, abs_path)
    }

    /// Allocates a module with semantic identity and registers it in the module registry.
    pub fn create_module_with_id(&mut self, id: phalcom_modules::ModuleId, kind: crate::heap::ModuleKind, logical_name: &str, abs_path: &str) -> ObjRef {
        let module_sym = self.interner.intern(logical_name);
        let module = ModuleObject::new(id.clone(), kind, logical_name.to_string(), module_sym, abs_path.to_string(), None, false);
        let obj_ref = self.heap.alloc(Object::Module(Box::new(module)));
        self.module_registry.insert(id, crate::modules::ModuleRecord::prepared(obj_ref));
        obj_ref
    }

    /// Defines global `name_sym = val` directly on `module`.
    ///
    /// # Errors
    ///
    /// Propagates [`ModuleObject::define`](crate::heap::ModuleObject::define)
    /// errors (e.g. too many globals).
    pub fn define_global(&mut self, module: ObjRef, name_sym: Symbol, val: Value) -> PhResult<usize> {
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

    /// Maps a native RuntimeError to its surface kind Symbol value, if applicable.
    pub fn error_kind_symbol(&mut self, err: &RuntimeError) -> Option<Value> {
        let sym_str = match err {
            RuntimeError::DivideByZero => "divideByZero",
            RuntimeError::NonFiniteNumber(_) => "nonFiniteNumber",
            RuntimeError::NumericLimit(_) => "numericLimit",
            RuntimeError::InvalidShift(_) => "invalidShift",
            RuntimeError::InvalidBitIndex(_) => "invalidBitIndex",
            RuntimeError::UndefinedNumericOperation(_) => "undefinedNumericOperation",
            RuntimeError::ConcurrentMutation { .. } => "concurrentMutation",
            RuntimeError::SelectorPatternMismatch { .. } => "selectorPatternMismatch",
            RuntimeError::DepthExceeded { .. } => "depthExceeded",
            RuntimeError::DeadFrameError => "deadFrame",
            RuntimeError::NumericConversion { .. } => "numericConversion",
            RuntimeError::NumericOverflow { .. } => "numericOverflow",
            RuntimeError::NumericText { .. } => "numericText",
            RuntimeError::AbstractClass { .. } => "abstractClass",
            RuntimeError::InvalidHash { .. } => "invalidHash",
            RuntimeError::Type { .. }
            | RuntimeError::TypeConversion { .. }
            | RuntimeError::Arity { .. }
            | RuntimeError::InstantiateNonClass { .. }
            | RuntimeError::AccessFieldsNonInstance { .. }
            | RuntimeError::IncompatibleMethodLayout { .. } => "type",
            _ => return None,
        };
        let sym = self.interner.intern(sym_str);
        Some(Value::Symbol(sym))
    }

    /// Converts a native numeric `RuntimeError` into a surface `RuntimeError::Raise`
    /// carrying an `Error` instance with the correct `kind` symbol.
    ///
    /// This is the **single numeric-error construction path** (U-NUMBERS-05 §1):
    /// every numeric failure goes through here so the surface `error.kind` is
    /// always set at the moment of raise, not deferred to a fiber-boundary wrap.
    pub fn raise_numeric_error(&mut self, err: RuntimeError) -> crate::error::PhError {
        let rendered = err.to_string();
        let kind_val = self.error_kind_symbol(&err);
        let error_cls = self.universe.classes.error_class;
        let field_count = self.heap.class(error_cls).field_count;
        let mut inst = crate::heap::InstanceObject::new(error_cls, field_count);
        inst.slots[0] = self.alloc_string_value(rendered.clone());
        if let Some(k) = kind_val {
            inst.slots[1] = k;
        }
        let error = Value::Obj(self.heap.alloc(crate::heap::Object::Instance(inst)));
        crate::error::PhError::Runtime(RuntimeError::Raise {
            error,
            rendered,
            traceback: None,
            help: None,
        })
    }

    /// Pushes a call frame, refusing once the `.ph` call-depth ceiling is reached.
    ///
    /// Every recursive frame push goes through here so the ceiling cannot be
    /// bypassed by a new call site
    /// ([PDR-0007](../../../docs/decisions/0007-bounded-call-depth-and-native-reentrancy.md) §1).
    ///
    /// # Errors
    ///
    /// [`RuntimeError::DepthExceeded`] when the frame stack is already at
    /// [`MAX_CALL_DEPTH`](crate::vm::MAX_CALL_DEPTH).
    pub(crate) fn push_frame(&mut self, frame: crate::frame::CallFrame) -> PhResult<()> {
        if self.frames.len() >= crate::vm::MAX_CALL_DEPTH {
            return Err(RuntimeError::DepthExceeded {
                what: "call depth",
                limit: crate::vm::MAX_CALL_DEPTH,
            }
            .into());
        }
        self.frames.push(frame);
        Ok(())
    }

    /// Checks the native re-entrancy ceiling before a recursive `run_until`.
    ///
    /// This counter is checked *before* recursing, not after: each native re-entry
    /// consumes a real Rust stack frame, and overflowing that aborts the process —
    /// there is no after (PDR-0007 §4).
    ///
    /// # Errors
    ///
    /// [`RuntimeError::DepthExceeded`] when already at
    /// [`MAX_NATIVE_REENTRY`](crate::vm::MAX_NATIVE_REENTRY).
    pub(crate) fn check_native_reentry(&self) -> PhResult<()> {
        if self.native_reentry_depth >= crate::vm::MAX_NATIVE_REENTRY {
            return Err(RuntimeError::DepthExceeded {
                what: "native re-entrancy depth",
                limit: crate::vm::MAX_NATIVE_REENTRY,
            }
            .into());
        }
        Ok(())
    }

    /// Discards all execution state at a REPL cell boundary, closing any open
    /// upvalues first (U-REPL §D10).
    ///
    /// `run_in_module`'s raw `frames.clear(); stack.clear()` is **not** equivalent:
    /// `open_upvalues` is keyed by absolute value-stack index, so clearing the stack
    /// beneath it aliases the previous cell's captured slots onto the next cell's
    /// values — silent corruption, not a crash.
    pub fn unwind_cell(&mut self) {
        self.unwind_to(0, 0);
    }

    /// Runs a compiled cell top-level closure within `module`, returning its value,
    /// and unwinding execution state at the cell boundary (U-REPL §D1, §D10).
    pub fn run_cell(&mut self, module: ObjRef, closure: ObjRef) -> PhResult<Value> {
        let frame = self.new_call_frame(closure, crate::frame::CallContext::Module { module }, 0, 0, None);
        self.push_frame(frame)?;

        // Report *before* unwinding. `unwind_cell` truncates `frames` to zero, and
        // `runtime_error` builds its traceback by walking exactly that vector — so a
        // caller that reported after `run_cell` returned always rendered an empty
        // traceback, which is what every REPL runtime error did
        // (PDR-0008 §2). `runtime_error` always returns `Err`, so the `map` only
        // adjusts the type.
        let res = match self.run() {
            Ok(value) => Ok(value),
            Err(err) => self.runtime_error(err).map(|()| Value::Nil),
        };

        self.unwind_cell();
        res
    }

    /// Captures a slice of stack frames as `FrameRecord`s.
    pub fn capture_frames(&mut self, start_idx: usize) -> Vec<crate::error::FrameRecord> {
        let mut records = Vec::new();
        let end_idx = self.frames.len();
        for idx in start_idx..end_idx {
            let frame = &self.frames[idx];
            let closure = self.heap.closure(frame.closure);
            let module_id = closure.module;
            let module = self.heap.module(module_id);
            let module_sym = module.name_sym;

            let is_main = matches!(frame.context, crate::frame::CallContext::Module { module: ctx_module } if ctx_module == closure.module)
                && closure.callable.name_sym == module.name_sym;
            let method_sym = if is_main {
                self.interner.intern("<main>")
            } else if let Some(token) = frame.home_frame_token {
                let enclosing = self
                    .frames
                    .get(token.frame_index)
                    .filter(|home| home.generation == token.generation)
                    .map(|home| self.heap.closure(home.closure).callable.name_sym)
                    .unwrap_or(closure.callable.name_sym);
                let enclosing_str = self.resolve_symbol(enclosing);
                let name_str = format!("<closure in {}>", enclosing_str);
                self.interner.intern(&name_str)
            } else {
                closure.callable.name_sym
            };

            let span_index = frame.ip.saturating_sub(1);
            let span = closure.callable.chunk.span_at(span_index);
            let source = module.source_at(closure.callable.chunk.source_id);
            let line = source.as_ref().map_or(0, |text| closure.callable.chunk.line_at(span_index, text));

            records.push(crate::error::FrameRecord::Normal {
                module: module_sym,
                method: method_sym,
                line,
            });
        }
        records
    }

    /// Captures the parked frames of `fiber_ref` as `FrameRecord`s.
    pub fn capture_parked_frames(&mut self, fiber_ref: ObjRef) -> Vec<crate::error::FrameRecord> {
        let mut records = Vec::new();
        let fiber = self.heap.fiber(fiber_ref);
        let end_idx = fiber.frames.len();
        for idx in 0..end_idx {
            let frame = &fiber.frames[idx];
            let closure = self.heap.closure(frame.closure);
            let module_id = closure.module;
            let module = self.heap.module(module_id);
            let module_sym = module.name_sym;

            let is_main = matches!(frame.context, crate::frame::CallContext::Module { module: ctx_module } if ctx_module == closure.module)
                && closure.callable.name_sym == module.name_sym;
            let method_sym = if is_main {
                self.interner.intern("<main>")
            } else if let Some(token) = frame.home_frame_token {
                let enclosing = fiber
                    .frames
                    .get(token.frame_index)
                    .filter(|home| home.generation == token.generation)
                    .map(|home| self.heap.closure(home.closure).callable.name_sym)
                    .unwrap_or(closure.callable.name_sym);
                let enclosing_str = self.resolve_symbol(enclosing);
                let name_str = format!("<closure in {}>", enclosing_str);
                self.interner.intern(&name_str)
            } else {
                closure.callable.name_sym
            };

            let span_index = frame.ip.saturating_sub(1);
            let span = closure.callable.chunk.span_at(span_index);
            let source = module.source_at(closure.callable.chunk.source_id);
            let line = source.as_ref().map_or(0, |text| closure.callable.chunk.line_at(span_index, text));

            records.push(crate::error::FrameRecord::Normal {
                module: module_sym,
                method: method_sym,
                line,
            });
        }
        records
    }
}
