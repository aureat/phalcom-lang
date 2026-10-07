# `universe.random` performance checkpoint

These self-checking programs measure the source-authored generator against
the source MINSTD baseline already used by the math benchmark corpus. They are
manual benchmarks, not CI tests.

Build the optimized CLI once, then run each program in a fresh process. On
macOS, `/usr/bin/time -l` also reports peak resident memory; use the platform's
equivalent resource report elsewhere.

```sh
cargo build --release -p phalcom-core --bin phalcom
for file in benchmarks/random/*.ph; do
  /usr/bin/time -l target/release/phalcom "$file"
done
```

Every program prints `true` only after checking that its workload produced a
valid result. The loop counts match the implementation plan: one million
`nextU64`, `nextFloat`, and bounded integer calls; 100,000 128-bit bounded
samples; 1 KiB of bytes; and shuffles of 100 and 10,000 elements. The MINSTD
program provides a one-million-call source baseline.

The random source uses exact arbitrary-precision `Int` arithmetic to preserve
its frozen 64-bit stream. Values above the immediate signed range and temporary
masked arithmetic can allocate `LargeInt` objects. There is no per-call
allocation counter exposed by the current VM, so the measured memory figure is
whole-process peak RSS, not an allocation count. `random_foundation.ph` also
forces collection while retaining the generator and verifies the subsequent
reference outputs.

Current checkpoint: optimized syntax startup passes after the general
expression-owner stack repair. The first million-word run was interrupted at
225.51 seconds; sampling exposed idle OS polling. The reactor repair passes
its tests, but the final release rebuild and renewed benchmark runs remain
pending. No completed workload throughput or allocation counts are claimed.
See the random completion handoff for continuation instructions.
