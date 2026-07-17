# The metaclass tower — source map at HEAD

Read-only investigation. Nothing in the tree was modified. Claims are marked
VERIFIED (read the exact line, or ran the program and matched output) or
INFERRED. Anchors are `file.rs::Type::method` @ ~Lxxx (symbol-first; line
numbers are current as of this read but rot faster than symbols).

## THE ANSWER: a `Copy` generational arena handle (`ClassId = ObjRef`) — NOT `Rc`, NOT a raw pointer, NOT an embedded sub-object. The apex "self-cycle" is real but is a 2-node loop, not a 1-node self-reference — and this contradicts two doc comments in the source itself.

The settling line, `heap/mod.rs` L80:

```rust
pub type ClassId = ObjRef;
```

`ObjRef` (`heap/mod.rs` L64-73) is a `slotmap::new_key_type!` — an
index-plus-generation `Copy` key into a `slotmap::SlotMap<ObjRef, Object>`
(`heap/mod.rs::Heap` L88-97). It is **not** `Rc<RefCell<_>>`/`Weak`, **not**
`&ClassObject`/`*const`, and **not** an embedded/owned sub-object — `ClassObject`
stores its metaclass and superclass as `ClassId` *values*, resolved through
`&Heap` on every access ([ADR-0009](../../adr/accepted/0009-handle-arena-heap.md)).
This matches the "name/handle resolved through a heap" candidate exactly.

**The apex cycle, corrected.** Both `heap/class.rs`'s module doc (L1-8) and the
`ClassObject.class` field doc (L28-29) describe the kernel wiring as *"a handle
that points at itself"* and say *"For `Metaclass` this is a self-cycle"* —
citing `Metaclass.class == Metaclass` as the example. **This is not what the
bootstrap code builds, and it is not what running the program shows.** The
actual apex is `Behavior`/`Class`/`Metaclass` plus a **distinct fourth row**,
`Metaclass class` (`metaclass_metaclass` in `create_core_classes`), and the
cycle closes across *two* rows:

```
Metaclass.class            == Metaclass class      (a different ClassId)
(Metaclass class).class    == Metaclass
```

No row's `class` field ever equals its own `ClassId` at HEAD — `heap.alloc_class`
returns a fresh slotmap key on every call, and `create_core_classes` never
assigns a row's own freshly-minted id back into itself. Live proof (see §6):
`Metaclass.class == Metaclass` prints **`false`**; `Metaclass.class.class ==
Metaclass` prints **`true`**. `verify_invariants` (§4) enforces exactly the
2-node loop (`invariants.rs` L57-58: *"Metaclass.class should be Metaclass
class"*), never a 1-node self-reference. So: the *type* (`ClassId` as a plain
`Copy` value) could represent a literal self-loop — nothing in the
representation forbids `class == self`   — but the kernel HEAD actually builds
does not use one; the two doc comments calling it "a self-cycle" are
imprecise relative to their own bootstrap code. This is exactly the kind of
mismatch worth flagging plainly rather than pattern-matching to the docstring.

(`docs/learn/metaclass-tower/recon.md` §1, an earlier phase's scouting note in
this same investigation, repeats the docstring's "handle that points at
itself" / `Metaclass.class == Metaclass` framing without independently
running the program — this source-map corrects it.)

---

## 1. The core row — `heap/class.rs::ClassObject` @ ~L25

Module-level doc, `heap/class.rs` L1-8 (quoted verbatim, including the claim
this doc corrects above):

```rust
//! Classes and metaclasses — the rows of the object-model tower.
//!
//! A [`ClassObject`] is a heap [`Object`](crate::heap::Object) referenced by a
//! [`ClassId`]. Its links to its metaclass and superclass are plain [`ClassId`]
//! handles ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)), so the
//! kernel's cyclic wiring (e.g. `Metaclass.class == Metaclass`) is just a handle
//! that points at itself — no `Rc`, no `Weak`, no `RefCell` (`object-model.md`
//! §5–6). Method lookup walks the superclass chain through the heap.
```

Confirmed: this is exactly the "cyclic wiring == a handle that points at
itself" language the task asked me to check — but as shown above, the
concrete example it names (`Metaclass.class == Metaclass`) is false at HEAD;
the real loop is 2 rows, not 1 (§THE ANSWER, §6).

Full struct, `heap/class.rs::ClassObject` L25-66:

```rust
#[derive(Debug, Clone)]
pub struct ClassObject {
    /// The class's display name (e.g. `"Number"`, `"Number.class"`).
    pub name: String,
    /// Handle to this class's metaclass (its `class`). For `Metaclass` this is a
    /// self-cycle.
    pub class: ClassId,
    /// Handle to this class's superclass, or `None` at the tower's apex
    /// (`Object`).
    pub superclass: Option<ClassId>,
    /// Methods defined directly on this class, keyed by selector [`Symbol`].
    pub methods: MethodsMap,
    /// Instance fields, keyed by name [`Symbol`] to their slot offset (ADR-0011).
    pub field_slots: IndexMap<Symbol, u16>,
    /// Number of instance slots (ADR-0011).
    pub field_count: u16,
    /// Class-side stored fields (static fields), stored as a fixed-size slot vector (ADR-0017).
    pub static_slots: Box<[Value]>,
    /// The per-class **base-name index** (selectors.md §3.1, U16-Open): …
    pub base_names: HashMap<Symbol, Vec<Symbol>>,
    /// Attribute instances attached via `Object#__attach` …
    pub attributes: Vec<Value>,
    /// Set by `Object#__freezeAttributes` once class-definition codegen has
    /// finished attaching every class-level attribute — further `__attach`
    /// calls are rejected (`attr.frozen`).
    pub attributes_frozen: bool,
}
```

(`base_names`/`attributes`/`attributes_frozen` doc comments elided above for
length — not load-bearing to the tower question; full text is in the source.)

`ClassId` and `ObjRef` are the same type (`ClassId = ObjRef`, `heap/mod.rs`
L80) — `ClassId` is purely a **documentation alias**, not a distinct key type
(`heap/mod.rs` L75-80: *"This is a documentation alias — it sharpens intent at
class-typed fields and signatures without introducing a distinct key type."*).

`ClassObject::bare(name)` (`heap/class.rs` L93-106) creates an **unwired**
row: `class: ClassId::default()` (the null slotmap key), `superclass: None`.
Every kernel/core row starts here; bootstrap patches `class`/`superclass`
afterward (§4) — this is the "allocate-then-patch" the module doc and ADR-0009
both name.

---

## 2. The superclass walk — `heap/class.rs::lookup_method_in_hierarchy` @ ~L74

```rust
pub fn lookup_method_in_hierarchy(heap: &Heap, mut class: ClassId, selector: Symbol) -> Option<ObjRef> {
    loop {
        let current = heap.class(class);
        if let Some(&method) = current.methods.get(&selector) {
            return Some(method);
        }
        match current.superclass {
            Some(superclass) => class = superclass,
            None => return None,
        }
    }
}
```

Traversal is **handle-by-handle through the heap**, not a borrowed reference
chain: `class` is reassigned to the next `ClassId` and re-resolved via
`heap.class(class)` each loop iteration. Per its own doc comment (L68-73):
*"Traversal follows `ClassId` handles through `heap`, so it neither borrows
nor clones any object across steps."* Each iteration's `&ClassObject` borrow
from `heap.class(class)` is dropped before the next iteration re-borrows —
there is no held reference spanning more than one hop, which is exactly what
lets this walk a cyclic graph (the kernel apex) without a borrow-checker
conflict.

---

## 3. Two distinct "class-of" paths

### 3a. Row→row: `Heap::class(ClassId)` — `heap/accessors.rs::Heap::class` @ ~L23

```rust
pub fn class(&self, id: ClassId) -> &ClassObject {
    match self.get(id) {
        Object::Class(class) => class,
        _ => panic!("ObjRef {id:?} is not a ClassObject"),
    }
}
```

Resolves an already-known `ClassId` to its `ClassObject` row. **Panics** if
`id` is stale (swept and generation-bumped) or does not name an
`Object::Class` variant — via `Heap::get` (`heap/mod.rs` L182-189), which
itself panics with `"dangling ObjRef {id:?}"` on a stale handle. This is a
pure heap accessor; it has no notion of "value" or "immediate" — it only
ever answers "give me the row this handle names."

### 3b. Value→class: `Value::class(&self, vm: &VM)` — `value/mod.rs::Value::class` @ ~L121

```rust
pub fn class(&self, vm: &VM) -> ClassId {
    match self {
        Value::Nil => vm.universe.classes.nil_class,
        Value::Bool(b) => {
            if *b {
                vm.universe.classes.true_class
            } else {
                vm.universe.classes.false_class
            }
        }
        Value::Number(_) => vm.universe.classes.number_class,
        Value::Symbol(_) => vm.universe.classes.symbol_class,
        Value::Obj(id) => match vm.heap.get(*id) {
            Object::Instance(instance) => instance.class,
            Object::Class(class) => class.class,
            Object::Method(_) => vm.universe.classes.method_class,
            Object::Module(_) => vm.universe.classes.module_class,
            Object::Str(_) => vm.universe.classes.string_class,
            Object::Closure(_) => vm.universe.classes.block_class,
            Object::Block(_) => vm.universe.classes.block_class,
            Object::BoundMethod(_) => vm.universe.classes.block_class,
            Object::List(_) => vm.universe.classes.list_class,
            Object::Fiber(_) => vm.universe.classes.fiber_class,
            Object::Map(_) => vm.universe.classes.map_class,
            Object::Set(_) => vm.universe.classes.set_class,
            Object::Tuple(_) => vm.universe.classes.tuple_class,
            Object::Range(_) => vm.universe.classes.range_class,
            Object::Family(_) => vm.universe.classes.family_class,
            Object::Upvalue(_) => panic!("upvalues are not surface values"),
        },
    }
}
```

This is a **different function on a different type** from 3a — it maps an
arbitrary `Value` (including every immediate: `Nil`, `Bool`, `Number`,
`Symbol`, none of which carry a `ClassId` field) onto its `ClassId` by
dispatch on the `Value`/`Object` tag, reading fixed handles off
`vm.universe.classes` (`CoreClasses`, §4) for immediates and natives, or the
row's own `.class` field for `Object::Instance`/`Object::Class`. Note the
`Object::Class(class) => class.class` arm: **a class value's `.class` is its
metaclass**, obtained by reading the row's `class` field — this is where 3a
and 3b connect (3b delegates to the same field 3a exposes, once it has
resolved down to a class row). `Value::lookup_method` (`value/mod.rs` L170-173)
composes 3b with §2's `lookup_method_in_hierarchy`, which is the whole
dispatch path from a live `Value` to a resolved method handle.

---

## 4. The bootstrap tie — `universe/core_classes.rs::Universe::create_core_classes` and `universe/invariants.rs::Universe::verify_invariants`

Two corrections to the task's own file-path guesses, both VERIFIED: `universe.rs`
is now a directory, `phalcom-core/src/universe/{mod.rs,core_classes.rs,
invariants.rs,primitives.rs}` — `create_core_classes` lives in `core_classes.rs`
(not `mod.rs`), `verify_invariants` in `invariants.rs`. `universe/mod.rs`'s own
module doc (L1-17) names the same seven-step order the ADR-0002 Decision
section states.

### 4a. `create_core_classes` — full, `universe/core_classes.rs` L15-195

```rust
pub fn create_core_classes(heap: &mut Heap) -> CoreClasses {
    // 1. Allocate the 8 apex rows bare (object-model.md §6 step 1).
    let object_class = heap.alloc_class(crate::heap::ClassObject::bare("Object"));
    let behavior_class = heap.alloc_class(crate::heap::ClassObject::bare("Behavior"));
    let class_class = heap.alloc_class(crate::heap::ClassObject::bare("Class"));
    let metaclass_class = heap.alloc_class(crate::heap::ClassObject::bare("Metaclass"));
    let object_metaclass = heap.alloc_class(crate::heap::ClassObject::bare("Object class"));
    let behavior_metaclass = heap.alloc_class(crate::heap::ClassObject::bare("Behavior class"));
    let class_metaclass = heap.alloc_class(crate::heap::ClassObject::bare("Class class"));
    let metaclass_metaclass = heap.alloc_class(crate::heap::ClassObject::bare("Metaclass class"));

    // 2. Wire instance-of (§6 step 2): every metaclass is an instance of
    //    Metaclass; Metaclass itself is an instance of Metaclass class,
    //    closing the loop; each ordinary class is an instance of its own
    //    metaclass.
    heap.class_mut(object_metaclass).class = metaclass_class;
    heap.class_mut(behavior_metaclass).class = metaclass_class;
    heap.class_mut(class_metaclass).class = metaclass_class;
    heap.class_mut(metaclass_metaclass).class = metaclass_class;
    heap.class_mut(metaclass_class).class = metaclass_metaclass;
    heap.class_mut(object_class).class = object_metaclass;
    heap.class_mut(behavior_class).class = behavior_metaclass;
    heap.class_mut(class_class).class = class_metaclass;

    // 3. Wire instance-side superclasses (§6 step 3).
    heap.class_mut(object_class).superclass = None;
    heap.class_mut(behavior_class).superclass = Some(object_class);
    heap.class_mut(class_class).superclass = Some(behavior_class);
    heap.class_mut(metaclass_class).superclass = Some(behavior_class);

    // 4. Wire metaclass-side superclasses by the parallel rule (§6 step 4,
    //    ADR-0002): (X class).superclass == (X.superclass) class.
    heap.class_mut(object_metaclass).superclass = Some(class_class);
    heap.class_mut(behavior_metaclass).superclass = Some(object_metaclass);
    heap.class_mut(class_metaclass).superclass = Some(behavior_metaclass);
    heap.class_mut(metaclass_metaclass).superclass = Some(behavior_metaclass);

    // 5. The remaining core classes, each with its own metaclass wired by
    //    the same parallel rule (§6 step 5).
    let number_class = make_core_class(heap, "Number", object_class, metaclass_class);
    // … (String, Nil, Bool, True, False, Function, Block, Method, Symbol,
    //    Module, System, Option, Some, None, Iterable, List, Map, Set,
    //    Tuple, Range, Message, Error, MessageNotUnderstood, Fiber,
    //    CannotYieldAcrossNativeFrame, Family — all via the same
    //    `make_core_class` helper, elided here; full list in source)

    CoreClasses { /* … 32 handles, elided … */ }
}
```

(Body elided above after step 5's first call for length — every one of the
~24 remaining core rows is a `make_core_class(heap, name, superclass,
metaclass_class)` call; the full unelided text is in
`phalcom-core/src/universe/core_classes.rs` L54-194.) The **allocate-then-patch**
pattern is exactly steps 1 vs. 2-4: step 1 allocates all 8 apex rows with
`ClassObject::bare` (null `class`, `superclass: None`), then steps 2-4
`heap.class_mut(id).class = …` / `.superclass = …` **after the fact**, once
every row's `ClassId` is already known — this is what makes the cyclic wiring
constructible at all (you cannot pass `metaclass_class`'s id into
`object_metaclass`'s constructor before `metaclass_class` itself has been
allocated, so every id is minted first and every field patched second).

### 4b. `make_core_class` helper — `universe/core_classes.rs` L205-221

```rust
fn make_core_class(heap: &mut Heap, name: &str, superclass: ClassId, metaclass_class: ClassId) -> ClassId {
    let metaclass_superclass = heap.class(superclass).class;

    let metaclass = heap.alloc_class(crate::heap::ClassObject::bare(&format!("{name} class")));
    {
        let meta = heap.class_mut(metaclass);
        meta.class = metaclass_class;
        meta.superclass = Some(metaclass_superclass);
    }
    let class = heap.alloc_class(crate::heap::ClassObject::bare(name));
    {
        let class_ref = heap.class_mut(class);
        class_ref.class = metaclass;
        class_ref.superclass = Some(superclass);
    }
    class
}
```

Signature: `fn make_core_class(heap: &mut Heap, name: &str, superclass: ClassId, metaclass_class: ClassId) -> ClassId`.
This is the allocate-a-class-plus-its-metaclass-pair helper the task asked
for — every non-apex core class goes through it, applying the parallel rule
(`meta.superclass = superclass.class`) automatically. Its doc comment (L198-204)
states the precondition explicitly: *"`superclass` must already have its
`class` link wired"* — i.e. callers must respect allocation order (seen in
`create_core_classes`'s comment about `Method` needing `Function` allocated
first, L69-75).

### 4c. `verify_invariants` — full, `universe/invariants.rs` L19-193

```rust
pub fn verify_invariants(&self, heap: &Heap) -> Result<(), String> {
    let c = &self.classes;

    let object_metaclass = heap.class(c.object_class).class;
    let behavior_metaclass = heap.class(c.behavior_class).class;
    let class_metaclass = heap.class(c.class_class).class;
    let metaclass_metaclass = heap.class(c.metaclass_class).class;

    if object_metaclass == c.object_class {
        return Err("Object.class must not equal Object itself".to_string());
    }
    if heap.class(c.behavior_class).superclass != Some(c.object_class) {
        return Err("Behavior.superclass should be Object".to_string());
    }
    if heap.class(c.class_class).superclass != Some(c.behavior_class) {
        return Err("Class.superclass should be Behavior".to_string());
    }
    if heap.class(c.metaclass_class).superclass != Some(c.behavior_class) {
        return Err("Metaclass.superclass should be Behavior".to_string());
    }
    if heap.class(c.object_class).superclass.is_some() {
        return Err("Object.superclass should be None".to_string());
    }

    if heap.class(object_metaclass).class != c.metaclass_class {
        return Err("Object.class.class should be Metaclass".to_string());
    }
    if heap.class(behavior_metaclass).class != c.metaclass_class {
        return Err("Behavior.class.class should be Metaclass".to_string());
    }
    if heap.class(class_metaclass).class != c.metaclass_class {
        return Err("Class.class.class should be Metaclass".to_string());
    }
    if heap.class(metaclass_metaclass).class != c.metaclass_class {
        return Err("Metaclass.class.class should be Metaclass".to_string());
    }
    // The closed loop: Metaclass.class == Metaclass class, and
    // (Metaclass class).class == Metaclass.
    if heap.class(c.metaclass_class).class != metaclass_metaclass {
        return Err("Metaclass.class should be Metaclass class".to_string());
    }

    if heap.class(object_metaclass).superclass != Some(c.class_class) {
        return Err("Object.class.superclass should be Class".to_string());
    }
    if heap.class(behavior_metaclass).superclass != Some(object_metaclass) {
        return Err("Behavior.class.superclass should be Object.class".to_string());
    }
    if heap.class(class_metaclass).superclass != Some(behavior_metaclass) {
        return Err("Class.class.superclass should be Behavior.class".to_string());
    }
    if heap.class(metaclass_metaclass).superclass != Some(behavior_metaclass) {
        return Err("Metaclass.class.superclass should be Behavior.class".to_string());
    }

    // R-INV-0.2 — the parallel rule (ADR-0002) holds for *every* ordinary
    // (non-apex) core row, not just `Number` …
    let ordinary_rows: [(&str, ClassId); 24] = [ /* Number, String, Nil, Bool, True,
        False, Method, Function, Block, Symbol, Module, System, Option, Some,
        None, Iterable, List, Map, Set, Tuple, Range, Message, Error,
        MessageNotUnderstood */ ];
    for (name, class_id) in ordinary_rows {
        let meta = heap.class(class_id).class;
        let superclass = heap
            .class(class_id)
            .superclass
            .ok_or_else(|| format!("{name}.superclass should be set (parallel rule)"))?;
        let expected_meta_super = heap.class(superclass).class;
        if heap.class(meta).superclass != Some(expected_meta_super) {
            return Err(format!("{name}.class.superclass should be {name}.superclass.class (parallel rule)"));
        }
    }

    // … R-INV-1.5 (Method < Function), R-INV-3.1 (Block < Function),
    // R-INV-0.3 (None/Nil distinct, None singleton is an Instance of None),
    // R-INV-0.4 (Some/Message/Error/MessageNotUnderstood field_count),
    // R-INV-6.1 (MessageNotUnderstood < Error < Object) — elided here,
    // full text in source, L118-176.

    // Every metaclass's superclass chain terminates (bounded walk guards
    // against a cycle turning into a hang instead of a failure).
    let mut current = heap.class(c.number_class).class;
    let mut steps = 0;
    loop {
        steps += 1;
        if steps > 64 {
            return Err("metaclass superclass chain did not terminate within 64 steps".to_string());
        }
        match heap.class(current).superclass {
            Some(next) => current = next,
            None => break,
        }
    }

    Ok(())
}
```

This is the **explicit assertion of the parallel rule**
(`(X class).superclass == (X.superclass) class`, ADR-0002) plus the closed
2-node metaclass loop (`Metaclass.class == Metaclass class`, `(Metaclass
class).class == Metaclass` — read directly off `object_metaclass`/
`behavior_metaclass`/`class_metaclass`/`metaclass_metaclass` all being
required to have `.class == Metaclass`, and `Metaclass.class` specifically
required to equal `metaclass_metaclass` — never `Metaclass` itself). Called
once, in `vm/bootstrap.rs::VM::new`, right after `install_primitives`,
`finalize_all_core_base_names`, and `run_core_module` (i.e. after `core.ph`
itself has run) — the caller `.expect()`s the `Result` (`vm/bootstrap.rs`
L151-153), so a violated invariant aborts VM construction outright rather
than surfacing as a later runtime bug.

**Full ordered bootstrap sequence** (VERIFIED by reading `vm/bootstrap.rs::VM::new`,
`universe/mod.rs::Universe::new` L135-150, and the two functions above):

1. `Universe::new(&mut heap)` → `Self::create_core_classes(heap)`: allocate 8
   apex rows bare → wire instance-of → wire instance-side superclasses →
   wire metaclass-side superclasses (parallel rule) → `make_core_class` for
   ~24 more core rows.
2. `VM::new` stamps fixed-slot layouts on the four directly-Rust-built rows
   (`Some`, `Message`, `Error`, `MessageNotUnderstood`) that have no `.ph`
   `construct`.
3. `Universe::install_primitives(&mut vm)` — binds native fn pointers
   (`class`/`superclass`/… ) onto the kernel rows' `methods` maps.
4. `vm.finalize_all_core_base_names()`.
5. `vm.run_core_module()` — compiles and runs `core.ph`, attaching `.ph`
   reopens (`List`, `Option`, `Some`, `None`, `System`, …) to their already-
   bootstrapped kernel rows.
6. `vm.universe.mark_leaf_tostring_pristine()`.
7. Inline assertion that the `None` global resolves to the singleton value,
   not the class object.
8. `vm.universe.verify_invariants(&vm.heap).expect(…)` — the last step
   before `VM::new` returns.

---

## 5. The kernel rows — `Object`, `Behavior`, `Class`, `Metaclass`

All four exist and are wired at HEAD (VERIFIED, `core_classes.rs` L17-51):

- **Instance-side chain:** `Object` (superclass `None`, the sole apex root)
  ← `Behavior` (superclass `Object`) ← `Class` (superclass `Behavior`);
  `Metaclass` also has superclass `Behavior` (a sibling of `Class`, not a
  subclass of it) — matching ADR-0003's *"`Class` and `Metaclass` both
  inherit from `Behavior`"*.
- **Metaclass-side chain (parallel rule):** `Object class`.superclass ==
  `Class`; `Behavior class`.superclass == `Object class`; `Class
  class`.superclass == `Behavior class`; `Metaclass class`.superclass ==
  `Behavior class`.
- **Instance-of:** every one of the four metaclasses (`Object class`,
  `Behavior class`, `Class class`, `Metaclass class`) has `.class ==
  Metaclass`; `Metaclass` itself has `.class == Metaclass class` (the 2-node
  loop, §THE ANSWER); `Object`/`Behavior`/`Class` each have `.class` equal to
  their own dedicated metaclass.

Helper for the ordinary (non-apex) case: `fn make_core_class(heap: &mut Heap,
name: &str, superclass: ClassId, metaclass_class: ClassId) -> ClassId`
(§4b) — allocates a class+metaclass pair and wires both by the parallel rule
in one call.

---

## 6. Run live — reflection surface

Built once: `cargo build -p phalcom-core --bin phalcom` — succeeded, no
errors (one unrelated dead-code warning, `init_selector_cache`).

### 6a. Written-and-run scratch probes (VERIFIED, actual stdout below)

`probe1.ph`:
```
class Point {}
let p = Point.new()

System.print("p.class.name = " + p.class.name)
System.print("Point.class.name = " + Point.class.name)
System.print("Point.class.class.name = " + Point.class.class.name)
System.print("Point.class.class.class.name = " + Point.class.class.class.name)

System.print("Metaclass.class == Metaclass -> " + (Metaclass.class == Metaclass).toString)
System.print("Metaclass.name = " + Metaclass.name)
System.print("Metaclass.class.name = " + Metaclass.class.name)

System.print("Object.class.superclass == Class -> " + (Object.class.superclass == Class).toString)

System.print("1.class.name = " + 1.class.name)
System.print("true.class.name = " + true.class.name)
System.print("false.class.name = " + false.class.name)
```

Actual output (`cargo run -q -p phalcom-core --bin phalcom -- probe1.ph`):
```
p.class.name = Point
Point.class.name = Point.class
Point.class.class.name = Metaclass
Point.class.class.class.name = Metaclass class
Metaclass.class == Metaclass -> false
Metaclass.name = Metaclass
Metaclass.class.name = Metaclass class
Object.class.superclass == Class -> true
1.class.name = Number
true.class.name = True
false.class.name = False
```

`probe2.ph`:
```
System.print("Metaclass.class.class == Metaclass -> " + (Metaclass.class.class == Metaclass).toString)
System.print("Metaclass.class.class.name = " + Metaclass.class.class.name)
System.print("Behavior.name = " + Behavior.name)
System.print("Behavior.class.name = " + Behavior.class.name)
System.print("Class.superclass == Behavior -> " + (Class.superclass == Behavior).toString)
System.print("Metaclass.superclass == Behavior -> " + (Metaclass.superclass == Behavior).toString)
System.print("Object.superclass == nil -> " + (Object.superclass == nil).toString)
```

Actual output:
```
Metaclass.class.class == Metaclass -> true
Metaclass.class.class.name = Metaclass
Behavior.name = Behavior
Behavior.class.name = Behavior class
Class.superclass == Behavior -> true
Metaclass.superclass == Behavior -> true
Undefined variable 'nil'.
```

Notes on these results:
- `Metaclass.class == Metaclass` → **`false`**; `Metaclass.class.class ==
  Metaclass` → **`true`**. This is the live, direct confirmation of the
  2-node loop over the 1-node self-cycle the doc comments describe (§THE
  ANSWER).
- `Point.class.class` (a user class's metaclass's metaclass) reports name
  `"Metaclass"`, i.e. it *is* the `metaclass_class` row itself — matching
  `make_core_class`'s wiring (`meta.class = metaclass_class` for every
  ordinary core/user class's metaclass).
- `nil` is **not** a bindable surface identifier — `Undefined variable
  'nil'.` — consistent with `value/mod.rs`'s module doc: `Value::Nil` is *"a
  **private** uninitialized-slot sentinel with no surface class; user code
  can never produce or observe it"*. This bounds what the doc can claim:
  there is no `.ph`-visible `nil` literal to test `Object.superclass ==
  nil` against; `Object.superclass` itself is simply never read as `nil`
  because reflection selectors return `Option`-wrapped values in `.ph`
  where applicable, not the private sentinel.

### 6b. Existing golden fixtures re-run live (VERIFIED — matched `.expected` exactly)

Three fixtures from `phalcom-core/tests/lang/{classes,metaclass}/` re-run via
`cargo run -q -p phalcom-core --bin phalcom -- <path>`, output compared
against the checked-in `.expected` file:

| Fixture | Output | Matches `.expected` |
|---|---|---|
| `classes/is_metaclass_discriminator.ph` (`Point is Class`, `Point is! Class`) | `true` / `false` | yes |
| `metaclass/metaclass_metaclass_of_metaclass_is_a_class.ph` (`Point.class.class.isA(Class)`, `Point.class.isA(Class)`) | `true` / `false` | yes |
| `metaclass/metaclass_parallel_rule_builtin.ph` (`Number.class.superclass == Object.class`) | `true` | yes |

The middle result is a genuine, pinned asymmetry the fixture's own header
comment calls out: *"the doubly-lifted object reports `isA(Class)` true,
while the metaclass itself (`Point.class`) reports `isA(Class)` false — the
tower's `Class` membership is not uniform across metaclass rungs in the
current implementation."* Not independently re-derived here beyond running
it; noted because it is a real, documented quirk a reader of this map should
not be surprised by.

Additional fixtures read (not re-run by me, but present as golden `.ph`/
`.expected` pairs at HEAD, so presumed passing under the project's own golden
suite): `metaclass/metaclass_is_a.ph` (`3.isA(Number)` → `true`,
`3.isA(String)` → `false`), `metaclass/metaclass_is_a_object_root.ph`
(`3.isA(Object)` → `true`), `metaclass/metaclass_parallel_rule_user_class.ph`
(`Point.class.superclass == Point.superclass.class` → `true`),
`reflection/reflection_class_of_instance_vs_class_vs_metaclass.ph`
(`p.class.name`/`Point.class.name`/`Point.class.class.name` →
`Point`/`Point.class`/`Metaclass`, matching my own `probe1.ph` exactly).

### 6c. What reflection surface actually exists at HEAD (bounds the doc's claims)

`class` (`Object::class`, getter, native), `superclass` / `superclass=`
(`Behavior::superclass`, getter+setter, native — `universe/primitives.rs`
L87-89), `name` (`Behavior#name` shadows `Object#name` for class receivers,
`universe/primitives.rs` L95), `==` (identity by default), `isA(_)` (`.ph`,
derived over `class`/`==`/`superclass`, `core.ph`), and `is`/`is!` (surface
syntax over the same). All of these ARE exposed to `.ph` and were exercised
above. `class=(_)` exists as a selector but is a hard error by design
(`primitive/object.rs::object_set_class` L91-98, always returns
`RuntimeError::InvalidSetClass` — *"an object's class is fixed"*), so runtime
class mutation is not a thing this tower supports, INFERRED-confirmed by the
function body (not separately tried live).

---

## 7. Use sites / blast radius (`graphify affected "ClassObject"`, depth 2)

| Subsystem | Symbol | Role |
|---|---|---|
| Heap accessors | `heap/accessors.rs::Heap::{class,class_mut,as_class}` (~L23,35,43) | typed row access, panic/`Option` variants |
| Heap allocation | `heap/mod.rs::Heap::alloc_class` (~L146) | mints a fresh `ClassId` for a bare row |
| GC / tracing | `heap/trace.rs::trace_object`, `Object::Class` arm (~L79-96) | marks `class`, `superclass`, `methods`, `static_slots`, `attributes` as outgoing edges (§8) |
| Object enum | `heap/object.rs::Object` (~L24), `Class(Box<ClassObject>)` variant | tags an arena slot as a class row |
| Value dispatch | `value/mod.rs::Value::class`/`lookup_method` (~L121,170) | Value→ClassId, then §2's hierarchy walk |
| Compiler / inline caching | `chunk.rs::InlineCache.class: ClassId` (L10-12) | `ClassId` doubles as the per-call-site inline-cache tag (ADR-0012) |
| Universe / bootstrap | `universe/core_classes.rs::create_core_classes`, `universe/invariants.rs::verify_invariants` | builds and asserts the tower (§4) |

---

## 8. GC angle

Mark-through, non-moving — `heap/trace.rs::trace_object`, `Object::Class` arm
(~L79-96):

```rust
Object::Class(class) => {
    push(class.class);
    if let Some(sup) = class.superclass {
        push(sup);
    }
    for method in class.methods.values() {
        push(*method);
    }
    for slot in class.static_slots.iter() {
        trace_value(*slot, push);
    }
    for attr in &class.attributes {
        trace_value(*attr, push);
    }
}
```

Both `class` (the metaclass link) and `superclass` are pushed as outgoing
edges for the mark worklist — the collector **marks through** them. The
collector is confirmed **non-moving, mark-sweep**:
`heap/mod.rs::Heap::collect` doc (~L262-292) states it runs *"one full
**non-moving, precise, stop-the-world mark-sweep**"* per
[ADR-0050](../../adr/accepted/0050-non-moving-mark-sweep-collector.md), and
explicitly: *"**Non-moving** (Invariant M1): a surviving object keeps its
`ObjRef` for life"* and *"Cycles — including the kernel's own (`Metaclass` is
an instance of itself) — terminate because an already-marked object is never
re-pushed (Invariant M5)."* So `class`/`superclass` `ClassId` handles are
never patched or relocated by a collection — only ever read, marked, and
(for a genuinely unreachable class) swept.

---

## 9. Spec / ADR — bounded

- **ADR-0002 (metaclass-tower-parallel-rule), accepted.** Decision: `(X
  class).superclass == (X.superclass) class`; `Object class`'s superclass is
  `Class`, closing the tower; `Metaclass` is stated to be *"an instance of
  itself"* in this ADR's own prose (the same simplification §THE ANSWER
  corrects against the actual bootstrap code) — and it enumerates the same
  seven-step bootstrap order `universe/mod.rs`'s module doc and
  `create_core_classes` implement.
- **ADR-0003 (introduce-behavior-kernel-class), accepted.** Decision: add
  `Behavior` as an abstract kernel class owning the shared method-dictionary/
  superclass/lookup/instantiation protocol; `Class` and `Metaclass` both
  inherit from it; `Behavior` inherits from `Object`. Explicitly named as
  what "unifies the kernel and removes the need for asymmetric
  special-casing of `Metaclass` versus `Class`" — i.e. it displaced an
  asymmetric-special-case design, not a menu of fresh alternatives.
- **ADR-0009 (handle-arena-heap), accepted — supersedes an `Rc`/`new_cyclic`
  representation.** Decision: objects live in a central `Heap`, referenced
  by `Copy` integer handles (`ObjRef`/`ClassId`); *"No `Rc`, no `RefCell`, no
  `MaybeWeak`. The kernel cycle is expressed as handles that refer to each
  other with no ownership paradox."* Its own "Alternatives considered"
  section names the design it replaced: *"`Rc<RefCell<T>>` + intentional
  process-lifetime kernel cycle (the current substrate, with `Weak` only
  where needed) … Rejected as the design baseline"* — confirming the tower
  was originally built on `Rc`/`Weak`/`RefCell` and was re-represented onto
  handles by this ADR.
- **`docs/spec/v0.2/core/core-classes.md` §3 "Kernel tower classes"** (Object
  ~L100, Behavior ~L132, Class ~L157, Metaclass ~L170): matches the code —
  `Object` is the root (no superclass); `Behavior` (superclass `Object`) is
  *"the home of everything that has instances"*, superclass of both `Class`
  and `Metaclass`; `Class` (superclass `Behavior`) is *"the class of every
  named class"*; `Metaclass` (superclass `Behavior`) is *"the class of
  metaclasses … `(X class).class == Metaclass`"* — note the spec itself
  states the relation in the **correct** 2-hop form (`(X class).class ==
  Metaclass`, not `Metaclass.class == Metaclass`), consistent with
  `verify_invariants` and the live run, and in tension with the two source
  doc-comments quoted in §THE ANSWER/§1. Status line for `Metaclass`: *"✅
  structurally complete — verified by `verify_invariants`."*

---

## What was inferred vs. verified — summary

**Verified** (read the exact line, or ran the program and matched output):
`ClassId = ObjRef` type alias and `slotmap` key definition; the full
`ClassObject` struct and its module doc; `lookup_method_in_hierarchy`;
`Heap::class`/`class_mut`/`as_class`; `Value::class` (all match arms,
including every immediate); `create_core_classes` and `make_core_class` in
full, including the exact allocate-then-patch ordering; `verify_invariants`
in full, including the exact 2-node-loop assertion; the `VM::new` call order
(`Universe::new` → field-slot stamping → `install_primitives` →
`finalize_all_core_base_names` → `run_core_module` → `mark_leaf_tostring_pristine`
→ None-singleton assert → `verify_invariants`); `trace_object`'s
`Object::Class` arm; `Heap::collect`'s non-moving mark-sweep doc; the
`class`/`superclass` floor-primitive bindings in `universe/primitives.rs`;
two scratch `.ph` probes run live against a freshly built `phalcom` CLI
(`cargo build -p phalcom-core --bin phalcom` succeeded); three existing
golden fixtures re-run live and matched their `.expected` files exactly;
ADR-0002/0003/0009 Decision (and 0009's Alternatives) sections;
`core-classes.md` §3's four kernel-row entries.

**Inferred, not independently re-derived from source**: `object_set_class`'s
`RuntimeError::InvalidSetClass` behavior was read from its body but not
separately exercised live; the remaining ~21 golden fixtures in
`tests/lang/{classes,metaclass,reflection}/` were read but not re-run (their
`.expected` files were read alongside them and are internally consistent
with the live-run results, but "presumed passing" is not the same as
"observed passing" for those specific files); `docs/spec/v0.2/object-model.md`
§5-6 itself was not opened in this pass (out of the bounded ADR/spec list the
task specified) — `core-classes.md` §3 and the ADRs were treated as the
bounded spec surface instead, per instruction.

**Confirmed not to exist / not applicable at HEAD**: no `Rc`/`RefCell`/`Weak`
anywhere in the class/metaclass representation (superseded by ADR-0009); no
row whose `class` field equals its own `ClassId` (the apex "self-cycle" is
actually a 2-node loop — §THE ANSWER); no bindable `.ph` `nil` literal
(`Value::Nil` is a private sentinel); no live class-mutation surface
(`class=(_)` is a hard error, not a partial/stubbed feature).
