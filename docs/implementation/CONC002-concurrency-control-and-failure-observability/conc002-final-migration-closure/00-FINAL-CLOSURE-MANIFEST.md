---
kind: migration-closure-manifest
prepared: 2026-09-12
repository: aureat/phalcom-lang
---

# Final CONC002 restructuring closure manifest

Verified remaining gaps after the C1-C4 plan files were added to `main`:

1. CONC002 `PROGRAM.md` still says C2.P2 is unpublished and C3/C4 are only planned.
2. CONC002 `STATUS.md` says the same.
3. The C2 checkpoint still labels P2 planned and incorrectly assigns structured concurrency/channels/select to C4.
4. The legacy executable C3 checkpoint/plans remain `PROPOSED`.
5. Both `CONC001/C2-scheduler-and-reactor` and the new historical `CONC001/C2-scheduler` coexist.
6. `stdlib/reactor.md` still requires the pre-C2 guest completion-pump design and links a missing U-REACTOR implementation record.
7. `system.md` and several live stdlib/PDR records still point to obsolete implementation owners.

This bundle closes those migration gaps. It does not create C5/C6 implementation plans.

Apply order:

1. Replace CONC002 `PROGRAM.md` / `STATUS.md`.
2. Rename C2 to `C2-vm-suspension-and-coroutine-semantics` and replace its checkpoint.
3. Mark legacy C3 plans SUPERSEDED.
4. Remove/archive duplicate legacy CONC001 C2 after link migration.
5. Adopt the reactor/system normative replacements.
6. Apply live link patches.
7. Run `VERIFICATION-REPORT.md`.
