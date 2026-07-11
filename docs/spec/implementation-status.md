# Implementation Status

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

Divergence between this specification and the current source tree. The tree today
is a Wren/clox-style VM (arity dispatch, operators-as-opcodes, visible `nil`,
dynamic field maps, no blocks). The spec is a Smalltalk-semantics language. Most of
the spec is greenfield, not refactor — nothing is *wrong*, it simply predates the
spec.

> **Redesign direction (as of July 2026):**
> - **Front end:** the parser/AST is being hand-written; LALRPOP is removed
>   (planned per ADR-0016). `phalcom-ast` is the active write-set for this work.
> - **Object graph / heap:** `Rc<RefCell<T>>` is replaced by a handle/arena heap —
>   `ObjRef`/`ClassId` are `Copy` integer handles into a central `Heap`
>   ([ADR-0009](../adr/0009-handle-arena-heap.md)). `phalcom-core/src` is the
>   active write-set for this work (U1).
> - **Selectors:** arity-only dispatch is replaced by label-encoded selector symbols
>   ([ADR-0012](../adr/0012-selector-signature-encoding-and-dispatch.md));
>   `Signature { selector, kind, positional_arity, variadic }` keyed by interned
>   `Symbol`.
> - **Value repr:** `Value` becomes a tagged `enum` with a private `Nil` sentinel
>   ([ADR-0010](../adr/0010-tagged-value-enum.md)).
> - **Absence:** surface `nil` is removed; absence is `Option`/`Some`/`None`
>   ([ADR-0007](../adr/0007-option-as-abstract-with-some-none.md)).
> - **Bindings:** `let` (immutable) and `var` (mutable) are ratified
>   ([ADR-0014](../adr/0014-let-and-var-bindings.md)).

---

## Tier S — foundational (each blocks an invariant; do first)

| # | Spec | Requires | Current state | ADR |
|---|------|----------|---------------|-----|
| 1 | [Messages & Selectors](messages-and-selectors.md) | Selector = **name + labels**, interned; lookup one hashmap hit | Arity-only: `SignatureKind::Method(u8)` (`method.rs`), `Invoke(u8, u16)` carries argc (`bytecode.rs`), `MethodDef.params: Vec<String>` — no label channel (`ast.rs`). **In progress (planned per ADR-0012).** | [ADR-0012](../adr/0012-selector-signature-encoding-and-dispatch.md) |
| 2 | [Blocks](blocks.md) | Blocks are the keystone; method = block bound to a selector | **No block/lambda anywhere** — no AST node, no `Closure`/`Call`/`MakeClosure`/jump opcodes. `=>` lexes but has no AST. **Planned.** | [ADR-0013](../adr/0013-closure-upvalues-and-frame-token-return.md) |
| 3 | [Control Flow](control-flow.md), Inv. 1 | Operators & control flow **desugar to sends** (compiler may inline) | Hardwired opcodes `Add/And/Or/Negate/…` (`bytecode.rs`); `BinaryOp::And/Or` eager (`ast.rs`). `a+b` never sends `+(_:)`; `number_add` primitive is dead on that path; `and`/`or` are not lazy. **Planned.** | [ADR-0012](../adr/0012-selector-signature-encoding-and-dispatch.md) |
| 4 | [Values & Absence](values-and-absence.md), Inv. 4 | No surface `nil`; absence is `Option` | `nil` is first-class: keyword, `Expr::Nil`, `Bytecode::Nil`, `Value::Nil`. No `Option`/`Some`/`None`. **Planned per ADR-0007/ADR-0010.** | [ADR-0007](../adr/0007-option-as-abstract-with-some-none.md), [ADR-0010](../adr/0010-tagged-value-enum.md) |

## Tier A — major subsystems absent

| # | Spec | Requires | Current state | ADR |
|---|------|----------|---------------|-----|
| 5 | [Classes §1](classes.md) | `construct` → alloc + body + implicit `self`, on the metaclass | No `construct` token/node. `SignatureKind::Initializer(u8)` exists but is unreachable from surface. `ClassMember` = Method/Getter/Setter only. **Planned.** | [ADR-0011](../adr/0011-static-instance-slot-layout.md) |
| 6 | [Classes §2](classes.md) | Static slot layout; implicit field decl; read-before-write compile error; private/non-inherited | `InstanceObject.fields: IndexMap<Symbol, Value>` — a dynamic per-instance map. No field-set collection, no read-before-write check. `GetField/SetField(u16)` exist but the store isn't slotted. **Planned per ADR-0011.** | [ADR-0011](../adr/0011-static-instance-slot-layout.md) |
| 7 | [Method Lookup §2](method-lookup.md) | Failed send → `Message` → `doesNotUnderstand(_:)` | Lookup returns `Option<Method>`; miss is a hard error. No `Message` value, no dNU hook. **Planned.** | — |
| 8 | [Messages & Selectors §4–5](messages-and-selectors.md) | Rest `*p`, spread, variadic table, `SEND_DYNAMIC`, `perform` | None. `*` is only multiply. **Planned per ADR-0012.** | [ADR-0012](../adr/0012-selector-signature-encoding-and-dispatch.md) |
| 9 | [Control Flow §3](control-flow.md) | Sacred-selector inliner with deopt guard | No jump opcodes at all; no inline caches. **Planned (IC shape reserved per ADR-0012).** | [ADR-0012](../adr/0012-selector-signature-encoding-and-dispatch.md) |
| 10 | [Blocks §5](blocks.md) | Non-local `return` via frame token + `DeadFrameError` | None (no blocks). **Planned per ADR-0013.** | [ADR-0013](../adr/0013-closure-upvalues-and-frame-token-return.md) |

## Tier B — surface / lexer gaps

| Spec | Gap |
|------|-----|
| [Lexical §5](lexical-structure.md) | String interpolation: none — `lex_string` only strips quotes. |
| [Lexical §4](lexical-structure.md) | Numeric separators `1_000_000`: unsupported (`Number` regex). |
| [Lexical §1](lexical-structure.md) | Newline suppression state machine: absent (raw `Newline` tokens). |
| [Lexical §4/§6](lexical-structure.md) | Tuple/list/map/set literals + brace disambiguation: no AST nodes. |
| [Open Q1 → ADR-0014](open-questions.md) | `var` keyword: only `let` exists. Planned per [ADR-0014](../adr/0014-let-and-var-bindings.md). |
| [Lexical §3](lexical-structure.md) | Field token not lexically distinguished (identifier regex swallows `_name`). Minor — `Expr::Field` exists. |
| [Blocks §1–4](blocks.md), [Classes §3](classes.md) | `=>` expression bodies, trailing-block sugar, unbraced arrow: none. |

## What already aligns (keep)

- **Dot-send pipeline** → `MethodCall`/`GetProperty`/`SetProperty` compiling to
  `Invoke` **with a selector constant index**. Right shape — label encoding slots
  into the existing selector constant ([ADR-0012](../adr/0012-selector-signature-encoding-and-dispatch.md)).
- **Symbol interning**; **`static` flag** end-to-end (`Method(u16, bool)` +
  `primitive_static!` installing on the metaclass).
- **Getter ≠ method**, **setter `name=(_)`** already modeled.
- **`GetField/SetField` slot opcodes** — right direction even though the store is
  wrong ([ADR-0011](../adr/0011-static-instance-slot-layout.md) will fix the backing).
- **Metaclass objects exist** — though the tower has the parallel-superclass bug
  ([Object Model §5](object-model.md), rule 4).

## Recommended implementation order

> Items marked **[in progress]** are actively being worked on in the current
> sprint.  Items marked **[planned per ADR]** have ratified decisions but work has
> not yet started.

1. **Selector redesign** (#1) — `Signature`/`SignatureKind`, the `Invoke` opcode,
   the compiler. Every later feature (variadics, dNU, `perform`) assumes it.
   **[in progress — ADR-0012]**

2. **Blocks** (#2) — unblocks control-flow-as-message, non-local return, the
   inliner. **[planned per ADR-0013]**

3. In parallel: **operators → sends** (#3) and **`nil` → `Option`** (#4).
   **[planned per ADR-0007, ADR-0010, ADR-0012]**

4. **Metaclass tower fix** + `verify_invariants()`
   ([Object Model §5–6](object-model.md)) — small and self-contained; can land any
   time as the foundation for `construct` (#5).  **[planned per ADR-0002, ADR-0003]**

5. **Handle/arena heap** — replaces `Rc<RefCell<T>>`, enabling the GC-ready
   ownership model. **[in progress — ADR-0009]**

6. Then #5–#10 as features on the corrected core.
