# Deferred improvements register

Out-of-scope optimizations / DX / speed / security observations noticed while
landing a forge unit, but deliberately not implemented in that unit. Each
entry: file:line, category, one-line rationale.

## Open entries

| file:line | Category | Rationale |
|---|---|---|
| `docs/spec/v0.2/object-model.md` §5:210-211; `docs/spec/.../implementation-status.md` | docs-drift | Note claims "every metaclass's superclass wired to `Class`, breaking it" — stale pre-U2; native tower now satisfies ADR-0002 rule 4 (tested) and U-INH extends the same rule to user classes. Re-point both. |
| `phalcom-core/src/bytecode.rs` (`SuperSend`) | perf / IC follow-on | `SuperSend` is uncached (DEC-INH-F). Wire the inline-cache seam **with U15/U16** so a `superclass=` (U15) / override-epoch bump (ADR-0018) invalidates a cached `SuperSend` the same way it invalidates `Invoke`. |
| `docs/forge/units/README.md`, phase INDEX | docs roster | Add the `U-INH` roster row (landed). Not edited in-unit — shared-file concurrent-session hazard. |

## Homed entries

Every other deferral has been homed in its owning unit's plan — each carries an
**Adopted debt** note in its write-set section:

| Debt | Owning unit |
|---|---|
| `vm.rs:107-110` — `impl Default for VM` is `todo!()` | [U-INH](units/U-INH/plan.md) §4 |
| `primitive/number.rs:~34` — type-error message hardcodes `"value"` | [U12](units/U12/plan.md) §3 |
| `primitive/nil.rs:~64` — broken rustdoc link → private `wrap_some` | [U-ERR](units/U-ERR/plan.md) §3 |
| `core/README.md` — stale floor baseline (80/64, should track 88) | [U-ERR](units/U-ERR/plan.md) §3 |

Add a new entry here **only** when a debt has no plausible owning unit; otherwise
fold it into the relevant `units/<U>/plan.md` write-set as an **Adopted debt** note.
