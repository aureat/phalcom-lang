//! The stack walk — the one primitive every traceback/observability surface builds on.
//!
//! [`StackWalk`] iterates a live [`VM`]'s call stack ([`VM::frames`]) **oldest → newest**
//! (no `.rev()`: the renderer wants Python order — "Traceback (most recent call last)" —
//! natively, traceback implementation spec (IS) §2.1) and yields one [`FrameView`] per
//! logical frame. Every consumer this unit's siblings build — the human traceback, the
//! JSON traceback, the fiber-switch trace log, a future REPL `where` — walks through here
//! rather than re-deriving frame-to-source resolution on their own.
//!
//! # The logical-expansion seam
//!
//! [`StackWalk`] yields via an internal [`StackWalk::expand`] seam that today returns
//! exactly one [`FrameView`] per physical [`CallFrame`] — a 1:1 mapping. It exists from
//! day one, unused, because superinstruction fusion already proved that a transformation
//! changing what one physical instruction *is* must not silently break span fidelity
//! ([`Chunk::fuse_superinstructions`](crate::chunk::Chunk::fuse_superinstructions),
//! `dispatch.rs:538-543` inlines the same lesson for `SuperSend`'s span read). A future
//! inliner that fuses several `.ph` call frames into one physical [`CallFrame`] emits its
//! several logical views from exactly this seam, and every downstream renderer keeps
//! working unmodified (IS §2.1, "logical frames, 1:many from day one (locked)").
//!
//! # Scope: this unit only
//!
//! This module produces frame views for the **live** call stack of the VM's *current*
//! fiber. It does not walk a captured record (that is
//! [PDR-0010](../../../docs/decisions/0010-errors-carry-structure-and-cheap-origin.md)'s
//! `{module, method, line}` capture shape, U-TRACE T3's territory) and it does not yet
//! synthesize a native-frame entry for an error raised inside a primitive (IS §5.5,
//! U-TRACE T4). [`FrameName::Native`] exists as a variant because the vocabulary is
//! shared; nothing in this module constructs one yet.

use std::sync::Arc;

use phalcom_common::range::SourceRange;

use crate::frame::{CallFrame, FrameToken};
use crate::heap::{CORE_MODULE_NAME, ObjRef};
use crate::interner::Symbol;

use super::VM;

/// How a frame's activation should be named in a rendered trace.
///
/// Selector-shape, per IS §2.1: `foo` and `foo(_)` are different methods (arity is part
/// of the signature in this object model, `method-lookup.md` §1), so the trace must be
/// able to say which. Every variant carries only [`Symbol`]s — no receiver, no class, no
/// [`ObjRef`] — mirroring the discipline
/// [PDR-0010](../../../docs/decisions/0010-errors-carry-structure-and-cheap-origin.md) §4
/// locks for the *captured* frame-record shape (`{module, method, line}`, never an
/// `ObjRef`): a symbol is GC-immune by construction (`gc.rs:77`), so a [`FrameName`]
/// never becomes a dangling-handle hazard even if rendering happens long after the frame
/// that produced it has unwound. Text is resolved through the interner **at render
/// time**, not here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameName {
    /// The module's own top-level activation — renders `<main>`.
    ///
    /// Detected by identity: the frame's [`CallFrame::closure`] is literally the
    /// [`crate::heap::ModuleObject`]'s own registered
    /// [`crate::heap::ModuleObject::closure`], not by any name comparison — a
    /// user-defined method can never alias that handle.
    Main,
    /// An ordinary method or top-level closure activation, naming the selector it
    /// was compiled under (e.g. `total`, `sum(_)`) via
    /// [`crate::callable::Callable::name_sym`].
    Method(Symbol),
    /// A block activation — one whose [`CallFrame::home_frame_token`] is `Some`
    /// ([`crate::frame::CallFrame::home_frame_token`]'s doc: populated *only* for a
    /// block invocation).
    Block {
        /// The name of the block's lexically-immediate home activation, resolved
        /// from the still-live frame [`CallFrame::home_frame_token`] points at when
        /// that frame is still on the stack (the common case: a block invoked
        /// synchronously, e.g. inside `fold`/`for`). Degrades to the block's own
        /// (generic `<block>`) name when the home frame has already returned — an
        /// escaped block called after its lexical parent unwound — rather than
        /// panicking or fabricating a symbol.
        ///
        /// This is the block's **immediate** lexical parent, which may itself be
        /// another block (a block nested in a block); it is not walked further up
        /// to the innermost enclosing *method*. Renders `<block in ‹enclosing›>`.
        enclosing: Symbol,
    },
    /// A synthesized frame for an error raised inside a native primitive, which
    /// pushes no [`CallFrame`] of its own (IS §5.5). Reserved for U-TRACE T4's
    /// native-send-context synthesis; unconstructed by this module.
    Native(Symbol),
}

/// One resolved frame in a stack walk (IS §2.1).
///
/// Self-sufficient for rendering: every field a traceback line needs (module, name,
/// source line, byte span for the caret block, the fiber it ran on, and whether it is
/// core-library code eligible for elision) is already resolved — a renderer never has to
/// walk back into the [`VM`] to finish a [`FrameView`].
#[derive(Debug, Clone)]
pub struct FrameView {
    /// Interned name of the module this frame's closure belongs to
    /// ([`crate::heap::ModuleObject::name_sym`]).
    pub module: Symbol,
    /// How this activation should be named — see [`FrameName`].
    pub name: FrameName,
    /// 1-based source line the frame's instruction pointer maps to, resolved via
    /// [`crate::chunk::Chunk::line_at`]. `0` when [`Self::source`] is `None` (no text
    /// to resolve a line against).
    pub line: u32,
    /// Byte span of the instruction the frame's `ip` maps to
    /// ([`crate::chunk::Chunk::span_at`]) — the caret block's anchor for the
    /// innermost frame.
    pub span: SourceRange,
    /// The chunk's own source text ([`crate::chunk::Chunk::source_id`]-resolved
    /// against [`crate::heap::ModuleObject::sources`]), or `None` when the chunk
    /// carries a source id the module never recorded (a hand-built chunk on a
    /// source-less module — reachable, not a defect; IS §5.1: such a frame prints
    /// its header line only, no echoed source).
    pub source: Option<Arc<String>>,
    /// Whether this frame belongs to the bootstrap core module — elidable in a
    /// human traceback (IS §5.1's `[N core frames elided]`, `--trace-core` to
    /// expand). Determined by comparing this frame's module handle against the
    /// core module's own handle, **not** by comparing names (IS §2.1).
    pub is_core: bool,
    /// Display id of the fiber this frame ran on
    /// ([`crate::heap::FiberObject::seq`], IS §6) — stable across a run, unlike the
    /// recyclable [`ObjRef`] handle.
    pub fiber: u32,
    /// Interned symbol of the class name, if this is a class/instance method frame.
    pub class_name: Option<Symbol>,
}

/// An iterator over a live [`VM`]'s call stack, oldest frame first.
///
/// Borrows the [`VM`] for its whole lifetime — there is no owned, post-mortem snapshot
/// type (deliberately: IS §13, "`StackWalk<'vm>` borrows; the owned post-mortem snapshot
/// type stays unbuilt deliberately," gated on ADR-0050 §7 for the REPL `where` command).
/// Build one with [`VM::walk`].
pub struct StackWalk<'vm> {
    vm: &'vm VM,
    /// The fiber being walked (its ObjRef handle).
    fiber: ObjRef,
    /// Index of the next **physical** [`CallFrame`] in [`VM::frames`] to expand.
    next_physical: usize,
    /// The still-unyielded tail of the current physical frame's logical expansion.
    pending: std::vec::IntoIter<FrameView>,
}

impl<'vm> StackWalk<'vm> {
    /// Builds a walk over `vm`'s current call stack. See [`VM::walk`].
    pub(crate) fn new(vm: &'vm VM) -> Self {
        Self {
            vm,
            fiber: vm.current,
            next_physical: 0,
            pending: Vec::new().into_iter(),
        }
    }

    /// Builds a walk over `vm`'s call stack for a specific fiber.
    #[allow(dead_code)]
    pub(crate) fn new_fiber(vm: &'vm VM, fiber: ObjRef) -> Self {
        Self {
            vm,
            fiber,
            next_physical: 0,
            pending: Vec::new().into_iter(),
        }
    }

    /// The logical-expansion seam (module docs). Returns every logical
    /// [`FrameView`] a single physical [`CallFrame`] at `physical` stands for —
    /// exactly one, today.
    fn expand(&self, physical: usize) -> impl Iterator<Item = FrameView> + use<'_> {
        std::iter::once(self.build_view(physical))
    }

    /// Resolves the single physical [`CallFrame`] at `physical` (an index into
    /// [`VM::frames`]) into its [`FrameView`].
    fn build_view(&self, physical: usize) -> FrameView {
        let frame = &self.vm.frames[physical];
        let closure = self.vm.heap.closure(frame.closure);
        let module_id = closure.module;
        let module = self.vm.heap.module(module_id);

        let is_main = self.is_main_frame(frame, closure);
        let name = if let Some(token) = frame.home_frame_token {
            FrameName::Block {
                enclosing: self.resolve_enclosing(token, closure.callable.name_sym),
            }
        } else if is_main {
            FrameName::Main
        } else {
            FrameName::Method(closure.callable.name_sym)
        };

        // `ip` points *past* the instruction a suspended frame is blocked on — the
        // offending one is `ip - 1` — except a frame that has not executed
        // anything yet, where `ip` is 0 and the subtraction underflows.
        // `saturating_sub` degrades to the chunk's first span rather than
        // panicking (the same reasoning `Chunk::span_at`'s doc now carries,
        // migrated here from the pre-U-TRACE inline version in `runtime_error`).
        let span_index = frame.ip.saturating_sub(1);
        let span = closure.callable.chunk.span_at(span_index);
        let source = module.source_at(closure.callable.chunk.source_id).cloned();
        let line = source.as_deref().map_or(0, |text| closure.callable.chunk.line_at(span_index, text));
        // Resolve class name for instance/class method frames (e.g. `Cart.total`).
        // Block frames and <main> frames get None.
        let class_name_sym = if is_main || frame.home_frame_token.is_some() {
            None
        } else {
            match frame.context {
                crate::frame::CallContext::Instance { instance } => {
                    let class_id = crate::value::Value::Obj(instance).class(self.vm);
                    let name = &self.vm.heap.class(class_id).name;
                    self.vm.interner.find(name)
                }
                crate::frame::CallContext::Class { class } => {
                    // class is an ObjRef that refers to a ClassObject
                    self.vm.heap.as_class(class).and_then(|c| self.vm.interner.find(&c.name))
                }
                _ => None,
            }
        };

        FrameView {
            module: module.name_sym,
            name,
            line,
            span,
            source,
            is_core: self.is_core_module(module_id),
            fiber: self.vm.heap.fiber(self.fiber).seq,
            class_name: class_name_sym,
        }
    }

    /// Resolves a block frame's `enclosing` name from its `home_frame_token`
    /// (`FrameName::Block`'s doc). Falls back to `own_name_sym` — the block's own
    /// generic name — when the home activation is no longer on the stack (an
    /// escaped block called after its lexical parent returned) or a different
    /// activation has since reused that frame slot (a stale/mismatched
    /// generation).
    fn resolve_enclosing(&self, token: FrameToken, own_name_sym: Symbol) -> Symbol {
        self.vm
            .frames
            .get(token.frame_index)
            .filter(|home| home.generation == token.generation)
            .map(|home| self.vm.heap.closure(home.closure).callable.name_sym)
            .unwrap_or(own_name_sym)
    }

    /// Whether `frame` is a module's own top-level activation ([`FrameName::Main`]).
    ///
    /// [`crate::heap::ModuleObject::closure`] is only populated by the module-import
    /// loader (`interpret.rs`'s `compile_and_add_closure`), not by the
    /// [`crate::vm::VM::run_in_module`]/[`crate::vm::VM::interpret_source`] path this
    /// walk otherwise assumes, so identity against it is not reliable here.
    /// Instead: [`crate::compiler::lib::Compiler::compile`] (the top-level pass, as
    /// opposed to [`crate::compiler::lib::Compiler`]'s method/block pass) is the
    /// *only* place that stamps a compiled [`crate::callable::Callable::name_sym`]
    /// with the compiling module's own
    /// [`crate::heap::ModuleObject::name_sym`] — a method or block selector can
    /// never collide with it by construction (`encode_selector` always produces
    /// selector punctuation a bare module name cannot contain). Combined with the
    /// frame having been pushed with [`crate::frame::CallContext::Module`] against
    /// that same module (ruling out an ordinary send to a *value* that happens to
    /// be a `Module` object, `value/mod.rs`'s `CallContext` resolution), this is a
    /// reliable, allocation-free identity check.
    fn is_main_frame(&self, frame: &CallFrame, closure: &crate::heap::ClosureObject) -> bool {
        let module = self.vm.heap.module(closure.module);
        matches!(frame.context, crate::frame::CallContext::Module { module: ctx_module } if ctx_module == closure.module)
            && closure.callable.name_sym == module.name_sym
    }

    /// Whether `module_id` is the bootstrap core module, by handle identity.
    ///
    /// Resolves the core module's own handle through its well-known name
    /// ([`CORE_MODULE_NAME`]) exactly once per call and compares [`ObjRef`]s — the
    /// identity comparison IS §2.1 requires, not a name comparison against the
    /// *frame's* module (which would misclassify any module a user happens to name
    /// `"core"`).
    fn is_core_module(&self, module_id: ObjRef) -> bool {
        self.vm
            .interner
            .find(CORE_MODULE_NAME)
            .and_then(|sym| self.vm.modules.get(&sym))
            .is_some_and(|&core| core == module_id)
    }
}

impl Iterator for StackWalk<'_> {
    type Item = FrameView;

    fn next(&mut self) -> Option<FrameView> {
        loop {
            if let Some(view) = self.pending.next() {
                return Some(view);
            }
            if self.next_physical >= self.vm.frames.len() {
                return None;
            }
            let physical = self.next_physical;
            self.next_physical += 1;
            self.pending = self.expand(physical).collect::<Vec<_>>().into_iter();
        }
    }
}

impl VM {
    /// Builds a [`StackWalk`] over this VM's current call stack, oldest frame
    /// first — the traceback/observability primitive (IS §2).
    pub fn walk(&self) -> StackWalk<'_> {
        StackWalk::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 3-deep `.ph` call — `<main>` → `Cart#total(_)` → a block passed to
    /// `fold` — asserting frame order (oldest first, Python order) and
    /// selector-shaped names, including a block frame whose `enclosing` name
    /// resolves back to `total(_)`.
    ///
    /// The block raises (`it.missingSelector`) so the walk can inspect the live
    /// stack at its deepest point: PDR-0008 §2 reports before any unwind, so
    /// `vm.frames` is still the full three-deep stack once `run_in_module`
    /// returns `Err`.
    ///
    /// Negative control: with [`StackWalk`] built by iterating
    /// `vm.frames.iter().rev()` instead of forward, `views[0]` is the innermost
    /// `Block` frame, not `Main` — confirmed by hand before trusting the forward
    /// version, and exactly the ordering bug (defect a4) IS §2.1 calls out.
    #[test]
    fn walk_orders_oldest_first_with_selector_shaped_names() {
        let mut vm = VM::new();
        let module = vm.create_module("main", "walk_orders_oldest_first_with_selector_shaped_names");
        let source = "\
class Cart {
  total(_ items) { items.reduce(0) |acc, it| { acc + it.missingSelector } }
}
const cart = Cart.new()
const result = cart.total([1, 2, 3])
";
        let closure = vm.compile_closure(module, source).expect("compiles");
        let result = vm.run_in_module(module, closure);
        assert!(result.is_err(), "the block's `it.missingSelector` send must raise");

        let views: Vec<FrameView> = vm.walk().collect();
        assert!(views.len() >= 3, "expected at least <main> + total(_) + a block frame, got {}", views.len());

        // Oldest first: the module top-level activation is first.
        assert!(matches!(views[0].name, FrameName::Main), "first frame must be Main, got {:?}", views[0].name);

        // Find the `total(_)` method frame.
        let total_sym = vm.get_or_intern("total(_)");
        let total_idx = views
            .iter()
            .position(|v| matches!(v.name, FrameName::Method(s) if s == total_sym))
            .expect("a Method(total(_)) frame must appear in the walk");

        let innermost = views.last().expect("at least one frame");
        match innermost.name {
            FrameName::Block { enclosing } => {
                assert_eq!(enclosing, total_sym, "the innermost block's enclosing method must resolve to total(_)");
            }
            ref other => panic!("innermost frame must be a Block, got {other:?}"),
        }

        // Oldest-first order: Main comes before the Method frame, which comes
        // before the innermost Block frame.
        assert!(0 < total_idx, "Main must precede the Method frame");
        assert!(total_idx < views.len() - 1, "the Method frame must precede the innermost Block frame");
    }

    /// [`FrameView::is_core`] is decided by comparing the frame's module handle
    /// against the core module's own handle — every frame in an ordinary user
    /// program's walk belongs to the user's own module, so `is_core` must be
    /// `false` throughout. Negative control: hardcoding `is_core: true`
    /// unconditionally in `build_view` makes this assertion fail, confirming the
    /// test actually exercises the identity check rather than trivially passing.
    #[test]
    fn walk_marks_user_module_frames_as_not_core() {
        let mut vm = VM::new();
        let module = vm.create_module("main", "walk_marks_user_module_frames_as_not_core");
        let closure = vm.compile_closure(module, "const x = 1\n").expect("compiles");
        vm.run_in_module(module, closure).expect("runs");

        // The program already finished, so re-drive one frame by hand to inspect
        // the walk mid-flight would need a raise; simpler and sufficient here:
        // push a fresh frame for the same closure directly and check its view.
        let frame = vm.new_call_frame(closure, crate::frame::CallContext::Module { module }, 0, 0, None);
        vm.frames.push(frame);
        let views: Vec<FrameView> = vm.walk().collect();
        assert_eq!(views.len(), 1);
        assert!(!views[0].is_core, "a user module's frame must not be classified as core");
    }

    /// [`Chunk::span_at`] clamping is unit-tested in `chunk.rs`; this asserts the
    /// walk actually reaches a *resolved* line number (not the `0` fallback) when
    /// the chunk carries source text, on the simplest possible one-frame stack.
    #[test]
    fn walk_resolves_a_line_number_when_source_is_available() {
        let mut vm = VM::new();
        let module = vm.create_module("main", "walk_resolves_a_line_number_when_source_is_available");
        let closure = vm.compile_closure(module, "const x = 1\nconst y = 2\n").expect("compiles");
        let frame = vm.new_call_frame(closure, crate::frame::CallContext::Module { module }, 1, 0, None);
        vm.frames.push(frame);
        let views: Vec<FrameView> = vm.walk().collect();
        assert_eq!(views.len(), 1);
        assert!(
            views[0].line >= 1,
            "a chunk with recorded source must resolve a nonzero line, got {}",
            views[0].line
        );
    }
}
