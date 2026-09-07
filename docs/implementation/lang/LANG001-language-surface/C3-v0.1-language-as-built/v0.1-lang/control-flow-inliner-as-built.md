# U5 — Control-Flow Inliner (as-built)

- **Status:** ✅ Landed — `83c908a` (`feat(u5): lower operators to sends + sacred-selector inliner with override guards`). In-tree on `main`, no worktree.
- **Realizes:** [ADR-0018](../../../adr/0018-sacred-selector-inliner-and-override-guard.md) (sacred-selector inliner + override-epoch deopt guard); builds on [ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md) (label-encoded selectors + IC-ready dispatch — the inliner is a coarse inline cache over the sacred selectors), [ADR-0013](../../../adr/0013-closure-upvalues-and-frame-token-return.md) (blocks). Spec: [control-flow.md](../../../spec/current/control-flow.md) §1–3.
- **Reviewer gate:** OFF per STATE.md review policy (U5 is not in the load-bearing hierarchy set U1/U2/U4/U6) — self-verified on the green gate + `cargo doc` + the §7 soundness tests.

## Mission
Make **every** control-flow construct an ordinary message send — arithmetic/comparison operators, `and`/`or`/`not`, `if`/`while`, and the `ifTrue:`/`whileTrue:` block families all dispatch through `Invoke` (Smalltalk semantics) — then add a compile-time **inliner** that lowers a fixed set of *sacred selectors* sent with **literal-block arguments** to guarded branch/jump bytecode, with a runtime **deopt guard** that falls back to a real send whenever the receiver is not the expected kernel `Bool`/`Block` **or** the sacred selector has been overridden, so the inlined path is observationally identical to the send it replaces (including non-local return).

## Surface / behavior
Operators and keyword control flow are surface sugar over sends; the two forms produce identical output.

```phalcom
if (x < 0) { "neg" } else { "pos" }     // desugars to  (x.<(0)).ifTrue({ "neg" }) ifFalse({ "pos" })
while (i < n) { i = i + 1 }              // desugars to  { i.<(n) }.whileTrue({ i = i + 1 })
a and b                                  // desugars to  a.and({ b })   — lazy, short-circuit
a + b                                    // desugars to  a.+(b)         — plain send, never inlined
```

A sacred send whose block arguments are **literal at the call site** compiles to guarded jumps with **zero closure allocation and zero call frames** on the common path. Overriding a sacred method on the kernel class (e.g. `Bool>>and(_:)`) is honored: the guard deopts to the real send.

## Implementation
Layer 0 — operators become sends (`compiler/lib.rs`, `bytecode.rs`, `vm.rs`, `primitive/*`):
- Removed the hardwired operator opcodes (`Add,Subtract,Multiply,Divide,Modulo,Equal,NotEqual,Less,LessEqual,Greater,GreaterEqual,And,Or,Negate,Not`) along with their `vm.rs` arms, the `binary_op!` macro, and `handle_primitive_op`. `Binary`/`Unary` now lower to `Invoke` via the single `encode_selector` helper.
- Backing primitives registered: `Number` (`+ - * / % == != < <= > >=` and unary negate; IEEE-754 division allows `inf`/`NaN`, no error), `Boolean` (`and(_:) or(_:) not() ifTrue(_:) ifFalse(_:) ifTrue(_:)ifFalse(_:)`), `Block` (`whileTrue(_:)`), `Object` (generic `==`/`!=` via `value_eq`), `String` (`+`).
- `and`/`or` are lazy: `a and b` lowers to `a.and({ b })` — laziness falls out of wrapping the RHS in a block.

Surface parsing (`phalcom-ast`, per DEC-E = "U5 owns"): `parse_if`/`parse_while` (and `parse_brace_block`) parse the keyword forms and desugar them at parse time to `MethodCall` sends over U4 block literals. `if`/`else` desugars to the combined `ifTrue(_:)ifFalse(_:)` sacred selector (not the spec's illustrative `ifNone` chain, which depends on U6's `Option`); plain `if` (no else) → `ifTrue(_:)`.

Layer 1 — the inliner (`compiler/inliner.rs`, new): recognizes a **sacred selector** sent with **literal-block** arguments and emits guarded jump bytecode; otherwise falls through to the ordinary Layer-0 send. Entry point `compile_sacred_call_want`, with per-selector emitters `compile_if_true`, `compile_if_false`, `compile_if_true_if_false`, `compile_and`, `compile_or`, `compile_while_true`; block bodies are spliced inline via `compile_inline_block_body` (no `ClosureObject` alloc), and `emit_sacred_send` emits the deopt fallback. New opcodes in `bytecode.rs`:
- `Jump(i32)`, `JumpIfFalse(i32)`, `Loop(i32)` — control transfer. Offset width is `i32` (not `i16`) because inlining can grow a body past ±32 KB and silent truncation would be a correctness bug.
- `GuardBool(i32)` / `GuardBlock(i32)` — the **deopt-guard** opcodes: peek the receiver; if it is not the kernel `Bool`/`Block` representation **or** the sacred flag is dirty, branch to the fallback offset without consuming the receiver.

The deopt guard = type check **+ override epoch** (the soundness core). Because `Value::Bool` is an immediate whose class is always the kernel boolean and U4 blocks are always kernel `Block`, the only override risk is replacing a sacred method **on the kernel class itself**. `universe.rs` holds coarse per-kernel-class pristine flags (`bool_sacred_pristine`, `block_sacred_pristine`) over the watched selector sets (`BLOCK_SACRED_SELECTORS = ["whileTrue(_:)"]`, and the Bool sacred set); set true at bootstrap, flipped false by `note_method_installed` when any sacred selector is re-installed on `Bool`/`Block` (hooked on `Bytecode::Method`). The guard deopts iff `!matches!(recv, expected)` OR `!pristine` — a coarse O(1) inline-cache invalidation. "Sealing" the sacred selectors was rejected: the spec wants `and`/`or` overridable.

Non-local return through inlined blocks: inlined bodies are spliced into the home method's chunk, so a `return` inside them is the method's ordinary `Return` and unwinds to the home method for free — identical to the send form's frame-token return.

`CallContext::Immediate` (`frame.rs`) was added because invoking a closure-backed sacred method on an immediate receiver (`Bool`/`Number`) — the post-deopt override path — previously panicked in `Value::to_context`.

## Invariants & tests
- Semantic equivalence goldens: `control-flow/control_flow_send_equivalence.ph` (keyword form ≡ explicit send), `control_flow_if_else.ph`, `control_flow_while_let.ph`, `arithmetic/arithmetic_comparisons.ph`, promoted `bool_short_circuit_{and,or}.ph`.
- Override honored (deopt correctness): `control-flow/control_flow_inline_override_honored.ph` — overriding a sacred selector runs the override, proving the site deopts on the dirty pristine flag.
- Non-local return transparency: `control-flow/control_flow_inline_non_local_return.ph` — a `return` inside an inlined block unwinds the home method.
- Guard-on-wrong-type: `runtime-errors/runtime_inline_guard_wrong_type.ph` and `runtime_and_non_boolean_operand.ph` deopt to a real send / hard type error.
- Two pre-existing goldens re-pinned to the corrected operator-dispatch behavior: `runtime_and_non_boolean_operand.expected`, `classes/class_instance_equality_identity.expected` (and `runtime_comparison_unsupported`).
- `verify.sh` green at both the Layer-0 gate and the Layer-1 gate; goldens otherwise byte-identical.

## Deviations & deferrals
- **Paired selector spelled `ifTrue(_:)ifFalse(_:)`** (keyword-labelled), not the spec's illustrative comma-form `ifTrue(_)ifFalse(_)` — matches Phalcom's actual selector model ([ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md)).
- **Jump offset width `i32`**, not `i16` (see above).
- **Class reopening added** to `Statement::Class` (attach to an existing same-named global instead of shadowing) so a sacred override is testable from surface Phalcom; `install_core` now also registers kernel `Function`/`Block` as globals (they were silently shadowed — a real bug fixed this session).
- **`if`/`else` desugars to `ifTrue(_:)ifFalse(_:)`**, not to an `Option`/`ifNone` chain — keeps U5 independent of U6.
- **`for (x in xs)` deferred** — it desugars to `xs.each { … }`, a non-sacred send needing an iterable protocol no kernel type defines until collections exist.
- **`repeat(_:)` deferred** — its receiver/semantics are unpinned (U5-plan BD-U5-2); only `whileTrue(_:)` is the inlinable loop selector. See [`docs/forge/DEFERRED.md`](../../DEFERRED.md) and [deferred-work.md](../../../spec/current/deferred-work.md).

## Sources
- Forge work order (`U5-plan.md`) folded into this spec (see git history); landing record: [`../../archive/phase2/u0-state.md`](../../archive/phase2/u0-state.md) "U5 — LANDED"; DEC-E in [`docs/forge/archive/phase2/PHASE2-INDEX.md`](../../archive/phase2/PHASE2-INDEX.md) §4.
- Code: `phalcom-core/src/compiler/inliner.rs` (new), `phalcom-core/src/compiler/lib.rs`, `phalcom-core/src/{bytecode,vm,universe,value,frame}.rs`, `phalcom-core/src/primitive/{number,boolean,string,object,block}.rs`; `phalcom-ast/src/{ast,parser}.rs`.
