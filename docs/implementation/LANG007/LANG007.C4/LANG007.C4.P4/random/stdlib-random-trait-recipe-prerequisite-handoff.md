# Trait recipe repair handoff

Historical prerequisite handoff. Superseded for continuation by
`stdlib-random-completion-handoff.md`; retain the established interfaces below.

Main, starting HEAD `800474d8`. Stable owner repair is committed in `fbf057ea`.
Recipe repair is IMPLEMENTED / FOCUSED_TESTED, uncommitted. Original random R0
syntax probe passes unchanged. No subagents or commits were used in this repair.

Read the checkpoint's current random prerequisite state and
`stdlib-random-implementation-incident.md`. Historical blocked recipe entries
are superseded by its current outcome.

## Stable takeover interfaces

- `CheckingContext::resolve_callable_signature` consumes the attached signature
  table or existing conformance view and records CallableSignature dependency.
- `publish_call_specialization` emits selected source method binder IDs after
  checked requirement inference; exact/forwarded proof keys remain instantiated.
- `TraitInvocationSpec::call_environment_recipe` carries the compiler's existing
  generic call-edge environment into selected trait execution.
- Pool remapping traverses the recipe; activation merges owner/method bindings
  and materializes canonical evidence slots. Abstract default slots are offset
  after the complete generic prefix. Witness identity validation remains active.

Do not reintroduce compiler inference, selector-based proof fallback, fake
declaration ownership, or exact-specialization callable identities.

## Next random action

The local source trait gap is closed. A removed provisional library slice found
separate canonical Universe blockers: module integer constants lack successful
static lowering, and new source Universe conformance/default products do not
reach typed client execution. The original plan requires consultation for the
static LargeInt gap. Private xoshiro fields must stay class-owned; accepted trait
rules permit an inherent `nextU64` witness but deny conformance private-field
authority. No provisional random production API remains.

First commands:

```sh
git status --short
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus random -- --nocapture
RUSTFLAGS='' cargo test -p phalcom-core --test core language::traits -- --nocapture
```

Four existing semantic trait tests are baseline failures, confirmed with this
repair's semantic production changes temporarily removed. The incident records
their selectors and causes. Preserve unrelated checkout work.
