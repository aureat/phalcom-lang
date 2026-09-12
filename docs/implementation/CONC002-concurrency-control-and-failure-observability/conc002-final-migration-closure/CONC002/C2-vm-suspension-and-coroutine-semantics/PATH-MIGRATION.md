# C2 path migration

Rename:

```text
C2-native-suspension-and-reactor-groundwork/
```

to:

```text
C2-vm-suspension-and-coroutine-semantics/
```

Move the existing P1-R1 and P2 files unchanged except for relative links.

The old path is semantically stale because C2 owns no reactor backend.

Update current implementation links, but do not rewrite historical raw/wiki snapshots only for path aesthetics.
