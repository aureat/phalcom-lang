// area: system
// spec: system.md; ADR-0007
// status: PASS
// `System.print` returns `Unit`, not its argument. The nested comparison is
// therefore false while the side effect still prints the inner value.

System.print(System.print(1) == 1)
