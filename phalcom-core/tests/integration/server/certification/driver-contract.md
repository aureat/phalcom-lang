# Phalcom golden certification driver contract

The golden package deliberately does not guess the repository's compiler CLI,
module graph, artifact directory, or diagnostic transport. Repository integration
supplies one executable adapter through `PHALCOM_GOLDEN_DRIVER`.

The adapter receives one command followed by the absolute package root:

```text
clean-build ROOT
persistence-build ROOT
incremental-noop ROOT
incremental-leaf ROOT
incremental-implementation ROOT
incremental-signature ROOT
negative-corpus ROOT
```

Every command must exit zero only when its scenario meets the contract below.

## `clean-build`

1. Remove semantic/incremental artifacts belonging to the golden package.
2. Compile all positive sources from source.
3. Link/run `golden/main.ph`.
4. Compare stdout with `certification/expected-output.txt`.

## `persistence-build`

1. Compile and persist the foundational surfaces (`domain`, `effect`, `http`, and
   application interfaces).
2. Start a fresh semantic/compiler session.
3. Compile application/testing/golden consumers from persisted imports.
4. Run and compare the same stable output.

The persisted representation must retain static applications (`Paid`, `Ok`,
`Count(3)`), HKT kind information, associated types, `Command<R>` case/result
relationships, record rows, tuple packs, and sequence packs.

## Incremental scenarios

`incremental-noop` rebuilds with no source change and checks that reuse does not
change semantics or output.

`incremental-leaf` changes only a golden leaf implementation and verifies that
unrelated semantic dependencies are not invalidated unnecessarily.

`incremental-implementation` changes a fixture implementation without changing
its exported type surface; public semantic identity must remain stable.

`incremental-signature` applies a public signature mutation (for example
repository `Id = OrderId` -> `UserId`, or gateway `Count(3)` -> `Count(4)`) and
must prove that stale dependents are invalidated and the formerly-valid consumer
is rejected.

## `negative-corpus`

Compile every fixture in `negative/manifest.tsv` independently. Each must fail in
the listed phase/semantic class for the listed principal reason; an unrelated
earlier failure is not acceptance evidence.
