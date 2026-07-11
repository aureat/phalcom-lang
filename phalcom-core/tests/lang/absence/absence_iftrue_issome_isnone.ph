// area: absence
// spec: values-and-absence.md §3.3; ADR-0007; ADR-0018
// status: PASS
// U-CORE-2: `isSome`/`isNone` (core.ph, derived over `match`) sent directly
// to an `ifTrue` result — only well-typed post-U-CORE-2, since `ifTrue` now
// returns a well-formed `Option` (`Some(A)` taken / `None` untaken) instead
// of the pre-U-CORE-2 half-Option, which left a raw non-`Option` value on the
// taken arm with no `isSome`/`isNone` to send.

System.print(true.ifTrue { 1 }.isSome)
System.print(true.ifTrue { 1 }.isNone)
System.print(false.ifTrue { 1 }.isSome)
System.print(false.ifTrue { 1 }.isNone)
