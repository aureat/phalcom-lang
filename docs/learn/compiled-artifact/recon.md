# Recon — Doc 2 `compiled-artifact.md` (Chunk / Callable / Closure / Block)

Phase-1 scout. Arms the two briefs and the synthesis. Not prose, not a trace.

## 1. Architecture vs representation

**Architecture (the shape).** A "compiled function" in Phalcom is **not one object** — it is a
stack of layers, each owning a different axis of the problem:

| Layer | Type | What it owns | Heap object? | Cite |
|---|---|---|---|---|
| Code | `Chunk` | `code: Vec<Bytecode>`, `constants: Vec<Value>` (the constant pool), `spans`, `caches`/`gcaches` | no (plain struct, `Clone`) | `chunk.rs::Chunk` L44 |
| Recipe | `Callable` | a `Chunk` **by value** + `max_slots`, `num_upvalues`, `upvalues: Vec<UpvalueDescriptor>`, `arity`, `name_sym` | no (plain struct, `Clone`) | `callable.rs::Callable` L21 |
| Instantiation | `ClosureObject` | `callable: Rc<Callable>` + `module: ObjRef` + `upvalues: Vec<ObjRef>` (filled cells) | **yes** | `heap/closure.rs::ClosureObject` L24 |
| Stamped instantiation | `BlockObject` | `closure: ObjRef` + `home_frame_token: FrameToken` | **yes** (`Copy`) | `heap/block.rs::BlockObject` L18 |

**Representation (where the consequences live).** Two facts settle the doc:

1. **The recipe is shared by `Rc`, not the chunk.** `ClosureObject.callable` is `Rc<Callable>`
   (`closure.rs:28`). The `Chunk` sits *inside* the `Callable` by value, so wrapping the recipe in
   one `Rc` shares the code, constant pool, and all side tables in a single refcount — no per-
   materialization `Chunk` clone. Comment names the driver: U-HOTPATH. This is the payoff of Doc
   1's **Lie #1** (the hoisted `Rc<Callable>` in `run_until_inner`, dispatch.rs `:489`/`:530`).

2. **Upvalue descriptors live on the recipe; upvalue *cells* live on the instance.** `Callable`
   carries `Vec<UpvalueDescriptor>` — static `{is_local, index}` capture instructions
   (`callable.rs:10`). `ClosureObject` carries `Vec<ObjRef>` — the actual heap cells, filled fresh
   each time `Bytecode::Closure` materializes a template into an instance (closure.rs doc-comment).
   Descriptor = the *plan* to capture; cell = the *captured thing*. (Upvalue mechanics are the
   upvalue doc's; here they are one tie, not the subject.)

**The grip, grounded (see §2).**

## 2. The grip

> **A "compiled function" is three things, not one: a `Callable` is the immutable recipe (code +
> constant pool + capture plan), a `ClosureObject` is one instantiation of that recipe (recipe +
> module + filled cells), and a `BlockObject` is an instantiation stamped with the frame it was
> born in. The recipe is shared by `Rc`; the instantiation is minted fresh; the stamp is what makes
> non-local return possible.**

The confusion it collapses: the reader thinks "a closure" is one heap object holding code and
captured variables together (the textbook picture). Phalcom splits *shared-and-immutable* from
*minted-and-mutable-per-materialization* from *identity-bearing*, and each split buys one thing.

## 3. What was actually deliberated (honesty basis)

- **ADR-0006** (`0006-function-as-abstract-callable-root.md`) — *Decision*: `Function` is an
  abstract kernel class; `Block` and `Method` are **sibling** concrete subclasses; `Method.bind(_)`
  reuses Block machinery. This is the object-model framing (why a method and a block are the same
  kind of thing). **Deliberated.**
- **ADR-0013** (`0013-closure-upvalues-and-frame-token-return.md`) — closure upvalues + frame-token
  non-local return. The `home_frame_token` on `BlockObject` is this ADR. **Deliberated.**
- The `Rc<Callable>` sharing is **U-HOTPATH / perf-log** (memory: "Change 4 committed: Rc<Callable>
  wrapping for block literal efficiency," 6081), a measured perf cut — **an outcome, not an object-
  model principle**. Say so: the *three-layer split* is deliberated design; *which layer got the
  `Rc`* is a hot-path optimization. Agent B must confirm the perf citation exists.
- The layer **count** (Chunk-inside-Callable vs Callable-owns-Chunk-by-value) is not an ADR
  decision — it is how the types fell out. The pedagogical "why not fewer layers?" walk is
  **reconstruction**; the doc must label it as such (§5.2 / §5.5).

## 4. Correction the plan needs (representation error, caught here)

The Doc-2 plan said "**Callable variants (bytecode vs native)**." **Wrong at HEAD.** `Callable` has
no native variant — it is *purely* a bytecode recipe (`callable.rs:21`, one struct, no enum). The
native-vs-bytecode fork lives one layer up, in **`MethodKind`**:

```
pub enum MethodKind {           // method/object.rs:17
    Closure(ObjRef),            // bytecode — a handle to a ClosureObject
    Primitive(PrimitiveFn),     // native Rust fn pointer
}
```

So a `MethodObject` is *either* a closure handle *or* a raw fn — and the closure branch points at
the exact three-layer stack above. This is a **fourth actor** and it is where "native" actually
lives. The doc must place `MethodObject`/`MethodKind` at the top of the stack and correct the naive
"the Callable is native or bytecode" expectation. This correction is itself a good predict-then-
check candidate.

## 5. Brief-steering notes

**Agent A (theory, no source).** Emphasis:
- Go deep on **why split a compiled function into shared-recipe vs per-instantiation** — the
  general PL problem (one code body, N live closures over it; template vs instance). Name the
  vocabulary: *code object / function prototype / closure / activation*. CPython (`code object` vs
  `function object` vs frame), Lua (`Proto` vs `Closure`/`LClosure`), JVM (`Code` attribute vs no
  per-instance closure) all draw this line — make each tempting.
- Go deep on **the constant pool**: why a side array of literals indexed by opcode, not immediates
  baked into the instruction stream. Who does this (JVM constant pool, CPython `co_consts`, Lua `k`
  array), what it buys (dedup, GC roots in one place, compact code), what it costs.
- One paragraph, not more, on **identity/home-frame stamping** for non-local return — theory of
  "a closure that remembers where it was created" (Smalltalk `^`-in-block, the classic). It is a
  bridge to a later doc, not this doc's subject.
- **Do NOT** tell A that Phalcom shares by `Rc` or that native lives in a separate enum — that is
  the answer. Give the design space (share code or copy it? one object or three? native as a
  variant of the code object or as a sibling?) as a *space*.

**Agent B (source map).** Must confirm, with line + (where behavioural) live output:
- The four type definitions in §1, quoted in full. The `Rc<Callable>` at `closure.rs:28` and the
  `Chunk`-by-value at `callable.rs:23` are the load-bearing lines.
- `MethodKind` is the native/bytecode fork (`method/object.rs:17`), **not** `Callable`. Confirm
  `Callable` has no native variant at HEAD.
- Constant-pool **read sites**: which opcodes index `chunk.constants` (`Constant`, and the selector/
  name indices carried by `Invoke`/`GetGlobal`/`Class`/`Method`). Give the arms + lines.
- The `Bytecode::Closure` opcode: where a template `Callable` becomes a live `ClosureObject` with
  filled cells — the materialization site (dispatch.rs). Quote it.
- Where a `BlockObject` is minted and stamped with `home_frame_token` (the block-literal opcode).
- **Constant dedup at HEAD**: does `add_constant` dedup, or store duplicate literals? (Memory 5050
  says no-dedup; memory 5964 says a `ConstKey` dedup was *specced* — resolve which shipped.) One
  disasm of a program with a repeated literal settles it.
- Confirm the perf provenance of the `Rc<Callable>` share (perf-log / U-HOTPATH) so the honesty
  pass can cite it rather than assert it.

## 6. Predict-then-check candidates (§5.4)

1. **The loop-materialization question** (primary): a block literal sits inside a loop that runs
   1000×. Each iteration executes `Bytecode::Closure`. *What is allocated fresh per iteration, and
   what is shared across all 1000?* (Answer to earn: `Callable` shared via `Rc` refcount-bump —
   one recipe; fresh `ClosureObject` + fresh upvalue cells each iteration; fresh `BlockObject` with
   *that iteration's* frame token. The reader who predicts "the whole closure is one shared object"
   or "everything is copied" is corrected by the layer split.)
2. **The native question** (secondary, from §4): "`1 + 2` calls `Number>>+`, a native method. Where
   in the four-layer stack does 'native' live — is the `Callable` for `+` a native `Callable`?"
   Answer: there is no `Callable` for `+`; native lives in `MethodKind::Primitive`, a sibling of the
   whole bytecode stack.

## 7. Fiber / GC touch (one honest paragraph each, no more)

- **Fiber:** upvalue cells are `Upvalue::Open{fiber, slot}` while open — a captured local still on a
  fiber's stack names *(which fiber, which slot)*, not a raw pointer. So an instantiation's cells
  can point into a live fiber stack. Detail is the upvalue doc's; here it is one sentence tying
  `ClosureObject.upvalues` to fiber identity.
- **GC:** `ClosureObject` and `BlockObject` are heap objects (traced); `Callable`/`Chunk` are plain
  Rust structs reachable *through* the `Rc` from every closure instance and from the constant pool's
  `Value`s. Trace reaches constants → they are GC roots living in one place (a constant-pool payoff).
  One sentence; the collector is U-GC's / Doc 5-adjacent, not here.

## 8. Marked lies this doc will tell (spiral)

- **Lie A:** "caches/gcaches are just two more arrays on the Chunk" — true shape, but *why* they are
  `Cell<Option<..>>` parallel side tables and how inline/global caching works is **Doc 5**
  (caches-and-fusion). Point forward.
- **Lie B:** "the module handle on a ClosureObject is just where globals resolve" — the global-
  resolution + version-guard machinery is **Doc 5** (gcaches) and the metaclass/module doc. Point
  forward.
- **Lie C:** "the frame token is just a number stamped on the block" — what a `FrameToken` *is*, how
  it is generated, and how a stale one raises `DeadFrameError` is **Doc 6** (frame-identity). Point
  forward.
- **Lie D (destroy Doc 1's Lie #1):** Doc 1 said "the hoisted `Rc<Callable>` — see Doc 2." This doc
  *is* that payoff: name it, show why the recipe is the thing worth hoisting and guarding on
  `closure_id`.
