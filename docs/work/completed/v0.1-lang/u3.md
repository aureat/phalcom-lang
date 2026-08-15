# U3 — Selector Dispatch (as-built)

- **Status:** ✅ Landed — `5758f3c` (`feat(u3): update AST, parser, compiler, and Signature for label-encoded selectors`) + `ae16924` (`feat(u3): propagate errors in VM Invoke handler and update binary op selectors`) + `845f2f9` (`feat(u3): update primitive macros and bootstrap registrations in universe.rs`), 2026-07-11.
- **Realizes:** [ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md) (label-encoded selectors + inline-cache-ready dispatch); spec [messages-and-selectors §2–3](../../../spec/current/messages-and-selectors.md), [method-lookup §1](../../../spec/current/method-lookup.md), [object-model §7](../../../spec/current/object-model.md), [selectors.md](../../../spec/current/selectors.md).
- **Reviewer gate:** the review policy lists dispatch (**U3**) among the load-bearing, reviewer-**ON** units. STATE.md records U3 as landed (`✅ (ADR-0012)`) but — unlike U1/U2 — does not log an explicit reviewer PASS/BLOCK verdict for it. (Unknown; not invented here.)

## Mission
Replace arity-only method dispatch with **label-encoded selector symbols**, so `foo`, `foo()`, `foo(_)`, `move(to,duration)` and `move(_,_)` are all distinct resolvable methods; give the method dictionary a single-probe lookup keyed by the selector symbol; and shape dispatch to host a future inline cache. Folds in the fixes for F1 (swallowed dispatch errors), F7 (0-arg metadata), F8 (divergent operator-selector encoder).

## Surface / behavior
A selector is an interned symbol encoding **name + argument labels** (Smalltalk-style): `add(_,_)`, `move(to,duration)`, `name=(_)`, `+(_)`, subscript `[_]`, variadic `sum(*)` are each their own selector. Because labels are baked into the symbol, lookup is one hashmap probe. Errored/failed sends now surface correctly (F1): a primitive error that was previously swallowed (e.g. `Point.new("abc")` → silent exit 0) now propagates.

## Implementation
- **`phalcom-core/src/method.rs`** (the plan's "signature.rs" is a dead empty stub — the real code lives here):
  - `SignatureKind` — `Initializer(u8)`, `Method(u8)`, `Getter`, `Setter`, `SubscriptGet(u8)`, `SubscriptSet(u8)`, and (added by U9) `Variadic(u8)`.
  - `Signature { selector: Symbol, kind: SignatureKind, positional_arity: u8, variadic: bool }`; `Signature::new` derives `positional_arity` from the kind. The `variadic` flag and a reserved internal-binding capacity anticipate external≠internal parameter names (Q3) and variadics **without changing selector identity**.
  - `encode_selector(name, labels, kind) -> String` — the **single source of truth** for the canonical label-encoded selector string, shared by the compiler and every runtime selector builder (`perform`, dNU forwarding) so they cannot diverge (F8 was exactly a divergent encoder). `decode_selector` (U8) is its total inverse. `make_signature` wraps `encode_selector` for a base name + kind.
  - `MethodKind` — `Closure(ObjRef)` (bytecode) or `Primitive(PrimitiveFn)`; `MethodObject` bundles kind + `Signature` + holder class, all by `Copy` handle.
- **`phalcom-ast` + `phalcom-core/src/compiler/lib.rs`** — AST/parser/compiler updated to lower method declarations and call sites to label-encoded selectors; binary operators lowered through the shared encoder.
- **`phalcom-core/src/vm.rs`** — the `Bytecode::Invoke(arity, selector_idx)` handler keeps its selector-constant operand (the dot-send → `Invoke(selector_const)` shape is retained; only what the selector encodes changed). It resolves via `receiver.lookup_method(self, selector_sym)` (a single-probe walk over the class's `methods` dictionary up the hierarchy) → `call_method`; a miss falls through (U8 wired this to `forward_does_not_understand`). `ae16924` fixed the handler to thread `call_method`'s `Result` (F1) — errors are `?`-propagated, not discarded.
- **IC-ready shape:** each call site is structured so a monomorphic cache slot (receiver `ClassId` → resolved method) can be added later, keyed by the stable class handle from [ADR-0009](../../../adr/0009-handle-arena-heap.md). Population is deferred (see below).

## Invariants & tests
- `phalcom-core/tests/lang.rs` gained coverage for the new selector model; the `tests/lang/` corpus was reorganized into labels (`arithmetic/`, `bindings/`, `dispatch/`, …) as part of this landing.
- F7 corrected: `object`'s static `new()` now carries correct 0-arg metadata. F8 corrected: operator selectors no longer intern the malformed `">( _)"` stray-space encoding — one encoder, byte-identical at compile and run time.
- Later units assert against this substrate: U8's `decode_selector` round-trip unit tests, and the floor-census / dispatch goldens.

## Deviations & deferrals
- **Inline-cache population is deferred** — dispatch is built IC-*ready* here; the cache itself is a speed item, not part of the accepted decision. See [deferred-work](../../../spec/current/deferred-work.md).
- Canonical selector spelling was **amended to comma form** (`move(_,to,duration)`, [selectors.md](../../../spec/current/selectors.md)); ADR-0012's original colon spelling is superseded — decision and reasoning unchanged, only the string encoding. (The as-built `encode_selector` emits the `_:`/label`:` internal form seen in `method.rs`.)
- The `Signature`'s reserved internal-binding field and variadic flag were added now but not populated until later units (variadics = U9).

## Sources
- forge: [`u0-state.md`](../../archive/phase2/u0-state.md) (U3 ✅ ADR-0012), [`PHASE2-INDEX.md`](../../archive/phase2/PHASE2-INDEX.md).
- code: `phalcom-core/src/method.rs` (`SignatureKind`, `Signature`, `encode_selector`/`decode_selector`, `make_signature`, `MethodObject`), `phalcom-core/src/vm.rs` (`Bytecode::Invoke` handler, `call_method`, `lookup_method`), `phalcom-core/src/compiler/lib.rs`, `phalcom-ast/src/{ast,parser}.rs`.
- ADRs: [0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md).
- landing: `5758f3c`, `ae16924`, `845f2f9`.
