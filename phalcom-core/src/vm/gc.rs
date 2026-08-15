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
use crate::heap::{ObjRef, trace_frame};

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

            // Handles a native primitive is holding in a Rust local across a
            // re-entrant call. Reachable from nowhere else for the duration —
            // missing this frees a live object under its holder (Invariant M3).
            temp_roots,

            // Module handles.
            modules,
            main_module,
            last_imported_module,

            // Named class handles.
            classes,
            // Symbols only (no object handles) — not a GC root.
            kernel_class_names: _,
            prelude_names: _,

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
            class_parents: _,
            switch_pending: _,
            native_reentry_depth: _,
            compiler_internal_dispatch_depth: _,
            native_method_contexts: _,
            next_frame_generation: _,
            // A `u32` counter, no object handles — not a GC root.
            next_fiber_seq: _,
            world_version: _,
            start_time: _,
            compile_mode: _,
            strip_contract_metadata: _,
            unit_kind: _,
            trace_core: _,
            native_selector: _,
            native_class: _,
            trace_format_json: _,
            trace_fibers: _,
            resources: _,
            strict_resources: _,
            numeric_policy: _,
            #[cfg(feature = "fiber-pool")]
                fiber_pool: _,
        } = self;

        for frame in frames {
            trace_frame(frame, &mut |id| out.push(id));
        }
        for value in stack {
            if let Some(id) = value.gc_obj_ref() {
                out.push(id);
            }
        }
        out.push(*current);
        out.extend(open_upvalues.values().copied());
        out.extend(ready_queue.iter().copied());
        out.extend(temp_roots.iter().copied());
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
    /// Safepoint-latched triggering ([`service_gc_safepoint`](Self::service_gc_safepoint))
    /// and the [`push_temp_root`](Self::push_temp_root) escape hatch that makes
    /// the native side safe are both live.
    pub fn force_gc(&mut self) -> usize {
        let mut roots = Vec::new();
        self.collect_roots(&mut roots);
        self.heap.collect(&roots)
    }

    /// Roots `value` for the collector until the matching
    /// [`truncate_temp_roots`](Self::truncate_temp_roots)
    /// ([ADR-0050](../../../docs/adr/accepted/0050-non-moving-mark-sweep-collector.md) §7).
    /// Immediates are ignored — they are not heap handles.
    ///
    /// Use when a native primitive holds a handle in a Rust local **across a
    /// re-entrant call** — `block_call`, `send_dynamic`, `invoke_method_object`,
    /// or anything else that re-enters `run_until`. The safepoint inside that
    /// call collects, and [`VM::stack`]/[`VM::frames`] do not describe a value
    /// that lives only in Rust.
    ///
    /// Holding a handle across a mere *allocation* needs no temp root: Invariant
    /// L makes `Heap::alloc` latch rather than collect. The re-entrant case is
    /// the one this exists for.
    pub(crate) fn push_temp_root(&mut self, value: crate::value::Value) {
        if let Some(id) = value.gc_obj_ref() {
            self.temp_roots.push(id);
        }
    }

    /// The current temp-root depth, to be restored by
    /// [`truncate_temp_roots`](Self::truncate_temp_roots).
    ///
    /// Depth-and-truncate rather than push-and-pop because a primitive's
    /// re-entrant call can return through several paths (`Ok`, a raised `Err`, a
    /// non-local return) and truncation is correct on all of them without the
    /// caller counting its own pushes.
    pub(crate) fn temp_root_depth(&self) -> usize {
        self.temp_roots.len()
    }

    /// Releases every temp root pushed since `depth`.
    ///
    /// Idempotent and safe if the stack is already shorter — [`Vec::truncate`]
    /// is a no-op then.
    pub(crate) fn truncate_temp_roots(&mut self, depth: usize) {
        self.temp_roots.truncate(depth);
    }

    /// Pushes `value` onto the operand stack to root it — **test scaffolding.**
    ///
    /// `VM::stack` is `pub(crate)`, so an integration test cannot root an object
    /// the way real code does (by having it on the stack). This exposes exactly
    /// that and nothing more. Not a temp root — for native code holding a fresh
    /// handle across a re-entrant send, use [`push_temp_root`](Self::push_temp_root).
    #[doc(hidden)]
    pub fn push_root_for_test(&mut self, value: crate::value::Value) {
        self.stack.push(value);
    }

    /// Pops the value [`Self::push_root_for_test`] pushed — **test scaffolding.**
    #[doc(hidden)]
    pub fn pop_root_for_test(&mut self) -> Option<crate::value::Value> {
        self.stack.pop()
    }

    /// Services a latched `gc_pending` — **safepoint only**.
    ///
    /// Call this exclusively from the dispatch-loop back-edge, where `VM::stack`/
    /// `frames` are the complete root truth. Never from `Heap::alloc` (Invariant L,
    /// memory-management.md §4), and never mid-opcode: several opcodes have a window
    /// where a value is popped or `split_off` the stack and held only in a Rust local.
    pub(crate) fn service_gc_safepoint(&mut self) {
        if self.heap.gc_due_at_safepoint() {
            self.force_gc();
        }
    }
}
