# Modules & Imports

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

**Governing ADRs:**
[ADR-0027](../../adr/0027-modules-as-files-with-public-by-default-imports.md)
(file = module; core is the auto-imported module; first-class `Module` object) ·
[ADR-0045](../../adr/0045-module-import-relative-path-whole-module-binding.md)
(Draft 0.1 narrowing: relative file-path resolution + whole-module binding;
`Module#doesNotUnderstand(_)` floor amendment)

`import` gives a Phalcom program a way to name, resolve, load, and bind another
compilation unit as a first-class [`Module`](object-model.md) namespace value —
open-question [Q8](open-questions.md).

## 1. A module is a file

Every `.ph` file is a module. Its top level — every `var`/`let`/`class` it
declares — becomes that module's members (ADR-0027 §1). The core library
(`core.ph`) is the module that happens to be auto-imported into every
compilation unit; a user module differs only in that you name it explicitly.

## 2. Resolution — relative file path (DEC-U15, ADR-0045)

```phalcom
import "./geometry/point" as Point
```

The path string is resolved **relative to the importing file's own
directory** — never the process's current directory, never the program
entry's directory. A `.ph` extension is appended if the string does not
already carry one. The resolved path is canonicalized before use, so two
different spellings of the same file (`"./a"` from one importer, `"../lib/a"`
from another) resolve to the **same** canonical key and therefore the same
`Module` (§4).

There is no logical-name resolver and no search path in Draft 0.1 — Draft 0.1
is "scripts plus a lib directory". `import "./x"` fails with a clean,
attempted-path-naming error if no such file exists; it never panics.

## 3. Binding — whole module (DEC-U15, ADR-0045)

```phalcom
import "path" as Name
```

`as Name` is **mandatory**. `Name` is bound to the imported unit's `Module`
object as an ordinary immutable global in the **importing scope only** — never
a namespace merge into the importer's own globals (no global-namespace
pollution). There is no bare `import "path"` and no selective
`import a, b from "path"` form yet; `from`/`export` are reserved keywords for
that future grammar (ADR-0027's qualified/selective forms), not yet lexed.

## 4. A module is a first-class object

A `Module` is an ordinary heap value (object-model.md §4). Its members are
reached through **ordinary sends**, exactly like any other object — never a
compiler-only qualified-name rewrite:

```phalcom
import "./math" as Math
System.print(Math.pi)          // a zero-arg getter send
System.print(Math.distance(1, 2))  // forwards to the member's own `call`
```

A member send that finds no match on `Module` itself is resolved against the
module's own top-level globals (`Module#doesNotUnderstand(_)`,
`primitive/module.rs`; [ADR-0045](../../adr/0045-module-import-relative-path-whole-module-binding.md)
Part 2). A zero-arg `Getter` selector returns the member value directly; any
other selector shape whose bound value is callable is forwarded to it via the
matching-arity `call(...)` send — "the member, called with these arguments"
falls out of "everything is a message" (README.md Invariant 1) rather than
needing a bespoke static-method dispatch path. A selector matching no member
at all falls through to `Object`'s ordinary `MessageNotUnderstood` raise.

Every top-level name is a member in Draft 0.1 — there is no `export` keyword
and no `_`-prefix privacy enforcement yet (ADR-0027 §2 is not enforced; see
ADR-0045's alternatives).

## 5. Compile-once, memoized by canonical path

A unit is compiled and its top level run **exactly once**, at first import.
`Universe::module_registry` (`HashMap<String, ObjRef>`, canonical path →
`Module`) memoizes the result: a second `import` of the same file — from the
same importer or a different one — returns the **identical** `Module` object.
A class defined inside an imported unit therefore has one identity across
every importer (`isA` stays sound).

## 6. Cyclic imports

A mutual import (`a` imports `b`, `b` imports `a`) terminates. The module is
inserted into the registry the moment it is **allocated**, before it is
compiled or run — so a re-entrant probe of the same canonical path (whether a
genuine second import or a cyclic import re-entering mid-load) always finds it
and never recompiles or loops.

**Partial-init hazard.** A name read across a not-yet-complete cyclic edge —
before the unit that defines it has reached that point in its own top level —
surfaces as the ordinary "member not found" `doesNotUnderstand` miss on the
still-partially-populated `Module`. This is a documented hazard, not a
compiler proof obligation (matching most languages' circular-import handling,
and consistent with ADR-0027's original "circular-import policy is an
implementation detail" stance): order your mutual imports so a value is
defined before the cyclic edge that reads it, or read it lazily (inside a
method body, not at the importing unit's own top level).

## 7. Kernel visibility

`Object`, `Number`, `List`, `Map`, … are visible in every module **without**
`import` — they are the bootstrap (§1's "core is the module you never have to
name"), not a module a unit opts into.

## 8. Evaluation order

`import` is an ordinary statement, evaluated in the position it is written —
not hoisted, not deferred. `Bytecode::Import` resolves/loads/runs the imported
unit's top level at the point the importing unit's own execution reaches the
`import` statement (a re-entrant push-one-frame-and-drain, mirroring
`VM::send_dynamic` — never the outermost program entry's stack-clearing path).
Side effects at import time are the author's responsibility, as in most
languages.

## 9. Not yet in scope (DEFERRED)

- **Compiled-bytecode imports.** `import` resolves to and compiles a `.ph`
  **source** file only. Loading a precompiled bytecode unit needs a bytecode
  verifier Phalcom does not have yet — loading unverified bytecode is a
  security hole. The `VM::import_module` resolve → obtain-a-compiled-chunk →
  instantiate-`Module` seam stays abstract enough for a future verified source
  to slot in behind it.
- **Selective import / `export` / `_`-privacy enforcement** (ADR-0027 §2/§3).
  `from`/`export` are reserved, unlexed keywords; the whole-module AST node
  can grow an optional selective list additively.
- **Logical-name resolution + a search path** (ADR-0027 §4). Relative file
  paths only, in Draft 0.1.
- **Path-traversal / sandboxing policy.** No root confinement is enforced —
  a relative import can walk outside the program's own directory tree via
  `../..`. Not a concern for a single-author script today; flagged for a
  future security ADR before Phalcom runs untrusted source.
