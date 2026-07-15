# Handoff — M-ATTR-ROOT implementation + triaged tree issues

Continuing: implement `M-ATTR-ROOT` (the `Attribute`/`@On`/tier-singleton/retention
mechanism — first foundational unit of `docs/forge/PLAN-DECORATORS.md`, the
Install/Dispatch/Runtime decorator work ratified this session under ADR-0054).

**Adopt `/forge senior` first.** Entry points below are pre-verified — do not re-survey.

## What's landed on `main` already (commits, this session)

- ADR-0052/0053 ratified (Proposed → Accepted).
- ADR-0057 (decorator vs proxy granularity split, both kept) — new, Accepted.
- ADR-0058 (Reactive tracking-context needs a native module, not class-side `.ph`
  state) — new, Accepted.
- ADR-0054 fully ratified (Install/Dispatch/Runtime tier unblocked; both gate
  conditions in Decision §2 satisfied).
- `docs/spec/v0.2/drafts/decorators-behavioral.md`, `decorators-dispatch-observability.md`,
  `decorators-observable.md` — new, ratified, all open questions (B-1/B-2/D-1/D-2/D-3/R-1..R-5)
  resolved to option (a) in every case, deferred alternatives filed in `docs/forge/DEFERRED.md`.
- `docs/spec/v0.2/drafts/reactivity.md` — ratified (R-1..R-5 resolved).
- `docs/spec/v0.2/decorators/on.md` — mechanism spec, A-1–A-5 resolved inline
  (A-6 explicitly deferred to v0.3, not a blocker).
- `docs/forge/PLAN-DECORATORS.md` — the full unit-dependency plan (commit `df259f3`),
  updated after the M-ATTR-ROOT blocker below (write-set expanded).
- `phalcom-core/core/core.ph` — added `Tracer`/`OffBehavior`/`Backoff` standalone
  library classes (commit `385fed9`), golden test `tests/lang/decorators/decorators_stdlib_helper_classes.ph`
  passing (`cargo test -p phalcom-core --test lang decorators`). These do **not**
  depend on M-ATTR-ROOT — pure library code, safe, already done.
- **NOT built:** `Flags` (`@featureFlag`'s registry) — cannot be pure `.ph`, needs a
  native module (no `.ph`-reachable class-side/module mutable state exists today,
  `concurrency.md:234`, same class of gap as ADR-0058's `Reactive` finding).
  `Backoff.fixed`/`.exponential` raise rather than working — `System.sleep(_)` doesn't
  exist yet (`system.md`: "still open").

## M-ATTR-ROOT — status: NOT STARTED (two attempts, both stopped correctly before writing code)

**Attempt 1** stopped and reported: the plan's stated dependency ("U-ANNOT-CONTRACTS
attribute registry, already LANDED") is wrong — `compiler/attributes.rs`'s
`expand_class_attributes`/`AttributeRegistry`/`ExpandCtx` exist but have **zero call
sites** anywhere; `compile_class` silently drops all `@Name(...)` attributes at
lowering. Plan was corrected, write-set expanded (now includes `compiler/lib/class_decl.rs`
+ possibly `compiler/lib/mod.rs`), committed at `df259f3`.

**Attempt 2** was dispatched with the corrected scope, spent ~14 minutes reading (53
tool calls, no `Edit`/`Write` calls — confirmed no code was written), was stopped
mid-flight by the orchestrator (context/time management, not a blocker) and asked to
compact its findings instead of continuing. Below is that compaction, verbatim from
the agent — **treat every file:line claim as agent-reported, not independently
re-verified by the orchestrator**, but the agent did actually read these files, so
this is real signal, not a guess.

### Confirmed insertion points

- **Wiring `expand_class_attributes`:** top of `compile_class`, `phalcom-core/src/compiler/lib/class_decl.rs:25`
  (fn starts) — insert BEFORE the field-collection Pass 1 (before line ~31/34).
  Contract weaving must run before field collection so woven checks' field refs get
  collected. Suggested shape: `let class_def = self.expand_and_validate_attributes(class_def)?;`.
  `expand_class_attributes` is at `attributes.rs:597`, returns `Result<ClassDef, CompilerError>`.
- **`ExpandCtx` build (borrow order matters):** read `self.vm.compile_mode` (Copy,
  `vm/mod.rs:181`) + `self.vm.strip_contract_metadata` (`vm/mod.rs:188`) into locals
  first; compute `is_attribute_class` first (read-only walk of `self.vm.class_parents`);
  THEN build `ExpandCtx { interner: &mut self.vm.interner, compile_mode, strip_metadata }`
  (fields at `attributes.rs:36-50`). Construct `AttributeRegistry::new()` (`attributes.rs:468`)
  locally per call — no cached instance exists.
- **Runtime class-definition sequence** in `compile_class` (`class_decl.rs`): member
  loop ends ~line 510; `self.emit(Bytecode::FinalizeClass, range)` at **518** (peeks,
  does NOT pop class); `self.emit(Bytecode::DefineGlobal(name_idx), range)` at **522**
  (CONSUMES/pops the class off stack). **Post-class-definition codegen hook goes AFTER
  line 522** — class is now a global; re-fetch via a synthesized `Expr::Var{class_def.name}`
  (→ `GetGlobal`). No pre-existing "run after class opcode" mechanism — this is new,
  built by `compile_expr`-ing synthesized attach ASTs + `Bytecode::Pop`.
- **`Bytecode::Invoke` shape** (`expr.rs`): `Bytecode::Invoke(argc: u8, selector_idx)`.
  Emission pattern (`expr.rs:101-107`): compile receiver, then each arg
  (`compile_expr(arg.expr)`), then `Invoke(args.len(), selector_idx)` where
  `selector_idx = self.add_constant(Value::Symbol(selector_sym))`. Getter send =
  `Invoke(0, ...)` (`expr.rs:147`). Constructor-alias redirect happens automatically
  (`expr.rs:74-107`) for `Var{ClassName}.new(...)` via `lookup_constructor_alias` — a
  synthesized `AttrName.new(args)` will resolve the alias.

### Confirmed API shapes / gotchas — do not re-derive

- **PARSER BLOCKER (real, forces a spec deviation):** attribute arg lists are
  **positional-only, no labels** (`parser.rs:1047-1065`, `parse_attribute_arg_list`
  calls `parse_expr()` per arg). `Attribute.args: Vec<Expr>` (`ast.rs:157-166`), not
  `Vec<Argument>`. **The spec's `@On(Method, tier: Install)` and `@Author(name: "Ada")`
  CANNOT PARSE.** `parser.rs` is outside M-ATTR-ROOT's write-set. Forced deviation:
  goldens must use positional args (`@On(Class)`, `@On(Method, Install)` — tier
  detected by matching arg `Var` name against `{Compile,Layout,Install,Dispatch,Runtime}`).
  **File this to DEFERRED.md** — labeled attribute-arg syntax needs a parser unit.
- **No symbol literals for method-level desugar** (`#fib` is U-LEX-HASH's paren-form,
  still pending — see `DEFERRED.md`'s existing `#[]`/`#+(_)` entries). Blocks
  `X.methodFor(#sel)`-style desugar. BUT method-level attach is feasible directly
  inside the member loop without symbols: after `Bytecode::Method` installs a method,
  stack top = class object; `Constant(method_obj_idx)` (push the method object — the
  SAME `ObjRef` now in the class dict, mutation is shared), `compile_expr(AttrName.new(args))`,
  `Invoke(1, __attach_idx)`, `Pop`. Verified stack-safe by the agent. Constructors
  reject attributes at parse time already (`parser.rs:1086`); field members emit no
  method object (skip); `_x`-getters emit `SetField` not a method object (skip).
- **List API for goldens:** no `.first` — use `.at(0)`. `filter`/`isA` exist
  (`core.ph:370`, `core.ph:33`). `methodFor` exists (`primitive/object.rs:198`, Symbol arg).
- **Heap/value shapes:** class value = `Value::Obj(ObjRef)`, `heap.get(id) = Object::Class`.
  Accessors: `class`/`class_mut`/`method`/`method_mut`/`module`/`module_mut`
  (`heap/accessors.rs:23/35/87/99/111/123`), `get`/`get_mut` (`heap/mod.rs:145/154`).
  List build: `Value::Obj(vm.heap.alloc(Object::List(ListObject::new(vec))))`
  (`heap/list.rs:28`).
- **Primitive registration idiom:** `PrimitiveFn = fn(&mut VM, &Value, &[Value]) -> PhResult<Value>`
  (`method/object.rs:13`). Register via `primitive!(vm, class, "base", SignatureKind, fn)`
  macro (`primitive/mod.rs:109`). New module needs `pub mod attribute;` added to
  `primitive/mod.rs` — technically outside the write-set but unavoidable glue; report
  the deviation, don't silently skip it.
- **Register all 3 new primitives (`__attributes` getter, `__attach` Method(1),
  `__freezeAttributes` Method(0)) on `object_class` instance-side**
  (`universe/primitives.rs:~38+`). Because `Object` sits at the bottom of every
  metaclass chain, this single registration site covers class objects, method
  objects, and module objects (confirmed via the ADR-0002 parallel rule) — no need
  for three separate registration sites.
- **`Behavior` class already exists** in the tower (`core_classes.rs:18`,
  `behavior_class` field), not yet reopened in `core.ph`. Neither is `Method`. Both
  need new reopen blocks for `attributes`/`attributesOfType(_)`.
- **`class_parents` is populated at COMPILE time** (`class_decl.rs:285`) for any class
  with an explicit `extends`; `vm.classes` is RUNTIME-only (empty during compile).
  Attribute-class detection **must** use a `class_parents` walk (resolve each symbol
  via `vm.resolve_symbol`, compare to `"Attribute"`) — simplest direct check:
  `class_def.superclass.name == "Attribute"`. No new `VM` field needed, avoids
  touching `vm/mod.rs` (outside write-set).
- **Struct field additions needed:** `MethodObject` (`method/object.rs:26`, init at
  `:54`), `ClassObject` (`heap/class.rs:25`, init at `bare()` `:84`), `ModuleObject`
  (`heap/module.rs:37`, init at `new()` `:57`) each need `attributes: Option<Vec<Value>>`
  + `frozen: bool`.
- **`core.ph` layout for insertion:** `Object` at line 1, `Class` at 36, `Metaclass`
  at 38, `Function` at 286, no existing `Method` class, `System` at 760, tail ends
  ~969 (`Backoff`, just added). Add `class Attribute {}`, `class On extends Attribute {}`
  (minimal/passive for now), a `Tier` value class + 5 singleton instances
  (`Compile`/`Layout`/`Install`/`Dispatch`/`Runtime`), and `Behavior`/`Method` reopen
  blocks for `attributes`/`attributesOfType(_)`. Reopen-with-fields on a bootstrap
  class trips read-before-write — the `Behavior`/`Method` reopens are method-only so
  this is safe.

### Ordered implementation plan (agent's proposal — verify before trusting blindly)

1. `heap/class.rs:25/84` + `method/object.rs:26/54` + `heap/module.rs:37/57`: add
   `attributes: Option<Vec<Value>>` + `frozen: bool`; add `attach_attribute(v)`/
   `attributes_slice()`/`freeze_attributes()` helpers, `///`-documented.
2. `primitive/attribute.rs` (NEW): `attribute_attach` (match receiver `Object`
   variant → append if `!frozen` else `Err(RuntimeError::Message("attr.frozen: ..."))`),
   `attribute_attributes` (read store → alloc `List`, empty if `None`),
   `attribute_freeze` (set frozen). `//!` module doc.
3. `primitive/mod.rs`: add `pub mod attribute;` (report as out-of-write-set glue).
4. `universe/primitives.rs:~38`: register the 3 primitives on `object_cls`.
5. `compiler/attributes.rs`: (a) register `"On"` → an `OnExpander` (no-op, legal on
   `Class`) in the registry at `:471`; (b) change the class-level (`:622`) and
   member-level (`:665`) loop `else` branches to NOT return `attr.unknown` —
   retain silently instead; (c) add an `is_attribute_class: bool` param to
   `expand_class_attributes:597`; (d) add `validate_attribute_class(...)`, run when
   `is_attribute_class`: parse `@On` args for tier (`Var` name match), map
   tier→reserved-hook-selector (Compile→`expand`, Layout→`finalizeLayout`,
   Install→`wrap`, Dispatch→`resolveMissing`, Runtime→`aroundSend`), scan members
   for those selectors, emit `attr.compile_tier_reserved` (Compile/Layout tier on a
   user class), `attr.missing_hook` (declared tier, no matching hook impl),
   `attr.undeclared_hook` (reserved-hook impl without a matching declared tier).
6. `compiler/lib/class_decl.rs`: (a) add an `is_attribute_class` helper
   (`class_parents` walk); (b) call `expand_class_attributes` at the top of
   `compile_class:25`; (c) after `DefineGlobal:522`: for each retained class-level
   attribute where the name isn't a registered builtin expander AND the class is a
   user `Attribute` subclass → emit `compile_expr(Var{ClassName}.__attach(AttrName.new(args)))`
   + `Pop`; if neither condition holds → `Err(attr.unknown)` (this is what makes
   `annotation_unknown_error.ph` pass); emit a freeze call after attaches; (d) member
   loop: after each `Bytecode::Method`, emit the member-level attach sequence
   described above, then freeze.
7. `core.ph`: add `Attribute`/`On`/`Tier`/5 singletons/`Behavior`+`Method` reopens.
8. Goldens: `tests/lang/decorators/` (PASS) — a positional-args `@Author("Ada")`
   retention round-trip (`Engine.attributesOfType(Author).at(0).name` → `"Ada"`,
   `Behavior.attributes`); `tests/lang/compile-errors/` (NEGATIVE) — `attr.missing_hook`
   via `@On(Method, Install)` on a class with no `wrap` method. Negative harness:
   nonzero exit + substring match (`support/mod.rs:142`, already used elsewhere).

### Still unknown — verify before implementing

- Does `@On` in the class-level loop (`attributes.rs:605`) actually route to the new
  `validate_attribute_class`, not the generic expander path? `@On`'s `legal_targets`
  must include `Target::Class`.
- Exact `strip_metadata` derivation from `compile_mode` + `strip_contract_metadata` —
  read a CLI call site + `method/object.rs:45`'s doc comment to confirm the truth
  table before hardcoding it.
- Does `compile_expr(Var{ClassName})` immediately after `DefineGlobal` in the same
  closure actually resolve at runtime (should — one closure compiles then runs, so
  `DefineGlobal` executes before the attach `GetGlobal`) — but confirm there's no
  compile-time "undefined global" rejection for a forward-in-same-closure reference.
- Does `Bytecode::Method`'s handler clone or share the method `ObjRef`? The
  member-level attach plan assumes it shares (mutating the dict's method object
  in place). Verify in `vm/mod.rs`'s `Bytecode::Method` arm.
- Freeze necessity is NOT gate-required by ADR-0053/A-5 (mutation must error
  eventually, but doesn't have to be enforced in this first landing). If member-loop
  freeze emission proves stack-fragile, ship class-level freeze only and defer
  member-level freeze — do NOT build an epoch counter instead (ADR-0053 explicitly
  rules this out).

## Separately triaged — 4 pre-existing red test groups + 1 new bug (read-only, not fixed)

Baseline `cargo test -p phalcom-core --test lang` fails 6 groups on a clean run,
**before** any M-ATTR-ROOT work starts. Two are the attribute-expansion gap above
(`compile_errors::annotation_unknown_error`, `runtime_errors::contracts_invariant_fiber_yield`
— should go green once M-ATTR-ROOT lands). The other four:

1. **`indexing`** — `indexing/independence.ph` fails to parse: `[](i) { return i * 2 }`
   (subscript-`[]` method-definition syntax). **Already tracked** in `DEFERRED.md`'s
   U-LEX-HASH entry: `#[]` was never lexed, so there's no `[](...)` method-def grammar
   to canonicalize against. Not a regression.
2. **`indexing_negative`** — `indexing/negative/empty_arg_dnu.ph` fails for the same
   root cause: `xs[]` (empty-arg subscript call site) doesn't parse. Same U-LEX-HASH gap.
3. **`errors`** — `errors/annotation_construct_own_fields.ph`'s own file header says
   `U-ANNOT-LAYOUT step 3`. It's a fixture for the concurrent U-ANNOT-LAYOUT session's
   not-yet-landed `@construct` derive (confirmed independently: that unit has only
   landed steps 1-2 of 7 — `FieldDef` grammar + layout wiring — as of this session).
   Expected-red until that unit progresses; not this thread's problem.
4. **`concurrency`** — **new, real, independent finding.** `concurrency_sched_fifo_order.ph`
   calls `System.schedule` three times but never `System.runScheduled`, expecting the
   documented "root-drive pump" (`VM::run` auto-draining the ready-queue once the
   top-level program ends, per `concurrency.md`'s "belt-and-suspenders pump"
   description) to fire it automatically. It doesn't — the program exits 0 with zero
   stdout. Either that pump was never actually wired despite being described as
   landed in `concurrency.md`/prior memory, or it regressed. **Nobody has looked into
   this yet** — worth a dedicated investigation, independent of everything else in
   this document. Entry point: `phalcom-core/tests/lang/concurrency/concurrency_sched_fifo_order.ph`,
   `docs/spec/v0.2/concurrency.md` §2 "Implementation" (the root-drive pump
   description), `VM::run` (locate via graphify).

## Next step for whoever picks this up

Start M-ATTR-ROOT fresh using the "Ordered implementation plan" above as a starting
point, not gospel — re-verify the "Still unknown" list first. This is a substantial
unit (new compiler wiring + new codegen path, no prior art for "run this after the
class opcode" in this codebase) — expect it to take real, focused implementation
time, not a quick patch. Use `phalcom-implementer` in an isolated worktree, gate on
`cargo build --workspace` + `cargo test -p phalcom-core --test lang` (confirm
`compile_errors`/`runtime_errors` go green; the other 4 pre-existing failures are
independent, don't let them block this unit).

The `System.schedule`/root-drive-pump bug (triage item 4 above) is unrelated and can
be picked up independently, any time, by anyone.
