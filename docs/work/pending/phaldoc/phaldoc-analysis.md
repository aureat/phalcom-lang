# Phaldoc: proposed documentation format

Phaldoc should be a **source documentation language layered over comments**, not a runtime feature and not another attribute system. Its job is to express human intent, examples, explanations, and semantic cross-references while allowing the compiler, LSP, `phalcom doc`, and native primitive manifest to merge in machine-readable facts—types, effects, raises contracts, stability, deprecation, invariants, provenance—without requiring authors to repeat them.

The repository already has an experimental Phaldoc proposal built around `///`, `//!`, Markdown, selector-keyed attachment, and documentation tags.  I would retain those foundations, but substantially tighten the format in light of the native metadata and attribute work we have just designed. In particular, `@since`, `@deprecated`, type information, effects, contracts, and other machine facts should **not** be Phaldoc prose when an authoritative metadata representation exists. This agrees with the more recent repository design principle that documentation is free comment trivia while executable/passive attributes remain the authoritative source for machine facts. 

The resulting philosophy is:

```text
Declaration       = what exists
Types             = what values it accepts/produces
Attributes        = machine-checkable semantics
Phaldoc           = what humans need to understand it
```

Or more tersely:

> **Phaldoc explains. Metadata specifies.**

---

## 1. Canonical syntax

Phaldoc has two source markers:

```phalcom
/// Outer documentation.
/// Documents the declaration immediately below.

//! Inner documentation.
//! Documents the containing module/package/class/protocol.
```

`///` is overwhelmingly the normal form.

```phalcom
/// Returns whether this collection contains `value`.
///
/// Equality is determined using [`Object#==(_)`].
contains(value) {
    ...
}
```

Class documentation:

```phalcom
/// An immutable UTF-8 string.
///
/// `String` values compare by textual content and are safe as map keys.
class String {
    ...
}
```

Module documentation:

```phalcom
//! Geometry primitives and transformations.
//!
//! This module contains the fundamental point, vector, and matrix abstractions.

class Point {
    ...
}
```

Class-inner documentation is also legal:

```phalcom
class Matrix {
    //! A rectangular immutable numeric matrix.
    //!
    //! Matrix operations preserve dimensional invariants.

    ...
}
```

But outer documentation above the declaration should be preferred where possible:

```phalcom
/// A rectangular immutable numeric matrix.
class Matrix {
    ...
}
```

### Why not docstrings?

Do not make this:

```phalcom
class String {
    "An immutable UTF-8 string."
}
```

special.

That creates runtime/parser semantics around what should remain compiler-free documentation and competes with ordinary string expressions.

### Why not `@doc(...)`?

Likewise, don't make documentation an attribute:

```phalcom
@doc("An immutable UTF-8 string.")
class String
```

Phaldoc should cost nothing in stripped builds. The repository has already reached essentially this same boundary: documentation comments should remain trivia rather than retained heap objects. 

---

# 2. No `/** ... */` in Phaldoc v1

I would remove the tentative block form from the current proposal.

Canonical Phaldoc v1 supports only:

```text
///
///!
```

Not:

```text
/**
 *
 */
```

The repository's existing draft already identifies a concrete reason: although block comments appear in lexical documentation, the current lexer does not actually treat `/* ... */` as inert trivia. 

Even after block comments eventually work, there is little need for a second canonical documentation notation.

One standard form gives better formatting, editing, extraction, and tooling consistency.

---

# 3. Phaldoc block formation

Consecutive documentation-comment lines form one block.

```phalcom
/// First paragraph.
///
/// Second paragraph.
///
/// Third paragraph.
foo
```

After removing the marker, the logical content is:

```markdown
First paragraph.

Second paragraph.

Third paragraph.
```

One optional ASCII space after the marker is removed:

```phalcom
/// Text
```

becomes:

```text
Text
```

while:

```phalcom
///    indented
```

preserves three spaces after the conventional one-space prefix removal.

A blank documentation line is written:

```phalcom
///
```

and represents a Markdown blank line.

A raw blank source line ends attachment:

```phalcom
/// This is dangling documentation.

foo
```

That should produce:

```text
phaldoc.dangling
```

from the LSP or `phalcom doc --check`.

---

# 4. Attributes do not break attachment

This must work:

```phalcom
/// An immutable sequence of bytes.
@sealed
class Bytes {
    ...
}
```

and:

```phalcom
/// Requires `amount` to be positive.
@requires(amount > 0)
deposit(amount) {
    ...
}
```

The semantic attachment is:

```text
Phaldoc
   ↓
attribute*
   ↓
declaration
```

not lexical "next line."

Therefore the formal attachment sequence is:

```text
doc-comment
    trivia-without-blank-line
    attribute*
    declaration
```

A comment between the Phaldoc and declaration should break attachment:

```phalcom
/// Something.

// implementation note
class Something
```

because the plain comment visually interrupts the documentation association.

---

# 5. Markdown dialect

Phaldoc should define a small, portable dialect:

> **CommonMark + fenced code blocks + pipe tables + Phaldoc semantic links + Phaldoc directives.**

Do not adopt every GitHub-specific extension.

The following are sufficient:

```text
paragraphs
emphasis
strong emphasis
inline code
ordered/unordered lists
blockquotes
headings
links
fenced code
tables
thematic breaks
```

Raw HTML should not be part of Phaldoc v1.

That keeps rendering deterministic across:

```text
terminal
HTML
IDE hover
editor preview
JSON/doc indexes
future offline documentation
```

Raw HTML supplied by users should be escaped as text rather than interpreted.

---

# 6. The first paragraph is always the summary

This is a strong convention worth enforcing.

```phalcom
/// Searches this collection for the first matching element.
///
/// Evaluation proceeds from left to right and stops after the first match.
///
/// ## Complexity
///
/// Linear in the number of elements inspected.
find(predicate) {
    ...
}
```

Normalized:

```text
summary:
    Searches this collection for the first matching element.

details:
    Evaluation proceeds from left to right...

    ## Complexity
    ...
```

Generated API lists and LSP completion can show only the summary.

Hover can show summary + relevant structured information.

Full documentation renders everything.

No explicit:

```text
@summary
```

is necessary.

If the document begins with a heading, directive, list, or code block, `phalcom doc --check` should emit `phaldoc.missing_summary`.

---

# 7. Keep the directive vocabulary deliberately small

This is where I would diverge most strongly from the old proposal.

Phaldoc does **not** need tags for every piece of documentation metadata because Phalcom increasingly knows those facts structurally.

The v1 directive vocabulary should be only:

| Directive | Purpose |
|---|---|
| `@param` | Human explanation of a parameter |
| `@returns` | Human explanation of the returned value |
| `@raises` | Human explanation of a particular language-level error condition |
| `@see` | Semantic/external cross-reference |

Reserve:

```text
@typeparam
```

for when generic declaration syntax actually lands.

Do **not** add:

```text
@since
@deprecated
@stability
@effect
@pure
@internal
@author
@type
@returntype
```

Those are either machine metadata, ordinary prose, or project-level information.

And do not add:

```text
@requires
@ensures
@invariant
```

because those names already correspond to actual executable Phalcom attributes. The existing experimental design correctly recognizes that executable contracts should be harvested directly rather than copied into documentation. 

---

# 8. Directive syntax

Canonical syntax:

```phalcom
/// Finds an element matching `predicate`.
///
/// @param predicate — Determines whether an element matches.
/// @returns — The first matching element, or `None` if nothing matches.
/// @raises ConcurrentMutationError — If the collection is structurally modified during traversal.
find(predicate) {
    ...
}
```

The em dash is canonical but not syntactically essential.

These are equivalent:

```text
@param value — Value to insert.
@param value - Value to insert.
@param value: Value to insert.
```

The formatter should canonicalize all of them to:

```text
@param value — Value to insert.
```

For multiline content:

```phalcom
/// @param predicate — Determines whether an element matches. The predicate
///     is evaluated at most once for each visited element and evaluation
///     stops immediately after a match.
```

Continuation lines must be indented relative to the directive.

Alternatively, a directive may have an empty first line:

```phalcom
/// @returns
///     `Some(value)` for the first match, otherwise `None`.
```

The formatter may choose the one-line form for short content and continuation form for longer content.

---

# 9. `@param` references the source parameter binder

This is one place I would change the old experimental proposal.

The old draft associates `@param` primarily with selector label positions.  That is too selector-centric for human documentation.

Consider:

```phalcom
replace(value, using: strategy)
```

There are two separate concepts:

```text
selector:
    replace(_,using)

parameter binders:
    value
    strategy
```

Phaldoc should say:

```phalcom
/// @param value — The value being replaced.
/// @param strategy — Determines how replacement is performed.
replace(value, using: strategy)
```

not:

```phalcom
/// @param using — ...
```

The LSP/compiler already knows that `strategy` occupies the `using` selector lane.

The generated signature can render:

```text
using strategy: ReplacementStrategy
```

without requiring documentation prose to repeat the label.

This also means refactoring:

```text
strategy → policy
```

can automatically update:

```text
@param strategy
```

because it is a semantic parameter reference, not arbitrary text.

### Anonymous parameters

If Phalcom permits genuinely anonymous argument binders, they simply cannot have `@param` prose.

That is reasonable: there is no user-facing parameter identity to document.

---

# 10. Types never appear in `@param`

Wrong:

```phalcom
/// @param value: String — The value to append.
```

if the method declaration already says:

```phalcom
append(value: String)
```

or native metadata says:

```rust
params = [String]
```

Correct:

```phalcom
/// @param value — The value to append.
```

Generated docs combine:

```text
machine fact:
    value: String

Phaldoc:
    The value to append.
```

into:

```text
value: String
    The value to append.
```

This avoids type documentation becoming stale.

---

# 11. `@returns` describes semantics, never type

Likewise:

```phalcom
/// @returns — `Option<Method>` containing the method.
```

is redundant when the type is already:

```text
Option<Method>
```

Prefer:

```phalcom
/// @returns — The exact accessible method, or `None` when lookup fails.
```

Native declaration:

```rust
#[phalcom::primitive(
    Object,
    "methodFor(_)",
    params = [Symbol],
    returns = Option<Method>,
    types = "(Symbol) -> Option<Method>",
)]
#[phalcom::phaldoc(r#"
Returns the method selected by `selector`.

The lookup is observational and does not invoke
[`Object#doesNotUnderstand(_:)`].

@param selector — The exact selector to probe.
@returns — The accessible method, or `None` when lookup fails.
"#)]
```

Generated output:

```text
Object.methodFor(_)

(Symbol) -> Option<Method>

Returns the method selected by selector.

Parameters
    selector: Symbol
        The exact selector to probe.

Returns
    Option<Method>
        The accessible method, or None when lookup fails.
```

No fact is stated twice.

---

# 12. `@raises` supplements the machine-readable `raises` set

For native primitives:

```rust
raises = [
    InvalidSelectorError,
    AccessError,
]
```

Phaldoc can provide the conditions:

```text
@raises InvalidSelectorError — If `selector` is structurally invalid.
@raises AccessError — If the requested method is inaccessible to the caller.
```

The generated representation combines them.

A documented error omitted from machine metadata is a mismatch:

```text
phaldoc.raises_not_declared

Object#foo(_)

documentation mentions:
    IndexError

native contract declares:
    [TypeError]
```

Conversely, declared errors do not require descriptions.

This is valid:

```rust
raises = [IndexError]
```

with no `@raises`.

Generated docs still show:

```text
Raises
    IndexError
```

just without explanatory prose.

For ordinary Phalcom code where no static `raises` contract exists yet, `@raises` is advisory documentation. Once a language-level error contract exists, the same cross-check becomes available automatically.

Executable contract failures are special:

```phalcom
@requires(index >= 0)
at(index) { ... }
```

Phaldoc must **not** require:

```text
@raises PreconditionError
```

The doc generator derives that from the real contract.

---

# 13. `@see`

`@see` is the main structured navigation primitive.

```phalcom
/// @see [`String#split(_)]
/// @see [`Collection#map(_)]
```

Multiple references can appear:

```phalcom
/// @see [`String#split(_)`], [`String#lines`]
```

An external URL is also legal:

```phalcom
/// @see [Unicode Standard](https://unicode.org/...)
```

The important case is semantic Phalcom references.

---

# 14. Semantic links should be a first-class Phaldoc extension

This is one of the most valuable features to design properly now.

Use Rustdoc-like Markdown syntax:

```text
[`String`]
[`universe.BoundMethodFamily`]
[`Object#methodFor(_)]
[`Number::new(_)]
[`#methodFor(_)]
[`::new(_)`]
```

I propose these meanings:

| Syntax | Meaning |
|---|---|
| ``[`String`]`` | type/value/declaration resolved lexically |
| ``[`universe.BoundMethodFamily`]`` | explicit universe binding |
| ``[`Object#foo(_)]`` | instance-side method |
| ``[`Object::foo(_)]`` | class-side method |
| ``[`#foo(_)]`` | instance member on current documented owner |
| ``[`::foo(_)]`` | class-side member on current documented owner |

The `#` versus `::` distinction is especially useful.

It maps directly to the primitive identity model:

```text
owner
side
selector
```

For example:

```text
[`Object#==(_)`]
```

resolves to:

```text
PrimitiveKey {
    owner: Object,
    side: Instance,
    selector: "==(_)"
}
```

and:

```text
[`Number::new(_)`]
```

to class side.

This is substantially better than a bare:

```text
#foo
```

because Phalcom selectors encode dispatch identity.

---

# 15. Bare method references should use canonical selector identity

Never make:

```text
foo
```

a sufficient method documentation identity when the context requires disambiguation.

Phalcom has distinct selectors such as:

```text
foo
foo()
foo(_)
foo(_,_)
foo(_,using)
```

Therefore documentation should internally key methods exactly as the runtime does:

```text
(owner, side, canonical selector)
```

The existing draft gets this principle right. 

The source author normally doesn't have to state it because adjacency resolves the declaration.

Explicit links do.

---

# 16. No detached-doc `selector:` escape hatch

The experimental draft proposed something like:

```text
/// selector: move(_,to,duration)
```

for detached docs. 

I would remove this.

Documentation should live next to the declaration it documents.

Detached documentation creates several problems:

```text
separate lookup rules
duplicate identity syntax
refactoring hazards
ambiguous ownership
documentation appearing far from implementation
```

If large conceptual material needs to live elsewhere, that belongs in a normal `.md` documentation page and should link to the API symbol.

Phaldoc itself remains declaration-attached.

---

# 17. Ordinary Markdown headings handle most sections

Do not invent:

```text
@example
@note
@warning
@remarks
@details
```

Use Markdown.

Example:

```phalcom
/// Parses an integer using the supplied radix.
///
/// Leading and trailing whitespace are not accepted.
///
/// ## Examples
///
/// ```phalcom
/// Int.parse("ff", radix: 16)
/// // Some(255)
/// ```
///
/// ## Complexity
///
/// Linear in the number of input digits.
///
/// ## Notes
///
/// The accepted alphabet is case-insensitive.
parse(text, radix:) {
    ...
}
```

This is more readable in raw source than a large directive vocabulary.

It also means headings remain extensible without changing the Phaldoc grammar.

A library can use:

```text
## Thread safety
## Performance
## Encoding
## Security
## Implementation notes
```

without registering a tag.

---

# 18. Code fences and doctests

Code blocks with:

```text
```phalcom
```

are Phalcom examples.

Initially they are just syntax-highlighted.

But design their info-string namespace now:

```text
phalcom
phalcom no_run
phalcom compile_fail
phalcom ignore
```

Semantics:

| Fence | Meaning |
|---|---|
| `phalcom` | Compile and execute as a doctest |
| `phalcom no_run` | Must compile, not executed |
| `phalcom compile_fail` | Must fail compilation |
| `phalcom ignore` | Display only |
| other language | Display only |

The doctest machinery can land later without changing source documentation.

For example:

```phalcom
/// ## Examples
///
/// ```phalcom
/// const method = object.methodFor(#toString)
/// assert(method.isSome)
/// ```
```

This should eventually become an executable documentation test.

---

# 19. Doctests should run in declaration scope

When executable doctests arrive, a method example should receive the module's ordinary import environment, not arbitrary compiler magic.

Conceptually:

```text
example source
    +
documented module's static import environment
    +
prelude
```

For public-package documentation, only public dependencies should be assumed.

Doctests must not have privileged access merely because documentation sits beside private code.

There can eventually be hidden setup support, but I would not specify that in v1.

---

# 20. Module/package documentation

As the future module system lands:

```phalcom
//! Geometric primitives.
//!
//! The package exports [`Point`], [`Vector`], and [`Matrix`].
```

at the top of:

```text
package.ph
```

documents the package itself.

At the top of:

```text
vector.ph
```

it documents that module.

This maps naturally to the future:

```text
Module < Package
```

model.

For the built-in `universe` package, its documentation can be synthesized partly from native metadata and optionally augmented with a native Phaldoc source.

For example conceptually:

```text
universe

The canonical package of built-in Phalcom language objects.

Not every universe binding is imported into the prelude.
```

---

# 21. Documentation for `universe` types

Semantic linking should understand prelude versus universe identity.

If:

```text
String
```

is in the prelude, generated documentation can display:

```text
String
```

even though the canonical identity is:

```text
universe.String
```

For a non-prelude binding:

```text
universe.BoundMethodFamily
```

the renderer should keep the qualification.

Therefore a native return type:

```rust
returns = BoundMethodFamily
```

can be rendered:

```text
universe.BoundMethodFamily
```

without the Phaldoc author manually qualifying it.

Again: documentation does not repeat machine metadata.

---

# 22. Native primitive Phaldoc

For Rust primitives, use exactly the same Phaldoc body language through the helper attribute we previously designed:

```rust
/// Rust implementation notes:
///
/// Lookup must remain observational and must not enter dNU.
#[phalcom::primitive(
    Object,
    "methodFor(_)",

    params = [Symbol],
    returns = Option<Method>,
    types = "(Symbol) -> Option<Method>",

    effects = pure,
    raises = [],

    side = instance,
    visibility = public,
    stability = stable,
)]
#[phalcom::phaldoc(r#"
Returns the exact accessible method selected by `selector`.

The lookup is observational and does not invoke
[`Object#doesNotUnderstand(_:)`].

@param selector — The selector to probe.
@returns — The accessible method, or `None` when no match exists.

## Examples

```phalcom
Object.methodFor(#toString)
```

@see [`Object#respondsTo(_)]
"#)]
pub fn object_method_for(
    vm: &mut VM,
    receiver: &Value,
    args: &[Value],
) -> PhResult<Value> {
    ...
}
```

Two distinct documentation channels remain:

```text
Rust ///
    documentation for runtime implementers

#[phalcom::phaldoc(...)]
    documentation for Phalcom users
```

They should not be implicitly merged.

That distinction is valuable.

---

# 23. The normalized Phaldoc model

Do not make tools repeatedly parse raw Markdown strings.

Parse into a VM-independent normalized representation.

Conceptually:

```rust
pub struct Phaldoc {
    pub summary: DocMarkup,
    pub details: DocMarkup,

    pub params: Vec<ParamDoc>,
    pub returns: Option<DocMarkup>,
    pub raises: Vec<RaiseDoc>,
    pub see: Vec<DocReference>,

    pub links: Vec<ResolvedDocLink>,
    pub examples: Vec<DocExample>,

    pub source: DocSource,
}
```

With:

```rust
pub struct ParamDoc {
    pub parameter: ParameterKey,
    pub body: DocMarkup,
}
```

```rust
pub struct RaiseDoc {
    pub error: TypeRef,
    pub body: DocMarkup,
}
```

```rust
pub enum DocReference {
    Symbol(DocSymbolRef),
    Url(String),
}
```

and:

```rust
pub struct DocExample {
    pub mode: ExampleMode,
    pub source: String,
    pub span: SourceRange,
}
```

The raw source should also be retained by tooling:

```rust
pub struct PhaldocSource {
    pub raw: Arc<str>,
    pub span: SourceRange,
}
```

so formatting and diagnostics retain exact source correspondence.

---

# 24. Documentation is attached to semantic declarations, not AST nodes forever

Parsing initially associates docs with syntax.

Normalization should re-key them to semantic identities.

Examples:

```text
class:
    DeclId(module, String)

instance method:
    MethodKey(String, Instance, "split(_)")

class method:
    MethodKey(String, Class, "fromUtf8(_)")

top-level binding:
    BindingKey(module, name)

module:
    ModuleId

package:
    PackageId
```

This is essential for:

```text
LSP symbol information
hover
rename
documentation generation
native/source unification
cross-reference resolution
incremental recompilation
```

AST locations are provenance.

They are not permanent documentation identity.

---

# 25. Runtime reflection should be optional and off by default

Phaldoc itself should not force:

```text
MethodObject {
    docs: String
}
```

on every runtime method.

Default model:

```text
source/compiler metadata
    yes

LSP/index metadata
    yes

phalcom doc
    yes

runtime heap retention
    no
```

A future build flag could optionally embed docs:

```text
--embed-docs
```

and expose something like:

```phalcom
method.documentation
```

But that is a packaging/runtime choice layered on top.

It should not determine the source format.

---

# 26. The generated documentation contract view

The important architectural consequence of all our recent work is that Phaldoc itself should not generate the entire API page.

The API page is an assembly of semantic sources.

For a method:

```rust
#[phalcom::primitive(
    Object,
    "methodFor(_)",
    params = [Symbol],
    returns = Option<Method>,
    types = "(Symbol) -> Option<Method>",
    raises = [],
    effects = pure,
    stability = stable,
)]
```

plus Phaldoc, the renderer should conceptually produce:

```text
Object#methodFor(_)

(Symbol) -> Option<Method>

stable · pure · native

Returns the exact accessible method selected by selector.

Parameters
    selector: Symbol
        The selector to probe.

Returns
    Option<Method>
        The accessible method, or None when no match exists.

See also
    Object#respondsTo(_)
```

For a source method:

```phalcom
@requires(index >= 0)
@requires(index < size)
at(index: Int) -> Element
```

plus:

```phalcom
/// Returns the element stored at `index`.
///
/// @param index — Zero-based position in the collection.
```

the generator can produce:

```text
at(index: Int) -> Element

Returns the element stored at index.

Parameters
    index: Int
        Zero-based position in the collection.

Requires
    index >= 0
    index < size

Raises
    PreconditionError
```

No `@raises PreconditionError` appears in Phaldoc because that would merely duplicate the contract.

That contract-view idea from the existing draft remains one of its strongest decisions. 

---

# 27. Stability and deprecation do not belong in Phaldoc

For native code:

```rust
stability = experimental,
since = "0.4.0",
deprecated_since = "0.7.0",
replacement = "newMethod(_)",
```

is authoritative.

For Phalcom source, use/passively retain language attributes once standardized:

```phalcom
@Deprecated(
    since: "0.7.0",
    reason: "Use newMethod(_)"
)
oldMethod(...)
```

The repository's metadata design is already moving toward that distinction. 

Therefore the old Phaldoc:

```text
@deprecated
@since
```

should be removed from the canonical format.

Generated documentation can still render:

```text
Deprecated since 0.7.0
Use newMethod(_)
```

It simply comes from semantic metadata rather than prose.

---

# 28. Effects do not belong in Phaldoc

Never write:

```text
@pure
@effect IO
```

in Phaldoc.

If native metadata says:

```rust
effects = [io]
```

the docs show it.

If future source-language effect semantics exist, the docs harvest those.

The same goes for:

```text
mutability
scheduling
allocation
blocking
unsafe/trusted status
intrinsic status
```

Phaldoc may explain consequences in prose, but it never declares them.

---

# 29. Parameter descriptions are optional

Do not create Java-style pressure to document every obvious parameter.

This is poor documentation:

```phalcom
/// Adds two numbers.
///
/// @param left — The left number.
/// @param right — The right number.
/// @returns — The sum.
add(left: Number, right: Number) -> Number
```

The signature already tells most of that story.

Perfectly good:

```phalcom
/// Returns the sum of `left` and `right`.
add(left: Number, right: Number) -> Number
```

Use structured parameter docs where they add information.

`phalcom doc --check` should therefore not enforce parameter-description completeness by default.

Projects may opt into a stricter lint.

---

# 30. Documentation inheritance: no implicit copying

I would make a firm decision here:

> Overridden methods do not inherit Phaldoc automatically.

If:

```phalcom
class Base {
    /// Performs the base operation.
    run() { ... }
}

class Specialized < Base {
    run() { ... }
}
```

`Specialized#run()` has no Phaldoc.

The generator may show:

```text
Overrides Base#run()
```

and the IDE may optionally offer the inherited member's documentation as a secondary fallback, clearly labeled:

```text
No documentation on this override.
Inherited contract/documentation from Base#run():
...
```

But it must not pretend the base prose is authored documentation for the override.

This avoids one of the nastier documentation-drift problems in inheritance-heavy APIs.

An explicit inheritance directive can be designed later if a real need emerges.

---

# 31. Documentation for inherited-but-not-overridden methods

Different case:

```phalcom
class Child < Parent {
}
```

where `Parent#foo()` is inherited untouched.

Then the API documentation for `Child` may list:

```text
Inherited from Parent
    foo()
```

and link directly to `Parent#foo()` documentation.

No duplication is necessary.

---

# 32. Generated methods

For methods synthesized by attributes such as future:

```text
@construct
@get
@set
@data
```

the generating mechanism should produce machine-generated documentation metadata where appropriate.

Do not automatically copy arbitrary nearby Phaldoc onto multiple generated methods.

For example:

```phalcom
/// Display name of the account.
@get
field name: String
```

can document the field.

If `@get` generates `name`, the doc generator can show:

```text
name -> String

Generated getter for `name`.

Display name of the account.
```

but that propagation rule belongs to `@get` semantics, not generic Phaldoc attachment.

---

# 33. LSP behavior

The LSP should parse Phaldoc incrementally and use the normalized representation directly.

For:

```phalcom
object.methodFor(#foo)
```

hover can combine native metadata and Phaldoc:

```text
Object#methodFor(_)

(Symbol) -> Option<Method>

Returns the exact accessible method selected by selector.

selector
    The selector to probe.

pure
```

Parameter signature help can display the relevant `@param` prose for the active argument.

Completion uses only the summary unless expanded.

Semantic links become go-to-definition targets.

Renaming a parameter should update its `@param` reference.

Renaming/moving a declaration should update semantic Phaldoc links where possible.

---

# 34. Diagnostic model

Phaldoc errors should not make ordinary program compilation fail.

That is important.

This program:

```phalcom
/// @pram x — typo
foo(x) {}
```

is still a valid Phalcom program.

The LSP reports:

```text
warning[phaldoc.unknown_directive]:
unknown Phaldoc directive `@pram`
did you mean `@param`?
```

`phalcom doc --check` may return a failing exit status.

Normal:

```text
phalcom run
phalcom build
```

must not become semantically dependent on good documentation.

Useful diagnostics include:

| Diagnostic | Meaning |
|---|---|
| `phaldoc.dangling` | outer doc block has no declaration |
| `phaldoc.unknown_directive` | unrecognized `@...` directive |
| `phaldoc.unknown_param` | `@param` does not identify a declaration parameter |
| `phaldoc.duplicate_param` | same parameter documented twice |
| `phaldoc.invalid_returns` | `@returns` on a callable that cannot return normally |
| `phaldoc.raises_not_declared` | native/typed contract doesn't declare documented error |
| `phaldoc.invalid_error_type` | `@raises` target is not an `Error` type |
| `phaldoc.unresolved_link` | semantic link cannot resolve |
| `phaldoc.ambiguous_link` | semantic link requires owner/side/signature qualification |
| `phaldoc.missing_summary` | no initial summary paragraph |
| `phaldoc.redundant_type` | prose directive restates declared type mechanically |
| `phaldoc.redundant_contract` | prose restates an executable contract mechanically |
| `phaldoc.invalid_inner_position` | `//!` occurs where no containing docs scope exists |

---

# 35. Unknown directives should not silently become directives

Suppose:

```phalcom
/// @performance This operation is O(1).
```

`@performance` is unknown.

Do not silently accept it into a generic extension map.

That creates the same metadata-cargo-cult problem as unrestricted annotation systems.

Instead issue:

```text
unknown Phaldoc directive @performance
```

and treat the line as ordinary prose for rendering so documentation is not lost.

The author should normally write:

```phalcom
/// ## Performance
///
/// This operation is O(1).
```

That distinction is useful:

```text
directives
    machine-associated prose

headings
    arbitrary human structure
```

---

# 36. Directive and code-attribute namespaces remain disjoint

Because both visually use `@`, retain the existing repository invariant:

```text
Phaldoc directives ∩ Phalcom attribute names = ∅
```

So if Phaldoc has:

```text
@param
@returns
@raises
@see
```

the language must never later define code attributes named exactly:

```text
@param
@returns
@raises
@see
```

without first changing one system.

Likewise Phaldoc must never introduce:

```text
@requires
@ensures
@invariant
@Deprecated
```

if those are actual language attributes.

The existing repository documentation explicitly calls out this namespace separation. 

Automate the check rather than leaving it as a review convention.

---

# 37. Suggested parser grammar

At a high level:

```text
DocBlock
    := OuterDocBlock
     | InnerDocBlock

OuterDocBlock
    := ("///" DocLine Newline)+

InnerDocBlock
    := ("//!" DocLine Newline)+
```

After marker stripping:

```text
Phaldoc
    := Summary Details* Directive*

Directive
    := ParamDirective
     | ReturnsDirective
     | RaisesDirective
     | SeeDirective
```

But directives do not need to appear strictly at the end. This should be legal:

```phalcom
/// Does something.
///
/// @param value — The value.
///
/// ## Behavior
///
/// More detailed behavior.
///
/// @see [`Other#method(_)]
```

So the real parser treats directives as block-level Markdown extensions anywhere outside fenced code.

A line beginning:

```text
@param
```

inside:

```text
```text
@param
```
```

is ordinary code content, not a directive.

Likewise inline:

```text
Use `@param` to document a parameter.
```

is ordinary prose.

---

# 38. Canonical formatting

`phalcom fmt` should format the comment wrapper but should be conservative about prose reflow.

It may normalize:

```text
///@param x: thing
```

to:

```text
/// @param x — thing
```

It should preserve paragraph wrapping unless a documentation-format option is enabled.

Fenced code content should never be text-reflowed by the outer formatter; it should optionally be passed to the Phalcom formatter independently.

---

# 39. A complete example

A normal Phalcom method:

```phalcom
/// Finds the first element satisfying `predicate`.
///
/// Traversal proceeds in iteration order and stops immediately after a
/// successful predicate evaluation.
///
/// @param predicate — Determines whether an element matches.
/// @returns — The matching element, or `None` if no element matches.
///
/// ## Examples
///
/// ```phalcom
/// const values = [1, 3, 8, 11]
///
/// assert(values.find(|value| { value.isEven }) == Some(8))
/// ```
///
/// ## Complexity
///
/// O(n) predicate evaluations in the worst case.
///
/// @see [`Iterable#any(_)]
/// @see [`Iterable#filter(_)]
find(predicate: Block<Element, Bool>) -> Option<Element> {
    ...
}
```

The authored Phaldoc says nothing about:

```text
Element
Block<Element, Bool>
Option<Element>
purity
stability
visibility
```

because the semantic model already knows those.

---

# 40. Complete native example

```rust
/// Performs the VM's exact method-table probe.
///
/// This implementation must not invoke dNU and must preserve the access
/// authority of the calling native frame.
#[phalcom::primitive(
    Object,
    "methodFor(_)",

    params = [Symbol],
    returns = Option<Method>,
    types = "(Symbol) -> Option<Method>",

    raises = [],
    effects = pure,

    side = instance,
    visibility = public,
    stability = stable,

    abi = value,
    flow = value,
)]
#[phalcom::phaldoc(r#"
Returns the accessible method selected by `selector`.

Lookup is exact and observational. A missing selector does not invoke
[`Object#doesNotUnderstand(_:)`].

@param selector — The exact selector to probe.
@returns — The accessible method, or `None` when no method matches.

## Examples

```phalcom
const method = object.methodFor(#toString)

method.ifSome(|m| {
    System.print(m.name)
})
```

@see [`Object#respondsTo(_)]
"#)]
pub fn object_method_for(
    vm: &mut VM,
    receiver: &Value,
    args: &[Value],
) -> PhResult<Value> {
    ...
}
```

The native interface generator has enough information to build one unified documentation record:

```text
owner          Object
side           instance
selector       methodFor(_)
params         [selector: Symbol]
returns        Option<Method>
raises         []
effects        pure
visibility     public
stability      stable
summary        Returns the accessible method selected by selector.
details        ...
param docs     selector → ...
return docs    ...
examples       ...
see            Object#respondsTo(_)
implementation phalcom_core::primitive::object::object_method_for
```

That is exactly the boundary we want.

---

# 41. What Phaldoc should deliberately not become

The format should resist turning into a second schema language.

Phaldoc should never be the authoritative place to declare:

```text
types
generic bounds
effects
purity
mutability
ownership
thread safety guarantees that are expressible semantically
visibility
stability
deprecation
version availability
contracts
intrinsics
native ABI
primitive ownership
selector identity
module exports
```

Those belong elsewhere.

Phaldoc may explain them:

```phalcom
/// This operation performs no allocation because...
```

but if the compiler needs to reason about "no allocation," that fact cannot live only in the prose.

---

# 42. Final proposed Phaldoc v1

The language can be surprisingly small:

```text
Markers
    ///
    //!

Markup
    CommonMark
    fenced code
    tables

Semantic links
    [`Type`]
    [`Owner#selector`]
    [`Owner::selector`]
    [`#selector`]
    [`::selector`]

Directives
    @param
    @returns
    @raises
    @see

Reserved
    @typeparam

Everything else
    ordinary Markdown
```

And the division of responsibility is very clean:

| Information | Canonical source |
|---|---|
| Summary/explanation | Phaldoc |
| Parameter meaning | Phaldoc |
| Return meaning | Phaldoc |
| Error circumstances | Phaldoc |
| Examples | Phaldoc Markdown |
| See-also links | Phaldoc |
| Parameter types | type metadata/declaration |
| Return type | type metadata/declaration |
| Declared errors | semantic/native metadata |
| Effects | semantic/native metadata |
| Preconditions | `@requires` |
| Postconditions | `@ensures` |
| Invariants | `@invariant` |
| Deprecation | passive semantic attribute |
| Stability | semantic/package/native metadata |
| Visibility | declaration metadata |
| Native implementation | primitive descriptor |
| Selector identity | declaration/primitive descriptor |

This is the version I would ratify before implementation: **small author-facing syntax, rich generated documentation, selector-aware semantic links, zero runtime cost by default, and no duplication of facts the language already knows.**