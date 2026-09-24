# Certification layer — Tasks 34–38

This directory turns the golden program into a repository-level regression gate
without hard-coding an unratified Phalcom compiler command line.

- `run-certification.sh` orchestrates all certification modes.
- `driver-contract.md` defines the tiny adapter surface the real repository must provide.
- `scenarios.tsv` is the scenario/acceptance manifest.
- `expected-output.txt` is the stable Task-38 runtime oracle.
- `../negative/manifest.tsv` is the Task-34 diagnostic oracle.

Tasks 35–37 are implemented here as executable scenarios but are not represented
as having passed until the repository supplies a driver and runs them against the
actual compiler/cache/persistence pipeline.
