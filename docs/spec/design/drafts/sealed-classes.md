# Sealed Classes & Variants — the closed set that shipped without a checker

- Status: **Draft** (exploration only — not proposed, not ratified, no owning unit)
- Date: 2026-07-15
- Depends on:
  [ADR-0041](../../../adr/accepted/0041-hierarchy-stability-policy.md) (hierarchy stability: single inheritance, sealed reparenting) ·
  [ADR-0026](../../../adr/accepted/0026-class-hierarchy-mutability.md) (methods open, superclass sealed) ·
  [ADR-0012](../../../adr/accepted/0012-selector-signature-encoding-and-dispatch.md) (selector identity) ·
  [ADR-0043](../../../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md) (no default arguments) ·
  [ADR-0044](../../../adr/accepted/0044-option-bootstrap-formalization-and-defer-niche-encoding.md) (Option bootstrap) ·
  [ADR-0054](../../../adr/accepted/0054-two-speed-ratification-annotation-decorator-tiers.md) (Compile/Layout tier, ratified)
- Related:
  [annotations-data.md](../experimental/annotations-data.md) (`@data`/`@sealed`/`@variant` — the *only* spec for this mechanism) ·
  [U-ANNOT-LAYOUT](../../../forge/units/U-ANNOT-LAYOUT/plan.md) (the owning unit; §3.4) ·
  [values-and-absence.md](../values-and-absence.md) (`Option`, `match(some,none)`) ·
  [open-questions.md](../open-questions.md) (Q7 residue — refutable patterns / `match` arms OPEN) ·
  `drafts/crypto.md` (the algorithm-set application; written concurrently — see §5)

> **Scope.** This doc audits a **built** mechanism and asks what an exhaustiveness
> checker would have to add. It does **not** specify `match`. A `match` *construct*
> remains OPEN (Q7 residue; [ADR-0046](../../../adr/accepted/0046-destructuring-bindings.md)
> shipped only irrefutable destructuring). Nothing here decides it.
>
> **Structure.** Exploration doc, expected to grow. Open questions are numbered S-1…
> and are meant to be appended to, not renumbered.

---

## 1. The finding: Phalcom has an ADT mechanism, and it is not a proposal

The brief that commissioned this doc framed sealing as a design question — *should*
Phalcom seal, how would it be spelled, where would it be enforced. **Every one of those
questions is already answered by shipped code.** Phalcom has `@sealed` + `@variant`: a
sealed class with declared variant arms that expand into sibling classes, plus a
generated eliminator. That is a sum type. It is closer to Kotlin's `sealed class` and
Rust's enums than to anything the brief anticipated.

What is *missing* is the other half: **a closed variant set exists at compile time, and
nothing consumes it as a totality proof.** That gap — not the sealing — is this doc's
subject.

### 1.1 As-built: what `@sealed` actually enforces

**It does two things, through two different code paths, reading two different sources
of truth.** This is the single most important fact in the doc; §2 argues the split is a
latent defect.

**(a) It blocks cross-unit `extends`** — `phalcom-core/src/compiler/lib/class_decl.rs:364-371`,
at the *subclass's* definition site:

```rust
if let Some(&sealed_in_module) = self.vm.sealed_classes.get(&sc_sym) {
    if sealed_in_module != self.module {
        return Err(CompilerError::Message(format!(
            "attr.sealed_violation: `{}` extends `@sealed` class `{}`, but was not declared in the same compilation unit",
            class_def.name, sc_ref.name
        )));
    }
}
```

It reads `VM::sealed_classes: HashMap<Symbol, ObjRef>` (`vm/mod.rs:194`) — class-name
symbol → the module handle that sealed it — written from the `@sealed` attribute at
`class_decl.rs:751-754`.

**(b) It gates `@variant`** — `attributes.rs:1540` computes `has_sealed` from the
**attribute list**, not the table, and threads it into `expand_variants`
(`attributes.rs:1265`), which rejects a `@variant` arm on an unsealed class:

```
attr.illegal_target: `@variant` requires its enclosing class `Shape` to also carry `@sealed`
```

So the answer to *"does `@sealed` enforce anything beyond gating `@variant`?"* is
**yes — it independently blocks `extends`, and the two enforcements do not share a
source of truth.**

### 1.2 As-built: what `@variant` expands to

`expand_variants` (`attributes.rs:1265-1345`), verified by reading it in full:

1. Collects `ClassMember::Variant(VariantDef)` members (`phalcom-ast/src/ast.rs:206`;
   `VariantDef { name, labels }` at `ast.rs:259`). Returns early if none.
2. Errors if `!has_sealed` (above).
3. **Strips** the variant members: `class.members.retain(|m| !matches!(m, ClassMember::Variant(_)))`.
   `ast.rs:203` says so — *"Never compiled directly — stripped and expanded."*
4. For each variant, synthesizes a **sibling top-level class**:
   - one `ClassMember::Field` per label, named `_{label}`, `mutable: true`;
   - `superclass: Some(SuperclassRef { name: class.name })` — the enclosing sealed class;
   - `attributes: vec![Attribute { name: "data" }]` — every variant is implicitly `@data`;
   - a `__matchArm(k1, k2, …)` method taking **positional** params named for *every*
     variant in the family (`lower_first` of each variant name, in declaration order),
     whose body is `return <ownKeyword>.call(self)`.
5. Generates `match(k1:, k2:, …)` on the enclosing class → `self.__matchArm(k1, k2, …)`.

The double-dispatch: each variant overrides the *same* `__matchArm` selector, differing
only in which positional block it calls. The code's own comment names the payoff —
*"a call site omitting or misnaming an arm is an ordinary missing-keyword-argument
dispatch miss, no new diagnostic needed."*

**The three expanders are deliberate no-ops.** `SealedExpander` (`attributes.rs:564`),
`VariantExpander` (`attributes.rs:591`), `DataExpander` (`attributes.rs:539`) all
`Ok(())` from `expand`. Registered at `attributes.rs:647-649`. `SealedExpander`'s doc
explains why: `@sealed`'s real work *"needs the compiling `Compiler`'s own module handle,
which `AttributeExpander`'s signature has no access to."* The registry rows exist only so
`attr.unknown` / `attr.illegal_target` fire correctly. `VariantExpander::legal_targets`
is `&[Target::Variant]`; the other two are `&[Target::Class]`.

### 1.3 As-built: it works, and it is tested

`phalcom-core/tests/lang/errors/annotation_variant_visitor_exhaustive.ph` — status
`PASS`, green in the tree:

```
@sealed
@data
class Shape {
  @variant Circle(radius:)
  @variant Rect(w:, h:)
}

let c = Circle.new(radius: 3)
System.print(c.match(circle: { circ => 3 * circ.radius }, rect: { rec => rec.w * rec.h }))
// → 9
System.print(c.toString)   // → Circle(3)
```

Full fixture inventory (`grep -rln "@sealed\|@variant" phalcom-core/tests/`):

| Fixture | Lane | Covers |
|---|---|---|
| `errors/annotation_variant_visitor_exhaustive.ph` | positive | expansion + visitor + `@data` `toString` |
| `compile-errors/annotation_variant_requires_sealed.ph` | negative | `@variant` without `@sealed` |
| `compile-errors/absence_option_sealed_violation.ph` | negative | `class MyOpt is Option {}` |
| `compile-errors/absence_some_sealed_violation.ph` | negative | ditto, `Some` |
| `compile-errors/absence_none_sealed_violation.ph` | negative | ditto, `None` |

**Coverage gap worth naming:** there is **no fixture for a user `@sealed` class being
extended from another module.** All three cross-unit `extends` fixtures test *core*
classes sealed at bootstrap. The `sealed_in_module != self.module` branch is therefore
exercised only via the bootstrap path, never via `@sealed` in user `.ph`. The decorator's
own headline enforcement is untested end-to-end. See S-2.

### 1.4 Owning unit and plan-vs-as-built divergence

The mechanism belongs to **U-ANNOT-LAYOUT** — its plan's title (`plan.md:1`) names
`@data`/`@sealed`/`@variant` explicitly, and §3.4 (`plan.md:206-262`) specifies the
expansion shapes.

**One real divergence, benign but recorded:** the plan (`plan.md:232-236`) specifies
`@sealed` as *"`finalize`-phase… At end-of-unit, verify every recorded
subclass-of-a-sealed-class was…"* — i.e. a deferred end-of-unit post-pass. The as-built
is an **immediate check at the subclass's definition site**. The code argues the
equivalence at `class_decl.rs:357-363`: *"Fires immediately rather than deferring to an
end-of-unit pass: the single-pass top-down discipline already guarantees a same-unit
sealed superclass is recorded before any of its subclasses reach this point."* That
reasoning holds **given** the top-down discipline, and `vm/mod.rs:185-193` repeats it.
The plan was not updated. Low-stakes, but the plan is now wrong about its own unit.

---

## 2. The defect: two sources of truth for "is this class sealed?"

The commissioning brief hypothesized that `Option` might be sealed *"by a different
mechanism than `@sealed`… two independent sealing mechanisms that do not know about
each other."* **That is not what is in the tree — but a narrower version of it is real.**

`8d401f4` (*"feat(core): seal Option/Some/None against user subclassing"*, 2026-07-14)
does **not** introduce a second mechanism. It writes **the same table**
(`vm/bootstrap.rs:215,220,261`):

```rust
self.sealed_classes.insert(option_sym, m);
self.sealed_classes.insert(some_sym, m);
self.sealed_classes.insert(none_class_sym, m);
```

Its message says so: *"Registers all three in `VM::sealed_classes` at bootstrap (keyed by
core module ObjRef), reusing the existing `attr.sealed_violation` enforcement in
`class_decl.rs`."* The `extends` enforcement is **unified**. `docs/adr/STATUS.md:74`
records this against ADR-0044, answering that ADR's open subclass-compatibility question
*"by ruling it moot"* — verified verbatim. (GC correctness is handled: the module
ObjRefs are rooted at `vm/gc.rs:107`.)

**The real seam is one level down.** Two predicates answer *"is `C` sealed?"*:

| Predicate | Reads | Enforces | Site |
|---|---|---|---|
| `class_attrs.iter().any(\|a\| a.name == "sealed")` | the **attribute list** | gates `@variant` | `attributes.rs:1540` → `1278` |
| `vm.sealed_classes.get(&sym)` | the **VM table** | blocks `extends` | `class_decl.rs:364` |

For a `@sealed` class in `.ph`, both agree — `class_decl.rs:751-754` derives the table
entry *from* the attribute. **For `Option`/`Some`/`None`, only the table has it.** The
attribute is absent, because — per `bootstrap.rs:207-211` and the commit message —
**`None` has no `.ph` class reopen to carry an annotation**: adding one would clobber the
`None` global back from the singleton to the class object.

The observable consequence, which no test covers:

> `Option` is sealed against `extends`, but does **not** carry `@sealed`. So a `@variant`
> arm declared inside an `Option` reopen would be rejected with *"requires its enclosing
> class `Option` to also carry `@sealed`"* — **even though `Option` is sealed.** The
> diagnostic would be false.

This is narrow (nobody is adding `@variant` to `Option` today) and it does **not**
undermine the `extends` guarantee. But it is a genuine design defect: *sealed* is a
property with two representations that can disagree, and the bootstrap path
deliberately populates only one. The fix is small — have `has_sealed` consult
`VM::sealed_classes` as well as the attribute list, or give the table a
`sealed_by_attribute` bit — but it is a **change to shipped semantics, not a doc edit**,
so this draft records it rather than making it. See S-1.

---

## 3. The gap: a closed set with no checker

Phalcom has, at compile time, for any `@sealed` class: **the complete list of its
variants** — `expand_variants` builds it (`attributes.rs:1286`, `variant_kw_names`) and
then *throws it away* after generating the visitor. It is not retained on `ClassObject`,
not in `VM`, not queryable. **The closed-world fact exists for the duration of one
function call.**

Nothing consumes it as a totality proof. And here is the subtlety that makes the gap
smaller than it looks:

### 3.1 Selector identity is already doing the checker's job

[ADR-0012](../../../adr/accepted/0012-selector-signature-encoding-and-dispatch.md)
encodes argument labels into selector identity, and
[ADR-0043](../../../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md)
refuses default arguments **specifically to keep that identity pristine**. Therefore:

- `match(circle:)` and `match(circle:, rect:)` are **different selectors**.
- Omitting an arm is not a "non-exhaustive match" warning — it is a **different message**,
  which the receiver does not understand.
- **Arity + labels = totality.** The arm set is not *checked* against the variant set; it
  is *named by the selector*. No wildcard `_` arm exists to write, because there is no
  wildcard.

`expand_variants`'s own comment claims exactly this — *"exhaustiveness for free"* — and
it is right, structurally. This is the Church / Böhm–Berarducci encoding: a sum type
*is* its eliminator. Phalcom arrives there not by intent but because keyword dispatch and
sealing compose that way.

**What sealing contributes is narrow and precise:** it keeps the variant set equal to the
eliminator's label set. Without it, `class Triangle is Shape {}` would be a `Shape`
for which `match(circle:, rect:)` has no arm — the eliminator silently partial.

### 3.2 So what would a `match` construct add?

Honestly: **less than the brief assumed, and it is genuinely open whether it is worth it.**

What the current mechanism already gives: a total eliminator over a sealed set, enforced
by dispatch, with no checker to write.

What it does **not** give, and what a checker/construct *could* add:

- **A diagnostic worth reading.** Today, a missing arm surfaces as a
  `doesNotUnderstand`-shaped miss on `match(circle:)`. It says nothing about *which*
  variant was forgotten. A checker that retained the variant list could say *"missing arm
  `rect:`"*. **This is the strongest concrete argument for the checker** — not soundness,
  which dispatch already delivers, but **diagnosis**.
- **Compile-time, not run-time, failure.** The arm-miss is a *dispatch* miss, i.e. it
  fires when the call executes. A sealed hierarchy is known statically, so the miss could
  be caught at compile time. Phalcom has no such pass today.
- **Binding/destructuring in arms.** ADR-0046 shipped irrefutable destructuring; arms
  currently bind via block params (`{ circ => … }`) and reach fields through `@data`
  accessors. Refutable patterns are Q7 residue.
- **Nested patterns**, which the eliminator cannot express at all — arms are blocks, so
  nesting means nesting `match` calls by hand.

**None of this is decided here.** (S-6.)

### 3.3 The generic hazard, for the record

A totality checker can only prove coverage over a **closed** variant set with
**unguarded** arms. Two ways to lose the proof:

1. **Guards.** An arm `case Circle(r) if r > 0` covers an undecidable subset — proving the
   guard's complement empty is theorem-proving. Rust, Scala, and Java all give up and
   assume a guarded arm covers nothing. **Phalcom has no guard surface**: arms are opaque
   blocks. Guards are expressible *inside* an arm without defeating anything, because the
   arm is total regardless of what its block does. **This is an accidental advantage — a
   syntactic guard would be the first construct that could make coverage undecidable.**
   See S-5.
2. **Open sums.** Phalcom's classes are open by default, so an eliminator over an
   arbitrary hierarchy can never be total. Exhaustiveness applies **only** to sealed
   hierarchies. That is arithmetic, not preference — and the tree implements exactly it.

The honest limit, which `annotations-data.md` §"Hazards" already flags: **the per-unit
check means a `@sealed` class is only sealed against units the compiler compiles.** There
is no closed-world/link-time pass (Q8 territory). A late-loading unit is caught because
*its own* `extends` is compiled and rejected — a guarantee only as good as "all code goes
through this compiler."

---

## 4. What sealing does not touch: method reopening

[ADR-0026](../../../adr/accepted/0026-class-hierarchy-mutability.md) keeps **methods
open** — add/redefine at runtime, epoch-guarded — while sealing the **superclass** at
definition. [ADR-0041](../../../adr/accepted/0041-hierarchy-stability-policy.md) supplies
policy and enforcement; `primitive/class.rs:43` is the whole runtime half:

```rust
pub fn class_set_superclass(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::InvalidSetSuper.into())
}
```

with `class.rs:37-38` making the axis-split explicit: *"Method reopening… is a separate
axis and is unaffected by this seal."* Test: `phalcom-core/tests/invariants.rs:224`,
`sealed_hierarchy_rejects_runtime_reparent_and_keeps_invariants()`.

**Three different seals — conflating them is the main hazard here:**

| Seal | Axis | Where | When |
|---|---|---|---|
| `InvalidSetSuper` (ADR-0026/0041) | reparenting an *existing* class | `primitive/class.rs:43` | runtime, always |
| `attr.sealed_violation` (`@sealed`) | *adding* a subclass cross-unit | `class_decl.rs:364` | compile, per-unit |
| override epoch (ADR-0018/0053) | method redefinition | dispatch guard | runtime |

**Is "sealed variant set + open methods" coherent for exhaustiveness? Yes**, and the
reason is precise: exhaustiveness quantifies over the *variant set*, not over *behavior*.
Redefining `Circle#area` changes what an arm does; it cannot mint a `Shape` that is
neither `Circle` nor `Rect`. Totality survives arbitrary method mutation because method
reopening does not create classes.

Two consequences this doc does not resolve:

- **The eliminator is itself reopenable.** `core.ph:530` routes derived methods through
  `match` *"so a user-overridden `match` is respected (R-INV-2.4)"* — deliberate, and
  documented. So the *arm set* is sealed while the *eliminator* is not.
- **`__matchArm` overrides are ordinary methods**, hence reopenable, hence the
  double-dispatch is subvertible per-variant.

Neither breaks the variant-set claim. Both mean "exhaustive" here is **"every variant is
named"**, not **"every variant is correctly handled."** That distinction must survive into
any future `match` spec. See S-7.

---

## 5. The security case: sealed algorithm sets and algorithm confusion

The application that motivates caring. **Cross-reference `drafts/crypto.md` for the
crypto surface — this section covers only the sealing argument.** (That file did not
exist in-tree when this was written.)

### 5.1 The bug class

JWT's `alg` header is **attacker-controlled** and **selects the verification routine**.
That sentence is the entire vulnerability class.

- **`alg: none`.** Disclosed by Tim McLean (Auth0), late March 2015 — [*"Critical
  vulnerabilities in JSON Web Token libraries"*](https://auth0.com/blog/critical-vulnerabilities-in-json-web-token-libraries/),
  corroborated by the [oss-sec thread, 2015 Q2](https://seclists.org/oss-sec/2015/q2/3).
  (The live page shows a 2020 byline — a re-publish timestamp, not the disclosure date.)
  `none` selected a **no-op verifier that returned success**: strip the signature, set
  `alg: none`, forge any payload. Named as affected: node-jsonwebtoken, PyJWT, jjwt,
  php-jwt, jsjwt, namshi/jose.
- **RS256 → HS256 confusion.** Vulnerable libraries expose one `verify(token, key)` where
  the header decides whether `key` is an *RSA public key* or an *HMAC secret*. RSA public
  keys are public by construction. Flip the header to HS256, compute
  `HMAC-SHA256(header.payload, <public key bytes>)`, and the server — told by the attacker
  which routine to run — recomputes the same HMAC and accepts. Auth0: *"it will think the
  public key is actually an HMAC secret key."*

### 5.2 Verified CVEs

| CVE | Library | Class |
|---|---|---|
| CVE-2015-9235 | node-jsonwebtoken `>=0.1.0 <4.2.2` | RS→HS confusion; fixed 4.2.2 (NVD published 2018 — ID reserved 2015, publication lagged) |
| CVE-2016-10555 | jwt-simple ≤0.3.0 | algorithm not enforced in `jwt.decode()`; fixed 0.3.1 |
| CVE-2016-5431 | gree/jose-php <2.2.1 | `_verify()` auto-detected the algorithm from the unverified header |
| CVE-2017-11424 | PyJWT ≤1.5.0 | `prepare_key`'s blocklist missed the PKCS1 PEM public-key header; fixed 1.5.1 |
| CVE-2018-1000531 | **Inversoft prime-jwt** ≤1.3.0 | `alg: none` treated as pre-verified |
| CVE-2022-29217 | PyJWT 1.5.0–2.3.0 | public-key formats (incl. `ssh-ed25519`) usable as HMAC keys; fixed 2.4.0 |
| CVE-2022-23540 | node-jsonwebtoken ≤8.5.1 | `jwt.verify()` defaulted to allowing `none`; fixed 9.0.0 |

**Corrections to the commissioning brief, flagged not silently fixed:** CVE-2018-1000531
is **Inversoft prime-jwt**, *not* `io.jsonwebtoken`/jjwt and not Auth0's library. **No CVE
could be verified for jjwt's 2015 issue** — cite the Auth0 post, not a CVE.
**CVE-2022-21449** ("Psychic Signatures", Neil Madden / ForgeRock, Java 15–18) is a
**different bug class** and is excluded: a missing `r,s ≠ 0` check in Java's pure-Java
ECDSA rewrite, so an all-zero signature verifies against any key. It affects `ES256` JWTs
but has nothing to do with `alg` selection.

### 5.3 The fix *is* a sealed set

Every ecosystem converged on "close the variant set, supply it out-of-band":

- **node-jsonwebtoken**: `jwt.verify(token, key, { algorithms: ['RS256'] })`.
- **PyJWT**: `algorithms=` became **mandatory in v2.0.0** — changelog: *"Require explicit
  `algorithms` in `jwt.decode(...)` by default."*
- **[RFC 8725](https://datatracker.ietf.org/doc/html/rfc8725) §3.1**: *"Libraries MUST
  enable the caller to specify a supported set of algorithms and MUST NOT use any other
  algorithms when performing cryptographic operations."* §2.1 describes the `none` bug;
  §3.2 the RS/HS confusion. (Not §2.7 "Substitution Attacks" — that is audience replay.)

**The general shape:** an *open* algorithm set means an attacker-supplied identifier can
select a variant the author never enumerated. A `@sealed` set + an eliminator whose
selector names every arm makes the unenumerated case **impossible to express** — omitting
an arm is a *different selector* (§3.1).

Note what Phalcom's shape buys over the mainstream framing. RFC 8725 §3.1's "MUST enable
the caller to specify a set" is a **runtime allow-list** — an API convention, checked at
runtime, and every CVE above is someone not passing the list. In a sealed Phalcom
hierarchy the allow-list is **not a parameter that can be forgotten; it is the selector**.
`alg: none` becomes a variant that does not exist in the sealed set, so the header value
fails to resolve to any arm *before* any verification routine is chosen. The failure mode
moves from **silent success** to **no such variant**.

Honest caveats: (a) this is an argument about *shape* — the crypto surface is
`drafts/crypto.md`'s business; (b) **parsing an attacker-controlled string into a sealed
variant is still a lookup that must fail closed** — sealing makes the arm set total, it
does not make the *parser* safe, and that lookup is exactly where `alg: none` would
re-enter; (c) none of this is specified, proposed, or owned. See S-8.

---

## 6. Beyond exhaustiveness: what the closed set could buy dispatch

The overlay records the first-order consequence (`overlay.md:17`): sealed reparenting
*"substitutes for an IC-invalidation scheme on the hierarchy axis: slot offsets +
`ClassId` dispatch are provably stable, so a future IC keys on `ClassId` with **no
invalidate-on-reparent case**."* ADR-0040's row (`overlay.md:24`) says the same for
`SuperSend`: the original "invalidate on `superclass=` bump" note is *"now half-moot."*

**A sealed *variant set* is strictly more information than sealed reparenting**, and the
tree has not spent it — `expand_variants` discards the list (§3). Reparent-sealing says
*existing* edges are stable. Variant-sealing says **the receiver set at a call site with a
statically-known sealed receiver is finite and enumerable at compile time.** Inputs to:

- **Monomorphic devirtualization.** A sealed call site with one variant is a direct call.
  Swift's SE-0117 names exactly this: non-overridable-by-default lets *"the vast majority
  of class methods be trivially devirtualized."*
- **PIC pre-population.** [ADR-0012](../../../adr/accepted/0012-selector-signature-encoding-and-dispatch.md)'s
  IC is designed, population deferred. A sealed *n*-variant hierarchy gives an *n*-way PIC
  that is **provably complete** — no megamorphic fallback, no unseen-receiver miss path.
  Today's ICs learn by observation; a sealed set is known before the first send.
- **The `@variant` visitor specifically.** `Shape#match(circle:, rect:)`'s double-dispatch
  is *n* virtual calls' worth of machinery to express what is, over a sealed set, a jump
  table on `ClassId`.

**This is speculation, not a plan.** No measurement, no unit. ADR-0051's measure-first
discipline explicitly forbids acting on it without a benchmark, a profile attributing cost
to a named mechanism, and a recorded before/after. Recorded as opportunity, not
recommendation. See S-9.

The override epoch still applies to all of the above — sealing the variant set does not
seal method redefinition (§4). Sealing removes *one* invalidation axis, not both.

---

## 7. Precedent, with consequence

Re-aimed at the mechanism that actually exists: a per-unit-sealed class with declared
variant arms and a generated eliminator.

| Language | Rule (verified) | What it cost |
|---|---|---|
| **Kotlin** | `sealed`: nested-only (1.0) → same **file** (1.1) → same **compilation unit + package** (1.5; interfaces prototyped 1.4.30) | The loosening is the story. [KEEP `sealed-interface-freedom`](https://github.com/Kotlin/KEEP/blob/master/proposals/sealed-interface-freedom.md) names the pain: *"It's painful to create complex sealed class hierarchy — nesting is too deep"* (KT-11573), and the need to *"split large sealed class hierarchies into several files."* Kotlin **rejected a Java-style `permits` clause** as *"against Kotlin tradition of avoiding source-code repetition of information that could be inferred by the compiler"* — **which is precisely Phalcom's `@variant` design**: arms are declared *inside* the class body, inferred, never repeated. Cost accepted: *"Once a module with a sealed interface is compiled, no new implementations can be created."* |
| **Java 17** | JEP 409 (previews 360/JDK15, 397/JDK16). `sealed … permits`; same module, or same package if unnamed | Needed a JEP chain **plus** `non-sealed` — *"the first hyphenated keyword proposed for Java"* — because sealing controls only the *immediate* permits list: a subclass *"reverts to being open… A sealed class cannot prevent its permitted subclasses from doing this."* Exhaustive matching was an **explicit goal** (JEP 409 Goals: *"a foundation for the exhaustive analysis of patterns"*; JEP 441 §"Exhaustiveness and sealed classes"). **Most relevant detail:** even a statically-exhaustive sealed switch gets a **synthetic `default` throwing `MatchException`**, because separate compilation means a novel implementation can appear at runtime. **Java does not trust its own proof at link time — and Phalcom's per-unit check (§3.3) has the same hole, without the backstop.** |
| **Scala** | `sealed trait`: same **file**. Exhaustivity is a **warning** | The weaker choice cost real safety: `-Xfatal-warnings` (later `-Wconf`) is needed to make it bite, and community consensus treats the default as insufficient; `@unchecked` disables it outright. Scala 3's `enum` cuts boilerplate and improved the checker's *precision* (guards, unsealed types no longer silently disable it) but **did not** promote it to an error; the unmerged [SIP "Sealed Types"](https://contributors.scala-lang.org/t/sip-sealed-types/5082) shows the tension is live. Scala's file-scoping rationale is a compilation constraint: scanning the whole program *"would prevent separate compilation and be rather slow."* **Phalcom sidesteps the warning-vs-error question entirely — a missing arm is a dispatch miss, which is neither.** |
| **Rust** | Enums sealed **by construction** | The expression problem undiluted: adding a variant breaks every exhaustive `match` downstream. [RFC 2008](https://rust-lang.github.io/rfcs/2008-non-exhaustive.html) `#[non_exhaustive]` is the **opt-out** that re-opens the sum — *"will force downstream crates to add a wildcard arm… ensuring that adding new variants is not a breaking change"* — trading exhaustiveness for evolvability. API Guidelines caveat it: *"should be used deliberately and with caution… can make your code much less ergonomic."* **Phalcom has no `#[non_exhaustive]` analogue and its break is worse — see §8.** |
| **Swift** | SE-0117 (Swift 3): `public` = usable outside module, **not subclassable**; `open` = both | **A different axis, and the distinction matters.** Swift's is *per-class finality*, not a *permitted-variant set*. Notably Swift's `public` is **sealed-to-the-module by default** — Phalcom's `@sealed` landing point, reached by defaulting rather than annotating. Took **three review rounds**. Separately SE-0192 (Swift 5) `@frozen`/`@unknown default`: Apple found only ~6 of 60+ Foundation enums genuinely needed exhaustiveness — which is why they chose a frozen/non-frozen split over "everything sealed." |
| **Smalltalk / Self** | **No sealing at all.** Classes open forever; `become:` can swap an object's class at runtime | **The most important precedent — the road Phalcom is leaving.** With nothing statically provable closed, Self bought "dispatch over a small known set of shapes" *at runtime, per call site, forever*: type feedback → customization (Chambers & Ungar, POPL '89) → polymorphic inline caches (Hölzle, Chambers, Ungar, ECOOP '91, LNCS 512) → **dynamic deoptimization** (Hölzle, Chambers, Ungar, PLDI '92, pp. 32–43) to unwind speculation when an assumption breaks. That four-part machine **is** ADR-0018's machinery. Sealing is how a language buys at compile time what Self pays for at runtime, continuously. |

**The convergence worth naming:** Rust (`#[non_exhaustive]`), Java (`non-sealed`), and
Swift (`@frozen`) independently landed on *"closed by default, with an explicit per-type
annotation to declare it open."* Phalcom is the **inverse** — open by default, `@sealed`
to close — which is the Smalltalk inheritance showing through. Whether that default is
right is not this doc's call (S-4).

**And the scope question was settled by accident.** Kotlin walked file → module over five
years; Phalcom's `class_decl.rs:364` started at the compilation unit. ADR-0027's *file =
module* survives via [ADR-0045](../../../adr/accepted/0045-module-import-relative-path-whole-module-binding.md),
so Phalcom's "unit" is Scala's "file" and Kotlin's "module" **simultaneously** — the
precedents' disagreement dissolves at Phalcom's current scale, and **re-opens the moment
modules become multi-file.** See S-3.

---

## 8. What this precludes

Costs already paid, since the mechanism shipped:

- **Users cannot extend `Option`/`Some`/`None`.** No user Option variant. Deliberate
  (ADR-0044's row calls it moot) — and *exactly the cost Kotlin accepted*.
- **`@variant` names are top-level siblings, not namespaced.** `annotations-data.md`
  §"Draft 0.1 simplification" is explicit: `Circle`, not `Shape.Circle`, and Phalcom has
  no namespacing. The doc records this as *"deferred, not foreclosed"* — a future feature
  can rescope "without changing the sealed/exhaustiveness semantics." **Unverified claim;
  check it before relying on it.** Note the collision risk is real *now*: two sealed
  families with a `Circle` arm in one unit collide at top level.
- **Every `@variant` is implicitly `@data`** (`attributes.rs:1329-1332`), with `mutable:
  true` `_`-prefixed fields. A variant that wants different field semantics has no path.
- **No `non-sealed` / `#[non_exhaustive]` / `@frozen` escape hatch exists.** Java, Rust,
  and Swift each needed one. Phalcom has none — and its break is **sharper than any of
  them**: because arms are *selectors*, adding a third `@variant` to `Shape` renames
  `match(circle:, rect:)` to `match(circle:, rect:, tri:)` **at every call site in the
  program.** Maximally visible, maximally viral. This is the expression problem arriving
  through ADR-0012's front door, and it is the strongest argument against sealing anything
  casually. Not resolved here (S-10).
- **Sealing does not seal behavior** (§4). "Exhaustive" = every variant is *named*.
- **Sealing is compile-time and per-unit** (§3.3). No closed-world pass; bounded by "all
  code goes through this compiler." Java at least backstops with `MatchException`.

---

## 9. Spec-side divergence

Reported loudly, per the brief:

1. **`@sealed`/`@variant` are absent from [`decorators-stdlib.md`](decorators-stdlib.md).**
   Grepped: no `sealed`/`variant` rows (the only hits are `@invariant`, an unrelated
   attribute, at lines 129-132). The stdlib decorator catalogue does not list two shipped
   Compile-tier attributes.
2. **The only spec is [`experimental/annotations-data.md`](../experimental/annotations-data.md).**
   A built, tested, green mechanism whose sole specification sits in `experimental/`.
   Read against the code, it **matches** — expansion shapes, the visitor, the
   subclass-site placement, the `attr.sealed_violation` name all line up, and it correctly
   flags its own cross-module gap. It is accurate; it is just filed as experimental while
   the code is real.
3. **[`attribute-classes.md`](../decorators/on.md) does not mention them either**
   (hits at 366/452/533 are `invariant`/"variant" in prose, unrelated).
4. **U-ANNOT-LAYOUT's plan is stale about its own unit** (§1.4): specifies a finalize-phase
   end-of-unit post-pass; as-built is an immediate subclass-site check.

Net: **the mechanism is ahead of its documentation.** `annotations-data.md` should
probably graduate out of `experimental/`, and `decorators-stdlib.md` should gain rows.
Both are doc moves with ratification implications (ADR-0054 already ratified the Compile
tier), so this draft names them rather than making them. See S-11.

---

## 10. Open questions

| # | Question | Notes |
|---|---|---|
| S-1 | ~~**Unify the two "is sealed?" predicates?**~~ **PARTLY RESOLVED 2026-07-15 (CB-3).** | The **gate** is fixed: `has_sealed` now reads `sealed_by_attr \|\| sealed_by_table` (union), so the false *"`@variant` requires `Option` to also carry `@sealed`"* is gone. **The unification is not done** — filed as DEFERRED #35. Note the fix this doc implied ("read `VM::sealed_classes`") would have **inverted** the bug: a user's own `@sealed class` is not in that table while its body expands (`class_decl.rs` inserts it *after* the body compiles), so a table-only gate rejects every user `@variant`. Neither source is complete; the union is required. Blocker for true unification: `Option`/`Some` have `.ph` reopens that could carry `@sealed`, but **`None` has none** — plus a seal-ownership question when bootstrap and `core.ph` both write the table. |
| S-2 | ~~**Test the decorator's own headline enforcement.**~~ **DISSOLVED 2026-07-15 — the test cannot be written.** | Not a coverage gap: the scenario is **unreachable**. (1) **Ordering** — `extends` resolves at *compile* time, `import` binds at *runtime*; give the imported lib a `System.print` and it never runs, the `Unknown superclass` error fires first. (2) **Naming** — `extends S.Shape` does not parse, and ADR-0045's whole-module binding leaks no globals. So `attr.sealed_violation` is **dead for user classes**; module structure already supplies the protection, and the check is live only for the globally-visible bootstrap-sealed kernel. **`@sealed`'s only live effect on a user class today is gating `@variant`.** Pinned by `compile-errors/decorators_sealed_cross_unit_needs_isolation.ph`, which must change if cross-module class references ever land. Positive half: `decorators/decorators_sealed_same_unit_subclass_allowed.ph`. |
| S-3 | Does per-unit sealing survive multi-file modules? | Dormant while ADR-0045/0027 keep file = module. Kotlin's file→module walk is the precedent for what happens when it wakes. No closed-world/link-time pass exists (Q8). |
| S-4 | Open-by-default + `@sealed`, or closed-by-default + an escape hatch? | Rust/Java/Swift converged on the latter; Phalcom has the former via Smalltalk inheritance. Probably settled by inertia — but it should be settled *on purpose*. |
| S-5 | Should a `match` surface admit **guards**? | §3.3. Arms are opaque blocks, so guards are already expressible *inside* an arm without defeating totality. A *syntactic* guard would be the first construct that could make coverage undecidable. **Not having one is an advantage; notice it before spending it.** |
| S-6 | Is a `match` **construct** wanted, given `match(circle:, rect:)` already works? | **OPEN — Q7 residue; ADR-0046 shipped only irrefutable destructuring.** §3.2 argues the real gains are *diagnosis* (naming the missing arm) and *compile-time* failure, not soundness. That is an argument, not a ruling. `overlay.md:145` states the tension: real `match` must respect selector identity + the open-classes/sealedness axis. |
| S-7 | Is a reopenable eliminator acceptable? | §4. `core.ph:530` deliberately routes derived methods through `match` so a user override is *respected* (R-INV-2.4). `Option#match` can be overridden to ignore `none:`. Totality survives; behavior does not. Feature or hazard? |
| S-8 | Should the sealed-algorithm-set argument (§5) become a real `drafts/crypto.md` obligation? | Cross-ref only. Note §5.3's caveat (b): the string→variant lookup must fail closed, and that is where `alg: none` re-enters. |
| S-9 | Spend the closed variant set on dispatch? | §6. Complete PICs, devirtualization, `ClassId` switch. `expand_variants` currently **discards** the list. **ADR-0051 forbids acting without benchmark + profile + recorded before/after.** |
| S-10 | Migration story for adding a variant? | §8. Arms are selectors, so a new variant renames the eliminator at every call site. No `non-sealed`/`#[non_exhaustive]`/`@frozen` analogue. Sharpest cost of the current design. |
| S-11 | Graduate `annotations-data.md` out of `experimental/`; add `decorators-stdlib.md` rows; refresh U-ANNOT-LAYOUT's plan. | §9. Doc moves with ratification implications (ADR-0054 ratified the Compile tier). |
| S-12 | Do `@variant` sibling names need namespacing sooner than "deferred"? | §8. Two sealed families with a `Circle` arm in one unit collide at top level *today*. |

---

## Appendix: verification log

All claims checked against the working tree at `4c6c83f` (branch `main`), 2026-07-15.
`graphify query` for orientation (sealed enforcement; `class_set_superclass`/
`InvalidSetSuper`), then `git show 8d401f4 --stat` and targeted `grep`/`sed` for
line numbers. `SealedExpander`, `VariantExpander`, and `expand_variants` were read in
full.

**Findings that contradicted the briefs, recorded per this repo's rules:**

1. **The original brief framed `@sealed`'s spelling / scope / enforcement point as open
   design questions.** All three are shipped. Spelling: a Compile-tier decorator
   (ADR-0054, ratified), not a keyword. Scope: the compilation unit. Enforcement:
   `class_decl.rs:364`, at the subclass's site. (The brief's argument that a *keyword* is
   right *because* the check is definition-time does not discriminate — ADR-0054's Compile
   tier already **is** definition-time.)
2. **The original brief expected the sealing check adjacent to `class_set_superclass` →
   `InvalidSetSuper`.** It is not, and should not be: that is a *runtime reparent* guard
   on an existing class; `@sealed` is a *compile-time* check on a *new subclass's*
   definition. Different axes (§4's table).
3. **The original brief expected exhaustiveness to arrive "the day `match` lands."** The
   generated visitor is green now (`annotation_variant_visitor_exhaustive.ph`), and
   `Option#match(some:, none:)` has been the eliminator in `core.ph:450` since before
   this session.
4. **The follow-up's hypothesis — *"two independent sealing mechanisms that do not know
   about each other"* — is NOT what is in the tree.** `8d401f4` writes the *same*
   `VM::sealed_classes` the decorator writes; `extends` enforcement is unified and the
   commit message says so. **But a narrower real defect exists** and is probably the most
   valuable thing here: the `@variant` gate reads the *attribute list* while `extends`
   reads the *table*, so `Option` is sealed-against-`extends` yet not `@sealed`-carrying
   (§2, S-1).
5. **The brief's thesis that `Option`'s sealing was done for an unrelated reason is
   correct** (ADR-0044 bootstrap), **but the mechanism was not incidental** — it reused a
   general `@sealed` built for `annotations-data.md`. Only the *application to `Option`*
   was the happy accident.
6. **CVE corrections** (§5.2): CVE-2018-1000531 is Inversoft prime-jwt, not jjwt or
   node-jsonwebtoken; no CVE verified for jjwt's 2015 issue; CVE-2022-21449 is a different
   bug class and is excluded.
