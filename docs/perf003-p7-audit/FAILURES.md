# PERF003.C2.P7 failures and deferred limits

## Current observed failures

### Runtime suite — classified, not passed

The release runtime suite recorded 20 attempts: 10 source-backed and 10 artifact-backed. All are `RUNTIME_ERROR`. The CSV error-detail field is blank, so the underlying error is not known from this evidence. These rows are not used for runtime speed or successful-execution claims. See [RAW-RUNTIME.csv](RAW-RUNTIME.csv) and `logs/runtime-suite.txt`.

### Workspace commands — inherited baseline drift

`cargo test --workspace --no-run` and `cargo test --workspace` both stop while compiling the monolithic semantic integration target. It reports 412 API fixture errors involving old `BTreeMap` values where the current APIs require `AnalysisMap` and `ModuleAuthorityMap`. The same failure signature is recorded at clean predecessor `08e2b2f5`; detailed current logs are `logs/workspace-no-run.txt` and `logs/workspace-test.txt`. These fixtures were left unchanged because they are outside P7's focused acceptance surface.

## Resolved historical failure

The earlier `ProductKind` VM bootstrap error is not the current runtime-suite explanation. The branch corrected intrinsic facade dispatch and the SlotMap VM bootstrap regression passes in the focused core suite. The current runtime rows still fail after that bootstrap discriminator; their detail field is blank, so no more specific cause is asserted.

## Claims deliberately left unverified

- The peak RSS values do not prove retained-memory reduction or leak freedom.
- Runtime execution speed is not measured successfully.
- The broad workspace suite is not green because of inherited fixture drift.

These limits do not change the P7 compiler-image acceptance ruling: R.G6 and R.G7 are PASS on the focused evidence listed in the certification matrix.
