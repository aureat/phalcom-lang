# Retained evidence

- [Original audit prompt](original-audit-prompt.md): complete requested scope.
- [Diagnostic probe](runtime_audit_probe.rs): exact Rust program used to reproduce current behavior.
- [Probe output](probe-output.txt): completed run, including intentionally caught panic messages.
- [Chunk test output](chunk-tests.txt): 10 focused tests passed.

The probe was temporarily placed at `phalcom-core/examples/runtime_audit_probe.rs` and executed using:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-core --example runtime_audit_probe
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib chunk::tests
```

The temporary example was removed after preserving it here. To rerun, copy the evidence source to an unused example path and execute that example; do not overwrite another contributor's file. Expected current probe results are explicitly buggy behavior, not passing regression assertions. Convert them to invariant assertions during remediation.

The full-Universe super reproduction uses `VM::new`; literal, layout and missing-edge probes use kernel bootstrap, and the real numeric primitive probe uses native bootstrap. Generated source tests exercise the compiler API; no frontend validity claim beyond successful compilation is inferred for unrelated examples.

## Probe interpretation

- `CONSTANT input=65536 index=0`: representability violation.
- `SOURCE execution=Ok(0)`: observable compiler/VM wrong result.
- `GC lexical_owner_survives=false`: confirmed omitted strong edge with no alternative retaining root.
- `NATIVE recursion_40=Ok(40)`: bounded diagnostic primitive demonstrates native-only counter bypass.
- `SHIPPING_NATIVE wrong_arity_panics=true`: actual number primitive reached with an empty argument slice.
- `BYTECODE empty_chunk_panics=true`: internal bytecode validation is absent at execution admission.
- `SUPER before=Ok(1)` / `SUPER old_receiver_after_rebind=Ok(2)`: old receiver's super anchor follows new name binding.

## Native lifecycle continuation

[Probe source](native_lifecycle_probe.rs) and [completed output](native-lifecycle-output.txt) record four bounded kernel-native cases at HEAD `94b9f14361333dfda06cd8abd792f672d12db06a`. The shipping JSON traceback renderer demonstrates missing/misattributed native frames; the final caught-error/success case returns 7. No panic injection or runtime fix was used.

Executed serially with `RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-core --example native_lifecycle_probe` (exit 0). The temporary example was removed after execution. This is a diagnostic probe recording defects, not a passing regression test. No other Cargo validation was run in this continuation.

## Numeric contract continuation

[Exact probe](numeric_contract_probe.rs), [completed output](numeric-contract-output.txt): four valid arithmetic sends using native bootstrap. Executed with `RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-core --example numeric_contract_probe`, exit 0. Results record two numeric contract divergences, not passing regression assertions. The temporary example was removed; no other Cargo checks ran in this slice.
