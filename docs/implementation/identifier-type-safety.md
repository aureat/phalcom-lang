# Identifier Type Safety Implementation Plan

**Goal:** Make Nano ID, UUID, and Snowflake values safer to pass between domains and make validated format-specific operations visible in Phalcom types.

**Architecture:** Keep the three packages independent and retain their existing untagged entry points. Add nominal phantom-tagged wrappers at the package boundary. Checked parsing and UUIDv7 refinement return explicit `Result` types; a Snowflake wrapper retains its generator's actual layout as well as a user-chosen layout tag. The checker remains the authority for nominal applied-type distinctions.

**Tech Stack:** Phalcom package sources, the canonical semantic test harness, and focused `phalcom-modules` parser/interface/linker tests.

---

## Contract and limits

- `Tag` distinguishes application domains; `Format` distinguishes caller-chosen Snowflake layouts. Neither type parameter is inferred from text or raw integer bits. Crossing from an untagged value to a tagged one is an explicit trust boundary.
- A generated layout-tagged Snowflake retains the exact layout used to produce it. Parsing into a layout-tagged value requires an explicit layout, and the caller owns the choice of `Format`. No tag alone proves that two independently configured layouts have identical parameters.
- A checked UUIDv7 wrapper may expose a non-optional timestamp only after checking the RFC variant and version. Non-v7 UUIDs remain valid `UUID`s.
- Nano ID's existing `Id<Tag>` is preserved. Typed generation should be available on configured generators, and parsing should report a specific typed validation error rather than an undifferentiated `Error`. `Id<Tag>.wrap` is an explicit, unchecked domain assertion and must be documented as such.
- Public data wrappers use private constructors and preserve exact generic type identity. Existing `UUID`, `SnowflakeId`, `SnowflakeNode`, `NanoId`, and `Id<Tag>` APIs remain available.
- The embedded Universe cannot currently execute the authored `Bytes` and SHA-1 dependencies. No Universe loader cutover belongs to this plan.

## Tasks

### Task 1: Prove type separation

**Files:** `phalcom-semantic/tests/semantic/adts/data_declarations.rs` (only if the existing generic-data tests do not establish the negative call/assignment case); semantic implementation only if the focused test exposes a defect.

- [x] Prove a matching tagged argument is accepted and a mismatched tag yields exactly one `ArgumentMismatch` in a source-level call. The focused test uses a test-only `numbers` exposure override because the current embedded Universe baseline blocks normal workspace setup.
- [x] Run `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic phantom_tagged_identifier_rejects_cross_domain_argument` (PASS). `data_phantom_applications_remain_distinct` also passes. The ordinary `data_declaration_publishes_nominal_components` test independently reproduces the unrelated embedded-Universe exposure failure.
- [x] No checker fix required: the requested generic nominal distinction is already enforced.

### Task 2: Nano ID validated typing

**Files:** `packages/nanoid/src/id.ph`, `packages/nanoid/src/errors.ph`, `packages/nanoid/src/package.ph`, `packages/nanoid/tests/src/typed-ids-test.ph`, `packages/nanoid/README.md`. Keep the `Id<Tag>` declaration in `nanoid.id` to preserve canonical identity. `TypedNanoIdGenerator<Tag>` wraps the existing generator in that same module, avoiding a generator/id import cycle.

- [x] Add a checked, structured Nano ID parse error surface; document `wrap` as the unchecked assertion boundary.
- [x] Add `TypedNanoIdGenerator<Tag>` with `fromGenerator`, `configured`, and tagged `generate` methods using the configured generator. An initial split of `Id<Tag>` into another module changed nominal declaration identity; the final design retains it in `nanoid.id`.
- [x] Author custom-alphabet, round-trip, and typed parse-error fixtures. Package source parsing and symbolic root-export identity linking passed; Telos runtime fixtures await the authored Universe.

### Task 3: Tagged UUID and checked v7

**Files:** `packages/uuid/src/typed.ph` (new), `packages/uuid/src/package.ph`, `packages/uuid/tests/src/typed-test.ph` (new), `packages/uuid/tests/src/package.ph`, `packages/uuid/tests/src/main.ph`, `packages/uuid/README.md`.

- [x] Add `TaggedUUID<Tag>` with private storage, explicit `fromUUID`, `parse`, `value`, and typed generation, without implicit conversion between tags.
- [x] Add checked `UUIDv7.fromUUID` and `UUIDv7.parse` with precise failure variants; `timestamp -> Int` is available only on a successful wrapper. Add a typed result for `UUIDv7Generator` without changing the existing `generate()` contract.
- [x] Author generation, version-rejection, and tagged round-trip fixtures; parsing and exports passed, runtime execution remains staged.

### Task 4: Layout-bound Snowflake values

**Files:** `packages/snowflake/src/typed.ph` (new), `packages/snowflake/src/package.ph`, `packages/snowflake/tests/src/typed-test.ph` (new), `packages/snowflake/tests/src/package.ph`, `packages/snowflake/tests/src/main.ph`, `packages/snowflake/README.md`.

- [x] Add `TypedSnowflake<Domain, Format>` with private ID and immutable layout, plus checked `parse`, explicit `fromId`, and layout-aware observations.
- [x] Add `TypedSnowflakeNode<Domain, Format>` that captures a `SnowflakeNode` and returns the corresponding typed values on `generate()` and `generateAt(_ millis: Int)`.
- [x] Author custom-layout, typed parsing, and invalid-input fixtures; parsing and package symbolic linking passed, runtime execution remains staged.

### Task 5: Focused integration and delivery

**Files:** `phalcom-modules/tests/nanoid_package.rs`, `phalcom-modules/tests/uuid_package.rs`, `phalcom-modules/tests/snowflake_package.rs` only where their existing checks need to assert new exports or parse new fixture files.

- [x] Run the three focused package checks (Nano ID: 2 tests; UUID: 1 test; Snowflake: 2 tests) and the source-level checker regression (1 selected test). All passed with `RUSTFLAGS=''`. The checker test covers both a matching tagged call and a mismatched one through a generic factory.
- [x] Report source/link versus runtime verification distinctly: typed fixtures parse and Nano ID/Snowflake symbolic links resolve; Telos runtime execution remains unverified until the authored Universe is active. The unrelated default semantic baseline still fails to expose `universe.numbers.capabilities` without the test-only provider override.
- [x] Review the scoped diff and preserve unrelated worktree changes; run file-scoped Rust formatting checks.
