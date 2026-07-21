# U9 — Variadics (rest parameters) (as-built)

- **Status:** ✅ Landed — `c9805d0` (runtime + acceptance corpus + forge docs, single commit)
- **Realizes:** [ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md) (amendment — adds the `Variadic` signature kind and `(*)` selector spelling); spec [messages-and-selectors.md §4](../../../spec/current/messages-and-selectors.md)
- **Reviewer gate:** OFF per the load-bearing-only review policy (STATE.md) — self-verified on the green gate (`./scripts/verify.sh` exit 0, `cargo doc` clean, clippy clean).

## Mission
Let a method declare a trailing rest parameter (`*name`) that binds all extra positional
arguments into a single kernel `List`. Extend the selector/signature model with a
`Variadic` kind and a canonical `(*)` selector spelling, collapse the extra args in the VM
call prologue, and add a derived-selector miss probe so a plain send can resolve to a
variadic method. Deliberately scoped *not* to include call-site spread (`f(*args)`).

## Surface / behavior
- **Rest parameter `*name`** — must be the **last** parameter and may not carry or follow a
  label; violations are clean parser diagnostics, not panics.
- Extra positional args at the call site are collapsed into one `List` bound to the rest slot.
- A variadic method and a same-name fixed-arity method **coexist** — the fixed selector wins
  when arity matches exactly; otherwise the `(*)` candidate is probed.

```phalcom
class Math {
  sum(*numbers) {
    var total = 0
    numbers.each { n => total = total + n }
    return total
  }
  format(fmt, *args) { /* F = 1 fixed prefix + rest */ ... }
}
Math.new().sum()          // rest = empty List → 0
Math.new().sum(1, 2, 3)   // rest = List[1,2,3] → 6
```

## Implementation
- **`phalcom-ast` (`ast.rs`, `parser.rs`)** — `ParameterDef.is_rest`; `parse_param_list`
  parses an optional leading `*` and rejects a rest param that isn't last or that
  carries/follows a label. Block-literal params are parsed by a separate scanner in
  `parse_primary` and never reach `parse_param_list`, so block variadics still don't parse
  ([forge/DEFERRED.md](../../DEFERRED.md) #9, confirmed still open).
- **`method.rs`** — new **`SignatureKind::Variadic(u8)`**; the payload is the fixed/minimum
  positional arity `F`. The selector spelling is always the bare `<name>(*)`, independent of
  `F` — `sum(*numbers)` and `format(fmt, *args)` both intern as `sum(*)`/`format(*)`; only
  `Signature.positional_arity` / `Signature.variadic` (set from the payload in
  `Signature::new`) distinguish them at runtime. `decode_selector`'s `Variadic` arm
  round-trips the name but **not** `F` (documented limitation — the selector text never
  carries it; only the dNU `Message`-reification path uses this, which doesn't need real `F`).
  *(The plan named `signature.rs`; that module is a dead stub — the work landed in `method.rs`.)*
- **`compiler/lib.rs`** — the `ClassMember::Method` arm computes `F = params.len() - 1` and
  selects `SignatureKind::Variadic(F)` over `SignatureKind::Method(arity)` when the last param
  `is_rest`. `compile_block` needed no change (`params.len()` already counts the rest param as
  an ordinary trailing local slot).
- **`vm.rs`** — call prologue in `call_method`'s `MethodKind::Closure` arm: if the target's
  signature is variadic, `Vec::split_off` the trailing `arity - fixed_arity` positional args,
  wrap them in one `List` via `heap.alloc_list`, and push it back. `receiver_idx`/`stack_offset`
  are computed before this mutation, so `CallFrame` slot addressing is unaffected. Runtime
  dispatch probe filling the `[U9 SEAM]` in `Bytecode::Invoke`'s miss arm: only an
  all-positional `SignatureKind::Method` selector (via `decode_selector`) probes for a
  `<name>(*)` candidate through one ordinary `lookup_method` walk; a hit dispatches only if
  `arity >= positional_arity`, else falls through to the existing `forward_does_not_understand`
  — no new error variant, no duplicated dNU body.

## Invariants & tests
- `variadics` PASS golden group: zero-prefix, fixed-prefix (`F=1`) prologue math,
  fixed-vs-variadic coexistence / dispatch ordering, dNU-fallback-preserved, real-`List` rest
  binding, and a stack-depth invariant (200 variadic calls in a loop — a black-box check that
  the tail collapse leaves the value stack balanced).
- 2 `syntax-errors` NEGATIVE goldens: rest param not last, rest param labelled.
- 2 pre-existing `clippy::useless_conversion` nits in `parse_param_list` cleaned in passing.

## Deviations & deferrals
- **No new "variadic table"** — reuses `ClassObject.methods: IndexMap<Symbol, ObjRef>` under
  the `(*)` selector; a same-name duplicate variadic silently overwins, same as any
  duplicate-selector redefinition → [forge/DEFERRED.md](../../DEFERRED.md) #24.
- **No `callable.rs`/`closure.rs` changes** — the variadic flag is read from
  `MethodObject.signature` directly in `call_method`.
- **No `Bytecode::SendDynamic` / call-site spread (`f(*args)`)** — U8's DEFERRED #21
  forward-note ("U9 owns the opcode") is superseded; spread-call syntax remains a future
  unit's job; `bytecode.rs`/`disasm.rs` untouched.
- `decode_selector` does not recover `F` from a `(*)` selector (see Implementation).

## Sources
- [forge/archive/phase2/STATE.md](../../archive/phase2/STATE.md) "U9 — LANDED"; [forge/archive/phase2/PHASE2-INDEX.md](../../archive/phase2/PHASE2-INDEX.md).
  Per-unit planning record (`U9-plan.md`, `U9-implementation-spec.md`) folded into this spec; see git history.
- Commit `c9805d0`.
- Code: `phalcom-ast/src/{ast.rs,parser.rs}`, `phalcom-core/src/{method.rs,compiler/lib.rs,vm.rs}`.
