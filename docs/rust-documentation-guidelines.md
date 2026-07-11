# Rust Documentation Guidelines (mandatory)

**Status: Standing rule — the default way code is written in this repo (ratified 2026-07-11).**

All Rust code in this workspace MUST ship with proper, professional documentation following
Rust's official conventions ([rustdoc book](https://doc.rust-lang.org/rustdoc/),
[RFC 1574 — API documentation conventions](https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html),
[Rust API Guidelines §Documentation](https://rust-lang.github.io/api-guidelines/documentation.html)).
This is not optional polish; it is part of "done." A diff that adds or changes a public item
without docs is incomplete and a reviewer must block it.

## Scope — document every level

1. **Crate level** — every crate's `lib.rs` / `main.rs` opens with a `//!` inner doc comment:
   what the crate is, its role in the workspace, and an entry-point orientation. (See the
   crate table in `CLAUDE.md` for each crate's role — the crate doc should say the same in prose.)
2. **Module level** — every module (`mod foo` / `foo.rs`) opens with a `//!` inner doc comment
   stating the module's responsibility and how it fits the surrounding subsystem.
3. **Item / member level** — every **public** item carries a `///` outer doc comment:
   `pub` functions, methods, structs, enums, traits, trait methods, type aliases, constants,
   statics, macros, and **every public field** and **every enum variant**.
   - Private items that are non-obvious (invariants, subtle algorithms, `unsafe` internals)
     SHOULD also be documented. Trivial private helpers may be left undocumented, but a
     reviewer may still ask for a one-line `//` where intent isn't obvious.

## Content conventions (rustdoc / RFC 1574)

- **Summary line first.** The first line is a single, complete sentence ending in a period —
  it becomes the item's one-line summary in generated docs and in IDE hovers. Start method/
  function summaries with a third-person verb: "Returns…", "Computes…", "Registers…",
  "Allocates…". Do not start with "This function…".
- **Then a blank line and detail.** Body paragraphs explain behavior, rationale, and how the
  item relates to the object model / VM design. Prefer explaining *why* and *invariants* over
  restating the signature.
- **Standard sections**, in this order, whenever they apply:
  - `# Examples` — a runnable ` ```rust ` (or ` ```ignore `/` ```no_run ` when it can't run)
    doctest for non-trivial public API. Doctests are tests: they run under `cargo test`.
  - `# Errors` — for every `fn` returning `Result`, describe each condition that yields `Err`.
  - `# Panics` — for every `fn` that can panic, state exactly when. If it never panics on
    valid input, prefer returning `Result` instead of documenting a panic.
  - `# Safety` — **required** on every `unsafe fn`: the invariants the caller must uphold.
    Every `unsafe { }` block also gets a `// SAFETY:` comment justifying it.
- **Intra-doc links.** Link related items with `[\`Type\`]`, `[\`Type::method\`]`,
  `[\`module\`]` so the docs are navigable. Don't paste bare type names as prose.
- **Spec grounding.** Where an item realizes a spec rule or an ADR, cite it in the doc
  (e.g. "Implements the parallel-superclass rule — see `docs/spec/object-model.md` §5 /
  ADR-0002."). This keeps code, spec, and decisions cross-referenced.
- **Keep docs true.** When behavior changes, the doc changes in the same diff. A stale doc is
  a bug.

## Enforcement

- Each crate's root declares **`#![warn(missing_docs)]`** (raise to `#![deny(missing_docs)]`
  once a crate is fully documented) so undocumented public items surface as warnings.
- `cargo doc --workspace --no-deps` must build **without warnings**; broken intra-doc links
  fail under `#![deny(rustdoc::broken_intra_doc_links)]`.
- Doctests run as part of `cargo test` (already in `./scripts/verify.sh`); keep them green.
- **Reviewers block on missing/stale docs** the same way they block on a failing test.

## Minimal shape (reference)

```rust
//! Method lookup and dispatch.
//!
//! Resolves a selector to a [`Method`] by walking the class/metaclass hierarchy,
//! implementing the lookup order of `docs/spec/method-lookup.md` §1.

/// A resolved, callable method bound to a selector.
///
/// Wraps either a bytecode [`Closure`] or a native primitive. Constructed by
/// [`Class::define`] and looked up via [`Class::lookup`].
pub struct Method { /* … */ }

impl Method {
    /// Returns the selector this method answers to.
    pub fn selector(&self) -> Symbol { /* … */ }

    /// Invokes the method against `receiver` with `args`.
    ///
    /// # Errors
    /// Returns [`RuntimeError::WrongArity`] if `args.len()` doesn't match the
    /// selector's positional arity.
    ///
    /// # Panics
    /// Never panics on a well-formed [`Chunk`]; a malformed chunk is a compiler bug.
    pub fn invoke(&self, receiver: Value, args: &[Value]) -> PhResult<Value> { /* … */ }
}
```
