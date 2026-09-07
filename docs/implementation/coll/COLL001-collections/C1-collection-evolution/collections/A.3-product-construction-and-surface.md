# Spec A.3 — Product Construction, Structural Semantics, and Surface Integration

Status: implementation specification. Requires A.1 and A.2 landed. This phase removes the temporary Tuple compatibility architecture, makes Tuple/Record/Unit syntax fully executable, installs the minimal raw product surface needed to realize ratified behavior, and closes Spec A with structural equality/hash and lane/field access foundations.

## 1. Mission

Compile explicit product AST nodes directly into canonical runtime construction, validating static and dynamic Symbol labels and duplicate identities without routing through List. Make `()` and `#{}` evaluate to the same Unit value; make positive Tuple and Record literals allocate their A.2 native representations; expose the required Tuple lane projections and Record field substrate; implement Tuple order/lane-sensitive equality/hash and Record order-insensitive equality/hash; then retire the old `Tuple.fromList` construction dependency.

A.3 must stop at the product boundary. General negative indexing/slicing is Spec C. Collection expansion operators and variadic packs are Spec F. Map conversion is Spec B. Typing, reflection, destructuring, printing policy, Record copy/update APIs, and general product capability hierarchy remain out of scope.

## 2. Required semantic outcomes

At the end of A.3 these laws must be executable:

```text
() == #{} == canonical Unit

positive Tuple
    = immutable positional lane + immutable labeled lane
    total order = positionals then labeled values
    duplicate labels invalid
    equality/hash are lane-, label-, and order-sensitive

positive Record
    = immutable Symbol-field product
    encounter order is preserved
    duplicate fields invalid
    equality/hash ignore encounter order

computed product label
    = expression must evaluate to Symbol
    no String -> Symbol coercion
```

`(x)` remains grouping; `(x,)` is a Tuple. Positive Tuple and Record remain distinct families even if their components look similar. Only the closed zero-coordinate boundary collapses to Unit.

## 3. Compiler/runtime construction design

### 3.1 Add dedicated build bytecodes

Do not resurrect parser desugaring to ordinary sends and do not construct temporary Lists. Add these compact build instructions to `phalcom-core/src/bytecode.rs`:

```rust
BuildTuple {
    positional: u16,
    labeled: u16,
},
BuildRecord {
    fields: u16,
},
```

Use the repository's existing enum formatting conventions when spelling the fields; the operands and semantics above are fixed.

A Chunk is already a `Vec<Bytecode>`, so there is no byte-width pressure justifying an opaque encoding. Update every required bytecode registry/index/name/disassembly match. `BYTECODE_NAMES`, `Bytecode::VARIANTS`, `Bytecode::index`, CLI disassembly, and execution-loop exhaustive matches must remain synchronized.

Zero products do not need a build opcode. Compile empty Tuple/Record syntax directly as a `Value::Unit` constant. The A.2 runtime finalizers must nevertheless continue normalizing zero because future dynamic expansion/capture can reach the same boundary.

### 3.2 Stack layout

Tuple source grammar guarantees all positionals precede labeled entries. Compile entry expressions strictly in lexical evaluation order:

```text
for each positional:
    push value

for each labeled entry:
    evaluate/push label Symbol value
    evaluate/push field value

BuildTuple(P, L)
```

The VM can therefore pop `L` `(label,value)` pairs plus `P` positional values, restore source order, validate labels, and call `finish_tuple`.

For Record:

```text
for each field in encounter order:
    evaluate/push label Symbol value
    evaluate/push field value

BuildRecord(F)
```

The VM restores encounter order and calls `finish_record`.

A static label has no source-level side effect, so compiling it as a `Value::Symbol` constant before its value expression preserves the same observable order as conceptually evaluating the label then the value. A computed `[expr]: value` must evaluate `expr` before `value`.

### 3.3 Static Symbol canonicalization

Centralize Symbol lowering. Extend the existing `Expr::Symbol` compiler logic so both ordinary Symbol literals and static product labels call one helper that maps `SymbolLiteralKind` to the canonical interned `Symbol`:

- `Name(name)` interns the name;
- `Selector { name, labels }` goes through the same `encode_selector` routine method definitions already use;
- `Quoted(text)` interns the literal Symbol spelling directly.

Do not duplicate selector encoding in product code. The following labels must resolve to identical Symbol identity when their spellings are semantically equivalent:

```phalcom
name: 1
#name: 1
[ #name ]: 1
```

For selector-shaped forms, bare and explicit variants must similarly converge through the shared selector canonicalizer.

### 3.4 Computed-label validation

The VM build instruction must require each dynamic label stack value to be `Value::Symbol`. Any other runtime value is an error; there is no implicit conversion from String.

Add a named runtime error variant or existing structured error form suitable for “product label must be Symbol”. It must carry enough context for the diagnostic layer to identify Tuple versus Record if that distinction is useful. Do not rely on a panic from `Value::as_symbol`.

The compiler should reject syntactically obvious non-Symbol computed labels early where existing compiler diagnostics already classify literals, for example a literal Number/String/Bool used directly in `[expr]:`. Do not attempt flow-sensitive inference; variables and general calls defer to runtime.

For runtime-computed labels, preserve label-before-value failure timing. Prefer a small non-overridable `RequireSymbol`/`GuardSymbol` bytecode emitted immediately after compiling a computed label expression and before compiling that field's value. It validates the top stack value and leaves the Symbol in place. This ensures `[badLabelExpr]: sideEffectingValue()` fails on the label without first evaluating the value. If HEAD already has an equivalent guard facility, reuse it. Static labels need no guard.

## 4. Duplicate detection and failure timing

Static duplicate labels/fields should fail at compile time once their canonical Symbol identities are known. This includes equivalent source forms such as a bare and explicit Symbol spelling that canonicalize to the same Symbol.

Dynamic computed labels are validated by the build instruction/finalizer at runtime. A duplicate causes the entire product construction to fail. There is no overwrite behavior. The first implementation may detect dynamic duplicates at finalization after all literal entries have been evaluated; pin that behavior in tests so it is deliberate. The separate immediate Symbol guard above still ensures a non-Symbol computed label fails before evaluating its associated value.

Preserve evaluation semantics: entries preceding the duplicate are evaluated in source order; the implementation must not reorder expressions merely to preflight dynamic duplicates. For fully static labels, a compile-time duplicate error may prevent all execution because the program does not compile.

Use one duplicate-check implementation in the A.2 finalizers for runtime safety even if the compiler catches static cases. Future Spec F expansions will depend on that invariant.

## 5. Tuple raw substrate

The existing Tuple floor consists of `fromList`, `size_`, and `at_`. Migrate it to a lane-aware minimum.

Retain:

```text
size_    # total component count
at_(i)   # raw total-order value access used by core.ph/iterator substrate
```

Add the smallest internal observations needed for labeled semantics and lane projection:

```text
positionalSize_ # positional lane count
labelAt_(i)     # Symbol at labeled-lane index i, or None on raw miss
positionals_    # native projection through finish_tuple
labeled_        # native projection through finish_tuple
```

Do not use a raw `value-or-None` label lookup primitive: a Tuple is allowed to contain the surface `None` value, so such an API would conflate “missing label” with “present label whose value is None”. Core code can scan `labelAt_`, then translate the labeled-lane index to the total value index using `positionalSize_` and existing `at_`.

The exact spelling may follow existing underscore-floor naming conventions, but do not expose mutable access.

After all source construction has moved to `BuildTuple`, remove the `Tuple.class::fromList(_)` primitive registration and its implementation unless another non-product subsystem still legitimately depends on that public API. Search the entire repository before removal. Existing Bytes code that conceptually snapshots to Tuple must be migrated to a normal language-level conversion path or an internal helper rather than keeping `fromList` alive solely for historical convenience.

Because this changes the admitted primitive set, update floor governance in the same change. Do not silently edit only the census. Add/supersede the relevant ADR record according to the repository's current ADR process, explaining the retirement of the construction primitive and admission of the minimal lane observers. The expected Tuple net delta is small and should be stated explicitly after HEAD inspection.

## 6. Record raw substrate

Record needs a minimal native observation floor because `.ph` cannot see a native `RecordObject`'s fields. Add only operations that are not derivable from existing public state:

```text
size_       # field count
labelAt_(i) # encounter-order Symbol at raw nonnegative slot
valueAt_(i) # encounter-order value at raw nonnegative slot
```

A dedicated raw Symbol lookup is unnecessary in this phase: `.ph` can scan the small immutable label array and read the matching slot. This also avoids an ambiguous “value or None” primitive when `None` itself is a legitimate stored field value. If later profiling justifies a native Symbol-to-slot lookup, add it as an optimization without changing semantics.

No mutation primitive, generic constructor primitive, shape-reflection object, merge/update primitive, Map conversion primitive, or expansion primitive belongs here.

Register these on `Record` and add a scoped primitive-floor amendment together with the Tuple lane changes. Update `docs/spec/current/core/floor-census.md` and the invariant test that verifies installed bindings. The implementation report must give the exact before/after binding count.

`Record` remains under `Object` in this spec. Do not define generic `iterate(_)` until a later protocol decision says what Record iteration yields. Encounter order is nevertheless fully preserved by `labelAt_`/`valueAt_` and will feed `**Record` in Spec F.

## 7. Tuple public surface

Rewrite the current `class Tuple` implementation in `phalcom-core/core/core.ph` against the new raw substrate. Preserve existing working positional behavior where it remains compatible, but remove assumptions that every component is positional.

### 7.1 `size`

`size` is the total number of components and continues to delegate to `size_`.

### 7.2 Value iteration

Tuple's ordinary iteration yields all component values in total order: positional values first, then labeled values. If the current `Iterable` implementation already derives cursor iteration from `size` plus indexed raw access, adapt only what is necessary. Labels are not yielded by ordinary iteration.

Do not make ordinary iteration define `*Tuple`; expansion is a separate lane projection in Spec F.

### 7.3 Lane projections

Implement the ratified intended properties:

```phalcom
tuple.positionals
tuple.labeled
```

Each returns another Tuple semantic value and therefore must route through the same canonical finalizer. Consequences:

- projecting an empty lane returns Unit, not an empty Tuple heap object;
- `positionals` returns a pure positional Tuple when nonempty;
- `labeled` returns a pure labeled Tuple when nonempty;
- both results are immutable.

Implement the projections with the `positionals_` and `labeled_` native getters admitted in §5. Each primitive slices/copies the relevant immutable lane and calls the A.2 `finish_tuple` finalizer, so an empty projection returns Unit automatically. The public properties are thin `.ph` wrappers over those raw getters.

Do not rebuild projections through a mutable List plus the retired `Tuple.fromList`: that loses labeled identity, performs unnecessary staging allocations, and reintroduces obsolete construction architecture. Include both projection getters in the same scoped floor amendment and record that they are underivable because `.ph` has no general dynamic Tuple builder.

### 7.4 Label lookup foundation

Tuple semantically supports Symbol label lookup. A.3 must provide the lookup substrate by scanning `labelAt_` and then reading the corresponding total-order value slot. Do not add the public `get` or strict Symbol `[]` wrapper in Spec A; Spec C owns the unified safe/strict lookup convention, `KeyError`, and the requirement to distinguish a missing label from a present label whose value is `None`.

Do not invent a String-label convenience overload.

### 7.5 Integer indexing boundary

Leave the existing total-order raw integer access working, but do not solve negative normalization, strict `IndexError`, or slicing here. Those belong to Spec C. If current `Tuple#[]` is total/None-returning due legacy behavior, preserve it temporarily and document the mismatch for C rather than mixing indexing redesign into A.3.

## 8. Record public surface

A.3 should expose only the stable foundation needed by later specs:

- `size`;
- Symbol-identity lookup substrate through the ordered raw fields;
- structural `==` and `hash`;
- enough ordered raw observations for expansion/conversion later.

Do not choose dot-field syntax, destructuring, Record iteration yield type, update/merge API, Map conversion, or public safe/strict lookup spelling here. Spec C will expose lookup over this substrate and must preserve the distinction between a missing field and a present field whose value is `None`. Any future Symbol-lookup surface must reject non-Symbol keys rather than coercing them.

## 9. Structural equality

### 9.1 Tuple

Tuple equality is family-sensitive and exact:

```text
same total arity
same positional/labeled boundary
same labeled Symbol identities in the same order
corresponding values equal
```

A pure positional `(1, 2)` is not equal to a labeled `(x: 1, y: 2)`. Reordering labeled components changes equality. Positive Tuple is never equal to positive Record.

Implement Tuple equality in `core.ph` over the raw observations so nested/user-defined element equality remains ordinary Phalcom `==` dispatch. Do not add a native equality primitive in Spec A. A future optimized fast path may exist only if it preserves the same dynamic element equality semantics; Rust handle equality is not a substitute.

Unit equality is already handled at the `Value` layer.

### 9.2 Record

Record equality ignores encounter order but compares the exact field set and values:

```text
same field count
for every left field Symbol:
    right contains same Symbol
    left value == right value
```

Use Symbol lookup rather than sorting by textual spelling. Encounter order remains stored but is not an equality operand.

The first implementation may be O(n²) through linear raw lookup for small Records. Do not introduce shape interning/canonical unordered shape IDs merely for an unmeasured fast path. The authoritative spec explicitly permits those as future optimizations.

Positive Record is never equal to positive Tuple. `#{}` never reaches Record equality because it is Unit.

## 10. Structural hashing

### 10.1 General law

For both products, hashing is available iff all contained values are hashable under Phalcom's existing hash protocol. Propagate a contained-value hash failure; do not silently substitute identity hashes for structurally unhashable contents.

### 10.2 Tuple hash

Combine all of the following into an order-sensitive digest:

- family/domain tag distinguishing Tuple from other hashable structures;
- positional count or an equivalent lane-boundary marker;
- each positional value hash in order;
- for each labeled entry in order, its Symbol hash and value hash.

Two equal Tuples must hash equal; moving a component across the lane boundary or changing/reordering labels should normally change the digest.

### 10.3 Record hash

Record hashing must be encounter-order-insensitive. Compute an independent contribution for each `(Symbol, value)` pair and combine contributions with a commutative order-independent operation, while also including a Record domain tag and field count. Avoid a naive ordered fold because equal Records constructed in different orders must hash identically.

Reuse the repository's existing numeric hash-combination conventions rather than introducing a second hashing subsystem. For Record, combine each `(label.hash, value.hash)` pair with an order-independent accumulator; for Tuple, extend the existing ordered Tuple fold with lane and label markers. The exact arithmetic is not normative, but it must stay within the repository's Number/hash assumptions and add adversarial tests for different Record encounter orders, swapped labels, and same values under different field names.

Because Record/Tuple are immutable, cached hashes are permitted but not required. Do not add cache state in Spec A unless profiling justifies it.

## 11. Printing and debug rendering

Exact public printing rules are deferred by the authoritative product spec. Preserve existing positional Tuple rendering where practical, make Unit render as `()`, and ensure debug/native rendering does not panic on labeled Tuples or Records. Do not spend this phase defining a permanent Record serializer or recursive pretty-printer.

If current Tuple `toString` would silently discard labels, it is better to use a clearly structural temporary rendering that includes them than to misrepresent the value. Mark such rendering non-normative in tests; do not make exact labeled/Record text a language conformance requirement yet.

## 12. Legacy construction retirement

Once build bytecodes and product surfaces are green:

1. remove A.1's compiler compatibility branch;
2. remove parser/plan comments claiming Tuple literals lower to `Tuple.fromList`;
3. remove the public native `Tuple.fromList` binding if repository search finds no legitimate remaining dependency;
4. migrate any internal callers to the A.2 product finalizer or an appropriate normal surface conversion;
5. update stale comments in `heap/tuple.rs`, `primitive/tuple.rs`, `core.ph`, `U-COLL`-adjacent implementation notes where they would actively mislead future implementers. Historical accepted ADR text should not be rewritten; add superseding documentation according to repository policy.

The runtime must end A.3 with one construction truth: product literals/builders finalize through the canonical Unit/Tuple/Record boundary, not via mutable List staging.

## 13. Diagnostics

Required named diagnostics/failures:

- duplicate static Tuple label — compile time;
- duplicate static Record field — compile time;
- duplicate computed/runtime Tuple label — runtime construction failure;
- duplicate computed/runtime Record field — runtime construction failure;
- computed label not evaluating to Symbol — runtime type failure;
- syntactically obvious computed non-Symbol literal — compile-time error where straightforward;
- positional-after-labeled remains A.1 syntax error.

Diagnostics should identify the offending label spelling/range where available. A bare `name:` and explicit `#name:` that collide should report the canonical Symbol identity rather than pretending they are distinct keys.

Do not use old `collections/10-diagnostics.md` rules that forbid explicit labeled arguments after `**`, treat Record as a `***` source, or model selector-valued labels as a separate product-key type; those rules are obsolete.

## 14. Tests

### 14.1 Zero normalization

Language-level goldens must prove:

```phalcom
() == #{}
().class == Unit
#{}.class == Unit
```

Also test that repeated evaluations denote the same immediate value behavior and allocate no Tuple/Record object. At Rust level, assert the heap live count does not change when evaluating/finalizing zero products.

### 14.2 Tuple construction

Cover:

```phalcom
(1,)
(1, 2)
(x: 1,)
(1, x: 2, y: 3)
```

Verify total size/order, lane lengths, Symbol lookup, positionals/labeled projections, projection-to-Unit for an empty lane, duplicate diagnostics, and computed labels.

Pin evaluation order with side-effecting helper calls: all positional values execute first in lexical order; each computed label expression executes before its associated value; labeled entries remain in source order.

### 14.3 Record construction

Construct equivalent Records in different field encounter orders and assert:

- each preserves its own encounter order through raw ordered observations;
- they compare equal;
- they hash equal;
- a changed field Symbol or value breaks equality;
- duplicate fields fail rather than overwrite;
- computed Symbol fields work and computed String/Number fields fail.

### 14.4 Structural hash/equality nesting

Test nested Tuples/Records and user values with overridden `==`/`hash` where supported, proving product operations use Phalcom semantics rather than ObjRef identity. Include Tuple-as-Map-key regression cases already present in `phalcom-core/tests/lang/collections/` and add Record-as-key coverage when every contained value is hashable.

### 14.5 Legacy regression

Keep old positive positional Tuple behavior that remains semantically valid, but delete/update fixtures that assert an observable empty `Tuple` class or `Tuple.fromList` as the language construction model.

Run the full workspace after primitive-floor changes; floor census/invariant failures are part of the feature, not test noise.

## 15. Expected write set

Primary files:

```text
phalcom-core/src/bytecode.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/error.rs
phalcom-core/src/vm/**                         # build-op execution
phalcom-core/src/product.rs
phalcom-core/src/primitive/tuple.rs
phalcom-core/src/primitive/record.rs           # new
phalcom-core/src/primitive/mod.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/core/core.ph
phalcom-core/src/value/render.rs                # only safe/non-normative rendering fixes
phalcom-core/tests/**
docs/spec/current/core/floor-census.md
relevant ADR/PDR supersession or floor-amendment document
```

The bytecode additions may require mechanical updates in disassembler/debug/learn-source-map code. Fix exhaustive matches, but do not broaden the feature.

## 16. Completion gate

Spec A is complete only when all of these are true:

1. `()` and `#{}` both compile and evaluate directly to canonical Unit;
2. no empty Tuple or closed empty Record object can be observed or allocated through language-facing construction;
3. positive Tuple literals preserve two ordered lanes and Symbol label identity;
4. positive Record literals preserve immutable Symbol fields and encounter order;
5. static and dynamic duplicate identities fail with no overwrite semantics;
6. computed labels accept Symbol only and never coerce String;
7. Tuple equality/hash are order/lane/label sensitive;
8. Record equality/hash are encounter-order insensitive;
9. Tuple lane projections are executable and themselves obey zero normalization;
10. Record has a safe Symbol-lookup foundation without prematurely choosing field syntax/iteration/update APIs;
11. the old parser/compiler `Tuple.fromList` construction architecture is removed;
12. the exact primitive-floor change is documented and census tests agree;
13. typing, reflection, destructuring, expansion, slicing, Map conversion, and full printing remain outside this spec;
14. `./scripts/verify.sh --full` is green.

The implementation report must include commit SHA(s), build-bytecode stack layouts, exact Unit/Tuple/Record construction paths, primitive-floor before/after counts, duplicate/type-error behavior, representative golden output, removed legacy construction references, and the final verification tail.
