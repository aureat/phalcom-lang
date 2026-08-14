# Recovery and Incomplete Programs

## Editors analyze invalid programs constantly

Examples:

```text
half-written selector
missing closing delimiter
unfinished type application
unknown import while user creates file
method body with syntax error
incomplete pattern
```

Semantic analysis should remain useful.

## Recovery layers

Parser recovery creates source nodes/ranges.
Semantic recovery creates unresolved/partial identities/facts.
Checker recovery suppresses cascades after a primary error.

Do not make recovery sentinel a real language value/type.

## Partial declarations

A class header may be usable for completion even if one member body fails. Build surface independently from body success where possible.

## Unresolved names

Retain occurrence with unresolved reason and candidate scope context. This supports diagnostics and completion without inventing a `Global(String)` fact as certainty.

## Dynamic versus unresolved

A genuinely dynamic operation is valid language semantics. An unresolved name due to broken source is an editor/error state. Keep separate.

## Range quality

Recovered nodes still need precise token/source spans for hover/rename targeting. Avoid highlighting whole methods when exact semantic occurrence is known.

## No panics

User syntax cannot be allowed to trip `expect` based on assumptions only true for fully valid AST. Internal invariants after validated lowering may use assertions.

## Incremental recovery

Removing a syntax error should replace the file's semantic contribution cleanly. No stale unresolved/candidate facts should survive.
