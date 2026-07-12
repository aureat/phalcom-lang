# Modules

A `.ph` file is a module — no `package`, no manifest, no build-system
ceremony. What decides what a file exports is a rule you already know from
fields.

## A file is a module

Every top-level definition in a file — a class, a top-level binding, anything
written outside a class body — is a member of that file's module:

```phalcom
// geometry.ph
class Circle {
  construct new(radius:) { _radius = radius }
  area => 3.14159 * _radius * _radius
}

class _Helper { ... }   // leading underscore — module-private
```

**Public by default; `_`-prefix is private.** A top-level name is exported
unless it starts with `_` — the same convention that already makes `_radius`
a private field, reused one scope up. `Circle` is importable from elsewhere;
`_Helper` is not. There is no `export` keyword and no separate export list —
see [ADR-0027](../adr/0027-modules-as-files-with-public-by-default-imports.md)
for why (mainly: it's one privacy rule instead of two, and it matches the
rest of the surface's low ceremony).

The core library isn't a special case — it's the one module that happens to
be auto-imported into every compilation unit, which is why `Int`, `String`,
`Option`, and the rest are in scope with no `import` at all. A module you
write differs only in that you have to name it.

## Three import forms

```phalcom
import geometry                     // qualified — reach in with geometry.Circle
import Circle, Rect from geometry   // selective — Circle, Rect land directly in scope
import geometry as geo              // aliased — reach in with geo.Circle
```

| Form | Syntax | What lands in scope |
|------|--------|----------------------|
| Qualified | `import geometry` | `geometry` itself |
| Selective | `import Circle, Rect from geometry` | `Circle` and `Rect`, unqualified |
| Aliased | `import geometry as geo` | `geo` — qualified, under a different name |

Qualified keeps provenance visible: reading `geometry.Circle` tells you where
`Circle` came from without scrolling to the top of the file. Selective is the
terse form for a name you'll use constantly and don't need to trace back.
`as` is for the case those two don't cover — two imports that would
otherwise collide on the same name.

## Name resolution

`import geometry` resolves `geometry` as a **logical module name**, not a
filesystem path — there is no `import "./geometry.ph"` form in the grammar.
A resolver maps the name to a file; the exact search-path mechanism is left
unspecified on purpose, so source code doesn't hard-code a directory layout
that reorganizing the project would break.

A selectively-imported name binds directly to the thing it names — after
`import Circle from geometry`, `Circle` in your file is `geometry`'s
`Circle`, the same object, not a copy. `as` renames the module reference
itself; there's no per-name form (`import Circle as C from geometry` isn't in
the grammar) — if you need to rename one imported member, import the module
qualified or aliased and reach through it.

## What's still open

A few pieces are deliberately undecided. Don't build on them yet:

- **Circular imports.** Hard error vs. lazy binding is called out as an
  implementation detail of "the module unit," not a language-level ruling.
- **First-class module objects.** Modules are files, not values — you can't
  pass one around or parameterize it. A first-class module object
  (Newspeak/ML-functor style) is deferred, not foreclosed, so this can grow
  later without breaking today's `import`.
- **Re-export.** Whether a name you import into your module is itself
  visible to whoever imports *your* module is unspecified. Treat it as an
  open question, not an assumption either way.

The full decision, including the rejected alternatives (a flat global image,
first-class modules now, explicit `export`, filesystem-path imports), is
[ADR-0027](../adr/0027-modules-as-files-with-public-by-default-imports.md).

---

Next: [Concurrency](concurrency.md) — `Fiber` as the one cooperative
primitive, `Future`, and the scheduler.
