# Dynamic-Language and Reflection Analysis

## Open world

Dynamic languages allow behaviors not visible from one lexical body:

- method lookup by runtime receiver;
- reflection/dynamic perform;
- module imports;
- native code;
- method-table mutation where allowed;
- dynamic packs/selectors.

Static analysis must model uncertainty rather than assume the currently indexed source is closed.

## Receiver analysis

A dynamic send can be analyzed when receiver abstract value is constrained. Resolve the canonical selector across possible receiver classes and join results/effects.

If receiver is top/dynamic, completion can still offer heuristic/common members, but correctness checks must not claim a guaranteed target.

## `doesNotUnderstand`

Missing ordinary selector may not mean immediate VM failure if the language routes through a missing-message hook. Static semantics should model the specified hook while diagnostics may still flag likely errors.

## Reflective perform

If selector value is exact, analysis can resolve it like an ordinary send with reflective access rules. If selector is a bounded set, analyze each. If unknown, dynamic effect.

## Method mutation

If reflective APIs can add/replace methods, closed-world call targets become generation-dependent. Use class/member revision assumptions and invalidate.

## Dynamic labels/packs

Computed labels/expansions can make concrete selector shape unknown until runtime. Represent selector family/unknown arity rather than fabricating one signature.

## Native primitives

Treat native signatures/effects as trusted summaries only after contract validation. Otherwise they are dynamic boundaries.

## Advisory versus correctness modes

LSP may choose a heuristic to be helpful:

```text
receiver likely String -> suggest String members
```

Checker/prover must tag whether this is guaranteed. Share evidence but not confidence policy blindly.
