# Current verification status

The active Hypothesis kernel has not been runtime-verified after the current
provider, strategy, search, and shrinking changes. The runtime suite was not run
for this implementation pass.

The previous recorded runtime output predates the compiler changes
`881bbe042347` and `03df529c4f54`; it does not establish current behavior. That
full historical report and the old release-verifier results are retained in
[`archive/legacy-0.1.0/historical-TEST-RESULTS.md`](archive/legacy-0.1.0/historical-TEST-RESULTS.md).

The active runtime fixtures can be run from the repository root with:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build -p phalcom-core --bin phalcom
python3 packages/hypothesis/scripts/run_runtime_tests.py \
  --phalcom target/debug/phalcom --probe-blocked
```
