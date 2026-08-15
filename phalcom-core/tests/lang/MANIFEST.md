# Phalcom Language Corpus Manifest

Spec-conformance corpus for end-to-end CLI validation, organized by label
directory. Each case is a `<name>.ph` plus a sibling `<name>.expected`.

## Model

- **PASS** — feature works today; `.expected` is the exact verified stdout. Runs by
  default (`check_pass`).
- **NEGATIVE** — malformed input; must exit non-zero with the `.expected` diagnostic
  as a substring, never panic. Runs by default (`check_negative`).
- **PENDING** — a spec target the language does not implement yet. Lives in
  `<label>/pending/`; `.expected` pins the *intended* spec output, not today's
  behavior. Wired as `#[ignore]` `check_pending`, so it stays out of the green run and
  graduates to PASS when the feature lands.

## Summary

- **Wren-suite `range`/`sequence` port delta (test/core/range/*.wren,
  test/core/sequence/*.wren):** +7 PASS in `collections/` —
  `range_contains_boundary_values`, `range_equality_bound_variants`,
  `range_first_last_negative_bounds`, `range_cursor_protocol_direct`,
  `range_map_filter_reduce_via_tolist`, `range_isa_class_hierarchy`,
  `range_size_zero_is_emptiness` — and +2 NEGATIVE in `collections/negative/` —
  `range_iterate_wrong_cursor_type`, `range_wrong_arity_constructor_rejected`.
  `Range` already had solid coverage (`range_basics`/`range_bound_equality_distinct`/
  `range_boundary_and_descending`/`range_inclusive_exclusive_and_laziness`); this
  delta adds the cursor-protocol pair (`iterate`/`iteratorValue`, Option-cursor
  idiom, ADR-0035 §1), the `toList`-bridge combinator pattern (`Range` has no
  own `map`/`where`/`reduce` — not in tuple-and-range.md §2's selector table —
  so `.toList.map`/`.toList.filter`/`.toList.reduce` is the idiomatic bridge),
  `isA`/`.class` type tests (no `is` operator or `Sequence`/`Iterable` root
  yet), and two adversarial negatives (wrong-arity `Range.new`, non-`Option`
  `iterate` cursor). Skipped with no Phalcom analog: Wren's backwards-range
  walk (Phalcom clamps `start > end` to empty, already pinned), `join`/`min`/
  `max`/`from`/`to`/`isInclusive` (no such selectors — `first`/`last`/`size`
  already cover the ascending-only equivalent), `toString` (Range has no
  custom `toString`, no spec anchor to pin an intended format), and the
  Wren `sequence/` trait breadth with no spec-anchored Phalcom
  counterpart (`all`/`any`/`count`/`skip`/`take`/abstract-root
  `no_constructor` — catalog-delta.md's `List` combinator set is exactly
  `map`/`filter`/`reduce`/`includes`/`isEmpty`, nothing wider).
- **Wren-suite `map`/`map_entry` port delta (test/core/map/*.wren,
  test/core/map_entry/new.wren, ~31+1 files):** +11 PASS in `collections/` —
  `map_wren_churn`, `map_wren_contains_key`, `map_wren_count`,
  `map_wren_cursor_roundtrip`, `map_wren_empty_string_key`, `map_wren_key_types`,
  `map_wren_new`, `map_wren_remove`, `map_wren_reuse_tombstone`,
  `map_wren_to_string`, `map_wren_type` — and +5 NEGATIVE in
  `collections/negative/` — `map_wren_iterate_not_int`, `map_wren_iterate_not_num`,
  `map_wren_iterator_value_not_int`, `map_wren_iterator_value_not_num`,
  `map_wren_iterator_value_negative_rejected` — and +2 PENDING in
  `collections/pending/` — `map_wren_is_empty`, `map_wren_clear`. `Map`
  already had solid coverage (`map_set_basics`/`map_literal_construction`/
  `map_empty_ops`/`map_nested_value`/`map_overwrite_same_key`/
  `map_remove_then_readd_order`/`map_tuple_key_roundtrip`/
  `negative/map_mutable_key_rejected`); this delta ports Wren's dedicated
  `map/` suite onto the actual selector surface — `containsKey`→`includes(_)`,
  `count`→`size`, Wren's `remove(_)`-returns-removed-value→Phalcom's
  `remove(_)`-returns-`self` (read via `at(_)` first instead), and the raw-int/
  `null` Wren cursor→Phalcom's `None`/`Some(_)` two-selector protocol
  (`iterate(_)`/`iteratorValue(_)`, no `MapEntry` wrapper — DEC-CT-E,
  `iteratorValue` yields the key). `key_types` swaps Wren's surface `nil` key
  for the `None` singleton (U6 removed surface `nil`) and swaps Wren's `is`
  operator for `isA(_)` (U-IS not yet landed) in `type`. Confirms Map's
  insertion-ordered (not hash-bucket-ordered) native `toString` is
  deterministic, unlike Wren's unspecified iteration order. Skipped with no
  Phalcom analog: the three `subscript_*` tests (`map[k]`/`map[k] = v` —
  Phalcom has no `[]` indexing sugar on `Map`, only `at(_)`/`at(_, put:)`),
  `key_iterate`/`value_iterate` and their `_not_int`/`_not_num` variants
  (redundant with `List`'s own cursor-protocol coverage — `keys`/`values`
  return a `List`, and the cursor is `Option`, not a Wren-style raw-int/`null`
  a bad-type cursor could even partially satisfy), `iterator_value_iterator_too_large`
  (Phalcom's `iteratorValue(_)` is total over an in-range index — `rawKeyAt`/
  `rawValueAt` return the `None` singleton past the end rather than erroring;
  see the past-the-end coverage folded into `map_wren_cursor_roundtrip`
  instead), `contains_key_not_value`/`remove_key_not_value` (Wren rejects a
  mutable-collection key on every keyed op; Phalcom's `is_mutable_collection_key`
  guard only fires on `rawPut` — `includes(_)`/`remove(_)` on a `List` key
  silently no-op/return `false` instead of raising, an asymmetry flagged in
  `docs/forge/DEFERRED.md` rather than pinned as a false NEGATIVE), and
  `map_entry/new.wren` (no `MapEntry` class — DEC-CT-E gives Map's cursor
  protocol no per-entry wrapper object at all).
- **Wren-suite `concurrency`/Fiber delta (ported from `wren/test/core/fiber/`,
  ~65 files):** Phalcom's `Fiber` (U-FIBER, ADR-0030) has a different surface
  than Wren's — `call`/`try` (not Wren's separate `call`/`try` split across a
  `transfer`-style symmetric-coroutine API), no `transfer`/`transferError`
  (Phalcom has one control-transfer primitive, resumer-chain `call`/`yield`,
  not Wren's fully general fiber-to-fiber transfer), and no `isDone`/`error`
  accessors yet (spec'd in concurrency.md §1's Interface table, unwired in
  `primitive/fiber.rs`). +9 PASS in `concurrency/` (`concurrency_fiber_wren_call_basic`,
  `_yield_sequence`, `_yield_with_value_implicit_none`,
  `_call_return_implicit_none`, `_try_without_error`,
  `_try_value_error_capture`, `_try_value_yield`, `_abort_string_captured`,
  `_abort_number_captured`) confirming call/yield/try semantics, implicit-`None`
  fiber completion, and `try`'s raw (unwrapped) abort-value/doesNotUnderstand
  capture match Wren's shape. +5 NEGATIVE in the existing `concurrency/negative/`
  lane (`fiber_call_finished_uncaught`, `fiber_try_finished_uncaught`,
  `fiber_reenter_direct_call`, `fiber_new_wrong_arg_type`,
  `fiber_yield_no_resumer` — the uncaught-at-top-level counterparts to the
  caught-via-nested-driver shapes already in `concurrency/`, plus the
  root-fiber-has-no-caller `yield` guard alongside the existing
  `fiber_abort_root_raises`). +1 PENDING (`concurrency/pending/`
  `concurrency_fiber_wren_is_done_and_error`, merging Wren's `is_done.wren` +
  `error.wren`) pinning the intended `isDone`/`error` surface. Skipped as
  fundamentally Wren-specific (no Phalcom analog): `transfer`/`transferError`
  and every `*_transfer*`/`resume_caller`/`call_root`/`yield_with_no_caller`
  file built on it (Wren's general fiber-to-fiber transfer has no Phalcom
  counterpart — only resumer-chain `call`/`yield`); `abort_null` (Wren
  special-cases a `null` abort payload as a no-op continuation, Phalcom's
  `Fiber.abort(_)` always raises); `new_wrong_arity`/`call_to_parameter`
  (Wren caps fiber entries at one parameter and silently fills a missing
  arg with `null`; Phalcom entries take arbitrary arity and raise `Arity` on
  mismatch — not a comparable behavior); `call_return_value` (an explicit
  bare `return` inside a fiber's block entry is a non-local return with no
  live home frame, `DeadFrameError` — a known, separately-tracked gap, not
  this delta's to paper over); `type.wren` (`is`/`Fiber.type` — `is`-operator
  surface is `U-IS`, PLANNED not landed; `.class`/`isA` cover the same ground
  and are already exercised elsewhere in `concurrency/`).
- **Labels:** absence, arithmetic, bindings, blocks, booleans, classes, collections,
  compile-errors, concurrency, control-flow, dispatch, errors, functions, imports,
  inheritance, iteration, lexical, list, messages, metaclass, runtime-errors, string,
  syntax-errors, system.
- **Wren-suite `string` delta (new label, ported from `wren/test/core/string*`):**
  `String`'s native floor is exactly `+(_)`/`hash`/static `new`/`toString`
  (`primitive/string.rs`) — no length, index, split, trim, or byte/codepoint
  accessor exists yet (that gap is `../../../docs/forge/units/U-STRING/u22-string.md`, not
  landed). +5 PASS (`string_concatenation`, `string_equality`, `string_to_string`,
  `string_type`, `string_new_coercion`) covering what already carries over from
  Wren's `test/core/string/{concatenation,equality,to_string,type,no_constructor}.wren`
  onto today's floor (content `==`/`!=`, `+`, `toString` identity, `isA`/`class`,
  and `String.new(_)`'s any-value coercion — a deliberate divergence from Wren's
  constructor-less `String` metaclass). +2 NEGATIVE filed in the existing
  `runtime-errors/` lane (`string_concatenation_wrong_arg_type`,
  `string_not_operator_unsupported` — the latter pins that Phalcom has no
  truthy-coercion model, unlike Wren's every-string-is-truthy `!` semantics).
  +2 PENDING in `string/pending/` folding the ~103 remaining Wren
  `string`/`string_byte_sequence`/`string_code_point_sequence` files into two
  representative future-shape fixtures (`string_split_trim_multiply`,
  `string_codepoint_sequence`) rather than a mechanical 1:1 port — the rest are
  skipped as either not-yet-implemented (subscript/count/indexOf/contains/
  startsWith/endsWith/join/from_byte/from_code_point — U-STRING's own future
  `strings` corpus is the right home once it lands) or structurally
  inapplicable (Wren's `[]` subscript sugar and 8-bit-clean `\0` escape have no
  Phalcom equivalent — no subscript operator, no escape-sequence table in the
  lexer beyond `\\` and `\(expr)` interpolation).
- **U15 delta (most recent; ADR-0045, `import` — relative file-path resolution +
  whole-module binding, member access as an ordinary send).** New `imports` label:
  +5 PASS (`imports_basic_member_access`, `imports_identity_memoized`,
  `imports_isolation_no_leak`, `imports_kernel_visible_without_import`,
  `imports_cyclic_load_no_hang`), +2 NEGATIVE in `imports/negative/`
  (`imports_missing_file`, `imports_cycle_partial_read_fails_cleanly`). `imports/lib/`
  holds **8** more `.ph` files (`answer`, `shared`, `isolated`, `kernel_user`,
  `cycle_a`/`cycle_b`, `cycle_bad_a`/`cycle_bad_b`) that are imported-by, never
  standalone cases — `collect_cases` does not recurse into subdirectories, so they
  are invisible to the harness but visible to a raw `find`; the case counts below
  count only actual cases (13), not the 8 library fixtures.
- **U-ERR delta (the aggregate total below predates it and is not
  reconciled here — see the reconcile-corpus-counts history):** `errors/pending/`
  is now empty and retired — its two placeholder fixtures
  (`errors_throw_try_catch_finally.ph`, `errors_result_bridge.ph`) graduated to
  PASS, rewritten onto the *ratified* ADR-0031 surface (`try`/`on T e {}`/
  `catch e {}`/`ensure{}`; the old drafts used a pre-ADR-0031 JS-style
  `catch (e: T) {} finally {}` placeholder spelling). +9 PASS total in `errors/`
  (`errors_try_on_typed_and_catch_all`, `errors_try_first_match_wins`,
  `errors_ensure_all_exits`, `errors_ensure_cleanup_supersedes`,
  `errors_result_combinators`, `errors_result_unwrap_bridges`,
  `errors_attempt_bridge`, plus the two graduated ones), +1 NEGATIVE
  (`compile-errors/compile_error_throw_non_error_literal.ph`), +1 NEGATIVE
  (`runtime-errors/runtime_error_throw_uncaught.ph`), -2 PENDING (the retired
  `errors/pending/` pair). `errors` moves `check_pending` → `check_pass`.
- **Wren-suite `bool`/`null`/`class`/`function`/`object`/`system` port delta**
  (`wren/test/core/{bool,null,class,function,object,system}/`, ~28 files):
  +14 PASS — `booleans/`: `bool_equality`, `bool_not`, `bool_to_string`,
  `bool_isa_type`; `absence/`: `absence_none_isa_type`; `classes/`:
  `class_equality`, `class_name`, `class_supertype`; `functions/`:
  `functions_block_arity`, `functions_block_type`, `functions_block_equality`,
  `functions_block_to_string`; `system/`: `system_print_dispatches_tostring`,
  `system_print_returns_none`. +6 NEGATIVE — `absence/negative/`:
  `absence_none_no_constructor`, `absence_none_not_operator_dnu`;
  `functions/negative/` (new subdir, wired `check_negative` in `lang.rs`):
  `functions_call_extra_arguments`, `functions_call_missing_arguments`,
  `functions_call_runtime_error`; `runtime-errors/`:
  `runtime_object_not_operator_dnu`. `object/`'s content folded into existing
  labels (no new `object` label) — `is.wren`/`type.wren` duplicate
  already-pinned `metaclass`/`reflection` fixtures (skipped), `to_string.wren`
  duplicates `values/value_object_default_tostring.ph` (skipped), `same.wren`
  has no analog (no `Object.same`/identity-vs-`==` primitive exists, skipped),
  `not.wren` ported to `runtime-errors/runtime_object_not_operator_dnu.ph`,
  `nonclass_on_right.wren`/`no_constructor.wren` skipped (Phalcom's `isA`
  with a non-class RHS returns `false` per ADR-0023 I-4 rather than erroring,
  and `Object.new()` succeeds rather than rejecting — both design
  divergences, not portable as either PASS or NEGATIVE without misrepresenting
  intent). `class/no_constructor.wren` and `class/type.wren` skipped likewise
  (`Class.new()` silently succeeds — divergence flagged in
  `docs/forge/DEFERRED.md`; the metaclass self-loop tower is already pinned
  AS-OBSERVED-DIVERGENT by `metaclass/metaclass_metaclass_of_metaclass_is_a_class.ph`).
  `function/new_wrong_arg_type.wren` skipped (no `Fn.new` constructor — blocks
  are literals only). `system/print_all*`/`write_all*`/`*_bad_to_string.wren`
  skipped (no `System.printAll`/`writeAll`/`write`, and `print` does not
  validate a `toString` override's return type). `bool/no_constructor.wren`
  skipped — `Bool.new()` (zero-arg) **panics** (`args[0]` index-out-of-bounds,
  `primitive/boolean.rs:34`) rather than raising a catchable error, so it
  cannot satisfy the harness's `assert_no_panic` NEGATIVE contract; filed as a
  correctness bug in `docs/forge/DEFERRED.md` instead of ported.
- **U-FIBER-REFLECT delta (this unit):** `Fiber#isDone`/`Fiber#error` land as
  pure reads over `FiberObject::status`/`result` (no scheduler dependency,
  `primitive/fiber.rs` + `universe/primitives.rs`) — +3 new PASS goldens
  (`concurrency_fiber_is_done_false_while_suspended`,
  `concurrency_fiber_is_done_true_once_done`,
  `concurrency_fiber_is_done_and_error_once_failed`) plus
  `concurrency_fiber_wren_is_done_and_error` graduated `pending/` -> PASS
  (its final `.error.message` read updated to unwrap the now-`Option`
  `error` via `match`, since the pending fixture predated the
  `Option`-wrapping decision). Net: +4 PASS, -1 PENDING in `concurrency`.
- **Case counts (RECONCILED 2026-07-12, post-U15):**
  PASS 297 · NEGATIVE 42 · PENDING 28 · **total 367 cases** (375 `.ph` files under
  `tests/lang` by raw `find`, minus `imports/lib/`'s 8 non-case library fixtures =
  367 harness-visible `.ph` files; PENDING = `*/pending/*.ph`; NEGATIVE =
  runtime-errors + compile-errors + syntax-errors + collections/negative +
  imports/negative lanes). Pre-U15 baseline was PASS 292 · NEGATIVE 40 · PENDING 28 ·
  total 360. The stale 163/34/32/229 line and its per-delta narrative below are
  superseded history.
  Net since 229: +91 adversarial goldens (waves 1-3: OO/collections/closures/absence,
  arithmetic/booleans/reflection/bindings/system, concurrency) + U-ERR's errors surface.
- **(historical)** PASS 163 · NEGATIVE 34 · PENDING 32 · **total 229** (U13
  hierarchy-stability policy, DEC-U13a=A: +1 NEGATIVE —
  `runtime-errors/runtime_error_superclass_reparent_rejected.ph` pins the
  sealed-hierarchy reject, `Can't set superclass of a class`, as a catchable
  runtime error rather than a panic, never a mutation of the class graph.
  Prior to that, +3 PASS —
  U-FUTURE Slice A (pure `.ph` settle-once `Future`; `../../../docs/work/pending/fiber-schedule/future/plan.md`):
  `concurrency/concurrency_future_value_error_isready.ph` (C-FUT-1 settled half,
  C-FUT-8), `concurrency/concurrency_future_settle_once.ph` (C-FUT-3), and
  `concurrency/concurrency_future_then_map_catch_settled.ph` (C-FUT-4 settled-only
  half); the suspending half — `async`/`await`, pending-continuation drain — is
  Slice B, gated on DEC-FUT-SCHED and unowned `U-SCHED`, and stays out of this
  corpus for now (see `concurrency/pending/concurrency_future_async_await.ph`);
  the U-ITER deferred item 5 fiber x generator fixtures `iteration/for_generator_suspends.ph`
  (C-ITER-8) and `concurrency/each_generator_raises.ph`, cut from U-ITER and graduated now
  that U-FIBER has landed, over the `U-COLLTYPES` `{k:v}`-map-literal wiring, landed after
  Phase 3; +1 PASS /
  -1 NEGATIVE — `negative/map_literal_pending.ph` retired (the map literal no
  longer raises a "pending" diagnostic) and replaced by
  `map_literal_construction.ph`. Phase 3 (`Range`) was +1 PASS over Phase 2
  (new `range_basics.ph`) — Phase 2 was +2 PASS / -1 PENDING over Phase 1
  (`literal_tuple.ph` graduated `pending/` → PASS plus a new `tuple_basics.ph`)
  — Phase 1 was +1 PASS / +1 NEGATIVE over the `U-COLL` collection-literal
  landing, itself +4 PASS / +2 NEGATIVE / +2 PENDING over the
  `U-INH`/`U-ITER`/`U-FIBER` three-worktree consolidation).
- **U-SCHED (native ready-queue + root-drive pump, `docs/forge/units/U-SCHED-FIBER/U-SCHED/plan.md`):**
  +6 PASS in `concurrency/` — `concurrency_sched_schedule_does_not_run_synchronously`
  (`System.schedule(_)` enqueues, does not run), `concurrency_sched_fifo_order`
  (three scheduled fibers run in enqueue order), `concurrency_sched_root_drive_runs_at_exit`
  (`VM::run`'s belt-and-suspenders drain fires even though `main` never calls
  `runScheduled`), `concurrency_sched_raising_fiber_does_not_abort_host` (an
  uncaught raise in one scheduled fiber, captured via `fiber_try`, does not stop
  a later one or the host's own exit), `concurrency_sched_next_scheduled_empty_is_none`
  (`System.nextScheduled` on an empty queue is the `None` singleton, never a raise),
  and `concurrency_sched_run_scheduled_drains_including_nested` (`System.runScheduled`
  drains to exhaustion mid-program, including a fiber a running scheduled fiber
  itself schedules). Ratifies DEC-FUT-SCHED (`U-FUTURE/plan.md` §9): `Future`'s
  eventual Slice B (`async`/`await`) now has this unit's ready-queue/drain seam
  as an owned, landed precondition — `concurrency/pending/concurrency_future_async_await.ph`
  itself stays PENDING (Slice B is separate follow-on work).
- Active suites (`cargo test -p phalcom-core --test lang`) are green; PENDING run only
  under `-- --ignored` and are expected to fail until their feature is implemented.
- Baseline recorded 2026-07-11 against `./target/debug/phalcom` at commit `037da3d`; the
  `absence` lane was reconciled at the U6 landing (`51f56e4`) — +7 PASS cases graduated
  (empty/value-less block & method bodies, false `ifTrue` branch, `print` result, root
  superclass, empty `match` none-branch, empty block call → all `<None instance>`).

## Label matrix

| Label | PASS | NEG | PEND | Harness | Spec anchor |
|---|---:|---:|---:|---|---|
| arithmetic | 18 (Wren-number-port: `number_zero_comparison`, `number_div_negative_zero`, `number_equality_cross_type`, `number_to_string`, `number_from_string`, `number_mod_precedence`) | – | 14 (Wren-number-port `pending/`: `number_abs`, `number_sqrt`, `number_floor`, `number_ceil`, `number_round`, `number_sign`, `number_pow`, `number_min_max`, `number_clamp`, `number_truncate`, `number_fraction`, `number_is_nan`, `number_is_infinity`, `number_is_integer` — Number instance methods beyond the `+ - * / % < <= > >= negated toString hash` floor, `phalcom-core/src/primitive/number.rs`) | `check_pass` + `check_pending` | values-and-absence.md; messages-and-selectors.md; control-flow.md |
| lexical | 10 | – | 7 | `check_pass` + `check_pending` | lexical-structure.md; values-and-absence.md; selectors.md |
| classes | 23 (Wren-class-port: `class_equality`, `class_name`, `class_supertype`) | 2 | 2 | `check_pass` + `check_pending` | classes.md; object-model.md; ADR-0011; ADR-0017 |
| inheritance | 8 | – | – | `check_pass` | object-model.md §5.1; method-lookup.md §1.14; ADR-0002; ADR-0040 |
| messages | 7 | – | 2 | `check_pass` + `check_pending` | messages-and-selectors.md; selectors.md; object-model.md |
| system | 8 (Wren-system-port: `system_print_dispatches_tostring`, `system_print_returns_none`) | – | 2 | `check_pass` + `check_pending` | system.md |
| bindings | 3 | – | 2 | `check_pass` + `check_pending` | values-and-absence.md; open-questions.md; ADR-0014 |
| control-flow | 3 | – | 5 | `check_pass` + `check_pending` | control-flow.md; blocks.md |
| dispatch | 3 | – | 5 | `check_pass` + `check_pending` | messages-and-selectors.md; method-lookup.md; object-model.md |
| metaclass | 2 | – | 1 | `check_pass` + `check_pending` | object-model.md |
| list | 9 | – | 3 | `check_pass` + `check_pending` | U-LIST-plan.md; ADR-0019; ADR-0020; collection-protocol.md §2 (U-SEQ, pending) |
| bytes | 5 (U-BYTES: `bytes_basics`, `bytes_bulk_ops`, `bytes_strings`, `bytes_equality_and_keys`, `bytes_iteration`; the law-8 yield row lives in `concurrency/concurrency_fiber_yield_through_block_call`) | 9 (`bytes/negative/`: every precondition raise of bytes.md law 1 — bad octet ×3, OOB set, bad fill/slice/copyInto, non-`Bytes` `equalsConstantTime`, `Bytes`-as-`Map`-key rejection) | – | `check_pass` + `check_negative` | bytes.md; PDR-0011; PDR-0013 ruling 4; collection-protocol laws 3/4 |
| collections | 32 | 9 | 3 | `check_pass` (+ `check_negative`, `check_pending`) | U-CORE-5 as-built.md; U-COLL: lexical-structure.md §4/§6/§7/§8; ADR-0029; ADR-0032; U-COLLTYPES: map-and-set.md; tuple-and-range.md; ADR-0039 |
| iteration | 9 | – | 2 | `check_pass` (+ `iteration_disasm`, `check_pending`) | ADR-0035; iteration.md; U-ITER specification |
| syntax-errors | – | 5 | – | `check_negative` | lexical-structure.md; implementation-status.md |
| runtime-errors | – | 11+1 (U-ERR: `runtime_error_throw_uncaught`)+2 (Wren-list-port: `runtime_list_not_operator_dnu`, `runtime_list_at_put_out_of_range`)+10 (Wren-number-port: `runtime_number_plus_operand_not_num`, `runtime_number_minus_operand_not_num`, `runtime_number_multiply_operand_not_num`, `runtime_number_divide_operand_not_num`, `runtime_number_mod_operand_not_num`, `runtime_number_lt_operand_not_num`, `runtime_number_le_operand_not_num`, `runtime_number_gt_operand_not_num`, `runtime_number_ge_operand_not_num`, `runtime_number_not_operand_not_bool`) | – | `check_negative` | messages-and-selectors.md; method-lookup.md; U-LIST-plan.md §3; ADR-0026; ADR-0041; error-handling.md §1/§4 |
| compile-errors | – | 12+1 (U-ERR: `compile_error_throw_non_error_literal`) | – | `check_negative` | values-and-absence.md; ADR-0014; ADR-0007; ADR-0021; object-model.md §5.1; ADR-0035 (break/continue outside loop); error-handling.md §1 |
| absence | 24 (Wren-absence-port: `absence_none_isa_type`) | 2 (Wren-absence-port: `absence_none_no_constructor`, `absence_none_not_operator_dnu`) | 3 | `check_pass` + `check_negative` + `check_pending` | values-and-absence.md; ADR-0007; ADR-0021; selectors.md |
| blocks | – | – | 3 | `check_pending` | blocks.md; functions.md |
| booleans | 11 (Wren-bool-port: `bool_equality`, `bool_not`, `bool_to_string`, `bool_isa_type`) | – | – | `check_pass` | control-flow.md |
| concurrency | 38 (Wren-fiber-port: 9 PASS cases; U-FIBER-REFLECT: +4 — `concurrency_fiber_is_done_false_while_suspended`, `concurrency_fiber_is_done_true_once_done`, `concurrency_fiber_is_done_and_error_once_failed`, and `concurrency_fiber_wren_is_done_and_error` graduated from `pending/`; U-SCHED: +6 — `concurrency_sched_schedule_does_not_run_synchronously`, `concurrency_sched_fifo_order`, `concurrency_sched_root_drive_runs_at_exit`, `concurrency_sched_raising_fiber_does_not_abort_host`, `concurrency_sched_next_scheduled_empty_is_none`, `concurrency_sched_run_scheduled_drains_including_nested`) | 8 (Wren-fiber-port: 5 NEG cases in `concurrency/negative/`) | 1 (`concurrency_future_async_await`, gated on U-SCHED/DEC-FUT-SCHED — U-SCHED itself has now landed as this row's own precondition; `Future` Slice B remains the open item) | `check_pass` + `check_negative` + `check_pending` | concurrency.md; ADR-0030; U-FIBER-REFLECT; U-SCHED |
| errors | 9 | – | – | `check_pass` | error-handling.md; result.md; ADR-0008/0031/0038 |
| functions | 7 (Wren-function-port: `functions_block_arity`, `functions_block_type`, `functions_block_equality`, `functions_block_to_string`) | 3 (Wren-function-port, new `functions/negative/`: `functions_call_extra_arguments`, `functions_call_missing_arguments`, `functions_call_runtime_error`) | 1 | `check_pass` + `check_negative` + `check_pending` | functions.md; selectors.md |
| imports | 5 | 2 | – | `check_pass` + `check_negative` | modules.md; object-model.md §4; ADR-0027; ADR-0045 |
| string | 5 | 2 (in `runtime-errors/`) | 2 | `check_pass` + `check_pending` | core/core-classes.md §String; object-model.md; Wren-suite port (`test/core/string*`) |

## Spec coverage

Every document in `docs/spec/` maps to at least one label:

| Spec doc | Labels |
|---|---|
| lexical-structure.md | lexical, syntax-errors |
| values-and-absence.md | arithmetic, bindings, absence, compile-errors |
| object-model.md | metaclass, dispatch, classes, inheritance |
| blocks.md | blocks, control-flow |
| functions.md | functions |
| messages-and-selectors.md | messages, dispatch, runtime-errors |
| selectors.md | messages/pending, lexical/pending, absence/pending, functions/pending |
| classes.md | classes |
| method-lookup.md | dispatch, runtime-errors, inheritance |
| control-flow.md | control-flow, booleans |
| error-handling.md | errors |
| concurrency.md | concurrency |
| system.md | system |
| U-LIST-plan.md | list, runtime-errors |
| lexical-structure.md §4/§6/§7/§8 (collection literals) | collections |
| modules.md | imports |

## Running

```sh
cargo test -p phalcom-core --test lang               # active PASS/NEGATIVE (green)
cargo test -p phalcom-core --test lang classes       # one label
cargo test -p phalcom-core --test lang -- --ignored  # PENDING spec targets (expected to fail)
```

## Notes

- The corpus deliberately mixes working regression guards with pending spec targets;
  the pending set is the executable to-do list for the remaining spec surface.
- Numbers are `f64`: `7/2` → `3.5`, whole results print without a decimal, `1/0` → `inf`,
  `0/0` → `NaN`.
- Three known bugs are pinned as pending spec targets: setter parameter name hardcoded
  to `value`; keyword-argument calls build selectors no method definition can match;
  user-defined `==(other)` never dispatched for instances (identity fallback).
- Adding a case: drop `<name>.ph` + `<name>.expected` in the label dir (or `pending/`);
  create the label dir first if new (a missing dir panics `collect_cases`), and wire the
  label in `../lang.rs` if it has no `check_*` test yet.
- **Multi-file fixtures (U15 precedent, `imports/`):** a case that needs companion
  files an `import` statement loads (not a standalone case in its own right) puts them
  in a `<label>/lib/` subdirectory. `collect_cases` only reads files directly inside
  the label dir (`path.is_file()`, no recursion), so `lib/` is invisible to the
  harness — the driver `.ph` case references its dependency with a path relative to
  *its own* directory (e.g. a case in `imports/` writes `import "./lib/x"`; a case in
  `imports/negative/` writes `import "../lib/x"`).
- U6/PDR-0033 (absence → immediate `Option` + `let`/`var`): surface `nil` was removed, so the old
  `lexical_nil_prints` / `system_print_nil` PASS cases became the single
  `compile-errors/compile_error_surface_nil` NEGATIVE (they were byte-identical), and the
  former `binding_let_reassignment` PASS became `compile_error_let_reassignment` (reassigning
  a `let` is now a compile error). The `compile-errors` lane holds compile-time semantic
  diagnostics (surface `nil`, `let` no-initializer, `let` reassignment, `Option` truthiness).
  The `absence`/`bindings` `pending/` cases stay pending: they pin the final surface (a pretty
  `None` printString and `Some(x)` sugar) are now PASS. Canonical construction is
  `Some(x)`/`Some.call(x)`; `Some.new(x)` remains only in compatibility coverage.
  `None` and all `Some` layers are immediate; no Option wrapper allocates.
- **U-SEQ (sequence combinators + lazy views):** +16 PASS cases in `sequence/` lane —
  `sequence_all_true`, `sequence_all_false_short_circuits`, `sequence_any_true_short_circuits`,
  `sequence_any_false`, `sequence_count_arity0`, `sequence_count_predicate`, `sequence_find_hit`,
  `sequence_find_miss_returns_none`, `sequence_join_default`, `sequence_join_custom_sep`,
  `sequence_join_empty_collection`, `sequence_tolist_from_range`, `sequence_tolist_from_view`,
  `sequence_mapview_basic`, `sequence_whereview_basic`, `sequence_skipview_basic`,
  `sequence_takeview_basic`, `sequence_takeview_repeatable` (law-2 compliance),
  `sequence_view_over_map_yields_keys` — and +2 NEGATIVE — `sequence_skip_negative_count_raises`,
  `sequence_take_non_number_count_raises`. Tests the combinator breadth (`all(where:)`/`any(where:)`/`count`/`count(where:)`/
  `find(where:)`/`join`/`join(sep)`/`toList`) plus the explicit `.iter` pipeline
  (`MapIterator`/`FilterIterator`/`SkipIterator`/`TakeIterator`), building over the cursor protocol
  (ADR-0048) and `Iterable` root. Predicate queries use labeled `where:` selectors; direct collection
  transforms are eager.
