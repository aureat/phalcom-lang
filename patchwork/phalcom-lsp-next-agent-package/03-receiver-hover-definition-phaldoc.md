# Implementation Spec 3 — Receiver-Qualified Hover/Definition and Declaration-Identity Phaldoc

**Repository:** `aureat/phalcom-lang`  
**Baseline commit:** `8f41ee4a7029f0617930cb01348454a111d072fb`  
**Prerequisites:** Specs 1 and 2 green.  
**Primary files:** `phalcom-lsp/src/backend.rs`, `hover.rs`, `semantic/mod.rs`, `semantic/surface.rs`.

## 1. Goal

Finish migration of hover and go-to-definition from workspace-global selector lookup to the same receiver-qualified semantic model used by completion.

Also fix Phaldoc so documentation belongs to a concrete declaration identity, not merely a selector string, and add class documentation hover.

## 2. What is already fixed on the baseline

Do not reimplement these:

- declaration hover calls `SemanticDb::member_at` and uses `return_for_callable`;
- receiver-targeted hover uses `semantic_receiver` and `receiver_member`;
- repeated `Counter.new()` vs `Other.new()` already has an integration regression test;
- unknown receiver-targeted hover intentionally avoids global-site fallback.

Build on this path.

## 3. Remaining baseline defects

### 3.1 Inherited hover reports receiver class instead of defining class

Current receiver hover resolves an inherited `MemberSurface`, but pushes:

```rust
SelectorSite {
    class: class.name.clone(), // receiver class
    kind: hover_member_kind(&member),
}
```

For `dog.speak()` where `Animal` defines `speak()`, hover can render “method on Dog”. The defining owner is `member.callable.owner`.

The same code selects the Phaldoc source URI from the receiver class module instead of the member owner module.

### 3.2 `semantic_definition_locations` is not inheritance-aware

It uses `member_surface(&class, selector)`, which checks only that exact class surface. It should use `receiver_member(class, selector, side)` and then navigate to `member.callable.owner`.

### 3.3 `goto_definition` still falls back globally after failed semantic receiver resolution

Baseline logic:

```rust
let semantic_locations = self.semantic_definition_locations(...);
if !semantic_locations.is_empty() { ... }
let occurrences = self.index.definitions(&selector); // global fallback
```

For `mystery.ping()` with unknown receiver and several `ping()` implementations, this can jump to every global definition. A receiver-qualified unresolved send must return no definition, not a workspace selector dump.

### 3.4 Selector-only return inference remains in hover fallback

`SemanticDb::return_for_selector` still exists and non-receiver hover fallback may call it. This is unsound whenever repeated selectors have different returns.

No user-facing semantic feature may infer a return by selector alone.

### 3.5 `SelectorSite` drops module-qualified owner identity

It stores only class text and member kind. If a bounded receiver union contains `A.User` and `B.User`, hover may render `User, User`.

Carry `ClassId` through rendering.

### 3.6 Phaldoc is selector-keyed inside a file

`hover::harvest_doc_for_selector` scans the defining file and returns the first doc block that resolves to the requested selector. If one file has:

```phalcom
class A {
  /// A docs
  ping() { }
}
class B {
  /// B docs
  ping() { }
}
```

hovering `B.ping()` can receive A’s docs.

### 3.7 Class Phaldoc/class hover is absent

`member_selector_at_line` only handles `ClassMember`s; a `///` block immediately above `class Grid` has no hover target. Cursor resolution also has no class-symbol hover path.

### 3.8 Documentation around attributes is fragile

Phaldoc adjacency must treat declaration attributes as part of the declaration:

```phalcom
/// Docs
@class
@private
foo() { }
```

The docs belong to `foo()`. Do not require `foo` to be on the line immediately following the `///` run.

## 4. Read only these files first

1. `phalcom-lsp/src/backend.rs`
   - `semantic_receiver`
   - `semantic_definition_locations`
   - `selector_at_position`
   - `hover_at`
   - `goto_definition`
2. `phalcom-lsp/src/hover.rs`
   - `PhaldocDoc`
   - `member_range`
   - `member_selector_at_line`
   - `harvest_doc_for_selector`
   - `SelectorSite`
   - `render_selector_hover_with_value`
3. `phalcom-lsp/src/semantic/mod.rs`
   - `member_surface`
   - `receiver_member`
   - `return_for_callable`
   - `return_for_selector`
   - `class_at`
   - `member_at`
   - `returns_for_callables`
4. `phalcom-lsp/src/semantic/surface.rs`
   - `ClassSurface`
   - `MemberSurface`
   - `source_range`, `name_range`, `callable.owner`
5. `phalcom-lsp/tests/stage4_hover.rs`
6. `phalcom-lsp/tests/workspace_semantics.rs` after applying the supplied patch.

Do not inspect VS Code TypeScript providers: the extension delegates hover/definition to the LSP.

## 5. Introduce one resolved member target model in the backend

Do not let hover and definition independently rediscover dispatch.

Add a crate-private structure near `Backend` helpers:

```rust
#[derive(Clone, Debug)]
struct ResolvedMemberTarget {
    /// Runtime receiver candidate used for dispatch.
    receiver: crate::semantic::ClassId,
    /// Actual declaration selected after inheritance lookup.
    member: crate::semantic::MemberSurface,
}
```

Add:

```rust
fn semantic_member_targets(
    &self,
    uri: &Url,
    position: Position,
    selector: &str,
) -> Option<Vec<ResolvedMemberTarget>>
```

Contract:

- `None` = cursor is not receiver-qualified / semantic receiver target cannot be formed;
- `Some(vec![])` = receiver-qualified, but receiver/member cannot be resolved; this is a conservative terminal result;
- `Some(nonempty)` = bounded semantic targets.

Implementation:

1. get open `Document`;
2. require `completion::target_at(doc, position)`;
3. call existing `semantic_receiver`;
4. for every receiver alternative, map `ReceiverKind` to `DispatchSide`;
5. call `self.semantic.receiver_member(&class, selector, side)`;
6. retain actual `MemberSurface`.

Deduplicate by `member.callable`.

This helper becomes the common semantic authority for hover and definition.

## 6. Declaration-site resolution

Before receiver resolution, declaration hover/definition should use the exact declaration under the cursor:

```rust
let offset = doc.line_index.offset(position);
if let Some(member) = self.semantic.member_at(uri, offset) {
    if member.callable.selector == selector {
        // exact declaration target
    }
}
```

No global lookup is needed at a declaration site.

For class symbols, add a separate class target (see §10).

## 7. Correct inherited owner semantics

When a target resolves:

- receiver = `Dog`;
- member owner = `Animal`.

Use:

```rust
let owner = member.callable.owner.clone();
```

for:

- hover “on …” text;
- definition URI;
- Phaldoc source URI;
- callable return lookup;
- visibility/declaration identity.

Do not use the runtime receiver class for those operations.

It is acceptable to optionally show both:

```text
`speak()` — method on Animal
receiver: Dog
```

but “on Animal” must refer to the declaration owner.

## 8. Make definition receiver-aware and conservative

Rewrite `semantic_definition_locations` to consume `semantic_member_targets`.

For each target:

```rust
let owner = &target.member.callable.owner;
if owner.module.as_str() == CORE_MODULE_URI {
    // no source location unless core source mapping exists
    continue;
}
let definition_uri = Url::parse(owner.module.as_str()).ok()?;
let range = with_source_snapshot(&definition_uri, |_, _, line_index| {
    line_index.range(target.member.name_range.start..target.member.name_range.end)
});
```

Then change `goto_definition` fallback policy.

Pseudo-code:

```rust
let Some((selector, _)) = self.selector_at_position(&uri, position) else {
    return Ok(None);
};

if let Some(targets) = self.semantic_member_targets(&uri, position, &selector) {
    // Receiver-qualified request: semantic resolution is authoritative.
    let locations = self.member_target_locations(targets);
    return if locations.is_empty() {
        Ok(None)
    } else {
        Ok(Some(GotoDefinitionResponse::Array(locations)))
    };
}

// Only a genuinely non-receiver legacy selector path may consult the index.
// Do not use it to recover a failed receiver resolution.
```

If unqualified call semantics can be resolved lexically, add a semantic unqualified resolver instead of falling back by selector. Keep the global index only where ambiguity is an intentional workspace/navigation feature.

## 9. Remove selector-only return semantics

Delete `SemanticDb::return_for_selector` if there are no remaining non-test callers after this migration.

Run:

```sh
rg -n "return_for_selector" phalcom-lsp
```

Every semantic return caller must use one of:

```rust
return_for_callable(&CallableId)
returns_for_callables(bounded_ids)
```

If a hover target is unresolved, omit the inferred return section. Never pick “the first summary with this selector”.

## 10. First-class class hover

### 10.1 Add class name range to the semantic surface

`ClassSurface` currently stores only the whole `source_range`.

Inspect `phalcom_ast::ast::ClassDef` and parser output for an existing class name range. If it exists, copy it into:

```rust
pub struct ClassSurface {
    pub id: ClassId,
    pub superclass: Option<ClassId>,
    pub members: BTreeMap<String, MemberSurface>,
    pub fields: BTreeMap<String, FieldSurface>,
    pub source_range: SourceRange,
    pub name_range: SourceRange, // NEW
}
```

If `ClassDef` lacks a name range, add it in `phalcom-ast` at parse time; do not approximate class hover by searching raw text for the class name.

### 10.2 Add `SemanticDb::class_name_at`

```rust
pub fn class_name_at(&self, uri: &Url, offset: usize) -> Option<ClassSurface>
```

It must match `name_range`, not the whole class body.

### 10.3 Render class hover

Add in `hover.rs`:

```rust
pub fn render_class_hover(
    class: &ClassId,
    superclass: Option<&ClassId>,
    phaldoc: Option<&PhaldocDoc>,
) -> String
```

Minimum output:

```text
`Widget` — class

<summary>
```

If superclass is explicit/known:

```text
`Dog` — class, is Animal
```

Do not list every workspace class with the same bare name.

## 11. Replace selector-keyed Phaldoc attachment with declaration identity

Keep the existing parser for summary/tags (`parse_doc_block`, `parse_tags`). Replace only target association.

### 11.1 New target structure

In `hover.rs`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeclarationDocTarget {
    Member {
        declaration: phalcom_common::range::SourceRange,
        name: phalcom_common::range::SourceRange,
    },
    Class {
        declaration: phalcom_common::range::SourceRange,
        name: phalcom_common::range::SourceRange,
    },
}
```

Add:

```rust
pub fn harvest_doc_for_declaration(
    text: &str,
    line_index: &LineIndex,
    target: DeclarationDocTarget,
) -> Option<PhaldocDoc>
```

The target is already semantically resolved; the harvester must not search for another declaration with the same selector.

### 11.2 Adjacency algorithm

Use source positions, not selector matching:

1. determine the declaration’s first syntactic line, including attributes;
2. walk upward over contiguous declaration attributes;
3. the immediately preceding non-attribute lines must be a contiguous `///` run;
4. a blank line breaks adjacency;
5. parse that exact run with existing `parse_doc_block`;
6. no selector search.

For class declarations there is no member selector involved.

### 11.3 Attributes

The AST already retains attributes on members. Add a helper in `hover.rs` or `surface.rs` that computes the earliest source start of the declaration including attribute ranges. If `member.source_range` already starts at the first attribute, use it and add a regression test proving it. If not, enrich `MemberSurface` with `doc_anchor_range`/`attribute_start` rather than parsing `@` lines heuristically.

Do not make documentation attachment depend on the visual spelling of an attribute.

### 11.4 Pinned Phaldoc compatibility

If `/// selector: foo(_)` is a supported existing Phaldoc feature, preserve it as an explicit detached-doc lookup path, but make the key declaration-qualified where ambiguity exists.

Recommended rule:

- adjacent docs always win by declaration identity;
- detached selector pin is used only when there is exactly one matching declaration in that module, otherwise do not guess.

Do not let a detached bare selector override the docs of a different resolved class member.

## 12. Carry module identity through hover rendering

Change `SelectorSite` from:

```rust
pub struct SelectorSite {
    pub class: String,
    pub kind: MemberKind,
}
```

to:

```rust
pub struct SelectorSite {
    pub owner: ClassId,
    pub receiver: Option<ClassId>,
    pub kind: MemberKind,
}
```

Renderer rules:

1. one owner: render bare class name (`Animal`);
2. several owners with distinct bare names: render bare names;
3. several owners sharing a bare name: disambiguate with module path/basename, e.g. `User (a.ph)`, `User (b.ph)`;
4. deduplicate by `ClassId`, not string.

Do not produce `Main, Main, Main`.

## 13. Hover resolution order

Use this exact priority:

1. keyword/contextual word;
2. class name declaration/reference if class target resolves;
3. exact member declaration under cursor;
4. receiver-qualified member target(s);
5. semantically resolvable unqualified member target;
6. documented top-level binding;
7. no hover.

A workspace-global selector listing is not a normal hover fallback.

If you retain an “ambiguous workspace selector” hover for some explicit reflective syntax, label it clearly as ambiguity and do not attach inferred return/doc from an arbitrary definition.

## 14. Existing Stage 4 test correction

The baseline `cross_file_hover_resolves_the_doc_from_the_declaring_file` fixture uses a class from another file without an import:

```phalcom
let m = Mover.new()
```

After Spec 2 removes workspace-global class leakage, correct that test to language-valid module scope:

```phalcom
import "./mover" as MoverModule
let m = MoverModule.Mover.new()
m.move(1)
```

Do not restore global class leakage merely to preserve the old test.

## 15. Regression tests supplied in the patch

The patch adds these acceptance tests:

1. `inherited_hover_reports_the_defining_owner_not_the_receiver_class`
2. `phaldoc_is_attached_to_the_resolved_declaration_not_the_first_matching_selector`
3. `class_hover_surfaces_adjacent_class_phaldoc`
4. `receiver_qualified_definition_does_not_fall_back_to_every_global_selector_match`
5. `unimported_workspace_class_is_not_semantic_authority_for_hover` (shared with Spec 2)

Do not weaken them.

## 16. Additional tests to add

In `stage4_hover.rs`:

- docs above `@class` + `@private` method attach correctly;
- two modules each define `User.new()` and union hover disambiguates module identity;
- inherited hover gets docs from ancestor file;
- class hover with same bare class name in different modules resolves exact module;
- receiver union shows only bounded actual implementations;
- unknown receiver hover is absent/conservative and does not list workspace classes;
- class-side and instance-side same-selector methods do not leak across sides.

For definition tests:

- `Dog.speak()` jumps to `Animal.speak()`;
- `Counter.new()` jumps only to Counter constructor;
- `A.User.foo()` and `B.User.foo()` jump to their respective modules;
- unknown receiver has no global-selector jump.

## 17. Implementation sequence

1. Add `semantic_member_targets` and use it only in tests first.
2. Fix inherited owner use in hover.
3. Migrate `semantic_definition_locations` to inherited/member-owner resolution.
4. Gate/remove global definition fallback for receiver-qualified requests.
5. Remove selector-only return fallback and `return_for_selector` callers.
6. Change `SelectorSite` to carry `ClassId` and add disambiguating rendering.
7. Implement declaration-identity Phaldoc for members.
8. Add class name range and class hover.
9. Add attribute-aware doc anchors.
10. Correct the old cross-file hover test to use an import.
11. Run focused and full LSP tests.

## 18. Commands

```sh
cargo test -p phalcom-lsp --test integration stage4_hover
cargo test -p phalcom-lsp --test integration inherited_hover_reports_the_defining_owner_not_the_receiver_class
cargo test -p phalcom-lsp --test integration phaldoc_is_attached_to_the_resolved_declaration_not_the_first_matching_selector
cargo test -p phalcom-lsp --test integration class_hover_surfaces_adjacent_class_phaldoc
cargo test -p phalcom-lsp --test integration receiver_qualified_definition_does_not_fall_back_to_every_global_selector_match
cargo test -p phalcom-lsp --test integration semantic_consistency
cargo test -p phalcom-lsp
cargo clippy -p phalcom-lsp --all-targets -- -D warnings
```

Then VS Code E2E only after Rust behavior is green.

## 19. Completion criteria

This unit is complete only when:

- hover/definition dispatch is receiver/class/side-qualified;
- inherited members report/navigate to their actual defining owner;
- selector-only return inference is gone from user-facing semantics;
- unknown receiver does not fall back to every global definition;
- same-named classes remain distinguishable by module identity;
- Phaldoc attaches to the resolved declaration, not first matching selector;
- docs above attributes work;
- classes have first-class hover and adjacent `///` docs;
- the global selector index remains only for workspace-symbol/reference-style features where global indexing is intentional.
