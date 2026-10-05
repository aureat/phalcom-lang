// Phase 04 — malformed strategy domains fail at construction time.

import Assert from hypothesis
import Gen from hypothesis

Assert.isTrue(|| { Gen.int(min: 4, max: 3) }.attempt().isErr)
Assert.isTrue(|| { Gen.float(min: 1.0, max: -1.0) }.attempt().isErr)
Assert.isTrue(|| { Gen.bytes(minSize: 4, maxSize: 3) }.attempt().isErr)
Assert.isTrue(|| { Gen.sampledFrom(const []) }.attempt().isErr)
Assert.isTrue(|| { Gen.oneOf() }.attempt().isErr)
Assert.isTrue(|| { Gen.list(of: Gen.int, minSize: 4, maxSize: 3) }.attempt().isErr)
Assert.isTrue(|| { Gen.set(of: Gen.int, minSize: -1, maxSize: 3) }.attempt().isErr)
Assert.isTrue(|| { Gen.map(keys: Gen.int, values: Gen.int, minSize: 5, maxSize: 2) }.attempt().isErr)

System.print("PASS invalid strategy construction")
