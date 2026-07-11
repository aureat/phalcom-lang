# Compiler Design

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** designing/critiquing the compiler pipeline — parsing/AST, name resolution, desugaring, IR choice, passes, codegen, incremental/error-recovery.

## Contents
- Pipeline shape
- Front-end representation
- Intermediate representation
- Name resolution & scoping
- Desugaring & lowering
- Pass ordering & phase coupling
- Error handling & recovery
- Compile-time evaluation & macros

## Pipeline shape
| Option | Langs | Consequence |
|---|---|---|
| Single-pass | Pascal/Wirth, early C | Fast, tiny memory; forces forward decls, no whole-program view |
| Multi-pass (batch) | most compilers, GCC, Clang | Clean phase separation; whole-file rescan per edit, no reuse |
| Query/demand-driven | rustc (red-green), Roslyn, Salsa | Incremental, memoized, IDE-grade; heavy infra, purity discipline |
| Pull-based sea-of-nodes | HotSpot C2, TurboFan | Passes are graph queries; hard to debug, order-insensitive |

**Impl.** query engine keys each derived fact on input fingerprints; a changed leaf invalidates only its transitive dependents, everything else replays from cache.

**Hazard — incremental/query compilation ⊗ global mutable state (CROWN JEWEL).** Memoized recompute is sound only if every query is a pure function of its inputs. Any hidden global — a mutable symbol counter, interner side-effect, ambient `TypeContext`, `thread_local` — makes a cache hit return a stale answer or a miss redo everything. All state must be an explicit tracked input. → overlay

## Front-end representation
| Option | Langs | Consequence |
|---|---|---|
| AST only | most batch compilers | Compact, easy passes; loses trivia, poor for refactoring tools |
| Lossless CST | rust-analyzer, Roslyn, Swift libsyntax | Round-trips source exactly; every node heavier, more memory |
| Red-green trees | Roslyn, rust-analyzer | Immutable shared "green" + positioned "red"; cheap edits, subtle API |
| Token stream + on-demand parse | some LSP servers | Lazy, low upfront cost; reparses regions repeatedly |

**Impl.** green node = kind + width + children, position-free and structurally shared; red node wraps it with absolute offset + parent, materialized lazily on traversal.

**Hazard — CST retention ⊗ memory.** Keeping every whitespace/comment token plus red-node wrappers for a large project multiplies front-end footprint several-fold; a compiler that never does refactoring or formatting pays that cost for nothing. Decide tooling ambitions before committing to lossless. → overlay

## Intermediate representation
| Option | Langs | Consequence |
|---|---|---|
| Tree-walk, no IR | early Ruby, bash, AST interpreters | Trivial to build; no optimization surface, slow |
| Stack bytecode | CPython, JVM, Smalltalk, Wren | Compact, easy codegen; implicit operands hide dataflow |
| Register bytecode | Lua, Dalvik, BEAM | Explicit temps, fewer dispatches; register alloc in compiler |
| SSA | LLVM, HotSpot, Cranelift | Each value defined once; φ-nodes; ideal for dataflow opts |
| CPS / ANF | SML/NJ, GHC (via STG), Scheme | Names every intermediate + control point; suits FP/tail calls |

**Impl.** SSA gives each assignment a fresh name and inserts φ at control-flow joins, so def-use is explicit and constant/copy propagation, DCE, GVN become local rewrites.

**Hazard — SSA φ-nodes ⊗ naive codegen.** φ-nodes are not machine instructions; lowering out of SSA must sequence parallel copies at edges, and doing it wrong (the "lost-copy" / "swap" problem) corrupts values when the φ operands interfere. Out-of-SSA needs real edge-splitting, not a per-φ mov.

## Name resolution & scoping
| Option | Langs | Consequence |
|---|---|---|
| Single resolve pass, lexical | C (decl-before-use), Pascal | Simple, one scan; no forward refs without prototypes |
| Two-pass (collect then bind) | Java, C#, most OO | Forward refs & mutual recursion free; needs full-scope pre-scan |
| Separate resolver → binding IDs | rustc, Roslyn | Uses point at binding IDs not strings; robust, extra pass |
| Late/dynamic binding | Smalltalk, Ruby, Python | Names resolve at send time; open classes, no static check |

**Impl.** resolver walks scopes building a symbol table, assigns each binding a unique id, and rewrites every use to reference that id — later passes never re-lex identifiers or re-walk scopes.

**Hazard — single-pass resolution ⊗ forward references.** In one lexical pass a use seen before its definition can't resolve — mutual recursion, later methods calling earlier state, hoisted types all fail unless you pre-declare or add a collection pass. Retrofitting forward refs onto a committed single-pass resolver is a rewrite. → overlay

**Hazard — macro expansion ⊗ hygiene.** A macro that introduces bindings must resolve its own identifiers in its definition scope, not the call site — otherwise expanded names capture or are captured by user names. See compile-time axis below.

## Desugaring & lowering
| Option | Langs | Consequence |
|---|---|---|
| Rich core, little desugar | C, Go | Fewer synthesized nodes; every surface form is its own pass |
| Small typed core (Core/GHC) | Haskell (Core), Idris (TT) | Whole language lowers to ~10 constructs; opts target core only |
| Surface → protocol calls | Smalltalk (`ifTrue:`→send), Python (`for`→iterator) | Control flow is ordinary dispatch; uniform, depends on dispatch speed |
| Elaboration to explicit | Rust (`?`→match), Scala | Sugar becomes explicit terms early; one code path downstream |

**Syntax.** `for x in it { … }` → `let mut i = it.iter(); while let Some(x)=i.next() {…}`; `a?` → `match a { Ok(v)=>v, Err(e)=>return Err(e.into()) }`; `x ifTrue: blk` → `x.ifTrue_(blk)` send.

**Hazard — desugar-before-typecheck ⊗ error messages (CROWN JEWEL).** If you lower `for`/`?`/operators to core before type checking, every diagnostic names the synthesized construct (`next`, `into`, a block send) the user never wrote. Source spans and a "desugared-from" origin must ride through lowering so errors point at surface syntax. → overlay

## Pass ordering & phase coupling
| Option | Langs | Consequence |
|---|---|---|
| Typecheck before desugar | Rust, Haskell (surface types) | Errors in user terms; typechecker handles all sugar |
| Desugar before typecheck | some ML front-ends | Simpler checker over small core; worse diagnostics |
| Monomorphize before opt | Rust, C++ templates | Opts see concrete types, inline well; code bloat, slow build |
| Fixed pipeline vs pass manager | GCC (fixed) vs LLVM (`PassManager`) | Manager reorders/repeats; ordering bugs become config bugs |
| Const-fold + inline interleaved | LLVM, V8 | Each exposes work for the other; fixpoint or miss opportunities |

**Impl.** an optimization declares the invariants it requires (e.g. "SSA form", "no critical edges") and preserves/invalidates others; the pass manager schedules to satisfy prerequisites and reruns analyses invalidated upstream.

**Hazard — pass ordering ⊗ optimization soundness (CROWN JEWEL).** An opt that assumes a prior pass's invariant silently miscompiles when reordered: GVN before edge-splitting, an alias analysis consumed after a pass that invalidated it, folding that assumes normalized constants. Each pass must state required + preserved invariants and the manager must enforce them, not the author's memory. → overlay

## Error handling & recovery
| Option | Langs | Consequence |
|---|---|---|
| Bail on first error | early Pascal | Trivial; one error per run, brutal edit loop |
| Panic-mode resync | yacc-style, many C compilers | Skip to sync token (`;`,`}`); cascade of bogus follow-on errors |
| Error productions | GCC, Clang | Grammar anticipates common mistakes; grammar bloat, targeted only |
| Error nodes in tree | rust-analyzer, Roslyn | Parse always yields a tree; IDE keeps working, downstream must tolerate |
| Diagnostics as values | rustc, Roslyn | Errors are data, batched + deduped + sorted; no exception control-flow |

**Impl.** recovering parser inserts an explicit error/missing node and continues, so a broken region is a subtree not a hard stop; every later pass treats error nodes as "already reported, don't cascade".

**Hazard — panic-mode resync ⊗ multi-error quality.** Skipping to the next sync token after an error routinely discards valid tokens and re-enters the grammar mid-construct, emitting a spray of phantom errors that bury the real one. Deduping and a "poisoned" marker on recovered nodes are what make multi-error output usable, not the resync itself.

## Compile-time evaluation & macros
| Option | Langs | Consequence |
|---|---|---|
| No metaprogramming | Go (pre-generics), C (bare) | Predictable, fast build; boilerplate, codegen lives outside |
| Textual/preprocessor | C `#define` | Cheap; unhygienic, no scope/type awareness, token soup |
| Hygienic macros | Scheme `syntax-rules`, Rust `macro_rules!` | Auto-renames introduced bindings; powerful, own sub-language |
| Procedural / AST macros | Rust proc-macro, Lisp `defmacro`, Template Haskell | Arbitrary compile-time code over AST; slow build, staging rules |
| Full comptime eval | Zig `comptime`, C++ `constexpr`/templates | Types/values computed at compile time; error-message + halting pitfalls |

**Syntax.** `@derive(Eq)` / `#[derive(Eq)]` desugars to a synthesized impl; Zig `fn List(comptime T: type)` runs at compile time to specialize a type; hygienic `let` inside a macro expands to a gensym'd binding.

**Hazard — macro hygiene ⊗ name capture (CROWN JEWEL).** An unhygienic macro that expands to `let tmp = …` captures (or is captured by) a user `tmp` in the call site, or its own free references resolve to whatever the caller happens to have in scope. Hygiene demands that introduced bindings get fresh names and free identifiers resolve in the definition environment — retrofitting it onto a textual macro system is intractable. → overlay

**Hazard — comptime eval ⊗ diagnostics + termination.** Turing-complete compile-time evaluation (templates, `comptime`, const-eval) can loop forever or fail deep inside a synthesized instantiation, producing errors at a location with no user source. Needs a recursion/step budget and an instantiation backtrace mapped to the originating call. → overlay
