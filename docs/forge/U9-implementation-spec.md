# U9 — Implementation Specification (supersedes U9-plan.md on conflict)

_Grounded against actual HEAD as of commit `83221e8` (U8 landed, green). This
document exists because `U9-plan.md` was written before U8 merged and contains
several assumptions that do not match the current tree. **Where this document
and `U9-plan.md` disagree, follow this document.** Where this document is
silent, `U9-plan.md` still governs (mission, guardrails, mandatory rules,
return-contract shape)._

Written for a **medium-effort implementer** — every non-trivial decision below
is already made. Do not re-derive them; do not re-read `messages-and-selectors.md`
§4/§5 looking for an answer this doc already gives. If you hit a fact that
contradicts this doc, STOP and report the conflict rather than guessing.

---

## 0. The five corrections to `U9-plan.md`

1. **`signature.rs` is a dead stub — do not edit it.** It is a one-line,
   registered-but-empty module (`pub mod signature;` in `lib.rs:25`). Every
   place `U9-plan.md` says "`signature.rs`" (§3 write-set, §4 design, §6
   fold-in), read **`phalcom-core/src/method.rs`** instead — `Signature`,
   `SignatureKind`, `encode_selector`, `decode_selector`, and `make_signature`
   all live there (confirmed at `method.rs:23` (`SignatureKind`), `method.rs:40`
   (`Signature`), `method.rs:82` (`encode_selector`)). Do not create or restore
   content in `signature.rs`.

2. **No new "variadic table" data structure — reuse the existing method map.**
   `U9-plan.md` BLOCKED-ON-DECISION #2 assumes a dedicated per-class table
   keyed by `(name, min_positional_arity)`. This is unnecessary: `ClassObject`
   already stores `methods: IndexMap<Symbol, ObjRef>` (`class.rs:16,34`), keyed
   by the exact **encoded selector symbol**. A variadic method is just another
   entry in that same map, under its own distinct selector (§2 below). The
   miss-path probe does a **second exact lookup** using a selector it derives
   from the failed call's bare name — no new storage, no `class.rs` changes at
   all. This resolves BLOCKED #2: **Option (a) from the plan (one variadic per
   name per class) is correct, but realize it via the existing map, not a new
   table.**

3. **Canonical variadic selector spelling: `<name>(*)`, always — independent of
   fixed-prefix arity.** `messages-and-selectors.md` §4 states literally: "A
   variadic method interns as `sum(*)`." It gives no fixed-prefix example. Do
   not invent one (the plan's `format(_:*)` guess is exactly that — a guess,
   and it makes the miss-path probe need to know `F` before it can compute the
   candidate selector, which the probe cannot know in advance). Instead: **the
   selector text never encodes the fixed-prefix count; only `Signature.variadic`
   and `Signature.positional_arity` do.** So `sum(*numbers)` and
   `format(fmt, *args)` both intern their defining selector as `sum(*)` /
   `format(*)` respectively — bare name, literal `(*)`. This makes the
   miss-path probe a pure function of the failed call's bare name: no probing
   over possible `F` values needed. Labels are still forbidden on a variadic
   parameter (per plan/spec, unchanged) — this is about selector *spelling*,
   not about what the parameter list may contain.

4. **No `callable.rs` / `closure.rs` changes at all.** The plan's write-set
   lists both. Neither needs to change:
   - `Callable`/`ClosureObject` carry no variadic flag today, and don't need
     one. `compile_block` (`compiler/lib.rs:362-449`) uniformly declares one
     local slot per parameter name regardless of arity kind — it already does
     the right thing with zero changes, because the *value* bound to the rest
     slot (an already-built `List`) is prepared by the VM's call prologue
     **before** the frame runs (§4 below), not by anything in `compile_block`.
   - The variadic flag the VM prologue needs is read from
     `MethodObject.signature.variadic` (already exists,
     `method.rs:48`/`Signature`), which `call_method` can reach directly since
     it already receives the method's `ObjRef` — no threading through
     `Callable` required.
   - This means **block variadics require zero of this unit's plumbing to
     "just work,"** which is exactly why they must be explicitly blocked at
     the parser/compiler boundary (§3 point 4) rather than silently
     accepted-but-broken.

5. **`Bytecode::SendDynamic` and call-site spread (`f(*args)`) are OUT OF
   SCOPE for this unit**, notwithstanding `DEFERRED.md` #21's forward-looking
   note ("U9 owns both the opcode and its handler"). Re-read `U9-plan.md` §0
   and §3: the mission and write-set are about **declaring** a rest parameter
   and **dispatching** a call to it — nowhere does the plan ask for spread
   syntax at call sites (that is `messages-and-selectors.md` §5, a separate,
   later section, not cited anywhere in U9-plan.md §4). Do not add
   `Bytecode::SendDynamic`, do not touch `bytecode.rs` or `disasm.rs` at all.
   Record in your `DEFERRED.md` addition that #21's forward note is
   superseded: spread-call syntax remains a future unit's job.

---

## 1. Preconditions — already verified, do not re-check

- U1, U4, U8 are merged and green (commit `83221e8`). `verify.sh` is green on
  `main` right now.
- `List` (U-LIST) is landed: `phalcom-core/src/list.rs` (`ListObject`),
  `Heap::alloc_list` (`heap.rs:123`), and the `.ph` `at`/`size`/`add`/`each`
  protocol in `core.ph`. **`reduce` does NOT exist** — `core.ph` explicitly
  defers `map`/`reduce`/`filter` to U-STD (see the comment directly above
  `class List {}` in `core.ph`). **`U9-plan.md`'s own test-strategy example
  (`numbers.reduce(0){acc,n => acc+n}`) is not runnable today.** Use `each`
  with a `var` accumulator instead (§6 below has the exact fixture).
- `Signature { selector, kind, positional_arity, variadic }` exists
  (`method.rs:40-49`); `variadic: bool` is currently always `false` (hardcoded
  in `Signature::new`, `method.rs:66`) — nothing sets it `true` yet. This is
  core U9 work.
- `callWith` does not exist as a primitive (checked: no match in
  `primitive/object.rs`). Per the plan, do not implement it. Note the
  no-op interaction in your `DEFERRED.md` entry only if you find yourself
  tempted to add it — otherwise no action needed.
- `ParameterDef { name: String, label: Option<String>, range: SourceRange }`
  (`phalcom-ast/src/ast.rs:55-59`) has no rest marker. `*` already tokenizes as
  `Token::Asterisk` (multiply operator, `lexer.rs:243`) — **no lexer change
  needed**, confirmed.
- `parse_param_list` (`phalcom-ast/src/parser.rs:662-682`) is the single
  parameter-list parser. **Before editing it, grep every call site** (it is
  very likely shared by method/constructor param lists *and* block-literal
  param lists). If it is shared with block literals, you must prevent a block
  literal from silently accepting `*rest` and mis-binding it (§3 point 4) —
  either gate rest-parsing on a caller-supplied `allow_rest: bool`, or parse it
  unconditionally and reject `is_rest` in the block-compiling path with a
  clean diagnostic. Pick whichever is less invasive once you've seen the call
  sites; document the choice in your return report.
- Run `./scripts/verify.sh` on your starting tree before the first edit to
  confirm your baseline is green.
- **Isolation:** work in-tree on `main`, sequentially (this mirrors how U8 was
  actually executed, not `U9-plan.md`'s worktree instruction — there is no
  parallel U11 work happening alongside you in this session). Commit at each
  green checkpoint, same convention as U8.

---

## 2. Design decisions (all BLOCKED-ON-DECISION items resolved)

**BLOCKED #1 (List availability):** Resolved — `List` is landed (U-LIST,
commits `c7c63fb`/`6fdf0c7`/`b2f7aec`/`333823a`). Proceed.

**BLOCKED #2 (variadic-table key / probe shape):** Resolved — no table. A
class's normal `methods: IndexMap<Symbol, ObjRef>` holds the variadic method
under its own literal selector symbol, `<name>(*)` (§0 point 3). At most one
variadic per bare name per class falls out naturally: if two variadic methods
with the same name are declared in one class body, they collide on the same
selector symbol and the second silently overwins the first in the `IndexMap`
— this is the **same pre-existing behavior as any duplicate-selector
redefinition** in this codebase today (not new, not this unit's job to guard
against; note it as a candidate `DEFERRED.md` entry if you want a clean
diagnostic later, but do not build one now).

**BLOCKED #3 (block variadics):** Resolved — **out of scope**. No `{ *xs => }`
grammar exists, and per §0 point 4 nothing downstream would give it correct
semantics without extra plumbing this unit deliberately skips. If your parser
change happens to make `{ *xs => ... }` parse, you MUST explicitly reject it
(clean diagnostic, not silent misbehavior) — see §1's `parse_param_list` note.
Add a `DEFERRED.md` entry for block variadics (folds/updates the existing
entry #9).

**Selector spelling:** `<name>(*)`, always (§0 point 3). No fixed-prefix
encoding in the selector text.

**Signature accounting for a variadic method:**
- `Signature.variadic = true`
- `Signature.positional_arity = F` — the **fixed/minimum** prefix arity (0 for
  `sum(*numbers)`, 1 for `format(fmt, *args)`). This is already exactly what
  the plan calls "min_positional_arity" — **no new field needed**, just this
  documented interpretation of the existing `positional_arity` field when
  `variadic` is `true`.
- The **compiled `Callable.arity`** (local-slot count) is `F + 1` (the fixed
  params plus one slot for the rest parameter, which the compiler already
  produces naturally since `params.len()` includes the rest param as an
  ordinary trailing entry — no `Callable` changes needed, per §0 point 4).

**Runtime dispatch rule (the actual "probe"):** On an exact-selector miss for
a `SignatureKind::Method` selector with an **all-positional** label list
(labels all `None` — never probe variadic for a labelled, getter, setter, or
subscript selector, matching "positional-only"), derive the bare name via
`decode_selector`, build the literal string `format!("{bare_name}(*)")`, intern
it, and do **one** ordinary `lookup_method` call (same superclass-chain walk
the exact probe already uses). On a hit, check `arity >= method.signature.positional_arity`
(the call supplied at least the fixed prefix); if true, dispatch via the
existing `call_method`. If the lookup misses, **or** hits but `arity` is too
low, fall through to `forward_does_not_understand` exactly as before — a
too-few-args variadic call is not distinguished from "no such method"; this
matches the plan's own dispatch-ordering test (§7) and needs no new error
variant.

**Call prologue (the subtle part — reasoned through and pinned):** In
`vm.rs::call_method`'s `MethodKind::Closure` arm, **before** building the new
call frame:
```rust
MethodKind::Closure(closure_id) => {
    let context = callee.to_context(&self.heap);
    let receiver_idx = self.stack.len() - arity - 1;
    let (variadic, fixed_arity) = {
        let sig = &self.heap.method(method).signature;
        (sig.variadic, sig.positional_arity as usize)
    };
    if variadic {
        // Collect the trailing `arity - fixed_arity` positional args into a
        // List bound to the rest slot. `receiver_idx` does not move — only
        // the tail past the fixed prefix is replaced by a single List value,
        // so `stack_offset` below is unaffected by this collapse.
        let rest = self.stack.split_off(receiver_idx + 1 + fixed_arity);
        let list_id = self.heap.alloc_list(rest);
        self.stack.push(Value::Obj(list_id));
    }
    let stack_offset = receiver_idx;
    let new_frame = self.new_call_frame(closure_id, context, 0, stack_offset, Some(source_range));
    self.frames.push(new_frame);
    Ok(())
}
```
Why this is correct: `receiver_idx` is computed from the stack length *before*
any mutation, using the true call-site `arity` (`K`). `Vec::split_off(receiver_idx
+ 1 + fixed_arity)` removes and returns everything from index `receiver_idx +
1 + F` onward — exactly the `K - F` rest arguments — leaving the stack at
`receiver_idx + 1 + F` elements (receiver + F fixed args). Pushing one `List`
value brings it to `receiver_idx + 1 + F + 1 = receiver_idx + (F+1) + 1`
elements, i.e. exactly `F+1` locals after the receiver — matching
`Callable.arity = F+1` from §2 above. `stack_offset = receiver_idx` is
unchanged from the non-variadic path, so `CallFrame`'s slot addressing
(`stack[stack_offset + i]`) is correct with no further change anywhere else in
the VM. **This is the one invariant to assert in a test** (§6): after a
variadic call returns, the stack depth is back to its pre-call depth (same
`stack.len()` check the plan's fuzz idea uses, made a normal test instead of
opt-in fuzz, since the reasoning above is exact, not probabilistic).

`arity >= fixed_arity` is guaranteed by the dispatch rule above (§2, "Runtime
dispatch rule") before `call_method` is ever invoked for a variadic target —
`call_method` itself does not need to re-check this.

---

## 3. Confirmed write-set (line numbers as of commit `83221e8` — re-grep before
editing since your own edits will shift later files in this list)

| File | Exact change |
|---|---|
| `phalcom-ast/src/ast.rs` | `ParameterDef` (currently `ast.rs:55-59`): add `pub is_rest: bool`. Full rustdoc on the new field. |
| `phalcom-ast/src/parser.rs` | `parse_param_list` (currently `parser.rs:662-682`): parse an optional leading `Token::Asterisk` before the identifier; set `is_rest`. Reject (clean parser diagnostic, not a panic): a rest param that is not the list's last entry; a rest param with a label (`Colon` following it). **First grep all callers of `parse_param_list`** to check block-literal sharing (§1). |
| `phalcom-core/src/method.rs` | **(this is the plan's "signature.rs", corrected — §0 point 1)**: (a) add `SignatureKind::Variadic(u8)` (payload = fixed/min positional arity `F`) alongside the existing `Method(u8)` etc. (`method.rs:23-36`); (b) `encode_selector` (`method.rs:82-131`): add one arm producing `format!("{name}(*)")` for `SignatureKind::Variadic(_)`, ignoring the payload in the spelling (§0 point 3); (c) `Signature::new` (`method.rs:53-68`): recognize `SignatureKind::Variadic(f)` and set `positional_arity: f, variadic: true` (every other arm keeps `variadic: false` as today); (d) extend `decode_selector` (added in U8) to round-trip a `(*)`-suffixed selector back to `SignatureKind::Variadic(0)` — note: decode cannot recover `F` from the selector text alone (by design, §0 point 3), so `decode_selector`'s variadic arm returns a fixed arity of `0` as a documented limitation (only used today by the dNU `Message` reification path, which doesn't need the real `F`). |
| `phalcom-core/src/compiler/lib.rs` | The `ClassMember::Method(method_def)` arm (currently `compiler/lib.rs:723-748`): if `method_def.params.last().is_some_and(|p| p.is_rest)`, compute `F = method_def.params.len() - 1` and build the selector via `SignatureKind::Variadic(F as u8)` instead of `SignatureKind::Method(arity as u8)` (both the `encode_selector` call and the `MethodObject::new_single` call). Reject (compiler diagnostic) any *other* param in the list with `is_rest` set (only the last may be). This is the **only** compiler-side change — `compile_block` itself (`compiler/lib.rs:362-449`) needs **no changes** (§0 point 4). |
| `phalcom-core/src/vm.rs` | Two changes: (1) `call_method`'s `MethodKind::Closure` arm (currently `vm.rs:396-402`) — add the prologue exactly as written in §2 above. (2) The `[U9 SEAM]` comment inside `Bytecode::Invoke`'s miss arm (currently `vm.rs:952-959`) — replace the seam comment with the derived-selector probe described in §2's "Runtime dispatch rule." Keep `forward_does_not_understand` as the single terminal fallback (do not duplicate its body). |
| `phalcom-core/tests/lang.rs` | Add a dedicated `variadics()` test function (mirror the `list()` test at `lang.rs:168-173`, which calls `support::check_pass("list")`) plus a fixture directory `phalcom-core/tests/lang/variadics/` with `.ph`/`.expected` pairs (§6). Do **not** repurpose the existing `functions()`/`functions_pending()` test — that label covers the broader `Function` protocol, out of scope here. |

**Explicitly NOT touched:** `phalcom-ast/src/lexer.rs` (no token change needed
— confirmed), `phalcom-core/src/callable.rs`, `phalcom-core/src/closure.rs`,
`phalcom-core/src/class.rs`, `phalcom-core/src/bytecode.rs`,
`phalcom-core/src/bin/phalcom/disasm.rs` (§0 points 4-5).

---

## 4. Build order

1. `ast.rs` — `ParameterDef.is_rest`.
2. `parser.rs` — parse `*name`, reject non-last/labelled rest, resolve the
   block-literal sharing question from §1.
3. `method.rs` — `SignatureKind::Variadic(u8)`, `encode_selector` arm,
   `Signature::new` arm, `decode_selector` arm.
4. `compiler/lib.rs` — variadic branch in the `ClassMember::Method` arm.
5. `vm.rs` — call-prologue in `call_method`, derived-selector probe filling
   the `[U9 SEAM]`.
6. `tests/lang.rs` + fixtures — the acceptance corpus (§6).

---

## 5. Fold-in cleanup

None beyond what the plan already says (leave the Setter-arity comment in
`method.rs` alone unless you are already touching that exact block).

---

## 6. Test strategy — concrete fixtures (replaces the plan's `reduce`-based
example, which does not compile — §1)

Create `phalcom-core/tests/lang/variadics/` with pairs like the existing
`list` corpus. Suggested `.ph` (adapt to this repo's actual class/method
syntax as seen in neighboring fixtures — check `tests/lang/list/*.ph` for the
exact surface syntax conventions before writing these):

- **Zero-prefix:** a class with `sum(*numbers) { var total = 0; numbers.each({ n => total = total + n }); return total }`; call `sum(1,2,3)` → `6`; call `sum()` → `0`.
- **Fixed-prefix:** a class with `format(fmt, *args) { return args.size }`; call with 1 and 3 trailing args — asserts the prologue's non-zero-`F` math (the subtle case per §2).
- **Coexistence:** a class defining both `sum(_,_)` (an ordinary two-arg method) and `sum(*numbers)` — call with exactly 2 args and confirm the **fixed** method's result wins (not the variadic's), then call with 3 args and confirm the **variadic** fires.
- **Dispatch ordering / no hard error:** an unknown call shaped like a miss with no variadic entry falls through to U8's `doesNotUnderstand` path (reuse whatever assertion the U8 dNU golden already uses, e.g. a class-level `doesNotUnderstand(_:)` override that returns a sentinel) — proves the probe doesn't swallow the dNU fallback.
- **Compile-time rejections** (put under `tests/lang/syntax-errors/` or wherever U8/U-LIST put their negative goldens): `foo(*rest, x)` (rest not last) and `foo(to:, *rest)` (labelled + variadic) each produce a clean diagnostic, not a panic.
- **Rest is a real `List`:** inside the body, call `.size`/`.at(_)`/`.each(_)` on the bound rest parameter and assert real values come back (not a `Vec` leak) — this also implicitly covers the `reduce`-free replacement of the plan's example.
- **Stack-depth invariant:** after a variadic call returns (any of the above), assert the VM's value-stack length is back to its pre-call depth — this is the exact-math check from §2's prologue reasoning, promoted from the plan's "opt-in fuzz" idea to a normal assertion since the math is exact, not probabilistic. (If there's no existing test hook to inspect `vm.stack.len()` from the test harness, a black-box equivalent is: run several variadic calls in a loop inside one `.ph` program and confirm the program still terminates with the expected final value — a stack leak would show up as wrong results or a panic well before N iterations.)

---

## 7. Mandatory rules (unchanged from U9-plan.md §8 — repeated for emphasis)

- Full rustdoc on every new/changed public item: `ParameterDef::is_rest`,
  `SignatureKind::Variadic`, the `encode_selector`/`decode_selector` arms, the
  prologue code (a `///` on whichever function/arm holds it, explaining the
  `stack_offset` invariant from §2). `cargo doc --workspace --no-deps` — zero
  new warnings.
- `./scripts/verify.sh` exits 0 is your sole sign-off — reviewer is OFF for U9.
- Selector discipline: the `(*)` spelling goes through `encode_selector` only.
- No `unsafe` expected. If any lands, `// SAFETY:` note + run
  `rust-sanitizers-miri`.
- Commit at each green checkpoint (per-file build-order step or logical
  cluster), not one batch at the end. Run `graphify update . --no-cluster`
  before every commit.
- **Hard stop when green:** do not begin U10/U11/U-LEX/U-STD.

---

## 8. Return contract (answer all of these — mirrors U9-plan.md §9)

- Confirm (or report a deviation from) each of the five corrections in §0.
- The exact selector spelling produced for a zero-prefix and a fixed-prefix
  variadic (should both be `name(*)` per §0 point 3 — confirm your
  `encode_selector` arm actually produces this).
- Whether `parse_param_list` turned out to be shared with block literals, and
  which guard you chose (§1/§3 point 4).
- The prologue code as actually committed in `call_method`, and which test
  asserts the stack-depth invariant.
- Confirmation the exact probe and the dNU fallback are otherwise unchanged —
  quote the replaced `[U9 SEAM]` block as actually committed.
- Files changed, `verify.sh` tail, `cargo doc` tail.
- Any new `DEFERRED.md` entries: block variadics (mandatory — resolve/fold
  entry #9), the superseding note on #21 (spread call sites, §0 point 5), and
  anything else surfaced.
