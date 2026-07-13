// area: system
// spec: system.md; ADR-0007
// status: PASS
// Ported from Wren `test/core/system/print.wren` (second half): Wren's
// `System.print` returns its own argument (`System.print(1) == 1` is
// `true`). Phalcom's `System.class::print(_)` deliberately returns the
// `None` singleton instead (`primitive/system.rs`'s `system_class_print` —
// a statement-like send must never leak the raw `nil` sentinel, ADR-0007
// Invariant 4), so the analogous comparison is `false`.

System.print(System.print(1) == 1)
