# Enum inference audit-follow-up walkthrough

Branch: `feat/integration-universe`; base: `9b283a03`; date: 2026-10-14.
Corrective slice: **IMPLEMENTED / FOCUSED_TESTED**. Full requirements: **PARTIAL**.

The expression regression first failed with premature ambiguity for both orders
of `Boxed<Int> | Boxed<String>` and distinct payload-discriminated owners.
Shared owner/case discovery plus isolated candidate application repaired those
failures without changing lexical precedence or canonical associated targets.
The unique winner is replayed live; losing solver/row state and publication
products remain detached. The solver isolation regression and incremental
rejected-owner edit test verify those boundaries.

A symbolic union discriminator exposed an owning annotation defect: hole
detection accepted unions while elaboration silently dropped them. The approved
correction recursively preserves each union member in the existing inference
context. The payload now projects as Int and the constructor publishes canonical
resolution; its formerly reachable fallback became correctly redundant and was
removed with adviser approval, strengthening direct payload/identity assertions.

Private construction discriminators exposed missing access enforcement in both
associated specialization and invocation. Both now use the existing declaration
access relation. Tests cover private-candidate elimination, qualified denial,
singleton/reference acquisition and authorized owner-context construction.

Three consultations clarified probe isolation, approved the union elaboration
correction and approved construction access enforcement. No authority moved
between semantic/compiler/runtime layers. No accepted proof, assertion or DAG
invariant was weakened.

Evidence: contextual 22 passed; associated lookup 25 passed; matching 163 passed
with 16 existing ignored; isolated type/row solver 1 passed; linker 19 passed;
scoped diff check passed. Exact command/order details are in the owning repair
record. The executable discriminator compiled but failed before its fixture on
the existing Universe Error runtime-read bootstrap blocker. Execution is not
verified; broad/release checks and broader semantic companion contracts remain
open. No commit or push was performed.
