// area: compile-errors
// spec: decorators/sealed.md; drafts/sealed-classes.md §1.3 / S-2;
//       ADR-0045 (whole-module binding); DEFERRED CB-3
// status: NEGATIVE
// **This fixture pins a gap, not a guarantee.** DEFERRED CB-3 / S-2 asked for a
// test of `@sealed`'s headline enforcement: a cross-unit subclass of a *user*
// sealed class raising `attr.sealed_violation`. **That test cannot be written** —
// the scenario is unreachable, on two independent grounds:
//
//  1. **Ordering.** `extends` resolves its superclass at COMPILE time; `import`
//     binds the module at RUNTIME. The error below fires before the imported
//     module is ever loaded — verified: give `lib/sealed_shape.ph` a
//     `System.print` side effect and it never runs. An imported class therefore
//     cannot be a superclass at all, sealed or not.
//  2. **Naming.** Even without (1): `extends S.Shape` does not parse (`extends`
//     takes a bare identifier, not a member access), and ADR-0045 binds the
//     whole module to `S` while leaking no globals — so this unit has no bare
//     `Shape` to name.
//
// So `attr.sealed_violation` is **dead for user classes**: the protection
// `@sealed` advertises is already supplied by module structure, and the check is
// reachable only for classes visible in every unit's globals at compile time —
// i.e. the bootstrap-sealed kernel (`Option`/`Some`/`None`), which is exactly
// what `annotation_variant_in_bootstrap_sealed_class.ph` exercises. `@sealed`'s
// only *live* effect on a user class today is gating `@variant`.
//
// The error below is therefore about SCOPE, not sealing. **If cross-module class
// references ever land, this fixture must change** — its expected output should
// become `attr.sealed_violation`. That is the point of keeping it: it fails
// loudly the day the assumption changes.

import "../imports/lib/sealed_shape" as S

class Square extends Shape {}
