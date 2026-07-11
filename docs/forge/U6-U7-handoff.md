# Handoff — U5 + U6 landed; U7 is next

**Repo:** `/Users/altunhasanli/dev/phalcom/phalcom`, working directly on `main`, no worktree, not pushed.
Authoritative state = `docs/forge/STATE.md`; map = `docs/forge/PHASE2-INDEX.md`. Navigate the code via
graphify (`graphify query/explain/affected`), not blind file sweeps.

## Status: U5 ✅ LANDED, U6 ✅ LANDED

### U5 — control-flow-as-message + sacred-selector inliner (commit `83c908a` + docs)
- Hardwired operator opcodes removed; all operators lower to `Invoke` message sends.
- Sacred-selector inliner (`compiler/inliner.rs`): guarded-jump inlining of `ifTrue:`/`ifFalse:`/
  `ifTrue:ifFalse:`/`and:`/`or:`/`whileTrue:` with an **override-epoch deopt guard** (`GuardBool`/
  `GuardBlock`) — **ADR-0018**. Deviations: `i32` jump offsets (not `i16`); selector spelling
  `ifTrue(_:ifFalse:)`; class reopening (same-named `class` reopens the global instead of shadowing);
  `CallContext::Immediate` for closure-backed methods on immediate receivers.

### U6 — absence → Option + let/var bindings (commits `3bc6ede` → `5b239ab` → `318e752` → `51f56e4` → `aa8bb8b`)
- **Absence = `Option`** (abstract) with `Some` (field `_value`) + `None` (shared singleton), mirroring
  Bool/True/False — **ADR-0007/0004**. Bootstrapped via **Rust primitives** (`primitive/nil.rs`), so U6
  is **independent of U7's `construct`**. Global `None` = the singleton value.
- **Construction is `Some.new(x)`** — there is **no bare `Some(x)` call syntax** in Phalcom (deviation).
- **Eliminator selector is `match(some:none:)`** on `Option` (deviation from any `match(some:)(none:)` spelling).
- **`let`/`var` bindings** — **ADR-0014**. `let` immutable (reassign = compile error `AssignToImmutable`);
  `var` mutable; `var x` no-init reads `None`; `let x` no-init = compile error `LetWithoutInitializer`.
- **`??`/`?.` are desugared in the parser** (no new `Expr` nodes): `a ?? b` → `a.orElse { b }`;
  `opt?.foo` → `opt.map { ‹recv› => ‹recv›.foo }` (synthetic receiver `" recv"`, non-lexable).
- **Invariant 4 (the load-bearing one): the private `Value::Nil` sentinel never reaches user code.**
  Established at its source: `VM::none_value()` is the *sole* producer of surface absence; `Bytecode::Nil`
  pushes the `None` singleton (not raw NIL); every surface-reachable primitive returns `None`; the raw
  sentinel exists **only** as an allocator storage default and is surfaced to `None` at every `Get*`/
  `Return` read boundary. **No opcode and no surface-reachable primitive yields the raw sentinel.**
- **Value-less block/method bodies yield `None`** (fall-off-end, empty body) — `compile_block` mirrors
  `compile_inline_block_body`, so **inlined ≡ non-inlined** (the speculative-opt ⊗ observable-semantics hazard).
- **Bare `return` → `None`** (pre-authorized, folded into the surfacing boundary).
- **BD-U6-1 shipped as Option A** — **ADR-0021**: Option/Some/None never implement the boolean-branch
  protocol → non-`Bool` condition is a hard runtime type error (via U5's `GuardBool`); PLUS the compiler
  rejects syntactically-literal Option conditions (`if(None)`, `if(Some.new(…))`) at compile time. Refines
  spec §3.5 "compile error" → "compile error where statically detectable + hard runtime type error otherwise."

### Reviewer gate (U6 was load-bearing → reviewer ON)
An independent `phalcom-reviewer` **BLOCKED once** on inlined ≠ non-inlined for value-less bodies (empty
method returned `self`, empty non-inlined block returned `<block>`, while the inliner returned `None`).
Fixed in `51f56e4`; independently re-confirmed (`true.ifTrue { }`, empty `match` `none:` block, and
`{ }.call()` all print `<None instance>`). Invariant-4 leak itself was verified **closed**. Green gate:
`./scripts/verify.sh` exit 0; `cargo doc --workspace --no-deps` clean.

## Deferred / known gaps (do not silently inherit)
- **U-STD owns the Option combinator bodies.** `core.ph` declares only Option/Some/None **skeletons**;
  `map`/`flatMap`/`filter`/`orElse`/`ifSome`/`unwrapOr`/… are U-STD's job (two method defs each, over
  `match`). **`??`/`?.` desugar to `orElse`/`map`, so their full end-to-end goldens depend on those bodies
  existing** — verify whether the `??`/`?.` positive goldens ran or were deferred to U-STD, and finish them there.
- **DEFERRED #13** — captured-`let` reassignment (through an enclosing frame / upvalue) is not rejected;
  only current-function-local and module-global `let` reassignment is caught. (`compiler/lib.rs` assignment path, ADR-0014.)
- **DEFERRED #14** — the `if(opt)` check is literal-only and `OptionTruthiness` carries no span. (ADR-0007.)
- **Incidental (trivial):** `is_option_literal`'s rustdoc in `compiler/lib.rs` links ADR-0007 as
  `0007-option-type.md`; the real file is `0007-option-as-abstract-with-some-none.md` — dead relative link,
  `cargo doc` doesn't flag it. Fix opportunistically.
- Pre-existing, not U6: `error.rs:30 extra_unused_lifetimes` clippy warning (DEFERRED #3); `bool_class_new`
  debug `println!`s (boolean.rs:32,34); `nil.rs:10 pub const NIL` now surface-unused.

## Next: U7
Follow `docs/forge/U7-plan.md` (+ `U7-U8-handoff.md`). U6 was deliberately built so `Some`/`None` do **not**
depend on U7's user-facing `construct` — U7 can now land field/`construct` dispatch on top of a stable
absence model. Reviewer stays ON for load-bearing units. Run `graphify update . --no-cluster` before each
commit; commit at each green checkpoint; do not push.
