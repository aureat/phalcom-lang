# The `@` mechanism — grammar, registry, `Attribute`, retention

- Status: **Verified against HEAD 2026-07-20.** The Compile-tier substrate and
  the retention/reflection layer are built; four recorded divergences (DEF-3…6)
  have fix plans below. The tier-hook dispatch surface (Install/Dispatch/
  Runtime) is specified and validated but never fired — its build plan lives in
  [behavioral.md](behavioral.md)/[interception.md](interception.md), not here.
- Ground truth: `phalcom-ast/src/parser.rs` (`parse_attribute`,
  `parse_attribute_arg_list`, `attach_attrs`), `phalcom-ast/src/ast.rs`
  (`struct Attribute`), `phalcom-core/src/compiler/attributes.rs`
  (`AttributeRegistry::new` — 12 rows, `AttributeExpander`,
  `expand_class_attributes`, `RESERVED_HOOKS`, `TIER_NAMES`,
  `validate_attribute_class`, `resolves_to_attribute_class`),
  `phalcom-core/src/primitive/attribute.rs` (`__attach`/`__attributes`/
  `__freezeAttributes`), `core.ph` (`class Attribute`, `class On`).
- Ratification base: ADR-0054 §1 (Compile/Layout normative), §2 (runtime tiers
  admitted, A-1–A-5 resolved, A-6 deferred); annotations-core.md D1–D3;
  annotations-legality-grammar.md (grammar, `Target`, legality table).

## 1. Grammar (as ratified; one divergence)

```ebnf
class-member  := attribute* member-decl
attribute     := "@" ident [ "(" attr-args? ")" ] NEWLINE*
attr-args     := attr-arg { "," attr-arg }
attr-arg      := expr                  (* bare ident parses as Expr::Var *)
```

- Attributes are **class-member-position only**. No statement, expression, or
  parameter attributes in v0.2. (The web draft's parameter binders `@body x: T`
  presuppose a grammar extension — recorded as W-1 in
  [frameworks.md](frameworks.md), not silently assumed.)
- An attribute binds to the **next member** across any number of newlines;
  `}`/EOF first ⇒ `attr.dangling`. Constructors cannot carry attributes today
  (`ConstructDef` has no `attributes` field; `attach_attrs` raises
  `attr.dangling`) — this asymmetry dissolves when ADR-0063 collapses
  `ConstructDef` into `MethodDef` ([placement.md](placement.md)).
- **Divergence (DEF-3):** the arg list is positional-expression-only. The
  ratified `@On(Method, tier: Install, inherited: true)` surface needs labeled
  attribute arguments, which do not parse. As built, tier is bare-positional
  (`@On(Method, Install)`), matched by name against `TIER_NAMES`.

## 2. Name resolution — two paths, one precedence

At expansion time a name `@name` resolves:

1. **Registry row** (`AttributeRegistry`, 12 rows on HEAD: `requires`,
   `ensures`, `invariant`, `construct`, `get`, `set`, `data`, `sealed`,
   `variant`, `On`, `native`, `ignore`). Compiler-owned; the only path that can
   run at Compile/Layout time.
2. **`Attribute` subclass** (`resolves_to_attribute_class` — a `class_parents`
   chain-walk to `core.ph`'s `class Attribute`). Instantiated and retained at
   class-definition time.
3. Neither ⇒ `attr.unknown`, hard compile error. Never warn-and-drop — an
   ignored decorator is a silent behavior change waiting to happen (the Java
   annotation-typo failure mode).

Registry wins over class if both exist. That precedence is currently
unobservable (no built name shadows a class) but must stay specified: a user
class named `data` must not capture `@data`.

### Naming resolution for COLL-3 (Proposed — PDR candidate)

Problem: the convention (README) makes stdlib Install/Runtime decorators
Capitalized `Attribute` subclasses — `@Retry`, `@Traced`, `@Lazy` — but
ADR-0057 deliberately kept proxy classes named `Retry`, `Trace`, `Lazy`. Two
classes cannot share one global name in one module, and the decorator/proxy
pairs are designed to ship together in the stdlib.

| Option | Shape | Cost |
|---|---|---|
| (a) lowercase attribute classes (`class retry is Attribute`) | keeps ADR-0057 spelling verbatim | violates the class-naming convention everywhere else; reads as a builtin, is not |
| (b) **suffix resolution: `@Name` tries `NameAttribute`, then `Name`** | `.NET` precedent (`[Foo]` → `FooAttribute`); proxy keeps `Retry`, decorator class is `RetryAttribute`, surface stays `@Retry` | second lookup on miss (compile-time only); one naming ceremony |
| (c) rename one side (`@Retryable` / `RetryProxy`) | no mechanism change | re-opens ADR-0057's ruling that both keep their natural names |

**Recommendation: (b).** It is the only option that preserves both ADR-0057's
surface and the naming convention, and the precedent's consequence is
well-understood: C# has lived with attribute-suffix resolution for two decades
without it being a source of confusion. The lookup order must be
suffix-first (`RetryAttribute` before `Retry`) so a proxy class can never be
accidentally instantiated as a decorator — the failure mode of the reverse
order. What it precludes: a class legitimately named `FooAttribute` that is
*not* an attribute becomes unusable as one word; acceptable.

## 3. The `Attribute` root, `@On`, and retention (built)

As specified in [on.md](../v0.2/decorators/on.md), verified: `@Name(args)`
desugars to `Name.new(args)` + `__attach(_)` at class-definition time; stores
live as `Vec<Value>` + frozen bit on `ClassObject`, `MethodObject`,
`ModuleObject`; post-freeze mutation raises `attr.frozen` (A-5); reflection via
`Behavior#attributes`/`attributesOfType(_)` in `core.ph`. The five reserved
hook selectors (`expand`, `finalizeLayout`, `wrap`, `resolveMissing`,
`aroundSend`) are validated (`attr.missing_hook`/`attr.undeclared_hook`/
`attr.compile_tier_reserved`) but never dispatched — a tier declaration on HEAD
is a validated claim, not behavior.

A-5's freeze is load-bearing beyond tidiness: it is what lets ADR-0053's
`has_runtime_interceptor` be a one-time bit instead of an epoch counter, and it
composes with PDR-0001 (classes are closed) into a clean invariant — **the
decoration of a class is fixed the moment the class is**. Any future
post-definition attach proposal must re-price ADR-0053 first.

## 4. Implementation plans (DEF-3…DEF-6)

Ordered by dependency; each is a small, self-contained unit. All are
parser/compiler-only — no floor amendment, no new bytecode.

### Plan §1 — labeled attribute arguments (DEF-3)

1. Extend `parse_attribute_arg_list` (`phalcom-ast/src/parser.rs`) to accept
   `ident ":" expr` alongside bare `expr`, producing
   `AttrArg { label: Option<String>, value: Expr }`; migrate
   `Attribute.args: Vec<Expr>` → `Vec<AttrArg>` (mechanical: every existing
   consumer reads `.value`).
2. Disambiguation is the same count-identifiers-before-colon trick ADR-0025's
   param labels already use; no new lexer state.
3. `validate_attribute_class` reads `tier:`/`inherited:` by label, keeping
   bare-positional acceptance for one release with a deprecation note in the
   error text ("write `tier: Install`").
4. Tests: golden positive (`@On(Method, tier: Install)` compiles), negative
   (`@On(tier: Bogus)` cites `TIER_NAMES`), snapshot of the parsed `AttrArg`
   list. Mutation-test the fixtures per the house rule.

Labeled args also unblock the behavioral family's surfaces
(`@Retry(times: 3, on: NetworkError)`) and `@get`'s enforced `(priv)`
argument — this plan is the critical-path item of the whole tree.

### Plan §2 — transitive attribute-class validation (DEF-4)

Replace `is_attribute_class`'s direct-parent check (`class_decl.rs`,
`superclass.name == "Attribute"`) with the same `class_parents` chain-walk
`resolves_to_attribute_class` already uses — validation and retention must
share one predicate or they drift again. A transitive subclass declaring a
reserved tier then correctly hits `attr.compile_tier_reserved`. Negative
fixture: `class A is Attribute {}` + `class B is A { wrap(m) {…} }`
with no `@On` ⇒ `attr.undeclared_hook`.

### Plan §3 — honor `inherited:` (DEF-5)

Gated on Plan §1 (needs the label to parse). `attributesOfType(cls)` in
`core.ph` grows a superclass walk guarded by each attribute's `inherited`
flag. Single inheritance ⇒ no diamond case (A-2's resolution). Until built,
`inherited:` stays rejected-at-parse rather than accepted-and-ignored — an
inert flag that parses is a lie in the API.

### Plan §4 — registry-wide argument validation (DEF-6)

Add `fn arity(&self) -> AttrArity` (`None`, `Exactly(n)`, `Labeled(&[...])`)
to `AttributeExpander`; the driver checks before `expand`. Every current
builtin declares `None` except `@requires`/`@ensures`/`@invariant`
(`Exactly(1)`) and `@get` (`Labeled(["priv"])` once DEF-7 resolves). New error
`attr.bad_args`, citing the expander's declared shape. This is deliberately a
registry-wide mechanism, not per-expander ad-hoc checks — native.md was right
to refuse to invent a one-off.

## Hazards

- **Two resolution paths must not disagree on one name** — registry-vs-class
  precedence is specified above; a test must pin it the day any stdlib
  `Attribute` subclass ships under a name that could shadow a builtin.
- **The freeze (A-5) is the IC guard's foundation.** Post-definition attach is
  not a small feature; it converts a bit into an epoch counter (ADR-0053's
  revisit trigger, shared with open-Q4).
- **`attr.unknown`'s helpfulness decays as the class path grows.** Once
  frameworks ship, a typo'd `@Colunm` fails as "unknown attribute" with no
  registry row to suggest. The error should list near-miss class names from
  the current scope's `Attribute` subclasses, not only registry rows.

## What this precludes

- **A third resolution path** (e.g. imported-module attribute namespaces with
  their own lookup rule). Attributes resolve like any other class name in
  scope, plus the registry. Module-qualified attribute references
  (`@Std.Retry`) are foreclosed until `extends`-position member access is —
  same parser limitation, one future decision.
- **User-defined Compile/Layout expanders** — inherited from A-3/ADR-0054;
  restated because this file is where an implementer would be tempted.
