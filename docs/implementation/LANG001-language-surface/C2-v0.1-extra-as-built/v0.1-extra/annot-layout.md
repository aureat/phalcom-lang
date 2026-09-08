# U-ANNOT-LAYOUT — Work order: `FieldDef` + `@get`/`@set`/`@construct` + `@data`/`@sealed`/`@variant`

_Self-contained implementation plan for **one** implementer. Compiler/AST/layout unit — touches
`phalcom-ast/src/ast.rs`, `phalcom-ast/src/parser.rs`, `phalcom-core/src/compiler/` (incl. the
`attributes.rs` registry U-ANNOT-CONTRACTS creates), and `phalcom-core/src/vm.rs`
(`ClassLayout`/`field_layouts`). **Strict dependency on U-ANNOT-CONTRACTS** — do not start until
that unit's `attributes.rs` scaffolding (registry, `Target`, `ExpandCtx`, span-hygiene) is
landed; this unit adds rows, it does not re-shape the trait. **Reviewer ON** (spine files
`parser.rs`/`compiler/lib.rs`, plus `ClassLayout`) — hand the diff to `phalcom-reviewer`; do not
self-approve. Green gate: `./scripts/verify.sh` exits 0 + `cargo doc --workspace --no-deps`
clean. Grounded in **[ADR-0054](../../../adr/0054-two-speed-ratification-annotation-decorator-tiers.md)**
and normative **[annotations-construct.md](../../../design/experimental/v0.2/annotations-construct.md)**,
**[annotations-construct-inheritance.md](../../../design/experimental/v0.2/annotations-construct-inheritance.md)**,
**[annotations-data.md](../../../design/experimental/v0.2/annotations-data.md)**,
**[annotations-legality-grammar.md](../../../design/experimental/v0.2/annotations-legality-grammar.md)**,
**[annotations-test-strategy.md](../../../design/experimental/v0.2/annotations-test-strategy.md)**.

> **Grounding correction, the single biggest sizing surprise in this whole plan (read before
> estimating effort).** `annotations-construct.md` §"Context" states `@construct` "has two
> prerequisites absent from the current tree": field-declaration syntax **and** the `construct`
> keyword itself ("`parse_class_member` has **no `construct` handling** today"). Verified false
> on HEAD as of 2026-07-13: **`construct` is fully built.** `Token::Construct` lexes (`lexer.rs`
> L285), `ClassMember::Construct(ConstructDef{name,params,body,range})` exists (`ast.rs`
> L140–153), `parser.rs` L937 `if self.eat(&Token::Construct) { … }` parses it, and
> `compiler/lib.rs` L1170–1209 compiles it end-to-end — including registering the
> `Counter.new()`-style call-site alias (`vm.constructor_aliases`) and the bare-allocator guard
> flag (`vm.has_new_construct`, set when the constructor's own name is `"new"`). This landed in
> **U7** (`../U7/u7.md`, "ADR-0011 slot layout · `construct` · ADR-0017
> static fields"). **Only Prerequisite 1 (field-declaration syntax) is genuinely unbuilt** —
> this unit's real net-new grammar work is smaller than `annotations-construct.md` implies:
> `@construct`'s derive step is "read declared `FieldDef`s, emit a `ClassMember::Construct`" —
> emitting the *already-existing* AST node, not inventing constructor semantics. Flag
> `annotations-construct.md` for a doc-sync pass in the return contract (out of write-set — do
> not edit it yourself).
>
> **A second, related correction:** `annotations-construct.md`'s own derive pseudocode
> (`MethodDef { name: "new", …, is_constructor: true, … }`) is wrong on HEAD — `MethodDef` has
> no `is_constructor` field; that flag is an internal `Compiler::compile_block` parameter
> (`compiler/lib.rs` L564/L575), not an AST field. The derive must construct a real
> `ConstructDef` and wrap it `ClassMember::Construct(..)`, matching what a hand-written
> `@constructor
new(...) { ... }` parses to. Getting this wrong (emitting a `Method` instead) would
> silently produce a *plain method* named `new` rather than an actual constructor — no compile
> error, wrong runtime behavior (skips `SignatureKind::Initializer` encoding, the
> `constructor_aliases` call-site registration, and `has_new_construct`'s bare-allocator guard
> interaction) — this is exactly the "silently wrong, not a build failure" class of bug flagged
> as the worst outcome in U-ITERABLE's own build-order note. Treat §5 step 3 below as atomic for
> this reason.

## 1. Mission (one sentence)
Add class-body **field-declaration syntax** (`FieldDef`, the one genuinely-missing prerequisite)
and, over it, land the layout-derive tier: `@get`/`@set` (accessor generation from a declared
field), `@construct` (constructor generation from declared fields, including the
inheritance-aware super-signature F-fix), and `@data`/`@sealed`/`@variant` (structural
`==`/`hash`/`toString`/`with(...)`, closed-hierarchy exhaustiveness, and the generated
keyword-argument visitor) — all **generate**-phase attributes that grow the instance-layout slot
vector (ADR-0011), the reason this tier is sequenced after, and depends on,
U-ANNOT-CONTRACTS's registry/pipeline scaffolding.

## 2. Preconditions (verify on actual HEAD — do not assume)
- **U-ANNOT-CONTRACTS landed**: `phalcom-core/src/compiler/attributes.rs` exists with
  `AttributeExpander`, `AttributeRegistry`, `Target`, `ExpandCtx`, and `expand_class_attributes`
  wired at the top of `Statement::Class` (`compiler/lib.rs` L763, precise line may have moved —
  re-verify). This unit adds registry rows (`"get"`, `"set"`, `"construct"`, `"data"`,
  `"sealed"`, `"variant"`); it does not touch the `weave`-phase expanders from that unit.
- **`construct` is fully built** (see banner above) — `ConstructDef`, `ClassMember::Construct`,
  `compiler/lib.rs` L1170–1209 (selector encoding via `encode_selector(&name, &labels,
  SignatureKind::Initializer(arity))`, the `constructor_aliases`/`has_new_construct` side
  tables). This unit's `@construct` expander target-emits this node; it does not add new
  constructor *runtime* semantics.
- **No `FieldDef`/field-declaration grammar exists.** `parse_class_member` has no `Token::Let`/
  `Token::Var` branch (those tokens dispatch only at statement position, `parser.rs` L409–410).
  `ClassMember` has 4 variants (`Method`/`Getter`/`Setter`/`Construct`) — this unit adds a 5th,
  `Field`.
- **Fields are today wholly inferred, not declared.** `compiler/lib.rs` L820–872 scans every
  non-static `Method`/`Getter`/`Setter`/`Construct` body for `_x = …` assignments
  (`collect_assigned_fields_stmt`) to build `own_instance_fields`, in **first-assignment
  source order** — this is the *current* field-order-is-API mechanism (no explicit R3 field list
  exists yet; order is an emergent property of scan order). `FieldDef` **replaces** this
  inference for any class that declares at least one field explicitly — mixing declared and
  inferred fields in one class is out of scope (§ hazard below); a class with any `FieldDef`
  member must declare *all* its fields that way, or this unit's derive attributes cannot see a
  complete field set.
- **`ClassLayout`** (`vm.rs` L34–40): `{ name, field_slots: IndexMap<Symbol,u16>, field_count,
  static_field_slots, static_field_count }`, keyed by class-name `Symbol` in
  `VM::field_layouts: HashMap<Symbol, ClassLayout>` (`vm.rs` L125), populated once per class at
  `compiler/lib.rs` L1024 (`self.vm.field_layouts.insert(name_sym, layout)`). A subclass already
  reads its superclass's layout for field **counts** at L973 (`self.vm.field_layouts.get(&sc_sym)`)
  — this is the **exact established pattern** the F-fix's super-constructor-signature lookup
  (§3.3) should follow, applied to methods instead of layouts.
- **Superclass constructor signature is readable without new side-table machinery.**
  `ClassObject.methods: IndexMap<Symbol, ObjRef>` (`class.rs` L17, `MethodsMap`) holds every
  compiled method including constructors, keyed by encoded selector `Symbol`. A `MethodObject`
  fetched via `self.vm.heap.get(obj_ref)` (already the pattern used elsewhere in the compiler
  for constant-pool access) carries `.signature.kind: SignatureKind` — filtering a superclass's
  `methods` for `SignatureKind::Initializer(_)` entries, then `decode_selector` (`method.rs`
  L181, already round-trips selector string → `(name, labels, SignatureKind)`) on each hit's
  selector, yields the label list without inventing a new reflection surface. **This is the
  concrete mechanism for "reads the parent's already-compiled constructor signature directly,
  not runtime reflection"** — confirm at implementation time that the superclass's `ClassObject`
  (and its `methods` map) is fully populated by the time the subclass's `Statement::Class` arm
  runs; single-pass top-to-bottom compilation plus U-INH's "superclass must already be defined"
  discipline (`compiler/lib.rs`'s reopen/superclass-resolution code, L875–909 region) makes this
  true for `extends`-declared superclasses the same way it's already true for `field_layouts`
  lookups at L973.
- **`@sealed`'s per-compilation-unit enforcement** needs a place to record "classes declared in
  this compile" and their declared-`extends` targets, checked at the **end** of compiling the
  unit (file/module) — no such end-of-unit hook currently exists in the compiler; confirm where
  a single `.ph` file's compilation is bounded (likely `Compiler::compile`'s top-level entry) and
  add a post-pass there, not inside the per-class `Statement::Class` arm (a `@sealed` class's
  full subclass set isn't known until every class in the unit has been seen).
- Baseline `./scripts/verify.sh` green before the first edit. Re-run `graphify affected
  "parser.rs"` / `"compiler/lib.rs"` / `"vm.rs"` / `"attributes.rs"` and check for concurrent
  editors, **especially U-ANNOT-CONTRACTS** — confirm it has actually landed on the branch this
  unit starts from, not merely planned.

## 3. Design (realize the ratified docs — do not re-litigate the model)

### 3.1 `FieldDef` — the one real prerequisite (annotations-construct.md §"Prerequisite 1")
```rust
// ast.rs
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,               // e.g. "_x"
    pub mutable: bool,               // var => true, let => false
    pub default: Option<Expr>,
    pub attributes: Vec<Attribute>,  // @get/@set attach here
    pub range: SourceRange,
}
```
Add `Field(FieldDef)` to `ClassMember`. Parser: `parse_class_member` gains a branch **before**
the existing `Construct`/`Static`/name dispatch (`parser.rs` L935+): if `peek()` is `Token::Let`
or `Token::Var`, parse a field declaration (`(let|var) _name ['=' expr] NEWLINE`) instead of
falling into the method/getter/setter path. This is the disambiguation
annotations-construct.md flags: same keywords as statement-position `let`/`var` (ADR-0014), two
roles, distinguished purely by position (class-body vs. statement) — no lookahead ambiguity
since `parse_class_member` is only ever called from `parse_class_body`.

**Field order is API** (R3, selectors.md §1, carried over unchanged) — `FieldDef`s appear in
`ClassMember` order exactly as written; this order is what `@construct`'s/`@data`'s generated
parameter lists key off. Document on `FieldDef` itself, not just in prose.

### 3.2 `@get`/`@set` — accessor derive (annotations-construct.md §"@get/@set")
Registry rows `"get"`/`"set"`, `Target::Field` only. `expand` reads the owning `FieldDef.name`
(`"_label"`) and emits:
- `@get` → `ClassMember::Getter(GetterDef{ name: "label", body: [Var("_label")], .. })`
- `@set` → `ClassMember::Setter(SetterDef{ name: "label", param: "value", body: [Assign("_label",
  Var("value"))], .. })`

**Collision with a hand-written accessor of the same selector is a compile error**
(`attr.accessor_collision`) — the expander's `generate` step must check the *rest* of the
class's member list (post-field-parse, pre-this-attribute's-own-emission) for an existing
`Getter`/`Setter` of the same derived selector before emitting; ADR-0012's "selector is sole
dispatch key, no last-wins" rule, same diagnostic class `@construct` and `@data` also use (keep
the check logic in one shared helper in `attributes.rs`, not three copies).
`@get(priv)` is **advisory naming only** — parse the bare `priv` arg (an `Expr::Var`, per
annotations-legality-grammar.md's own note) but do not gate anything on it; selectors.md §5's
no-visibility-syntax commitment stays untouched.

### 3.3 `@construct` — constructor derive + the inheritance F-fix
**Own-fields case** (annotations-construct.md): collect the class's own `FieldDef`s in
declaration order; emit
```rust
ClassMember::Construct(ConstructDef {
    name: "new".into(),
    params: fields.iter().map(|f| ParameterDef { name: strip_leading_underscore(&f.name), label: Some(strip_leading_underscore(&f.name)), is_rest: false, range: f.range }).collect(),
    body: fields.iter().map(|f| assign_stmt(&f.name, var_expr(&param_name(f)))).collect(),
    range: attr.range,   // D3 span hygiene
})
```
— a real `ConstructDef`, not a `MethodDef` (banner correction). Fields carrying a `default: Some(expr)`
are **omitted** from `params` (annotations-construct-inheritance.md's "supply-and-default is
mutually exclusive per field"); their `expr` is evaluated **per instance, at construct time,
before the derived body's own assignments**, in field declaration order — prepend
`assign_stmt(f.name, f.default.clone())` for each defaulted field, *before* the labeled-param
assignments, so a later non-default field's default expression (if any future field ordering
puts one after) still sees prior defaults already applied. `@get(priv)`... n/a here.

**Collision**: a class carrying both `@construct` and a hand-written `@constructor
new(...)` of the
same selector is `attr.accessor_collision` (annotations-construct-inheritance.md
§"Collision"); a differently-selectored hand-written `construct` (e.g. `@constructor
anonymous()`)
coexists — the collision check is selector-keyed, not "any hand-written construct present."

**Super-@constructor
chaining (the F-fix, annotations-construct-inheritance.md)**: if
`class_def.superclass.is_some()`, look up the superclass's compiled `ClassObject.methods` (§2
precondition) for `SignatureKind::Initializer(_)` entries:
- **Exactly one** → `decode_selector` its selector to get the label list; synthesized params are
  `super_params ++ own_params` (own = this class's own non-defaulted `FieldDef`s per the rule
  above); synthesized body is `super.new(<super_params by label>)` (an `Expr::MethodCall` with
  `SuperVar` receiver — confirm `Expr::SuperVar{range}` (`ast.rs` L341) is the right receiver
  node for a `super.new(...)` send; it should be, given `super.foo()` elsewhere in the language
  already resolves through it) **followed by** this class's own field assignments.
- **Zero** (no superclass constructor at all — e.g. superclass never defines one, relying purely
  on `Object`'s bare allocator) → own-fields-only case, no super-chain prepended.
- **More than one** (superclass has ≥2 differently-labeled `Initializer` selectors — an
  overloaded hand-written `new`) → **compile error**, `construct.super_ambiguous`, span on the
  subclass's class name (annotations-construct-inheritance.md: "Ambiguity, not ancestry, is the
  failure case").

This works identically whether the superclass's single constructor was hand-written or itself
`@construct`-derived, **because the lookup reads the compiled selector, not the source
attribute** — this is the entire point of the F-fix and the reason it's buildable without
runtime reflection: by the time the subclass's `Statement::Class` arm runs, the superclass's
`ConstructDef` (however it was produced) has already been compiled into a real `MethodObject`
with a real `Signature`.

### 3.4 `@data`/`@sealed`/`@variant` (annotations-data.md, verbatim expansion shapes)
- **`@data`**: `generate`-phase, reuses **exactly** §3.3's field-to-constructor derivation
  (own-fields-only shape — `@data` classes are not expected to also chain a superclass
  constructor in Draft 0.1's worked examples, but the F-fix logic applies unchanged if a `@data`
  class does `extends` something with its own constructor; don't special-case it away). If a
  class carries **both** `@construct` and `@data`, `@data`'s own constructor-generation step is
  a no-op (both target the same `new` selector via the same field list — detect via "a
  `ConstructDef` named `new` was already emitted earlier in this same expansion pass" rather
  than re-deriving and hitting a spurious self-collision). Additionally emits, in one pass over
  the same field list:
  - `==(other)` — conjoined field-by-field `==` (an `Expr::Binary` chain, `and`-folded — ground
    against whatever the current `and`/`&&`-equivalent AST shape is, likely
    `Expr::MethodCall` sending `and(_)`, matching the language's own short-circuit convention,
    not a native `&&` operator).
  - `hash` — a getter chaining `.hash.combine(...)` per field, per annotations-data.md's exact
    shape.
  - `toString` — a getter building an interpolated string; reuse whatever AST shape
    `\(expr)`-interpolation currently desugars to (ADR-0022) — do not hand-roll string
    concatenation if the interpolation desugar is available to the expander at this phase (it
    should be, since interpolation desugars at *parse* time per the lexer's `StringInterp` note,
    ast.rs L102–109 — confirm the expander can construct a `StringInterp`-shaped literal directly
    or must build the equivalent `+`-chain manually; the latter is safer/less coupled).
  - `with(...)` — a labeled-optional-arg method (`field.orElse { self.field }` per field),
    allocating one new instance via the same derived `new` selector.
  `==`/`hash` are derived **together or not at all** — if a class hand-writes one and derives
  the other via `@data`, that's `attr.accessor_collision` (same diagnostic, reused).
- **`@sealed`**: `finalize`-phase (not generate/weave) — record the class name + its
  `Target::Class` marker in a compile-unit-scoped table (§2 precondition: needs an end-of-unit
  hook, not per-class). At end-of-unit, verify every recorded subclass-of-a-sealed-class was
  declared **within the same compilation unit** — a subclass declared elsewhere is
  `attr.sealed_violation` at the *subclass's* definition site (per annotations-data.md, not at
  the sealed class's site). This needs the compiler to know, for every compiled class, both its
  `superclass` name and which compilation unit (file) it came from — confirm this metadata
  exists or must be added (likely a light addition: tag each `ClassDef` compile with its source
  file, already implied by `SourceRange`'s file association if `phalcom-common::range` carries
  one — verify before assuming).
- **`@variant`**: `generate`-phase, expands each `@variant Name(labels...)` found inside a
  `@sealed @data`-carrying class body **before** the enclosing class finalizes, into a
  standalone top-level `Statement::Class` for `Name`, `extends <enclosing>`, itself carrying
  `@data` and one `FieldDef` per label. This is generation-of-a-sibling-statement, not
  generation-of-a-member — the expander's return type (`Result<Vec<ClassMember>, ...>` per
  U-ANNOT-CONTRACTS's trait) is the wrong shape for "emit a new top-level class"; **this is a
  real signature gap this unit must close**: either (a) widen `expand_class_attributes`'s return
  to `(ClassDef, Vec<Statement>)` so `@variant`'s expansion can hand back sibling top-level
  statements for the caller (`compiler/lib.rs`'s statement-list driver) to splice in immediately
  after the enclosing class, or (b) have the `@sealed`/`@variant` combination special-cased at
  the `Statement::Class` arm itself (read nested `@variant` markers before calling the generic
  expander, synthesize the sibling `ClassDef`s there, then hand the now-`@variant`-stripped
  class body to the ordinary registry). **Recommend (a)** — keeps `attributes.rs` the single
  owner of all generate-phase logic; (b) leaks variant-specific knowledge into `compiler/lib.rs`
  itself, which every other attribute in this whole design avoids. Flag this signature widening
  explicitly in the return contract; it changes U-ANNOT-CONTRACTS's own trait shape retroactively
  (an actual coupling between the two units beyond "shared file, additive rows" — the one place
  this plan's stated "strict dependency, not reshape" assumption needs a caveat).
  The generated visitor (`match(circle:, rect:) { … }` + per-variant `__matchArm`) is a further
  `generate`-phase step on the **enclosing sealed class**, run once all its `@variant` children
  are known (i.e., after the sibling-statement splice, same end-of-class-body point) —
  keyword-argument list is the declared `@variant` names in declaration order, per
  annotations-data.md verbatim; zero new grammar (an ordinary keyword-message selector).

### Rubric — hazards & preclusion (mandatory)
- **Mixed declared/inferred fields (new hazard, not in any source doc).** A class with some
  fields via `FieldDef` and others still implicit-by-assignment (the old inference path) has an
  ambiguous field *order* — `FieldDef`s are position-known at parse time, inferred fields are
  only known after a full body scan. **Resolution**: a class using **any** `FieldDef` uses
  `FieldDef`s as the *complete* field list for that class (inference is skipped entirely for
  that class); a class using zero `FieldDef`s keeps the current pure-inference path unchanged.
  Do not attempt to merge/interleave the two orderings. Pin a golden proving a non-`FieldDef`
  class's behavior is byte-for-byte unaffected by this unit (regression, not just new-capability
  coverage).
- **`construct.super_ambiguous` needs a *real* ambiguous case to test against**, which requires
  the language to already support two differently-labeled `construct` selectors on one class —
  confirmed possible (`ConstructDef.name` is parser-free-form via `parse_method_name`, so
  `@constructor
new(x:)` and `construct new(x:,y:)` — wait, same base name `new` with different
  labels *does* produce different encoded selectors already (`SignatureKind::Initializer(arity)`
  + labels are part of `encode_selector`'s input) — confirm two same-named-different-arity/label
  constructors don't collide with the **bare-allocator alias** (`has_new_construct`, keyed by
  class only, not selector) before relying on this as the ambiguous-superclass test fixture; if
  `has_new_construct`'s guard logic assumes at most one `"new"`-named constructor per class in
  some other code path, the test fixture needs a *second, differently-named* `construct` (e.g.
  `@constructor
new(x:)` + `construct fromPair(a:,b:)`) instead — **verify against HEAD's actual
  `has_new_construct` consumer (the ctor-inherit guard, per the `ctor-inherit-guard-fix` prior
  session) before writing this golden**, do not assume.
- **`@variant`'s sibling-statement generation is the one place this tier's `generate` phase
  produces something other than members** — the signature-widening call above (§3.4) is
  load-bearing; get it wrong and `@sealed @data class Shape { @variant Circle(radius:) }`
  either fails to register `Circle` as a real global class or double-compiles it.
  **Representation/dispatch impact:** `@data`'s `==`/`hash` derivation interacts with
  the existing equality/hash machinery (`equality-and-hash.md`, not read in full for this plan —
  **flag for the implementer to read before coding `@data`'s `==`/`hash` shape**, since a
  mismatch with the kernel's existing hash-consistency invariant (`verify_invariants()`-checked
  elsewhere) would be a silent-wrong bug, not a build failure.
- **`@sealed` cross-module gap is documented, not solved** (annotations-data.md's own hazard) —
  do not attempt whole-program closed-world checking; per-compilation-unit only.
- **`with(...)` is shallow** — a `with(...)`-produced instance shares heap-object field values
  with its source (standard functional-update semantics, not a deep clone) — pin a golden that
  specifically distinguishes shallow from deep (a `@data` class with a `List`-valued field;
  `with(...)` copies the reference, mutating the copy's list is visible in the source's list
  too) so a future "helpful" deep-clone regression is caught.
- **Precedent:** Swift's memberwise-init cost for field-order-as-API (annotations-construct.md's
  own citation) — do not route around it with keyword reordering; C#'s `record` for `@data`'s
  shape (`with` expressions, structural equality); Rust's `#[derive(Clone)]`-style "opt-in
  derive, hand-written wins on collision" for the `attr.accessor_collision` policy.

## 4. Confirmed write-set (tight & disjoint; re-validate with `graphify affected` on HEAD)
| File | Why | Slice |
|---|---|---|
| `phalcom-ast/src/ast.rs` | `FieldDef` struct; `ClassMember::Field` variant | AST |
| `phalcom-ast/src/parser.rs` **(SPINE — reviewer ON)** | `let`/`var` field-decl branch in `parse_class_member`, disambiguated from statement position | parser |
| `phalcom-core/src/compiler/attributes.rs` **(shared with U-ANNOT-CONTRACTS, additive rows only)** | `"get"`/`"set"`/`"construct"`/`"data"`/`"sealed"`/`"variant"` expanders; the `expand_class_attributes` return-type widening for `@variant`'s sibling statements (§3.4 — the one non-additive change, coordinate with U-ANNOT-CONTRACTS's landed shape, do not diverge silently) | expander |
| `phalcom-core/src/compiler/lib.rs` **(SPINE — reviewer ON)** | splice sibling `Statement::Class` nodes from `@variant` expansion into the enclosing statement list; end-of-compilation-unit `@sealed` closed-world check | compiler wiring |
| `phalcom-core/src/vm.rs` | `field_layouts`/`ClassLayout` consumption for `FieldDef`-declared classes (no shape change to `ClassLayout` itself — same struct, populated from a different source); a compile-unit-scoped `sealed_classes`/`class_source_unit` tracking table if not already derivable from `SourceRange` | layout + sealed tracking |
| `phalcom-core/src/method.rs` / `phalcom-core/src/class.rs` | read-only use of `MethodsMap`/`decode_selector` for the F-fix lookup — no struct change expected, confirm before assuming | F-fix lookup |
| `phalcom-core/tests/lang/annotations/` (same label as U-ANNOT-CONTRACTS, additive files) | goldens per §7 | goldens |
| `phalcom-core/tests/lang.rs` | wire new test fns | test harness |

**Deliberately NOT in scope:** `@requires`/`@ensures`/`@invariant`, the `CompileMode` axis,
`MethodObject::contracts`, `FiberObject::checking` (all U-ANNOT-CONTRACTS); the `Attribute`/
`AttributeUsage` core class, `Behavior.defineMethod`, `Method.invokeOn`/`.replaceWith`, any
Install/Dispatch/Runtime-tier surface (`attribute-classes.md`, gated); real `match`-with-patterns
grammar (open-Q7, explicitly untouched per annotations-data.md); nested/namespaced `@variant`
naming (explicit Draft 0.1 simplification, not this unit's job to fix); `bytecode.rs` (no new
`Value`/opcode — `@data`'s derived methods are ordinary `.ph`-shaped bytecode, same as any
hand-written method).

### 4.1 Write-set collision risk (flag, don't resolve)
- **`attributes.rs` is shared with U-ANNOT-CONTRACTS** — start only after that unit lands;
  confirm the registry/`Target`/`ExpandCtx` shape on the actual landed diff, not this plan's
  description of it (which was written against a not-yet-built file). The `@variant`
  signature-widening (§3.4) is the one place this unit may need to touch code
  U-ANNOT-CONTRACTS wrote — coordinate explicitly, do not silently change the trait.
- **`parser.rs`/`compiler/lib.rs`/`vm.rs`** — spine files, confirm no concurrent unit holds them.
- **`core.ph`** — this unit likely needs zero `core.ph` edits (all derivation is Rust-side AST
  synthesis, not new native primitives) — confirm this stays true; if a kernel class needs
  `@data`/`@construct` applied to itself as a self-hosting exercise, that's explicitly **out of
  scope** here (a follow-on, not this unit's mission).

## 5. Build order (small, independently-green diffs)
1. **`FieldDef` grammar only**, no attributes yet. Parse `let`/`var _x [= expr]` at class-body
   position into `ClassMember::Field`; a class using it compiles today only if some other
   mechanism still produces its slot layout — since none does yet, this step's own goldens are
   AST-snapshot-only (parse-then-snapshot), not yet runnable `.ph` programs. Green (parser-level).
2. **`FieldDef` feeds layout** (no derive attributes yet, still needs at least one accessor/
   constructor written by hand to be a runnable class). Wire declared `FieldDef`s into
   `own_instance_fields`/`ClassLayout` construction, replacing inference **only** for classes
   using `FieldDef` (§ hazard). Green — a hand-written `construct` + `FieldDef`-only class
   compiles and runs identically to today's fully-inferred equivalent (regression golden).
3. **`@construct` own-fields-only.** No super-chaining yet. Emits a real `ConstructDef` (banner
   correction — atomic, verify the emitted node round-trips through the *exact* same
   `compiler/lib.rs` L1170–1209 path a hand-written `construct` does, not a parallel path).
   Green — `@construct class Point { var _x; var _y }` / `Point.new(x:,y:)` golden.
4. **`@construct` inheritance F-fix.** The `MethodsMap`/`decode_selector` lookup,
   zero/one/many-constructor cases, `construct.super_ambiguous`. Green —
   `construct_subclass_super.ph`, `construct_subclass_hand_written_parent.ph`,
   `construct_subclass_ambiguous_super.ph` (§7).
5. **`@get`/`@set`.** Accessor derive + collision check. Green.
6. **`@data`.** Constructor reuse/no-op-if-`@construct`-present, `==`/`hash`/`toString`/
   `with(...)`. Green — read `equality-and-hash.md` first (Rubric flag) before this step.
7. **`@sealed`/`@variant`.** The sibling-statement signature widening, end-of-unit closed-world
   check, generated visitor. Green — this is the largest remaining step; if the write-set gets
   hot, split `@variant`'s visitor generation to its own commit within this step (not a separate
   unit — the exhaustiveness/visitor pairing is one coherent piece of work).

Each step is a self-verifiable commit; never commit a non-compiling tree. Step 3 in particular
must not be sub-split further (per the banner's "silently wrong, not a build failure" warning).

## 6. Mandatory rules
- **Docs:** `///` on every new type/fn/field (`FieldDef`, `ClassMember::Field`, every new
  registry row's expander, the `@sealed` unit-tracking table, the widened
  `expand_class_attributes` return type) citing the spec § it realizes.
  `cargo doc --workspace --no-deps` clean.
- **Green gate:** `./scripts/verify.sh` exits 0; no new clippy; no `unsafe`.
- **Reviewer ON** (spine files) — `phalcom-reviewer` gates the diff; writer never self-approves.
  Every diagnostic (`attr.accessor_collision`, `construct.super_ambiguous`,
  `attr.sealed_violation`) recovers, never panics.

## 7. Test strategy (extends annotations-test-strategy.md's existing catalog — do not invent a parallel plan)
**AST snapshots (insta):**
- `expand__construct_params` — fields → labeled params → assignments; super-chaining. Add
  variants for: zero-superclass-constructor, one, and the ambiguous-error case (snapshot the
  diagnostic, not just success shapes).
- `expand__get_set_pair` — as named in the doc's table.
- New (not in the doc's table, required by this unit's own findings): `expand__data_derive`
  (all four/five generated members from one `@data` class), `expand__variant_sibling_split`
  (the sibling-statement splice — snapshot both the enclosing class post-strip and the
  generated sibling `ClassDef`s).

**Golden `.ph` corpus:**
- `construct_subclass_super.ph` — `Dog.new(name:, breed:)` sets both slots (exact case from
  annotations-construct-inheritance.md).
- `construct_subclass_hand_written_parent.ph` — the F-fix's own headline case: `@construct`
  subclass of a **hand-written**, non-`@construct` single-constructor parent infers correctly.
- `construct_subclass_ambiguous_super.ph` — `construct.super_ambiguous` compile error against a
  superclass with two differently-labeled constructor selectors (verify the fixture doesn't
  collide with `has_new_construct`'s guard assumptions — Rubric flag).
- `annotation_field_order_regression.ph` — a class with several `FieldDef`s in a specific order;
  assert the derived constructor's label order matches declaration order exactly (R3).
- `annotation_mixed_declared_inferred_unaffected.ph` — a **non**-`FieldDef` class's inference
  path is byte-identical pre/post this unit (the mixed-fields-hazard regression guard).
- `data_equality_hash_together.ph` — `@data`'s `==`/`hash` derived together; a hand-written `==`
  with `@data` still deriving `hash` is `attr.accessor_collision`.
- `data_with_shallow_copy.ph` — the shallow-vs-deep `with(...)` distinguishing case (Rubric).
- `sealed_same_unit_ok.ph` / `sealed_cross_module_violation.ph` — `attr.sealed_violation` fires
  at the *subclass's* site, not the sealed class's.
- `variant_visitor_exhaustive.ph` — `shape.match(circle:{...}, rect:{...})` dispatches correctly
  per variant; a call site omitting an arm is an ordinary missing-keyword-argument dispatch
  failure (no new diagnostic — exhaustiveness "for free," per annotations-data.md).
- `attribute_tier_ambiguous_error.ph`/`attribute_usage_violation_error.ph`/
  `attribute_compile_tier_forbidden.ph` — **these three rows in annotations-test-strategy.md's
  own table depend on the gated `Attribute`-class surface (`attribute-classes.md`, Install tier)
  and cannot be built by this unit** — flag explicitly in the return contract as
  correctly-out-of-scope, not silently dropped.

**Diagnostics catalog** (this unit's slice): `attr.accessor_collision`, `construct.super_ambiguous`,
`attr.sealed_violation` — all span-carrying, all recover under multi-error batching.

## 8. Decisions flagged
| ID | Decision | Resolution |
|---|---|---|
| **DEC-ANNOT-E** — resolved by grounding | Emit `MethodDef{is_constructor:true}` (per the spec's literal pseudocode) or a real `ConstructDef`? | **Real `ConstructDef`/`ClassMember::Construct`** — the field doesn't exist; the spec's pseudocode is stale (banner). |
| **DEC-ANNOT-F** — resolved by grounding | F-fix super-signature source: new side-table, or reuse `ClassObject.methods`? | **Reuse `ClassObject.methods` + `decode_selector`** (§2/§3.3) — no new persistent state, mirrors the existing `field_layouts` cross-class-read pattern. |
| **DEC-ANNOT-G** — flagged, needs a small in-unit call, not user sign-off | `@variant`'s sibling-statement generation: widen `expand_class_attributes`'s return type, or special-case in `compiler/lib.rs`? | **Recommend widen the return type** (§3.4) — keeps all generate-phase logic in `attributes.rs`. Coordinate with whatever U-ANNOT-CONTRACTS actually landed; this is the one place the "additive rows only" collision-avoidance promise has an exception. |
| **DEC-ANNOT-H** — flagged, not resolved | Mixed declared/inferred fields in one class: reject at compile time, or silently unsupported (§ hazard's "skip inference entirely once any `FieldDef` present")? | **Recommend**: not a hard error, just "any `FieldDef` present ⇒ inference off for this class" (§ hazard) — simplest, matches how `field_layouts`'s reopen-guard already treats "first definition is authoritative." A future revision could make partial-declaration a compile error if silent-skip proves confusing in practice; not blocking now. |

No item here is **BLOCKED-ON-DECISION** in the "needs the user" sense.

## 9. Must-not-preclude check
- **Install/Dispatch/Runtime tiers (gated, A-1–A-6):** *not touched.* No `Attribute` root class,
  no `Behavior.defineMethod`, no retained-instance store. `@data`/`@construct`/`@get`/`@set`
  remain pure compiler-side AST derives, exactly the user/compiler tier line
  `decorators.md`/`attribute-classes.md` already draw (Compile/Layout stay compiler-owned).
- **Real `match`-with-patterns (open-Q7):** *served, not precluded* — annotations-data.md's own
  point: when true pattern-matching syntax is designed, it can desugar to the same generated
  `__matchArm` visitor this unit produces, so the exhaustiveness mechanism carries forward
  unchanged. This unit must not hard-code the keyword-argument visitor shape in a way that
  assumes it's the *only* possible caller — keep `__matchArm` a plain overridable method, not
  something special-cased by the compiler.
- **Nested/namespaced `@variant` naming:** *deferred, not foreclosed* — `Circle` staying a
  global class name (Draft 0.1 simplification) doesn't bake namespace assumptions into the
  sealed/exhaustiveness machinery; a future namespace feature only needs to change *name
  resolution*, not this unit's closed-world/visitor logic.
- **Weak-reference / moving-GC per-receiver caching (ADR-0052's revisit trigger):** not
  applicable to this unit (no per-receiver runtime state here at all — that was U-ANNOT-CONTRACTS's
  `checking` set) — confirm `@data`'s `with(...)` and `@construct`'s derived bodies allocate no
  receiver-keyed side tables (they don't, by construction — flat field-copy/assignment only).
- **v0.3 per-instance behavior (A-6):** not touched — every derive in this unit produces
  ordinary class-dictionary methods (`Behavior`-owned, per `attribute-classes.md`'s own
  "install surface is `Behavior`-side, never per-instance" section), never anything scoped to a
  single receiver's own dictionary.
- **`equality-and-hash.md`'s existing invariants:** must not be silently violated by `@data`'s
  derived `==`/`hash` — this is the one place this unit's own work could regress an *existing*
  committed guarantee rather than merely fail to add a new one; the Rubric's explicit
  "read `equality-and-hash.md` first" flag is the mitigation, not a formality.

## 10. Return contract (report to `phalcom-reviewer`)
`FieldDef` + `ClassMember::Field` + the parser disambiguation · confirmation the mixed
declared/inferred field policy (DEC-ANNOT-H) was implemented as specified · the real-`ConstructDef`
derive (DEC-ANNOT-E) verified to round-trip through the *same* `compiler/lib.rs` L1170–1209 path
a hand-written `construct` uses (not a parallel path — the load-bearing check) · the F-fix's
`ClassObject.methods`/`decode_selector` lookup (DEC-ANNOT-F) + the zero/one/many-constructor
cases + `construct.super_ambiguous` · `@get`/`@set` + the collision check · `@data`'s five
derived members + the `@construct`-already-present no-op-not-collision handling + confirmation
against `equality-and-hash.md` · `@sealed`'s end-of-unit closed-world check + `@variant`'s
sibling-statement splice, **including the exact shape of whatever return-type widening
(DEC-ANNOT-G) was actually implemented** (report this precisely — it's the one place this unit
touches code the sibling unit wrote) · the generated `match(...)`/`__matchArm` visitor · all
AST-snapshot + golden-corpus results from §7, explicitly noting the three
`attribute_tier_ambiguous_error`/`attribute_usage_violation_error`/`attribute_compile_tier_forbidden`
rows as correctly out-of-scope (gated on the Install-tier `Attribute` class) · confirmation of
**zero `Value`/opcode change**, `ClassLayout`'s struct shape unchanged (only its population
source gained a second path) · a flagged doc-sync list: `annotations-construct.md`
(Prerequisite 2 already landed; derive pseudocode's `is_constructor` field doesn't exist) — not
edited by this unit, only flagged · `verify.sh` + `cargo doc` tails.
