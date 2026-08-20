# Implementation Specification: @variant Global Name Recognition

## Metadata

- **Status**: Approved design; implementation pending
- **Date**: 2026-08-20
- **Scope**: Phalcom compiler name predeclaration, @variant semantics, conformance coverage
- **Primary code owner**: phalcom-core compiler
- **Normative feature source**: [annotations-data.md](../../spec/design/experimental/annotations-data.md), @variant section

## Executive summary

@variant declarations generate ordinary top-level sibling classes, but the
compiler's pre-scan of known global names currently sees only source-level
Statement::Class names. A reference to a generated variant from an enclosing
class member is consequently misclassified as an implicit self-send.

Fix the declaration inventory used by predeclare_known_globals() so it also
records every direct ClassMember::Variant name before any class body is
compiled. This makes generated variant names obey the existing global-name
resolution rule without changing runtime expansion order, selector identity,
implicit-self semantics, or class construction.

## 1. Problem and observed behavior

This program is valid under the existing @data/@sealed/@variant model:

~~~phalcom
@data @sealed
class Ordering {
  @variant Less()
  @variant Greater()
  @variant Equal()
  @variant Unordered()

  @class less { Less.new() }
}

System.print(Less)
System.print(Ordering.less is Less)
~~~

The generated form is semantically equivalent to an ordinary global class:

~~~phalcom
class Ordering {
  @class less { Less.new() }
}

@data
class Less is Ordering {}
~~~

The explicit form works because Less is a top-level Statement::Class and is
predeclared as a known global. The attribute form fails in the class-side
method because Less remains a ClassMember::Variant until attribute expansion
runs inside compile_class().

### 1.1 Live compiler path

The current path is:

1. Compiler::compile() invokes predeclare_known_globals() before compiling
   program statements
   ([mod.rs:405-407](../../../phalcom-core/src/compiler/lib/mod.rs#L405-L407)).
2. predeclare_known_globals() currently records direct class names and
   binding-pattern names, but not names carried by class members
   ([mod.rs:458-497](../../../phalcom-core/src/compiler/lib/mod.rs#L458-L497)).
3. @variant is represented as ClassMember::Variant(VariantDef) and is
   explicitly documented as being stripped and expanded into a sibling
   top-level class
   ([ast.rs:412-417](../../../phalcom-ast/src/ast.rs#L412-L417)).
4. expand_variants() collects those members, removes them from the enclosing
   class, and synthesizes Statement::Class siblings
   ([attributes.rs:1746-1765](../../../phalcom-core/src/compiler/attributes.rs#L1746-L1765),
   [attributes.rs:1826-1844](../../../phalcom-core/src/compiler/attributes.rs#L1826-L1844)).
5. The compiler defines the enclosing class first, then recursively compiles
   generated siblings so their extends Ordering lookup can see the parent
   global
   ([class_decl.rs:253-263](../../../phalcom-core/src/compiler/lib/class_decl.rs#L253-L263),
   [class_decl.rs:1131-1143](../../../phalcom-core/src/compiler/lib/class_decl.rs#L1131-L1143)).
6. In a member body, bare-name resolution checks locals, upvalues, linked
   bindings, known globals, and then implicit self
   ([scope.rs:255-280](../../../phalcom-core/src/compiler/lib/scope.rs#L255-L280)).
7. A bare Less classified as ImplicitSelf emits self plus a zero-arity
   Invoke(Less), rather than GetGlobal(Less)
   ([expr.rs:1027-1044](../../../phalcom-core/src/compiler/lib/expr.rs#L1027-L1044)).

The generated Less class therefore exists in the runtime global namespace,
but the enclosing class method was compiled with the wrong static resolution
classification.

## 2. Semantic contract

### 2.1 Variant identity

For each declaration:

~~~phalcom
@variant V(label1:, label2:)
~~~

inside class Base, Phalcom defines one ordinary sibling class with the
semantic shape:

~~~phalcom
@data
class V is Base { /* generated fields and visitor method */ }
~~~

V is an ordinary global class name in the declaring compilation unit. It is
not a field, getter, lexical local, nested namespace member, or implicit
Base.V lookup. This preserves the existing feature specification
([annotations-data.md:107-123](../../spec/design/experimental/annotations-data.md#L107-L123))
and the AST contract for VariantDef
([ast.rs:461-479](../../../phalcom-ast/src/ast.rs#L461-L479)).

### 2.2 Declaration visibility during compilation

Every valid generated variant name is part of the compilation unit's known
global declaration inventory before any source member body in that unit is
lowered.

This is a compile-time visibility rule. It does not move runtime definition
or initialization:

- Base is still defined before its generated siblings.
- Generated siblings are still compiled immediately after Base.
- The generated sibling's superclass lookup still uses the ordinary
  GetGlobal(Base) path.
- A method body may be compiled against a known global before that global's
  runtime slot is initialized, exactly like other forward-known globals. A
  call must execute only after normal module initialization has defined the
  class.

### 2.3 Bare-name resolution precedence

The fix must not alter existing precedence:

~~~text
local → upvalue → linked binding → known global → implicit self
~~~

Therefore:

- A local or upvalue named Less continues to shadow the generated global.
- A linked/import binding continues to resolve before the generated global
  inventory entry.
- An unshadowed Less in a member body resolves as a global class value.
- An explicit self.Less remains an ordinary send to the receiver and is not
  rewritten to the generated class.
- A bare unresolved name in a member with a receiver continues to use
  implicit-self semantics; only declaration inventory becomes more complete.

The current object-model rule, “ordinary unresolved names use local → upvalue →
known global → implicit self,” remains governing
([object-model.md:62-65](../../spec/current/object-model.md#L62-L65)). Generated
variant names become known globals; they do not introduce a new resolution tier.

### 2.4 Class-side method behavior

For:

~~~phalcom
@class less { Less.new() }
~~~

self is the Ordering class object because @class installs the member on the
metaclass. Less is the global class object. Less.new() is then an ordinary
class-side constructor send on that class object. The resulting instance has
Less as its direct class and Ordering in its superclass chain, so:

~~~phalcom
Ordering.less is Less
~~~

evaluates to true.

### 2.5 Name collisions and invalid declarations

The fix does not add a new collision namespace or diagnostic. Generated
siblings continue through the existing Statement::Class compilation path,
including class-definition checks, superclass validation, sealed-hierarchy
checks, layout construction, and global definition.

Consequences:

- A generated variant name colliding with an explicit class name is handled by
  the existing class-already-defined path.
- A generated variant name colliding with an import is handled by the existing
  class/import collision path.
- @variant without @sealed still fails with the existing attr.illegal_target
  diagnostic. Predeclaring its name does not make an invalid declaration valid.
- Duplicate variant names remain subject to existing generated-class collision
  behavior; this change must not silently overwrite a global.

## 3. Goals and non-goals

### Goals

1. Make every @variant name resolve as a global class from enclosing member
   bodies in the same compilation unit.
2. Preserve existing global/implicit-self resolution precedence.
3. Preserve generated sibling compilation order and superclass initialization
   requirements.
4. Prove behavior through a language-level regression fixture, not only a
   compiler helper test.
5. Keep the declaration inventory extensible for future compile-time derives
   that generate top-level declarations.
6. Align compiler comments and semantic documentation with shipped behavior.

### Non-goals

- No new nested-class or namespaced-variant syntax.
- No change to @data, @sealed, @variant field, constructor, equality,
  toString, or visitor generation.
- No change to @class placement or metaclass lookup.
- No change to :: Family reference semantics.
- No change to runtime global initialization order.
- No new whole-program or cross-module sealed-world analysis.
- No broad rewrite of the compiler into a separate AST-expansion pass.
- No removal or weakening of implicit-self fallback.

## 4. Functional requirements

| ID | Requirement | Priority | Acceptance evidence |
|---|---|---:|---|
| FR-1 | Pre-scan direct Statement::Class names as before. | P0 | Existing class-forward-reference fixtures remain green. |
| FR-2 | For each direct class declaration, pre-scan every ClassMember::Variant(v) name. | P0 | New class-side variant-reference fixture passes. |
| FR-3 | Resolve an unshadowed variant name in a member body as BareNameResolution::Global. | P0 | Fixture behavior and optional bytecode assertion show GetGlobal, not implicit-self Invoke. |
| FR-4 | Keep local, upvalue, linked, and import precedence unchanged. | P0 | Existing name-resolution tests plus a shadowing regression if implementation adds one. |
| FR-5 | Keep generated sibling classes on the existing recursive compile path. | P0 | Existing visitor fixture and sealed-subclass behavior remain green. |
| FR-6 | Preserve existing invalid-variant and collision diagnostics. | P1 | Existing negative fixtures remain green; collision behavior is checked if a new fixture is added. |
| FR-7 | Document that generated variant names enter the known-global inventory before member lowering. | P1 | Updated semantic/compiler documentation reviewed for consistency. |

## 5. Technical design

### 5.1 Declaration-inventory helper

Update predeclare_known_globals() in
phalcom-core/src/compiler/lib/mod.rs.

Preferred shape:

~~~rust
fn collect_class_global_names(class: &ClassDef, out: &mut Vec<String>) {
    out.push(class.name.clone());
    for member in &class.members {
        if let ClassMember::Variant(variant) = member {
            out.push(variant.name.clone());
        }
    }
}
~~~

Use this helper from the existing Statement::Class arm:

~~~rust
match statement {
    Statement::Class(class) => collect_class_global_names(class, &mut names),
    Statement::Let(binding) => collect_pattern(&binding.pattern, &mut names),
    _ => {}
}
~~~

The implementation may use an equivalent iterator-based form, but it must keep
declaration inventory logic in one place and make the generated-global reason
explicit in comments. The helper must remain limited to names that are
semantically top-level globals. Variant labels, generated visitor keyword
parameters, fields, methods, getters, setters, and implementation hooks must
not enter this set.

### 5.2 Imports and AST dependencies

Import ClassDef and ClassMember from phalcom_ast::ast if the helper uses short
names. Avoid duplicating VariantDef name extraction logic in attributes.rs:
predeclare_known_globals() is responsible for compile-unit name visibility,
while expand_variants() remains responsible for generated class structure.

### 5.3 No changes to resolve_bare_name()

Do not add a variant-specific branch to resolve_bare_name(). After the inventory
change, the existing resolves_known_global() branch will handle variants
uniformly with explicit classes, lets, imports, and other known globals.

This keeps name resolution declarative and prevents future generated declaration
kinds from acquiring incompatible fallback behavior.

### 5.4 No changes to expansion order

Do not move expand_variants() before class compilation. The current order is
load-bearing because a generated sibling extends the enclosing class and must
resolve the enclosing class after its DefineGlobal has executed. The fix is
compile-time declaration visibility only; it must not alter bytecode execution
order.

### 5.5 Future extension seam

The helper should be named and documented as a declaration-inventory operation,
not as a one-off @variant workaround. If a future compile-time attribute
generates another top-level declaration, it must either:

1. expose that name to the same pre-scan inventory, or
2. participate in a future explicit declaration-expansion pass that runs before
   name predeclaration.

This specification does not introduce that broader pass. It records the
invariant so future derives do not repeat the defect.

## 6. Semantic lowering trace

For the failing source form:

~~~phalcom
@class less { Less.new() }
~~~

the corrected path is:

~~~text
VariantDef("Less")
    ↓ declaration inventory
known_globals contains Symbol("Less")
    ↓ compile class-side body
Expr::Var("Less")
    ↓ resolve_bare_name
BareNameResolution::Global
    ↓ expression lowering
GetGlobal(Symbol("Less"))
    ↓ ordinary member send
Less.new()
~~~

The incorrect path was:

~~~text
VariantDef("Less") not in known_globals
    ↓ resolve_bare_name inside a member with self
BareNameResolution::ImplicitSelf
    ↓ expression lowering
Invoke(0, Symbol("Less")) on self == Ordering
~~~

The corrected path is behaviorally identical to the explicit top-level class
form. No new opcode, runtime table, selector, heap representation, or
allocation is required.

## 7. Test plan

### 7.1 New end-to-end positive fixture

Add:

phalcom-core/tests/lang/errors/annotation_variant_global_reference.ph

Suggested source:

~~~phalcom
// area: errors
// spec: annotations-data.md @variant / this implementation specification
// status: PASS

@data @sealed
class Ordering {
  @variant Less()
  @variant Greater()

  @class less { Less.new() }
  @class greater { Greater.new() }
}

System.print(Less)
System.print(Greater)
System.print(Ordering.less is Less)
System.print(Ordering.greater is Greater)
~~~

Add the exact sidecar:

phalcom-core/tests/lang/errors/annotation_variant_global_reference.expected

with:

~~~text
Less
Greater
true
true
~~~

This fixture proves both a generated sibling's global identity and the
class-side-body resolution path. It also proves that all variant names are
predeclared together rather than becoming visible only after each generated
sibling is compiled.

### 7.2 Existing regression lanes

The implementation must leave these existing cases green:

- errors/annotation_variant_visitor_exhaustive.ph: generated sibling shape,
  @data behavior, and visitor dispatch.
- compile-errors/annotation_variant_requires_sealed.ph: invalid variant
  declarations remain rejected.
- decorators/decorators_sealed_same_unit_subclass_allowed.ph: same-unit sealed
  subclass behavior remains unchanged.
- compile-errors/decorators_sealed_cross_unit_needs_isolation.ph: cross-unit
  sealed restriction remains unchanged.
- Existing class-side and class-name fixtures under classes/ and metaclass/:
  ordinary class-object lookup remains unchanged.

### 7.3 Resolution-precedence coverage

No production code should change precedence, but implementation review must
verify these cases conceptually or with focused fixtures:

| Source occurrence | Required resolution |
|---|---|
| Less in an unshadowed member body | generated global class |
| local binding named Less | local binding |
| captured binding named Less | upvalue |
| linked/import binding named Less | linked/import binding |
| self.Less | send to current receiver |
| Less at module top level | global lookup |

If a shadowing fixture is added, use a distinct local value and assert that the
local still wins; do not change the language to make globals dominate locals.

### 7.4 Collision and negative coverage

The existing implementation path should be exercised for generated-class
collisions if the fixture harness can pin the diagnostic without unstable
source-span text. At minimum, manually verify that these programs do not
silently overwrite a binding:

~~~phalcom
@sealed class Ordering { @variant Less() }
class Less {}
~~~

~~~phalcom
import "./less" as Less
@sealed class Ordering { @variant Less() }
~~~

The expected outcome is an existing class/import collision diagnostic, not a
new variant-specific error. If exact fixture registration is added, update the
language corpus manifest counts.

## 8. Documentation updates required during implementation

The code change and fixtures must be accompanied by these focused edits:

1. Update the predeclare_known_globals() comment in
   phalcom-core/src/compiler/lib/mod.rs to state that generated variant
   sibling names are included in the declaration inventory.
2. Add one paragraph to
   docs/spec/design/experimental/annotations-data.md after the global-name
   paragraph: variant names are predeclared for member-body compilation, while
   runtime sibling definition remains after the enclosing class definition.
3. Clarify the known-global sentence in
   docs/spec/current/object-model.md to include generated compile-time global
   declarations, especially @variant siblings.
4. Add the new fixture and sidecar to the existing errors corpus. Update
   phalcom-core/tests/lang/MANIFEST.md only if its maintained case counts
   require the new PASS row to be reflected.

Do not mark the experimental annotation document as fully ratified as part of
this bug fix. This work aligns shipped behavior with its existing global-name
semantic claim; it does not ratify unrelated annotation design questions.

## 9. Alternatives considered

### A. Expand all generated declarations before predeclaration

This would make generated Statement::Class nodes visible to the existing
pre-scan automatically. It is a possible future architecture, but not the
recommended fix: expansion currently needs compiler/VM context, and generated
siblings deliberately compile after their parent has been defined. Moving all
expansion earlier would create ordering, borrow, source-span, and initialization
risks for no benefit in this defect.

### B. Change unresolved bare names to prefer globals at lowering time

Rejected. It would alter the language rule for every unresolved name in a
member body, changing implicit-self sends into global reads and potentially
breaking dynamic dispatch, DNU behavior, and existing programs. The defect is
an incomplete declaration inventory, not an incorrect fallback rule.

### C. Add a variant-specific expression-lowering exception

Rejected. It would require expr.rs to know about @variant declarations and would
create a second name-resolution mechanism separate from locals, imports,
globals, and implicit self. Generated declarations must enter the existing
inventory instead.

## 10. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Variant name is predeclared but runtime initialization is still later | A method invoked during an unusual class-initialization side effect could read an uninitialized global | Preserve sibling compile/execute order; do not claim predeclaration initializes values; add no eager runtime lookup |
| Helper accidentally adds fields or method names to globals | Name-resolution behavior changes outside this bug | Collect only ClassDef.name and direct VariantDef.name |
| Local shadowing behavior changes | Existing programs may resolve a different binding | Leave resolve_bare_name() order untouched; verify shadowing matrix |
| Generated name collision bypasses class checks | Silent global overwrite or inconsistent class table | Keep generated siblings as Statement::Class and route through compile_class_impl() |
| Future derive repeats the same bug | Ongoing compiler drift | Document declaration-inventory invariant and future expansion seam |
| Unrelated dirty worktree changes contaminate validation | False attribution of failures | Preserve current modified/untracked paths; inspect diff scope before staging or committing |

## 11. Implementation sequence

1. Add the class declaration-inventory helper and include variant names in
   predeclare_known_globals().
2. Add or update compiler comments explaining compile-time visibility versus
   runtime sibling definition order.
3. Add the positive language fixture and expected output.
4. Update the two semantic documentation locations and corpus bookkeeping if
   required.
5. Run formatting and whitespace checks.
6. Run the focused errors language lane, then the full lang target and relevant
   compiler integration targets.
7. Inspect the final diff to confirm only the named implementation/spec/test
   files changed; preserve unrelated dirty files.
8. Refresh Graphify after code changes with graphify update ..

## 12. Verification and acceptance gates

### Required commands

~~~text
cargo fmt --all -- --check
git diff --check
cargo test -p phalcom-core --test lang errors -- --nocapture
cargo test -p phalcom-core --test lang -- --nocapture
cargo test -p phalcom-core --test integration -- --nocapture
graphify update .
~~~

If integration is not a registered target in the current checkout, use the
repository's registered equivalent and record the substitution rather than
silently skipping that validation.

### Acceptance criteria

- [ ] New variant-global fixture prints Less, Greater, true, true.
- [ ] Less and Greater inside class-side bodies compile as global reads.
- [ ] Existing explicit top-level class behavior remains unchanged.
- [ ] Existing @variant visitor and @data fixtures remain green.
- [ ] Existing sealed and invalid-variant diagnostics remain green.
- [ ] Local/upvalue/import/linked precedence remains unchanged.
- [ ] Generated class collisions do not silently overwrite existing bindings.
- [ ] Runtime sibling definition order remains parent first, siblings next.
- [ ] No new opcode, runtime representation, selector, or allocation path is introduced.
- [ ] Documentation states both global identity and predeclaration timing.
- [ ] cargo fmt --all -- --check passes.
- [ ] git diff --check passes.
- [ ] Final diff excludes unrelated pre-existing worktree changes.

## 13. Definition of done

The bug is fixed when an unshadowed Less in an enclosing class member is
compiled through the existing global resolution path and the end-to-end fixture
observes a Less instance. The semantic work is complete when the spec and
source documentation agree that @variant creates an ordinary global sibling
whose name is known during member-body compilation, while runtime definition
order and all existing name-resolution precedence remain unchanged.
