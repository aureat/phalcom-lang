# CONC001.C2 — historical scheduler scope

This record supersedes the old `scheduler-and-reactor-scope-spec.md` as the live CONC001 scope summary.

CONC001.C2 owns historical provenance for the original ready queue/root-drive scheduler only.

Current implementation authority:

```text
scheduler hardening / executor semantics -> CONC002.C1/C2
reactor / worker completions / timers    -> CONC002.C3
Future library completion                 -> CONC002.C4
```

The old reactor worker/timer implementation record is no longer an executable plan.

Accepted PDR-0003/PDR-0004 and `docs/spec/current/stdlib/reactor.md` remain architectural/normative authorities independent of this program move.
