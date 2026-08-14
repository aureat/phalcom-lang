# Family Test Refresh Implementation Plan

> **For agentic workers:** Execute inline in the current checkout. No additional agents are needed.

**Goal:** Replace stale Open/Pinned family fixtures with current exact/pattern bound-family coverage and add missing language-level cases.

**Architecture:** Keep all changes inside `phalcom-core/tests/lang/family/`. Existing cases will be renamed where their filenames encode retired semantics, then rewritten with current selector forms. New fixtures will exercise exact getter/nullary/method/setter calls, pattern overload routing, deferred misses, and receiver evaluation.

**Tech Stack:** Phalcom `.ph` golden corpus, `.expected` sidecars, Cargo language integration target.

---

### Task 1: Rename stale fixture identities

**Files:**
- Modify/rename only within `phalcom-core/tests/lang/family/` and `phalcom-core/tests/lang/family/negative/`.

- [x] Rename `family_open_bound_labeled_call` to `family_pattern_bound_labeled_call`.
- [x] Rename `family_open_bound_positional_call` to `family_exact_bound_positional_call`.
- [x] Rename `family_pinned_bound_labeled_call` to `family_exact_bound_labeled_call`.
- [x] Rename `family_pinned_selects_exact_overload` to `family_exact_selects_overload`.
- [x] Rename `family_pinned_type_bound_static` to `family_exact_type_bound_static`.
- [x] Rename `family_type_bound_static` to `family_exact_type_bound_positional`.
- [x] Rename `family_inheritance_flattened` to `family_exact_inherited_method`.
- [x] Rename negative files to describe call-time missing selectors and exact shape mismatch; remove the obsolete bare-hash negative.

### Task 2: Adapt existing positive fixtures

**Files:**
- Modify renamed `.ph` and `.expected` pairs under `phalcom-core/tests/lang/family/`.

- [x] Convert `p::move` labeled dispatch to `p::move(...)` pattern dispatch and retain labeled-call output.
- [x] Convert positional and labeled pinned cases to exact selector references whose call labels match their stored selector shape.
- [x] Convert inherited and class-side references to exact nullary or positional selector syntax.
- [x] Update comments to cite `docs/spec/callables/family.md` §§1–5 and remove Open/Pinned claims.
- [x] Change DNU-backed exact Family invocation from `family()` to `family.get()` so it invokes the getter shape.

### Task 3: Adapt existing negative fixtures

**Files:**
- Modify renamed negative `.ph` and `.expected` pairs under `phalcom-core/tests/lang/family/`.

- [x] Make missing exact-selector construction succeed and invoke it with `.get()`; expect the default DNU substring `does not understand 'typo'`.
- [x] Make missing exact-method construction succeed and invoke it with the matching call shape; expect `does not understand 'typo(_)'`.
- [x] Keep exact unary Family shape rejection, updating the expected substring to `exact family \`move\` does not accept this call shape`.
- [x] Remove the obsolete bare hash-reference parser rejection, which the current parser accepts as a Family.

### Task 4: Add current family coverage

**Files to create:**
- `phalcom-core/tests/lang/family/family_exact_getter_and_nullary.ph`
- `phalcom-core/tests/lang/family/family_exact_getter_and_nullary.expected`
- `phalcom-core/tests/lang/family/family_pattern_named_overloads.ph`
- `phalcom-core/tests/lang/family/family_pattern_named_overloads.expected`
- `phalcom-core/tests/lang/family/family_exact_setter.ph`
- `phalcom-core/tests/lang/family/family_exact_setter.expected`
- `phalcom-core/tests/lang/family/family_receiver_evaluated_once.ph`
- `phalcom-core/tests/lang/family/family_receiver_evaluated_once.expected`
- `phalcom-core/tests/lang/family/negative/family_pattern_no_matching_route.ph`
- `phalcom-core/tests/lang/family/negative/family_pattern_no_matching_route.expected`

- [x] Test exact getter with `family.get()` and exact nullary method with `family()` in one fixture; expected output `11` then `12`.
- [x] Test `receiver::value(...)` routing nullary and unary `value` overloads through `family()` and `family(4)`; expected output `12` then `5`.
- [x] Test `receiver::value=(put)` plus `family.set(8)` mutating and reading the receiver; expected output `8`.
- [x] Test a factory receiver that prints `created`, then call an escaping exact Family twice; expected output `created`, `12`, `12`.
- [x] Test a missing structural-pattern route at call time; expect `does not understand 'missing()'`.

### Task 5: Verify corpus and repository state

- [x] Run the focused family lane with `cargo test -p phalcom-core --test lang family -- --nocapture` (passes both `family` and `family_negative`).
- [x] Fix only fixture syntax, expected output, or comments exposed by those tests; no runtime/compiler changes were needed.
- [x] Run `cargo test -p phalcom-core --test lang` (family lanes pass; unrelated `syntax_errors` baseline remains).
- [x] Run `cargo test -p phalcom-core --test integration` (family/runtime tests pass; unrelated missing example fixtures remain).
- [x] Run `graphify update .`.
- [x] Run `git diff --check` and review status/diff for scoped changes.
