# 27. A module is a file; exports are public by default; imports are qualified, selective, or aliased

- Status: Accepted
- Date: 2026-07-12
- Related: `docs/spec/core/decisions.md` Q4 (core = "the core module's exports,
  auto-imported"), `docs/spec/core/forward-compat.md` §3, `docs/spec/open-questions.md` Q8,
  `vm.rs::install_core`, `phalcom-core/core/core.ph`

## Context

The `import` token exists in the lexer but has no semantics. One boundary is
**already ruled** ([decisions.md Q4](../spec/core/decisions.md)): kernel names live in
*the core module, auto-imported into every compilation unit*; there is one flat global
namespace populated at boot, and user-facing `import` was deferred to "the module
unit." forward-compat §3 requires that a surface name be added *to the core module*
(not an ad-hoc string-keyed global table) so a future import system can re-scope or
shadow it without a breaking change.

Open-question Q8 designs that user-facing module system. The design must be the natural
continuation of the "core is a module you import implicitly" framing already committed.

## Decision

**A module is a file; user modules are just core modules you import explicitly.**

1. **File = module.** Each `.ph` file is a module. Its top-level definitions are its
   contents. The core library is the module that happens to be auto-imported into every
   unit — user modules differ only in that you name them explicitly.

2. **Public by default; `_`-prefix is private.** Every top-level name is exported unless
   its name begins with `_` (reusing the field-privacy convention already in the
   language). No `export` keyword — low ceremony, matching the rest of the surface.

3. **Three import forms.**
   ```phalcom
   import geometry                    // qualified:  geometry.Circle
   import Circle, Rect from geometry  // selective:  Circle, Rect into local scope
   import geometry as geo             // aliased:    geo.Circle
   ```
   Qualified access preserves provenance (you can see where `Circle` came from);
   selective import is the terse form; `as` avoids collisions.

4. **Logical-name resolution.** `import geometry` resolves a *logical* module name to a
   file via a resolver, **not** a raw filesystem path baked into source. (The exact
   search-path mechanism is an implementation detail of the module unit.)

5. **Names go into the core module, per forward-compat §3.** A unit that adds a surface
   name adds it to a module, never to an ad-hoc global table keyed by raw string — so
   import/shadow/re-scope stay non-breaking.

## Consequences

- **Real encapsulation without a module calculus.** Files give namespacing and
  privacy; there is no need to invent first-class module objects yet.
- **Consistent with the committed core-as-module model.** decisions.md Q4 said the core
  is an auto-imported module; this makes *every* module the same kind of thing, so the
  implicit core import is just the one module you do not have to name.
- **Deferred, non-foreclosed pieces.** Parameterized / first-class module objects
  (Newspeak/ML functors) are **not** adopted now, but a first-class module object can
  later subsume file-modules without breaking this design. Circular-import policy (hard
  error vs lazy binding) is an **implementation detail of the module unit**, not a
  language-level decision here.
- **`import` stops being decorative.** The existing token gets meaning; the flat boot
  namespace becomes "the core module, auto-imported," a special case of the general rule
  rather than a separate mechanism.
- **A privacy convention becomes load-bearing.** `_name` now means "module-private" at
  the top level in addition to "instance-private" on fields — one rule, two scopes.

## Alternatives considered

- **Flat global image (Smalltalk).** Matches today's one-namespace reality with zero new
  machinery, but gives no encapsulation, guarantees name collisions at scale, and leaves
  `import` meaningless. Rejected: it is a non-decision that does not scale past a
  single-author image.
- **First-class / parameterized module objects (Newspeak, ML functors).** Maximal power
  (modules as values, parameterizable, testable in isolation) and consistent with
  "everything is an object," but a large design and runtime surface the language does not
  need before it has a working stdlib. Deferred, not foreclosed.
- **Explicit `export` (only marked names escape).** Clearer API surface but more
  boilerplate; rejected in favor of public-by-default + `_`-private, which matches the
  language's low-ceremony feel.
- **Filesystem-path imports (`import "./geometry.ph"`).** Simple to implement but couples
  source to on-disk layout and breaks under reorganization; rejected for logical-name
  resolution.
