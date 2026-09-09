# AUD-SEM-007 — Subtype pair budget does not account for recursive work

## Classification

**Severity:** Medium. **Category:** Termination / Performance. **Status:** Confirmed; see the evidence qualification below. Audited HEAD `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`, 2026-09-09. No implementation fix.

## Location

`phalcom-semantic/src/types/relation.rs:205–247` (`check_subtype_bounded`, `check_subtype_impl`); `phalcom-semantic/src/types/outcome.rs:62–91` (budget dimensions/defaults).

## Observed behavior

A tuple comparison with a nested Int→Number obligation returns Proven with max_relation_pairs=1 and pairs_checked=1, despite visiting two distinct pairs. The retained assertion fails. The recursive path charges steps but never pairs and does not consult max_type_depth.

## Expected invariant

Advertised relation-pair and depth limits must bound the actual nested work they describe. Exhaustion must remain distinguishable from a mathematical refutation.

## Root cause

Only the public entry charges a pair. Recursive calls use check_subtype_impl, which charges a step and recurses without depth accounting. Generic supertype materialization/substitution can do additional work before re-entering that step budget.

## Minimal reproducer

`recursive_relation_charges_pair_budget` in [the probes](evidence/semantic_audit_probe.rs).

## Impact

The configured pair limit does not constrain structural expansion; depth safety remains unproven. Default step limits still exist, so this is not evidence of unlimited recursion or an observed stack overflow.

## Fix direction

Charge distinct obligations at recursive entry and enforce depth before descending, preferably with an explicit work stack. Thread shared control through substitution/materialization and retain terminal outcomes in inference instead of bool collapse.

## Tests required

Pair limit 0/1/2, depth below/at/above limit, cancellation mid-query, expanding generic supertype, shared DAG growth, and recovery after blocked comparison. Do not fix this solely by lowering the step count.

## Evidence and limitations

See [retained evidence](evidence/README.md) for exact probes, commands and completed results. Semantic invariant failures are expected audit failures, not fixed behavior. IDs 001–002 have source-analysis plus executable runtime evidence; 003 has static and runtime evidence; 004–006 have pure/source semantic evidence; 007 has an executed pair-accounting counterexample and source-led depth concerns. No workspace-release certification is claimed.
