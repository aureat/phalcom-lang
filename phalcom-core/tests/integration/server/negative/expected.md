# Expected-failure corpus

Every file in this directory is an independent mutation. Acceptance requires the
principal semantic failure listed in `manifest.tsv`; merely failing compilation
for an unrelated earlier reason does not count.

The corpus covers:

- typestate specialization,
- static status/retry/domain identity,
- higher-kinded kind checking,
- associated-type constraints,
- open/exact row behavior and row occurs checks,
- ordinary and cross-domain inference cycles,
- GADT-style command branch refinement and exhaustiveness,
- tuple/sequence packs,
- closure result inference,
- handler function compatibility,
- Fiber result specialization,
- post-inference witnesses,
- and runtime `is` refinement scope.

`manifest.tsv` is the machine-oriented oracle containing phase, semantic class,
and principal reason for every fixture.
