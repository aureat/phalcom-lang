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
| `phalcom-core/src/compiler/lib.rs` ClassDef field-slot build (~line 791) | correctness | Subclass field-slot assignment does **not** dedup field names already declared by the superclass: a subclass that reuses an *inherited* field name gets a fresh slot (`sc_field_count + i`) instead of the parent's offset. So `class Q extends P { construct new(x){ _v = x } }` where `P` owns `_v` + getter `v => _v` writes `_v` to slot 1 while `P`'s inherited getter reads slot 0 → returns nil (`Q.new(42).v` prints `None`). Repro confirmed. Orthogonal to the constructor guard (field layout, ADR-0011). Fix: when building `field_slots`, reuse the superclass layout's slot for a shadowed name rather than allocating a new one. Owning unit: U-INH field-layout follow-up (candidate U13). |
| `phalcom-core/src/compiler/lib.rs:~1081` (`has_new_construct` guard) | correctness | Guard is keyed on the receiver class name only, **not** inheritance-aware. A subclass that *inherits* a `new`-constructor but declares none is absent from `has_new_construct`, so a wrong-arity `Sub.new(...)` (e.g. `B.new()` when the only ancestor ctor is `new(t)`) silently falls through to the `Object.class::new` bare allocator and returns an **uninitialized** instance instead of the "No constructor matches" error the declaring class raises. Unique to `new` (named ctors have no bare-allocator fallback → they dNU, safe). Matching-arity inherited ctors already resolve correctly via `value.rs:128` `lookup_method`'s `init `-prefix metaclass walk — this is *only* the guard gap. Fix: walk the superclass chain in both the guard **and** the `constructor_aliases` lookup; needs a compile-time name→parent map (populate at ClassDef superclass resolution, `lib.rs:~764`). Owning unit: U13 (hierarchy policy). |

## Homed entries

Every other deferral has been homed in its owning unit's plan — each carries an
**Adopted debt** note in its write-set section:

| Debt | Owning unit |
|---|---|
| `primitive/number.rs:~34` — type-error message hardcodes `"value"` | [U12](units/U12/plan.md) §3 |
| `primitive/nil.rs:~64` — broken rustdoc link → private `wrap_some` | [U-ERR](units/U-ERR/plan.md) §3 |
| `core/README.md` — stale floor baseline (80/64, should track 88) | [U-ERR](units/U-ERR/plan.md) §3 |

Add a new entry here **only** when a debt has no plausible owning unit; otherwise
fold it into the relevant `units/<U>/plan.md` write-set as an **Adopted debt** note.
