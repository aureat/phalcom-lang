# Integrated runtime audit — retained checkpoint, 2026-09-07

## Status and assessment

**Provisional B: a sound foundation with important structural improvements needed.** Two source/compiler execution defects were reproduced: constant-index wraparound and super dispatch changing for an old instance after REPL class replacement. GC authority retention and native entry validation also have confirmed failures at their respective API boundaries. These warrant corrections, not replacement of the object model or interpreter.

This is an **investigation checkpoint**, consolidated at the user's request to conserve context. It is not certification that every area of the original prompt has been exhausted. No runtime implementation fixes, commits or broad release gates were performed. The original scope is retained in [the supplied prompt](evidence/original-audit-prompt.md).

Audited HEAD: `a2b86fb4ce35657780623ad7c532d8b1d1178839`, with pre-existing uncommitted work. In-scope bootstrap differences changed the native None binding to `Value::none()` and skipped `.class` names in one export path. Cargo, semantic, LSP and documentation changes also existed. They were preserved. Evidence applies to this working state, not a clean historical revision. Graphify was used only as an existing navigation index; findings were checked against live source.

## Major findings

| ID | Severity / confidence | Core problem and recommended direction |
| --- | --- | --- |
| [001](AUD-RUNTIME-001-unchecked-operand-narrowing.md) | High / confirmed constants | Unchecked `u16` narrowing changes source execution. Check representability at shared producers; audit locals, fields and jumps as one operand contract. |
| [002](AUD-RUNTIME-002-super-dispatch-loses-lexical-identity.md) | High / confirmed | Super lookup resolves a current name binding instead of retained lexical behavior. Bind a stable execution owner. |
| [003](AUD-RUNTIME-003-gc-misses-authority-edges.md) | High / confirmed missing edge | Rooted closures do not retain lexical-class authority through GC. Trace strong authority fields and make payload edge decisions mechanically visible. |
| [004](AUD-RUNTIME-004-native-entry-validation.md) | High at native boundary / confirmed | Host sends reach real primitives with wrong arity; native-only recursion bypasses the re-entry counter. Validate and account before activation. |
| [005](AUD-RUNTIME-005-grouped-contract-and-performance-findings.md) | Medium / differentiated per item | Trusted mutable chunks, repeated rest/native metadata work, large parallel cache tables, and raw execution-reset concerns. Contains lesser observations without overstating them. |

Each major document preserves invariant, producer/consumer evidence, failure scenario, recommended solution, constraints, required tests and unresolved questions.

## Verification completed

- Diagnostic probe built and executed with the pinned toolchain and `RUSTFLAGS='' RUSTC_WRAPPER=''`.
- Constant indices 0, 1, 65,534, 65,535 and 65,536 were exercised; the last wrapped to zero.
- A compiled source unit containing integers 0 through 65,536 returned `Ok(0)`.
- A rooted closure's otherwise-unretained lexical class was reclaimed by forced GC.
- Native-only recursion to depth 40 completed despite the configured 32 re-entry ceiling. No stack-overflow process crash was attempted.
- Wrong-arity host invocation of shipping `number_add` panicked at `args[0]`.
- An empty internal executable chunk panicked at instruction fetch.
- Full-Universe REPL reproduction: an old instance's super getter changed from 1 to 2 after class-name rebinding.
- `cargo test -p phalcom-core --lib chunk::tests`: **10 passed**, 0 failed, 93 filtered out. Includes basic fusion, branch-target protection, span access and two cache-invalidation tests.

Logs and exact diagnostic code are under [evidence](evidence/README.md). Probes intentionally record current failures; successful probe exit is not a claim that the bugs are fixed. Existing depth, arity, GC, Option and representation tests were inspected, but were not all run. No performance timings, all-opcode proof, complete boundary matrix, Miri run or workspace test result is claimed.

## Actual architecture

`Value` is two private `u64` words: payload plus tag/depth metadata. It is 16 bytes, alignment 8 on this host, `Copy`, and preserves full `i64`/`f64` bits. It is not NaN-boxed. Object payloads encode generational SlotMap keys, not pointers. `ClassId` aliases the object key type rather than enforcing class kind statically.

The VM owns a non-moving mark/sweep heap. Collection runs at interpreter safepoints; native code retaining fresh objects across re-entry uses temporary roots. Cycles need no reference-count ownership workaround. SlotMap generation checks prevent stale handles from silently becoming a new object in the same slot; `Heap::get` nevertheless panics when a stale reference is consumed.

An instance holds a class handle plus boxed fixed-size Value slots. A class contains its metaclass/superclass handles, Symbol-keyed method maps, field layout, static storage and reflection metadata. Ordinary fields use direct slot indices. Methods are heap objects containing signature, implementation and ownership metadata. Closures refer to immutable shared `Rc<Callable>` plus module, captured cells and authority. Blocks add a home-frame token wrapper. Frames are plain `Copy` records in a vector; the measured frame size is 120 bytes.

Chunks contain `Vec<Bytecode>`: 8-byte Rust enum instructions on this build, not a serialized variable-width byte stream. Constants, spans, per-IP inline/global caches and typed executable semantic pools are separate arrays. The interpreter increments IP before executing an instruction; branch offsets are relative to that advanced IP. It hoists the shared callable across instructions, guarded by closure identity, while rereading live frame IP/window on every iteration.

## Representative execution paths and stack contracts

Notation: `S` is the caller prefix; `r` receiver; `a` arguments. This records representative traced paths, not a formal proof for every opcode.

| Operation | Lowering and execution |
| --- | --- |
| `42` | `expr.rs` adds `Value::int(42)` → `Constant(u16)` → pool load → `S,42`. Literal strings allocate during compilation and are copied as handles during execution. |
| `object.foo` | Getter selector `foo`, distinct from `foo()` → `Invoke(0, selector)` → receiver class → cache/lookup → getter activation → `S,result`. A getter body accesses storage with `GetField`. |
| `object.foo()` / `foo(x)` / `foo(label:x)` | Static selector encodes kind/labels/arity, receiver then values are evaluated → `Invoke(n, selector)` → `S,r,a...` becomes `S,result`. Labels are part of selector identity; no label map is needed on ordinary exact hits. |
| Class-side call | Same send, but `Value::class` on a class yields its metaclass. |
| `Foo.new(...)` | `attributes.rs::lower_constructors` generates class factory → internal `_$new()` allocates fixed slots → hidden instance initializer → factory returns allocated instance. Native/abstract/enum-root generic allocation has explicit refusal paths. |
| Setter / index assignment | Selector send with RHS included; direct `SetField` consumes receiver/value and leaves assigned value. ADT payload writes are rejected. Subscript arity includes the put value. |
| `a + b` | Bilateral lowering in `expr.rs` evaluates operands into scratch slots → reflected-preference probe → `TryInvokeExact` plus Unsupported branches → numeric/native or language target. This is not the ordinary cached `Invoke` hot path. |
| Conditional / short circuit | Sacred-call recognition → guarded inline Bool branches or ordinary-send fallback → branch consumes Bool; no truthiness coercion. One-arm value use can wrap success in Some. |
| While / for | Guarded while skeleton or iterator cursor protocol → conditional exit, body result discarded, backward jump. Loop contexts patch break/continue and close captures on exits. |
| Closure/block call | `Closure` creates capture cells as needed, a fresh closure and block wrapper → Function gateway selects/binds shape → pushes frame through `EnteredFrame`. Synchronous combinators can still re-enter the interpreter. |
| Return | Pop/surface result → close open upvalues before truncating frame window → restore caller/result. Non-local block return validates home index+generation and unwinds through that activation. |
| Super | Original self remains receiver; lookup should start above defining behavior. Current name-based anchor is defective under replacement: issue 002. |
| Enum construction / match | Typed executable side tables connect semantic VariantId to VM RuntimeVariantId. General cases have variant ID plus immutable payload; singleton IDs can be immediate. Match bytecode and `value_is_variant` use runtime variant identity. |
| Error / catch | `run_until` preserves uncaught frames for diagnostics. Catch/ensure and fiber-floor paths use relative boundaries; `unwind_to` closes captures before truncation. Raw `run_in_module` reset is the outstanding API concern in 005. |

Trace anchors: `compiler/lib/{expr,scope,loops,jumps,class_decl,enum_decl,patterns,match_expr}.rs`, `compiler/{inliner,attributes}.rs`, `vm/{dispatch,send,adt}.rs`, `primitive/{block,class,map}.rs`, `heap/{class,instance,closure,upvalue,trace}.rs`.

## Invocation cost and actual hot maps

**Ordinary exact cached bytecode method:** read receiver from argument window; determine its class; load Symbol selector from constants; probe per-site class/world-version cache; check module-export special case; authorize selected method; obtain implementation/context; push one frame. On frame change the loop clones the callable Rc, not the bytecode. No selector-string construction, method-map hashing or required per-call heap allocation occurs on the ordinary warm exact path; vector capacity growth can allocate. Heap-backed receivers require an arena lookup to identify their category/class. Cold lookup recomputes receiver class in `Value::lookup_method` and probes one Symbol-keyed method map per hierarchy level until found.

**Getter:** the same dispatch with zero arguments. A storage getter then pops its receiver, resolves its arena object and indexes its fixed slot. `GetField` currently tries instance/class/ADT categories separately; there is no field-name hash on this path. It surfaces uninitialized storage as None, including an out-of-range read, whereas writes reject bad slot bounds.

**Native exact call:** shared dispatch/authorization followed by a Rust function pointer. Legacy ABI copies up to eight args into a stack array, more into a vector; clones/interns the diagnostic class name; tracks native authority; then replaces the window with the result. Shape-aware gateways can return `EnteredFrame` to avoid recursive interpreter driving.

**Other real hot lookups:** uncached hierarchy sends, rest-family maps, dynamic selector interning, super's class registry, cold global name resolution, and user Map/Set bucket lookup plus language hash/equality sends. Warm globals use cached module+slot. Bilateral `TryInvokeExact` currently performs hierarchy lookup without the ordinary call-site cache. Associated target caches are a distinct path and were not performance-measured.

## Performance direction

Highest expected payoff per complexity, to benchmark rather than assume:

1. Cache native diagnostic class Symbols instead of cloning/re-interning names per call.
2. Store immutable rest-call shape/label descriptors to avoid decode/intern on cache hits.
3. Reduce temporary argument copies in rest binding while preserving GC roots and order.
4. Measure an exact-resolution cache for bilateral operator attempts, preserving subtype/reflected/Unsupported rules.
5. Measure sparse per-site cache storage: both 24-byte cache arrays currently exist at every 8-byte instruction.

Allocations that remain semantically useful include object field storage, materialized closures/cells, exposed rest products, strings/results, and collection growth. Open captures use a BTreeMap keyed by stack position; close operations collect a temporary index vector. Inspect workload frequency before changing that structure.

Avoid premature NaN boxing, pointer tagging, JIT/threaded dispatch, broad instruction redesign or atomic/shared ownership changes. Existing guards, inline caching, callable hoisting and instruction fusion already address meaningful overhead. No wall-clock ranking has been established in this audit.

## Identity and future compatibility

Runtime class identifies behavior, not necessarily a complete applied type. Lists share one behavior class; Some depth stores nesting but not a generic argument. General ADTs distinguish semantic declaration/VariantId, VM enum/variant IDs and physical discriminants. `RuntimeTypingRegistry` separately links stable declarations and loaded metadata to behavior handles. This separation is a useful extension seam, not evidence that all runtime generics are reified today.

Preserve declaration identity independently from VM handles and runtime applied-type identity. If specialized behavior varies by an applied type, current class-only inline-cache keys must include an appropriate specialization discriminator or use distinct behavior identities. Do not make every generic application a distinct class by default. Correct lexical super anchoring and GC ownership before adding more retained identity metadata.

Duplicated metadata has purpose: signature/Callable arity serves different call paths, method holder/access owner/closure lexical owner serve different authority questions, and frame receiver context accompanies the stack receiver. These copies need explicit consistency contracts; they should not be collapsed mechanically. `Callable.num_upvalues` also duplicates descriptor count. No comprehensive divergence test was completed.

## Positive findings to preserve

- Private tagged Value words, full scalar payloads, checked Some-depth increment and `gc_obj_ref` that sees objects under Some wrappers.
- Generational handles and centralized heap mutation; no pointer-provenance trick in the reviewed representation.
- Direct fixed field slots and plain vector frames.
- Immutable shared callables; hoisting reloads live IP and stack window correctly across frame/fiber changes.
- Existing monomorphic/world-version cache and global-slot cache; the focused cache tests passed.
- Explicit method visibility checks, foreign layout guard and immutable ADT payload checks.
- Flat shape-aware closure gateways and home-frame generation tokens.
- Close-before-truncate unwind helper and iterative GC traversal.
- Checked arity/product counts and typed semantic side-table insertion: examples for fixing older unchecked producers.

## Next checkpoint

The highest-value next work is to turn issues 001–004 into fixes or focused plans, if authorized. If continuing the audit instead: reproduce GC expiry with ordinary REPL source; test locals/inherited-field limits; establish all retained side-table ownership; test raw-run reuse after failure; and complete the CFG/operand matrix. Do not repeat the already-preserved architecture reads and probes. Full performance benchmarks and exhaustive abnormal-control-flow testing remain open.

## Audit continuation — native lifecycle and parameter callers

At HEAD `94b9f14361333dfda06cd8abd792f672d12db06a`, a new bounded native lifecycle probe confirmed missing/misattributed native traceback frames after nested calls; see [005 §6](AUD-RUNTIME-005-grouped-contract-and-performance-findings.md#6-nested-native-calls-lose-or-misattribute-diagnostic-context). All four diagnostic cases completed. This is diagnostic evidence, not a passing regression suite. Authority push/pop was inspected, while shape-aware/fiber lifecycle and GC owner retention remain open.

L03's suspected labeled-rest misbinding was disproved for current compiler-produced paths: closures are positional, and method rest acceptance separately checks fixed-label prefixes. The general metadata helper remains weaker for manually constructed labeled shapes. Details and next steps are in [S2's continuation checkpoint](AUD-RUNTIME-S2-uninvestigated-leads-and-insights.md#continuation-checkpoint--2026-09-07).

Provisional B and incomplete coverage remain unchanged. No implementation fixes, commits, benchmarks or broad gates were performed. Unrelated concurrent working-tree changes were preserved.

## New slice — numeric and string contracts

[006](AUD-RUNTIME-006-numeric-domain-contracts.md) confirms incorrect float remainder signs and lost precision in mixed exact floor division through shipping native dispatch. Four valid arithmetic cases completed, including an integer-only control. [S3](AUD-RUNTIME-S3-new-numeric-and-string-leads.md) records new leads L19–L22 and the guarded UTF-8 slicing counterexample. Source compilation, boundary-panic execution and performance measurements were not performed. Provisional B and incomplete audit status remain unchanged.
