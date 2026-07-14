// area: strings
// spec: core-classes.md §String; docs/forge/units/U-STRING/plan.md §2.3
// status: NEGATIVE
// split(_) rejects a non-String delimiter with a clear ArgumentError
// instead of failing deep inside rawByteCount/rawByteAt dispatch.

"a,b".split(5)
