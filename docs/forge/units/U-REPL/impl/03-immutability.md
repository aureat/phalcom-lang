# §03 — Stage 2: immutability across cells

Implements plan.md **§D4**. Two binding sets, and a REPL exemption that relaxes both
checks for prior-unit entries.

**Phase A — lands on `main`.** This is the last stage touching `phalcom-core`.

## 1. Why this is not optional

Wren errors on redefinition (`wrenDefineVariable` returns `-1`,
`resources/wren/src/vm/wren_vm.c:1575`) and its REPL is unusable for it. V8 built a
dedicated REPL mode to make `let`/`const` redeclarable. Erlang needed `f(X)` to forget a
binding. **Rebinding is not optional in a REPL** — and today it works here only by
accident.

## 2. The current mechanism, verified

`Compiler.global_bindings: HashMap<Symbol, bool>` — name → is_mutable
(`compiler/lib/mod.rs:62`, initialized `:126`). Two checks read it:

- **const assignment** — `compiler/lib/expr.rs:303`:
  `if self.global_bindings.get(&name_sym) == Some(&false)` → error.
- **same-scope redeclaration** — `compiler/lib/scope.rs`, documented at `:118`
  (locals) and `:170` (globals): "Rejects a same-scope redeclaration exactly as
  `add_local` does for locals". Class declarations are exempt (decision 0066 registers
  them in the same map).

A `Compiler` lives for exactly one `compile_closure` call. So cross-cell rebinding
already works — **by lifetime accident, untested and unstated** (precondition 3). §D4
makes it deliberate.

## 3. The design

```
Compiler.global_bindings        // bindings from THIS unit
                                //   → assignment to a `false` (const) entry errors (expr.rs:303)
                                //   → redeclaring any entry errors                 (scope.rs)

ModuleObject.global_bindings    // bindings from PRIOR units, same shape
                                //   → both checks apply, UNLESS UnitKind::Repl
```

At the end of a successful compile, the unit's `global_bindings` are **merged into** the
module's. The next cell's fresh `Compiler` starts empty and consults the module's set for
prior-unit names.

### 3.1 The exemption must relax **both** checks

Stated in plan.md §D4 and easy to half-implement. Lifting only the const check leaves
`const x = 1` in cell 1 followed by `const x = 2` in cell 2 failing the **redeclaration**
ban — which is precisely Wren's behavior this unit exists to avoid. An implementation
that passes a `const`-then-assign test and fails `const`-then-`const` has missed the
point.

### 3.2 The matrix

| case | result |
|---|---|
| file: `const x = 1` … `x = 2` | **error** — ADR-0014 unchanged |
| cell 1 `const x = 1`; cell 2 `x = 2` | **allowed** |
| cell 1 `const x = 1`; cell 2 `const x = 2` | **allowed** |
| both in **one** cell | **error** — a real mistake, still caught |
| future non-REPL multi-unit path | **error** — no accidental pre-authorization |
| `xx = 2`, never declared | runtime error — unchanged (`SetGlobal` has no core fallback) |

`const` becomes a promise about the **binding**, enforced across units — strictly
stronger than today, where it holds only by the precondition-3 accident. The exemption is
confined to a named `UnitKind`, not a global weakening.

## 4. The interaction that makes this dangerous

**One undocumented lifetime now carries three independent rulings:**

1. §D4's two-set immutability (this stage),
2. U-BINDINGS' same-scope redeclaration ban (`scope.rs:118`, `:170`),
3. decision 0066's registration of class declarations in the same `global_bindings` map
   (which is what makes 0065 ruling 6's class shadowing work).

All three depend on `Compiler` being constructed **per cell**. A refactor that makes
`Compiler` session-lived — an obvious-looking optimization, since constructing one per
cell looks wasteful — silently breaks all three at once, and only one of them has a test.

Write §5's cross-cell test as the guard for **all three**, and say so in its doc comment.
This is the single most important test in the unit.

## 5. Tests

| Test | Asserts |
|---|---|
| `const_rebinds_across_repl_cells` | cell 1 `const x = 1`, cell 2 `x = 2` — succeeds under `Repl` |
| `const_redeclares_across_repl_cells` | cell 1 `const x = 1`, cell 2 `const x = 2` — succeeds (§3.1) |
| `const_assignment_errors_in_one_unit` | both statements in one cell — errors |
| `const_assignment_errors_in_file_mode` | same source as the first test, `UnitKind::File` — errors |
| `class_shadows_across_repl_cells` | cell 2's `class Foo` does not trip `class.already_defined` |

The fourth is the one that catches an over-broad exemption: if the relaxation leaks into
`File`, the REPL works and **every `.ph` file silently loses `const`**. Same source text
as the first test, different `UnitKind`, opposite expectation.

The fifth guards ruling 3 of §4 and belongs here rather than in [§02](02-session-and-cells.md)
because it fails through the same map.

## 6. Write-set

| Path | Change |
|---|---|
| `phalcom-core/src/heap/module.rs` | `ModuleObject.global_bindings: HashMap<Symbol, bool>` + merge method |
| `phalcom-core/src/compiler/lib/mod.rs` | consult module's set; merge on success |
| `phalcom-core/src/compiler/lib/expr.rs` | `:303` const check consults both sets, honors `UnitKind` |
| `phalcom-core/src/compiler/lib/scope.rs` | redeclaration ban consults both sets, honors `UnitKind` |
| `phalcom-core/tests/` | the five tests above |

**Conflict risk vs class work — the highest in the unit.** `expr.rs` is in U-CLASSNS's
write-set (`field_layouts` reads at `:258`, `:316`); this stage edits `:303`, between
them. `scope.rs` and `heap/module.rs` are not in either class unit's write-set.
`compiler/lib/mod.rs` is in all three.

Land this on `main` **before** U-CLASSNS if the sequence allows. If it cannot, expect a
real (not trivial) rebase on `expr.rs` and re-run the five tests after resolving — the
region is dense and a mechanical merge can produce something that compiles and checks the
wrong set.

## 7. Gate

Workspace green, 28 suites + five new tests, 0 failures.

Read the diff once more before committing, specifically asking: *does any relaxation
reach a `File` unit?* The fourth test answers it, but the question is worth asking of
the code as well — this is the change where a wrong answer is invisible until someone's
`const` stops holding in a real program.
