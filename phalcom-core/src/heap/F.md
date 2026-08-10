F.2 §15 still incomplete: evaluation-order proofs, fiber yield, full dynamic subscript lanes/collision ordering, dynamic arity 255/256, GC stress, dynamic compiler-internal authority, static disassembly, plus several generic-* and E.3 matrix rows.

---

Short version: these are evidence gaps, not confirmed runtime bugs.

- The boundedness fixture is deferred because `collections_pending` is ignored for the whole mixed pending directory. Its harness expects successful stdout, while `boundedness_*_unbounded_rejected.ph` correctly exits with compiler error on stderr. Running ignored test confirms this mismatch. [lang.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/tests/lang.rs:227), [support/mod.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/tests/support/mod.rs:168)

- E.3 behavior itself is covered by focused F.2 tests: unbounded spread rejects; bounded `take(3)` succeeds. [outgoing_packs_completion.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/tests/outgoing_packs_completion.rs:214)

- F.3 focused rest tests pass, but matrix closure is intentionally unclaimed. Open rows mean dedicated evidence is missing:

  - reflective duplicate-install path;
  - subclass non-accepting fallback and exact `super` boundary;
  - F.3-specific primitive-floor audit;
  - full GC-stress coverage;
  - before/after static-send benchmark.

  The implementation already contains installation validation and superclass-aware lookup, but coverage does not prove every required path. [dispatch.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/vm/dispatch.rs:257), [F.3 checklist](/Users/altunhasanli/dev/phalcom/phalcom/docs/work/pending/collections/F.3-rest-capture-and-rest-pattern-dispatch-amended.md:2044)

- GC has focused PackBuilder tracing proof; general `PHALCOM_GC_STRESS` remains deferred. Performance has disassembly evidence, not F.3 benchmark evidence. [F.2 gate](/Users/altunhasanli/dev/phalcom/phalcom/docs/work/pending/collections/F.2-outgoing-pack-assembly-and-dynamic-send-amended.md:2468)

- `variadic_send` and old paths remain in perf logs and superseded plans for historical provenance. They are not current F.3 acceptance evidence. [historical F.2 plan](/Users/altunhasanli/dev/phalcom/phalcom/docs/work/pending/collections/F.2-outgoing-pack-assembly-and-dynamic-send.md:3)

Focused checks run: rest dispatch, floor census, and PackBuilder GC passed; ignored collection pending lane failed at expected deferred negative-fixture classification.