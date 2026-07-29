# Checkpoint Artifact Protocol

Every implementation phase ends with one full project archive.

## Naming

```text
phalcom-hypothesis-phase-NN-<slug>.zip
```

The final release is:

```text
phalcom-hypothesis-complete.zip
```

## Required contents

Each archive includes:

- the entire current project tree;
- `CHECKPOINT.md`;
- `TEST-RESULTS.md`;
- `CHANGELOG.md`;
- `SHA256SUMS`;
- all tests accumulated through that phase;
- all examples accumulated through that phase.

## Response contract

The phase completion response provides:

- the archive download link;
- the checkpoint report link;
- the test-results link;
- a concise change summary;
- exact verification limitations, when present.

A delta patch may be included as an additional convenience, but never replaces the full archive.
