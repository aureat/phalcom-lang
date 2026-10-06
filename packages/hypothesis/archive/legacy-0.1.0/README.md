# Archived Hypothesis 0.1.0 material

Everything in this directory describes or supports the former monolithic
0.1.0 implementation. It is retained as historical design and migration
evidence; it is not the source tree, API contract, test suite, or verification
gate for the active package.

The current package lives in `../../src/`, with executable consumer fixtures
in `../../tests/runtime/` and the active runner in `../../scripts/`.

`historical-SHA256SUMS.txt` records checksums for the old package layout. The
files were relocated into this archive, so those original paths no longer
resolve and this file is not a checksum manifest for either the archive or the
current package. The archived `scripts/verify_*.py` files likewise encode the
old release surface and must not be used to validate the current kernel.
