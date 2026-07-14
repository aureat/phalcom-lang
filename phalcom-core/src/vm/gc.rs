//! Root enumeration and the collection entry point.
//!
//! The VM alone knows the root set, so [`VM::collect_roots`] lives here while the
//! mark/sweep itself lives on the [`Heap`](crate::heap::Heap)
//! ([`Heap::collect`](crate::heap::Heap::collect)). Realises the normative root
//! set of [memory-management.md §2.1](../../../docs/spec/v0.2/memory-management.md)
//! per [ADR-0050](../../../docs/adr/0050-non-moving-mark-sweep-collector.md)
//! (Accepted 2026-07-14; collection ships on by default, DEC-GC-A option A).
//!
//! **The asymmetry that governs this file:** a *missed* root frees a live object
//! (Invariant M3) — silent, arbitrarily-delayed corruption. An *extra* non-root
//! merely over-retains, which is safe and merely wasteful. When unsure whether
//! something is a root, root it.

use super::VM;
use crate::heap::{trace_frame, ObjRef};

impl VM {
    /// Collects the complete root set into `out`.
    ///
    /// Normative set: [memory-management.md §2.1](../../../docs/spec/v0.2/memory-management.md).
    /// Duplicates are fine — [`Heap::collect`](crate::heap::Heap::collect)
    /// de-duplicates through the mark set.
    ///
    /// **Written as an exhaustive destructure on purpose.** Adding a field to
    /// [`VM`] fails to compile here until it is explicitly classified as a root or
    /// a non-root. This is not stylistic: the original hand-audited root table
    /// missed `sealed_classes`, `checking` **and** `ready_queue` (forge finding
    /// F6), the last of which holds fibers that `System.schedule(_)` has enqueued
    /// but not yet resumed — reachable from nowhere else, so missing it would free
    /// a scheduled fiber. Do not replace this with field accesses.
    pub fn collect_roots(&self, out: &mut Vec<ObjRef>) {
        let VM {
            // The arena being collected — not a root.
            heap: _,

            // The running fiber's live state. `VM::frames`/`stack`/`open_upvalues`
            // are the *authoritative mirror* of `current`'s own buffers, which sit
            // empty while it runs (§2.3) — so rooting the mirror is what keeps the
            // running fiber's objects alive, not tracing its `FiberObject`.
            frames,
            stack,
            current,
            open_upvalues,

            // Fibers enqueued by `System.schedule(_)`, not yet started. Reachable
            // from nowhere else until the pump drains them.
            ready_queue,

            // Module handles.
            modules,
            main_module,
            last_imported_module,

            // Named class handles.
            classes,

            // The pinned kernel + the import registry (Invariant M5).
            universe,

            // Sealing class objects (U-ANNOT-LAYOUT `@sealed`/`@variant`). Sits
            // among the Symbol-only maps below and reads like a peer of them, but
            // its *values* are handles.
            sealed_classes,

            // Receivers under `@invariant` re-entrancy checking — the live mirror
            // of `FiberObject::checking` (U-ANNOT-CONTRACTS).
            checking,

            // --- Non-roots (§2.2): Symbols, integers, flags, and Rust-side state.
            // Symbols live in the interner, never the heap, and are never collected.
            interner: _,
            // `ClassLayout` holds only Symbols and slot indices.
            field_layouts: _,
            constructor_aliases: _,
            has_new_construct: _,
            class_parents: _,
            switch_pending: _,
            native_reentry_depth: _,
            next_frame_generation: _,
            start_time: _,
            compile_mode: _,
            strip_contract_metadata: _,
        } = self;

        for frame in frames {
            trace_frame(frame, &mut |id| out.push(id));
        }
        for value in stack {
            if let Some(id) = value.as_obj() {
                out.push(id);
            }
        }
        out.push(*current);
        out.extend(open_upvalues.values().copied());
        out.extend(ready_queue.iter().copied());
        out.extend(modules.values().copied());
        out.extend(main_module.iter().copied());
        out.extend(last_imported_module.iter().copied());
        out.extend(classes.values().copied());
        out.extend(sealed_classes.values().copied());
        out.extend(checking.iter().copied());
        universe.each_handle(&mut |id| out.push(id));
    }

    /// Forces one full mark-sweep now and returns the number of objects swept.
    ///
    /// This is the **unconditional** entry point: it collects wherever it is
    /// called, so it is only safe where `VM::stack`/`frames` are the complete root
    /// truth — i.e. at a dispatch-loop safepoint, never part-way through a native
    /// primitive holding a fresh handle in a Rust local
    /// ([memory-management.md §4](../../../docs/spec/v0.2/memory-management.md)).
    /// Automatic safepoint-latched triggering and the `temp_roots` escape hatch
    /// that makes the native side safe are U-GC step 4; until then this is driven
    /// only by tests.
    pub fn force_gc(&mut self) -> usize {
        let mut roots = Vec::new();
        self.collect_roots(&mut roots);
        self.heap.collect(&roots)
    }

    /// Pushes `value` onto the operand stack to root it — **test scaffolding.**
    ///
    /// `VM::stack` is `pub(crate)`, so an integration test cannot root an object
    /// the way real code does (by having it on the stack). This exposes exactly
    /// that and nothing more. Not a temp-root: the §4 `push_temp_root` escape
    /// hatch for native code holding fresh handles across a re-entrant send is
    /// U-GC step 4.
    #[doc(hidden)]
    pub fn push_root_for_test(&mut self, value: crate::value::Value) {
        self.stack.push(value);
    }

    /// Pops the value [`Self::push_root_for_test`] pushed — **test scaffolding.**
    #[doc(hidden)]
    pub fn pop_root_for_test(&mut self) -> Option<crate::value::Value> {
        self.stack.pop()
    }
}
