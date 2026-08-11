//! The central object [`Heap`]: arena storage keyed by `Copy` generational handles.
//!
//! Realizes [ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md). Every heap
//! object (instances, classes, methods, modules, closures and strings) lives in
//! one [`Heap`] and is referred to by an [`ObjRef`] — a small `Copy` handle, not
//! a pointer. Dereferencing goes through the heap ([`Heap::get`] and the typed
//! accessors), so the cyclic kernel graph (a metaclass that is an instance of
//! itself, [`crate::universe`]) is expressed as handles that point at each other
//! with no ownership paradox and no `Rc<RefCell<T>>` borrow-panic surface.
//!
//! ## Why `slotmap`
//!
//! The arena is a [`slotmap::SlotMap`]. `slotmap` gives us exactly the shape
//! [ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md) asks for with **zero
//! `unsafe`** in this crate:
//!
//! - keys ([`ObjRef`]) are `Copy` and generational, so a stale handle resolves
//!   to a clean `None` rather than undefined behavior (no use-after-free);
//! - interior mutability lives here, in the arena, instead of in a per-object
//!   `RefCell`, which removes the double-borrow panic hazard entirely;
//! - the stable-key design leaves room for a future tracing collector to
//!   relocate or reclaim entries behind the same [`ObjRef`] surface.
//!
//! `generational-arena` was the fallback; `slotmap` was chosen for its richer
//! typed-key ergonomics and `no unsafe`-at-the-call-site guarantee.
//!
//! NaN-boxing of [`crate::value::Value`] stays deferred behind this API
//! ([ADR-0010](../../../docs/adr/accepted/0010-tagged-value-enum.md)); it does not affect
//! the heap contract.

mod accessors;
mod block;
mod bytes;
mod class;
mod closure;
mod fiber;
mod instance;
mod list;
mod map;
mod module;
mod object;
mod pack_builder;
mod range;
mod record;
mod record_literal_builder;
mod string;
mod trace;
mod tuple;
mod upvalue;

pub use block::BlockObject;
pub use bytes::BytesObject;
pub use class::{ClassObject, lookup_method_in_hierarchy};
pub use closure::ClosureObject;
pub use fiber::{FiberObject, FiberResumeMode, FiberStatus};
pub use instance::InstanceObject;
pub use list::ListObject;
pub use map::MapObject;
pub use module::{CORE_MODULE_NAME, MAIN_MODULE_NAME, MAX_GLOBALS, ModuleId, ModuleObject, next_module_id};
pub use object::{BoundMethodObject, FamilyObject, Object};
pub use pack_builder::{ArgumentPackBuilderObject, PackBuilderError};
pub use range::RangeObject;
pub use record::RecordObject;
pub use record_literal_builder::RecordLiteralBuilderObject;
pub use string::StringObject;
pub use trace::{trace_frame, trace_object};
pub use tuple::TupleObject;
pub use upvalue::Upvalue;

use slotmap::{SecondaryMap, SlotMap, new_key_type};

new_key_type! {
    /// A `Copy` generational handle to an [`Object`] stored in the [`Heap`].
    ///
    /// An `ObjRef` is an index-plus-generation into the arena, **not** a
    /// pointer. It is cheap to copy, hash and compare, and comparing two
    /// `ObjRef`s tests *object identity*. Resolve it through the heap
    /// ([`Heap::get`] / [`Heap::class`] / …). Realizes
    /// [ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md).
    pub struct ObjRef;
}

/// An [`ObjRef`] whose referent is statically intended to be a [`ClassObject`].
///
/// This is a documentation alias — it sharpens intent at class-typed fields and
/// signatures without introducing a distinct key type. Resolve it with
/// [`Heap::class`] / [`Heap::class_mut`].
pub type ClassId = ObjRef;

/// The central arena owning every heap [`Object`], keyed by [`ObjRef`].
///
/// The [`crate::vm::VM`] owns exactly one `Heap`. Methods that historically
/// called `self.borrow()` / `self.borrow_mut()` now take `&Heap` / `&mut Heap`
/// and dereference a handle through it. Realizes
/// [ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md).
pub struct Heap {
    /// Backing arena. Generational keys make stale handles resolve to `None`.
    objects: SlotMap<ObjRef, Object>,
    /// Live-object count at which `alloc` next latches `gc_pending`.
    next_gc: usize,
    /// Set by `alloc` when `objects.len()` crosses `next_gc`; serviced only at
    /// the dispatch back-edge safepoint. See Invariant L.
    gc_pending: bool,
    /// Optional `PHALCOM_GC_STRESS=N` cadence, measured in legal dispatch
    /// safepoints rather than allocations.
    gc_stress_interval: Option<usize>,
    /// Safepoints elapsed since the last stress-triggered collection.
    gc_stress_safepoints: usize,
}

const INITIAL_GC_THRESHOLD: usize = 4096;
/// Threshold growth after a **productive** collection — one that reclaimed at
/// least [`GC_LOW_YIELD`] of the heap.
const GC_GROW_FACTOR: f64 = 1.5;
/// Threshold growth after an **unproductive** collection (see
/// [`GC_LOW_YIELD`]): back off harder, because tracing cost scales with the
/// *live* set while the benefit scales with the *garbage*, and a heap that is
/// nearly all-live pays the former to get none of the latter.
///
/// Skynet is the pathological case the flat 1.5× was never chosen for: its
/// ~1.1M fibers are all live until the program ends, so every collection traced
/// the entire heap, freed ~nothing, and re-triggered 1.5× later. `trace_object`
/// alone was ~20% of its samples ([perf-log F11](../../../docs/forge/perf-log/findings.md)).
const GC_UNPRODUCTIVE_GROW_FACTOR: f64 = 4.0;
/// Reclaimed fraction below which a collection is judged unproductive and the
/// next threshold grows by [`GC_UNPRODUCTIVE_GROW_FACTOR`] instead.
const GC_LOW_YIELD: f64 = 0.10;

fn parse_gc_stress_interval(raw: Option<&str>) -> Option<usize> {
    let Some(raw) = raw else {
        return None;
    };
    let raw = raw.trim();
    if raw.is_empty() || raw == "0" {
        return None;
    }
    match raw.parse::<usize>() {
        Ok(interval) if interval > 0 => Some(interval),
        _ => panic!("PHALCOM_GC_STRESS must be 0, 1, or a positive integer safepoint interval; got `{raw}`"),
    }
}

fn gc_stress_interval_from_env() -> Option<usize> {
    let raw = std::env::var("PHALCOM_GC_STRESS").ok();
    parse_gc_stress_interval(raw.as_deref())
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}

impl Heap {
    /// Creates an empty heap.
    pub fn new() -> Self {
        Self {
            objects: SlotMap::with_key(),
            next_gc: INITIAL_GC_THRESHOLD,
            gc_pending: false,
            gc_stress_interval: gc_stress_interval_from_env(),
            gc_stress_safepoints: 0,
        }
    }

    fn insert(&mut self, object: Object) -> ObjRef {
        let id = self.objects.insert(object);
        if self.objects.len() >= self.next_gc {
            self.gc_pending = true; // LATCH ONLY — never collect here (Invariant L)
        }
        id
    }

    /// Returns whether a garbage collection is pending.
    pub fn gc_pending(&self) -> bool {
        self.gc_pending
    }

    /// Returns whether collection is due at this legal VM safepoint.
    ///
    /// Normal threshold latching and stress cadence are independent: stress
    /// never collects from `alloc`, so Invariant L and the single-safepoint
    /// collector contract remain unchanged.
    pub(crate) fn gc_due_at_safepoint(&mut self) -> bool {
        let stress_due = self.gc_stress_interval.is_some_and(|interval| {
            self.gc_stress_safepoints += 1;
            if self.gc_stress_safepoints >= interval {
                self.gc_stress_safepoints = 0;
                true
            } else {
                false
            }
        });
        self.gc_pending || stress_due
    }

    /// Allocates `object` and returns its fresh [`ObjRef`].
    pub fn alloc(&mut self, object: Object) -> ObjRef {
        self.insert(object)
    }

    /// Allocates a [`ClassObject`] and returns its [`ClassId`].
    pub fn alloc_class(&mut self, class: ClassObject) -> ClassId {
        self.insert(Object::Class(Box::new(class)))
    }

    /// Allocates a [`StringObject`] from `value` and returns its [`ObjRef`].
    pub fn alloc_string(&mut self, value: String) -> ObjRef {
        self.insert(Object::Str(StringObject::from_string(value)))
    }

    /// Allocates a [`ListObject`] from `elements` and returns its [`ObjRef`].
    pub fn alloc_list(&mut self, elements: Vec<crate::value::Value>) -> ObjRef {
        self.insert(Object::List(ListObject::new(elements)))
    }

    /// Allocates an empty [`Object::Map`] and returns its [`ObjRef`].
    pub fn alloc_map(&mut self) -> ObjRef {
        self.insert(Object::Map(Box::new(MapObject::new())))
    }

    /// Allocates a compiler-owned dynamic Record literal builder.
    pub(crate) fn alloc_record_literal_builder(&mut self) -> ObjRef {
        self.insert(Object::RecordLiteralBuilder(Box::new(RecordLiteralBuilderObject::new())))
    }

    /// Allocates an empty [`Object::Set`] and returns its [`ObjRef`].
    pub fn alloc_set(&mut self) -> ObjRef {
        self.insert(Object::Set(Box::new(MapObject::new())))
    }

    /// Allocates an [`Object::Bytes`] and returns its [`ObjRef`]
    /// ([PDR-0011](../../../docs/decisions/0011-admit-bytes-native-octet-buffer.md)).
    pub fn alloc_bytes(&mut self, bytes: BytesObject) -> ObjRef {
        self.insert(Object::Bytes(bytes))
    }

    /// Mutably borrows **two distinct** [`BytesObject`]s at once — the
    /// aliasing-safe seam `Bytes::copyInto_(_,_)` needs for its
    /// source→destination memmove (`impl/bytes.md` §2.6). Returns `None` if
    /// the handles are equal, stale, or either is not an [`Object::Bytes`].
    pub fn bytes_pair_mut(&mut self, a: ObjRef, b: ObjRef) -> Option<(&mut BytesObject, &mut BytesObject)> {
        match self.objects.get_disjoint_mut([a, b])? {
            [Object::Bytes(first), Object::Bytes(second)] => Some((first, second)),
            _ => None,
        }
    }

    /// Allocates a positive-arity [`Object::Tuple`]. Product finalization owns
    /// zero normalization and duplicate rejection before this boundary.
    pub(crate) fn alloc_tuple_nonempty(&mut self, values: Box<[crate::value::Value]>, labels: Box<[crate::interner::Symbol]>) -> ObjRef {
        self.insert(Object::Tuple(TupleObject::new(values, labels)))
    }

    /// Allocates a positive-arity [`Object::Record`].
    pub(crate) fn alloc_record_nonempty(&mut self, labels: Box<[crate::interner::Symbol]>, values: Box<[crate::value::Value]>) -> ObjRef {
        self.insert(Object::Record(Box::new(RecordObject::new(labels, values))))
    }

    /// Allocates an [`Object::Range`] from optional endpoint values.
    pub fn alloc_range(&mut self, lower: Option<crate::value::Value>, upper: Option<crate::value::Value>, upper_inclusive: bool) -> ObjRef {
        self.insert(Object::Range(RangeObject::new(lower, upper, upper_inclusive)))
    }

    /// Borrows the [`Object`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or was never allocated in this heap.
    pub fn get(&self, id: ObjRef) -> &Object {
        self.objects.get(id).unwrap_or_else(|| panic!("dangling ObjRef {id:?}"))
    }

    /// Mutably borrows the [`Object`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or was never allocated in this heap.
    pub fn get_mut(&mut self, id: ObjRef) -> &mut Object {
        self.objects.get_mut(id).unwrap_or_else(|| panic!("dangling ObjRef {id:?}"))
    }

    /// Borrows the [`Object`] behind `id`, or `None` if `id` is **stale** —
    /// i.e. its object has been swept.
    ///
    /// The non-panicking counterpart to [`Self::get`], and the observable behind
    /// Invariant M6 (*defined staleness*): after a collection a handle to a swept
    /// object resolves here to `None`, never to a live object and never to
    /// undefined behaviour. The `SlotMap`'s generation bump is what makes this
    /// sound — a recycled slot never answers to the old handle.
    pub fn try_get(&self, id: ObjRef) -> Option<&Object> {
        self.objects.get(id)
    }

    /// Every live handle — **test scaffolding** for GC probes.
    #[doc(hidden)]
    pub fn iter_handles_for_test(&self) -> Vec<ObjRef> {
        self.objects.keys().collect()
    }

    /// Debug-renders the object behind `id` — **test scaffolding**.
    #[doc(hidden)]
    pub fn kind_of_for_test(&self, id: ObjRef) -> &'static str {
        match self.objects.get(id) {
            Some(Object::Instance(_)) => "Instance",
            Some(Object::Class(_)) => "Class",
            Some(Object::Method(_)) => "Method",
            Some(Object::Module(_)) => "Module",
            Some(Object::Closure(_)) => "Closure",
            Some(Object::Str(_)) => "Str",
            Some(Object::Block(_)) => "Closure",
            Some(Object::BoundMethod(_)) => "BoundMethod",
            Some(Object::Upvalue(_)) => "Upvalue",
            Some(Object::List(_)) => "List",
            Some(Object::Fiber(_)) => "Fiber",
            Some(Object::Map(_)) => "Map",
            Some(Object::Set(_)) => "Set",
            Some(Object::Bytes(_)) => "Bytes",
            Some(Object::Tuple(_)) => "Tuple",
            Some(Object::Record(_)) => "Record",
            Some(Object::Range(_)) => "Range",
            Some(Object::Family(_)) => "Family",
            Some(Object::LargeInt(_)) => "LargeInt",
            Some(Object::PackBuilder(_)) => "PackBuilder",
            Some(Object::RecordLiteralBuilder(_)) => "RecordLiteralBuilder",
            None => "<stale>",
        }
    }

    /// Number of live objects currently in the arena.
    ///
    /// The collection trigger keys on this (DEC-GC-B option A: count-based, not
    /// byte-based — post-`Box` the slot size is uniform at 40 B). Also the
    /// observable a GC test asserts against.
    /// The live-object count at which the next `alloc` will latch `gc_pending`.
    ///
    /// Test-only. Exists so the GC tests can assert Invariant L against the
    /// collector's *actual* threshold instead of hard-coding a constant: the
    /// threshold is a tuning decision (see `GC_UNPRODUCTIVE_GROW_FACTOR`) and a
    /// test that bakes in today's number fails the next time it is tuned, which
    /// says nothing about the invariant under test.
    pub fn next_gc_for_test(&self) -> usize {
        self.next_gc
    }

    pub fn live_count(&self) -> usize {
        self.objects.len()
    }

    /// Runs one full **non-moving, precise, stop-the-world mark-sweep**, freeing
    /// every object not reachable from `roots`. Returns the number of objects swept.
    ///
    /// Realises [memory-management.md §3](../../../docs/spec/v0.2/memory-management.md)
    /// per [ADR-0050](../../../docs/adr/accepted/0050-non-moving-mark-sweep-collector.md).
    ///
    /// `roots` is the **complete** root set of
    /// [memory-management.md §2.1](../../../docs/spec/v0.2/memory-management.md),
    /// enumerated by the VM (which alone knows it) — see
    /// [`VM::collect_roots`](crate::vm::VM::collect_roots). A *missed* root frees a
    /// live object (Invariant M3); an *extra* non-root over-retains, which is safe.
    /// Passing roots as a slice rather than a callback is deliberate: the VM must
    /// finish reading itself before it hands `&mut Heap` over, so there is no way
    /// to hold a `&VM` across the sweep.
    ///
    /// **Non-moving** (Invariant M1): a surviving object keeps its `ObjRef` for
    /// life, so inline-cache tags ([ADR-0012](../../../docs/adr/accepted/0012-selector-signature-encoding-and-dispatch.md)),
    /// `==` identity (`Value::value_eq`), and the `Value`s parked in suspended
    /// fiber stacks all stay valid across a collection. A swept handle becomes
    /// stale and resolves to the `dangling ObjRef` diagnostic in [`Self::get`],
    /// never to a live object (Invariant M6) — the `SlotMap` generation bump is
    /// what guarantees that.
    ///
    /// Marking uses an **explicit worklist**, never Rust recursion: a 100k-deep
    /// `List`/`Instance` chain must not overflow the native stack. Cycles — including
    /// the kernel's own (`Metaclass` is an instance of itself) — terminate because
    /// an already-marked object is never re-pushed (Invariant M5); this is why
    /// mark-sweep and not reference counting (ADR-0050 §Alternatives).
    ///
    /// Runs **no finalizers** and resurrects nothing: there is zero `impl Drop` on
    /// the object graph (Invariant M4).
    pub fn collect(&mut self, roots: &[ObjRef]) -> usize {
        // DEC-GC-D option B: a per-collection local, not a persistent field — no
        // stale-marks invariant to hold between cycles.
        let mut marked: SecondaryMap<ObjRef, ()> = SecondaryMap::new();
        let mut gray: Vec<ObjRef> = Vec::new();

        for root in roots {
            // A root may be stale only if the VM's own bookkeeping is corrupt;
            // `contains_key` keeps `collect` total rather than panicking mid-sweep.
            if self.objects.contains_key(*root) && marked.insert(*root, ()).is_none() {
                gray.push(*root);
            }
        }

        let objects = &self.objects;
        while let Some(id) = gray.pop() {
            let Some(object) = objects.get(id) else { continue };
            // Push children straight onto the worklist. `objects` is reborrowed
            // immutably inside the closure alongside `object`'s own borrow (both
            // shared, so they coexist); `marked`/`gray` are locals, disjoint from
            // the arena. This is what keeps marking allocation-free per object.
            trace_object(object, &mut |child| {
                if objects.contains_key(child) && marked.insert(child, ()).is_none() {
                    gray.push(child);
                }
            });
        }

        let before = self.objects.len();
        self.objects.retain(|id, _| marked.contains_key(id));
        let live = self.objects.len();
        // Scale the next threshold by how much this cycle actually reclaimed.
        // A collection that frees almost nothing has just traced the whole live
        // set for no benefit, and growing by a flat 1.5× schedules the same
        // wasted trace again almost immediately (F11).
        let reclaimed_fraction = if before > 0 { (before - live) as f64 / before as f64 } else { 1.0 };
        let factor = if reclaimed_fraction < GC_LOW_YIELD {
            GC_UNPRODUCTIVE_GROW_FACTOR
        } else {
            GC_GROW_FACTOR
        };
        self.next_gc = std::cmp::max(INITIAL_GC_THRESHOLD, (live as f64 * factor) as usize);
        self.gc_pending = false;
        before - live
    }
}

#[cfg(test)]
mod gc_stress_tests {
    use super::*;

    fn heap_with_stress(interval: Option<usize>) -> Heap {
        let mut heap = Heap::new();
        heap.gc_stress_interval = interval;
        heap.gc_stress_safepoints = 0;
        heap.gc_pending = false;
        heap
    }

    #[test]
    fn stress_interval_one_is_due_at_every_safepoint() {
        let mut heap = heap_with_stress(Some(1));
        assert!(heap.gc_due_at_safepoint());
        assert!(heap.gc_due_at_safepoint());
    }

    #[test]
    fn stress_interval_three_repeats_exactly() {
        let mut heap = heap_with_stress(Some(3));
        assert!(!heap.gc_due_at_safepoint());
        assert!(!heap.gc_due_at_safepoint());
        assert!(heap.gc_due_at_safepoint());
        assert!(!heap.gc_due_at_safepoint());
        assert!(!heap.gc_due_at_safepoint());
        assert!(heap.gc_due_at_safepoint());
    }

    #[test]
    fn ordinary_gc_pending_wins_before_stress_interval() {
        let mut heap = heap_with_stress(Some(100));
        heap.gc_pending = true;
        assert!(heap.gc_due_at_safepoint());
    }

    #[test]
    fn stress_configuration_parser_is_strict() {
        assert_eq!(parse_gc_stress_interval(None), None);
        assert_eq!(parse_gc_stress_interval(Some("0")), None);
        assert_eq!(parse_gc_stress_interval(Some("1")), Some(1));
        assert_eq!(parse_gc_stress_interval(Some("100")), Some(100));
    }
}
