# Current source dependency graph

The table records distinct module dependencies authored by top-level `import` and `from … import` statements in the current shadow `.ph` files. Relative paths follow the repository resolver rule: one leading dot uses the importing package, and each additional leading dot ascends one package level. This is a source-path inventory, not proof that every target exists or binding resolves; it does not establish initialization order or interface-only behavior. Implicit prelude names and references not expressed by imports are excluded.

Current resolved distinct edges: **218**.

The S0 copied baseline recorded **208** edges. The scalar split added 6 net edges; the foundational-type increment added 2 explicit package-facade edges. S4 retargeted 23 existing edges from `universe.collections.iterator_laws` to `universe.collections.iteration` without changing the edge count. S5 adds root imports of `universe.diagnostics` and `universe.features` and removes the redundant `universe.concurrent` import, leaving 217 edges; extracting `CompletionSource` adds its explicit `Future` import, bringing the current total to **218**.

| S3 change | From module | Resolved target module |
|---|---|---|
| Removed | `universe.scalar.number` | `universe.text.errors` |
| Added | `universe.scalar` | `universe.scalar.float` |
| Added | `universe.scalar` | `universe.scalar.int` |
| Added | `universe.scalar` | `universe.scalar.number` |
| Added | `universe.scalar.float` | `universe.scalar.number` |
| Added | `universe.scalar.float` | `universe.text.errors` |
| Added | `universe.scalar.int` | `universe.scalar.number` |
| Added | `universe.scalar.int` | `universe.text.errors` |

| Foundational-type change | From module | Resolved target module |
|---|---|---|
| Added | `universe` | `universe.result` |
| Added | `universe.scalar` | `universe.scalar.unit` |

| S4 iteration relocation | From module | Previous target | Current target |
|---|---|---|---|
| Retargeted (23 source dependencies) | collection, string, text, functional, and math consumers | `universe.collections.iterator_laws` | `universe.collections.iteration` |

| From module | Resolved target module | Evidence |
|---|---|---|
| `universe` | `universe.callable` | authored `import`/`from … import` path |
| `universe` | `universe.collections` | authored `import`/`from … import` path |
| `universe` | `universe.concurrency` | authored `import`/`from … import` path |
| `universe.concurrency.completion_source` | `universe.concurrency.fiber` | authored `from .fiber import Future` path |
| `universe` | `universe.diagnostics` | authored `import`/`from … import` path |
| `universe` | `universe.errors` | authored `import`/`from … import` path |
| `universe` | `universe.features` | authored `import`/`from … import` path |
| `universe` | `universe.fs` | authored `import`/`from … import` path |
| `universe` | `universe.functional` | authored `import`/`from … import` path |
| `universe` | `universe.io` | authored `import`/`from … import` path |
| `universe` | `universe.json` | authored `import`/`from … import` path |
| `universe` | `universe.math` | authored `import`/`from … import` path |
| `universe` | `universe.net` | authored `import`/`from … import` path |
| `universe` | `universe.numbers` | authored `import`/`from … import` path |
| `universe` | `universe.object` | authored `import`/`from … import` path |
| `universe` | `universe.option` | authored `import`/`from … import` path |
| `universe` | `universe.path` | authored `import`/`from … import` path |
| `universe` | `universe.process` | authored `import`/`from … import` path |
| `universe` | `universe.random` | authored `import`/`from … import` path |
| `universe` | `universe.reflection` | authored `import`/`from … import` path |
| `universe` | `universe.regex` | authored `import`/`from … import` path |
| `universe` | `universe.result` | authored `import`/`from … import` path |
| `universe` | `universe.scalar` | authored `import`/`from … import` path |
| `universe` | `universe.testing` | authored `import`/`from … import` path |
| `universe` | `universe.text` | authored `import`/`from … import` path |
| `universe` | `universe.time` | authored `import`/`from … import` path |
| `universe.collections.array` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.bag` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.bijection` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.buffer` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.bytes` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.indexed_dict` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.iteration` | `universe.text.builder` | authored `import`/`from … import` path |
| `universe.collections.list` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.list` | `universe.text.builder` | authored `import`/`from … import` path |
| `universe.collections.map` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.mutable_bag` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.ordered_set` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.pack` | `universe.collections.tuple` | authored `import`/`from … import` path |
| `universe.collections.range` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.range_map` | `universe.collections.interval` | authored `import`/`from … import` path |
| `universe.collections.range_map` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.set` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.slice` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.tuple` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.collections.vector` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.errors` | `universe.errors.argument` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.algebra` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.arrow` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.chain` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.effects` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.either` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.fold` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.folds` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.higher_kinded` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.journal` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.lens` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.optics` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.reader` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.reader_result` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.state` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.state_result` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.validation` | authored `import`/`from … import` path |
| `universe.functional` | `universe.functional.writer` | authored `import`/`from … import` path |
| `universe.functional.algebra` | `universe.functional.chain` | authored `import`/`from … import` path |
| `universe.functional.algebra` | `universe.numbers.capabilities` | authored `import`/`from … import` path |
| `universe.functional.arrow` | `universe.functional.either` | authored `import`/`from … import` path |
| `universe.functional.chain` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.functional.effects` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.functional.effects` | `universe.functional.chain` | authored `import`/`from … import` path |
| `universe.functional.effects` | `universe.functional.higher_kinded` | authored `import`/`from … import` path |
| `universe.functional.fold` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.functional.fold` | `universe.functional.chain` | authored `import`/`from … import` path |
| `universe.functional.fold` | `universe.numbers.capabilities` | authored `import`/`from … import` path |
| `universe.functional.folds` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.functional.folds` | `universe.functional.algebra` | authored `import`/`from … import` path |
| `universe.functional.folds` | `universe.functional.chain` | authored `import`/`from … import` path |
| `universe.functional.folds` | `universe.functional.validation` | authored `import`/`from … import` path |
| `universe.functional.higher_kinded` | `universe.functional.chain` | authored `import`/`from … import` path |
| `universe.functional.journal` | `universe.functional.algebra` | authored `import`/`from … import` path |
| `universe.functional.journal` | `universe.functional.chain` | authored `import`/`from … import` path |
| `universe.functional.optics` | `universe.functional.lens` | authored `import`/`from … import` path |
| `universe.functional.optics` | `universe.reflection.structural` | authored `import`/`from … import` path |
| `universe.functional.reader_result` | `universe.functional.reader` | authored `import`/`from … import` path |
| `universe.functional.state` | `universe.functional.lens` | authored `import`/`from … import` path |
| `universe.functional.state_result` | `universe.functional.lens` | authored `import`/`from … import` path |
| `universe.functional.state_result` | `universe.functional.state` | authored `import`/`from … import` path |
| `universe.functional.validation` | `universe.functional.algebra` | authored `import`/`from … import` path |
| `universe.functional.writer` | `universe.functional.algebra` | authored `import`/`from … import` path |
| `universe.math` | `universe.math.float_kernel` | authored `import`/`from … import` path |
| `universe.math` | `universe.math.integer` | authored `import`/`from … import` path |
| `universe.math` | `universe.math.numerical_errors` | authored `import`/`from … import` path |
| `universe.math` | `universe.math.reduction` | authored `import`/`from … import` path |
| `universe.math.complex` | `universe.math.complex.kernel` | authored `import`/`from … import` path |
| `universe.math.complex.kernel` | `universe.math.float_kernel` | authored `import`/`from … import` path |
| `universe.math.complex.kernel` | `universe.numbers.complex` | authored `import`/`from … import` path |
| `universe.math.complex.kernel` | `universe.numbers.float_complex` | authored `import`/`from … import` path |
| `universe.math.reduction` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.math.reduction` | `universe.math.float_kernel` | authored `import`/`from … import` path |
| `universe.math.reduction` | `universe.math.numerical_errors` | authored `import`/`from … import` path |
| `universe.numbers` | `universe.numbers.algebra` | authored `import`/`from … import` path |
| `universe.numbers` | `universe.numbers.capabilities` | authored `import`/`from … import` path |
| `universe.numbers` | `universe.numbers.complex` | authored `import`/`from … import` path |
| `universe.numbers` | `universe.numbers.quaternion` | authored `import`/`from … import` path |
| `universe.numbers` | `universe.numbers.rational` | authored `import`/`from … import` path |
| `universe.numbers.complex` | `universe.math.float_kernel` | authored `import`/`from … import` path |
| `universe.numbers.complex` | `universe.numbers.algebra` | authored `import`/`from … import` path |
| `universe.numbers.complex` | `universe.numbers.capabilities` | authored `import`/`from … import` path |
| `universe.numbers.complex` | `universe.numbers.float_complex` | authored `import`/`from … import` path |
| `universe.numbers.complex` | `universe.numbers.rational` | authored `import`/`from … import` path |
| `universe.numbers.float_complex` | `universe.math.float_kernel` | authored `import`/`from … import` path |
| `universe.numbers.quaternion` | `universe.numbers.algebra` | authored `import`/`from … import` path |
| `universe.numbers.quaternion` | `universe.numbers.capabilities` | authored `import`/`from … import` path |
| `universe.numbers.quaternion` | `universe.numbers.complex` | authored `import`/`from … import` path |
| `universe.numbers.rational` | `universe.math.integer` | authored `import`/`from … import` path |
| `universe.numbers.rational` | `universe.numbers.capabilities` | authored `import`/`from … import` path |
| `universe.random` | `universe.random.bernoulli` | authored `import`/`from … import` path |
| `universe.random` | `universe.random.distribution` | authored `import`/`from … import` path |
| `universe.random` | `universe.random.generator` | authored `import`/`from … import` path |
| `universe.random` | `universe.random.secure` | authored `import`/`from … import` path |
| `universe.random` | `universe.random.uniform_int` | authored `import`/`from … import` path |
| `universe.random` | `universe.random.xoshiro256pp` | authored `import`/`from … import` path |
| `universe.random.bernoulli` | `universe.random.distribution` | authored `import`/`from … import` path |
| `universe.random.bernoulli` | `universe.random.generator` | authored `import`/`from … import` path |
| `universe.random.distribution` | `universe.random.generator` | authored `import`/`from … import` path |
| `universe.random.uniform_int` | `universe.random.distribution` | authored `import`/`from … import` path |
| `universe.random.uniform_int` | `universe.random.generator` | authored `import`/`from … import` path |
| `universe.random.xoshiro256pp` | `universe.random.generator` | authored `import`/`from … import` path |
| `universe.reflection.structural` | `universe.collections.pack` | authored `import`/`from … import` path |
| `universe.reflection.structural` | `universe.collections.record` | authored `import`/`from … import` path |
| `universe.reflection.structural` | `universe.reflection.typing.kind` | authored `import`/`from … import` path |
| `universe.scalar` | `universe.scalar.float` | authored `import`/`from … import` path |
| `universe.scalar` | `universe.scalar.int` | authored `import`/`from … import` path |
| `universe.scalar` | `universe.scalar.number` | authored `import`/`from … import` path |
| `universe.scalar` | `universe.scalar.unit` | authored `import`/`from … import` path |
| `universe.scalar` | `universe.scalar.string_index` | authored `import`/`from … import` path |
| `universe.scalar.bool` | `universe.text.errors` | authored `import`/`from … import` path |
| `universe.scalar.char` | `universe.text.substring` | authored `import`/`from … import` path |
| `universe.scalar.float` | `universe.scalar.number` | authored `import`/`from … import` path |
| `universe.scalar.float` | `universe.text.errors` | authored `import`/`from … import` path |
| `universe.scalar.int` | `universe.scalar.number` | authored `import`/`from … import` path |
| `universe.scalar.int` | `universe.text.errors` | authored `import`/`from … import` path |
| `universe.scalar.string` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.scalar.string` | `universe.scalar.string_index` | authored `import`/`from … import` path |
| `universe.scalar.string` | `universe.text.errors` | authored `import`/`from … import` path |
| `universe.scalar.string` | `universe.text.substring` | authored `import`/`from … import` path |
| `universe.text` | `universe.text.builder` | authored `import`/`from … import` path |
| `universe.text` | `universe.text.cursor` | authored `import`/`from … import` path |
| `universe.text` | `universe.text.errors` | authored `import`/`from … import` path |
| `universe.text` | `universe.text.line_map` | authored `import`/`from … import` path |
| `universe.text` | `universe.text.lines` | authored `import`/`from … import` path |
| `universe.text` | `universe.text.patterns` | authored `import`/`from … import` path |
| `universe.text` | `universe.text.split` | authored `import`/`from … import` path |
| `universe.text` | `universe.text.substring` | authored `import`/`from … import` path |
| `universe.text` | `universe.text.traits` | authored `import`/`from … import` path |
| `universe.text` | `universe.text.writer` | authored `import`/`from … import` path |
| `universe.text.builder` | `universe.text.writer` | authored `import`/`from … import` path |
| `universe.text.cursor` | `universe.scalar.string_index` | authored `import`/`from … import` path |
| `universe.text.cursor` | `universe.text.substring` | authored `import`/`from … import` path |
| `universe.text.line_map` | `universe.scalar.string_index` | authored `import`/`from … import` path |
| `universe.text.line_map` | `universe.text.substring` | authored `import`/`from … import` path |
| `universe.text.lines` | `universe.text.substring` | authored `import`/`from … import` path |
| `universe.text.patterns` | `universe.text.substring` | authored `import`/`from … import` path |
| `universe.text.split` | `universe.text.substring` | authored `import`/`from … import` path |
| `universe.text.substring` | `universe.collections.iteration` | authored `import`/`from … import` path |
| `universe.text.substring` | `universe.scalar.string_index` | authored `import`/`from … import` path |
| `universe.text.traits` | `universe.scalar.string` | authored `import`/`from … import` path |
| `universe.text.traits` | `universe.text.substring` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.calendar.date` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.calendar.datetime` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.calendar.month` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.calendar.partial` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.calendar.period` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.calendar.time` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.calendar.weekday` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.clock` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.duration` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.offset` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.resolution` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.timestamp` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.timezone` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.zone_transition` | authored `import`/`from … import` path |
| `universe.time` | `universe.time.zoned` | authored `import`/`from … import` path |
| `universe.time.calendar` | `universe.time.calendar.date` | authored `import`/`from … import` path |
| `universe.time.calendar` | `universe.time.calendar.datetime` | authored `import`/`from … import` path |
| `universe.time.calendar` | `universe.time.calendar.month` | authored `import`/`from … import` path |
| `universe.time.calendar` | `universe.time.calendar.partial` | authored `import`/`from … import` path |
| `universe.time.calendar` | `universe.time.calendar.period` | authored `import`/`from … import` path |
| `universe.time.calendar` | `universe.time.calendar.time` | authored `import`/`from … import` path |
| `universe.time.calendar` | `universe.time.calendar.weekday` | authored `import`/`from … import` path |
| `universe.time.calendar.date` | `universe.time.calendar.month` | authored `import`/`from … import` path |
| `universe.time.calendar.date` | `universe.time.calendar.period` | authored `import`/`from … import` path |
| `universe.time.calendar.date` | `universe.time.calendar.weekday` | authored `import`/`from … import` path |
| `universe.time.calendar.date` | `universe.time.timestamp` | authored `import`/`from … import` path |
| `universe.time.calendar.date` | `universe.time.timezone` | authored `import`/`from … import` path |
| `universe.time.calendar.datetime` | `universe.time.calendar.date` | authored `import`/`from … import` path |
| `universe.time.calendar.datetime` | `universe.time.calendar.month` | authored `import`/`from … import` path |
| `universe.time.calendar.datetime` | `universe.time.calendar.period` | authored `import`/`from … import` path |
| `universe.time.calendar.datetime` | `universe.time.calendar.time` | authored `import`/`from … import` path |
| `universe.time.calendar.datetime` | `universe.time.duration` | authored `import`/`from … import` path |
| `universe.time.calendar.datetime` | `universe.time.offset` | authored `import`/`from … import` path |
| `universe.time.calendar.datetime` | `universe.time.timestamp` | authored `import`/`from … import` path |
| `universe.time.calendar.datetime` | `universe.time.timezone` | authored `import`/`from … import` path |
| `universe.time.calendar.partial` | `universe.time.calendar.date` | authored `import`/`from … import` path |
| `universe.time.calendar.partial` | `universe.time.calendar.month` | authored `import`/`from … import` path |
| `universe.time.clock` | `universe.time.duration` | authored `import`/`from … import` path |
| `universe.time.resolution` | `universe.time.calendar.datetime` | authored `import`/`from … import` path |
| `universe.time.testing.clocks` | `universe.time.clock` | authored `import`/`from … import` path |
| `universe.time.testing.clocks` | `universe.time.duration` | authored `import`/`from … import` path |
| `universe.time.testing.clocks` | `universe.time.timestamp` | authored `import`/`from … import` path |
| `universe.time.timestamp` | `universe.time.duration` | authored `import`/`from … import` path |
| `universe.time.timezone` | `universe.time.duration` | authored `import`/`from … import` path |
| `universe.time.timezone` | `universe.time.offset` | authored `import`/`from … import` path |
| `universe.time.timezone` | `universe.time.timestamp` | authored `import`/`from … import` path |
| `universe.time.timezone` | `universe.time.zone_transition` | authored `import`/`from … import` path |
| `universe.time.zone_transition` | `universe.time.offset` | authored `import`/`from … import` path |
| `universe.time.zone_transition` | `universe.time.timestamp` | authored `import`/`from … import` path |
| `universe.time.zoned` | `universe.time.calendar.datetime` | authored `import`/`from … import` path |

## Limits and bootstrap notes

- Package `expose` statements describe child-module visibility and are not import edges.
- Scalar owner modules now import `Number`; the scalar facade imports and re-exports `Number`, `Int`, and `Float`. The import graph records these edges, but no shadow linker/runtime initialization behavior was verified.
- Production bootstrap and native source anchors still use the legacy tree. Do not register both source trees in one production module graph or VM.
- Any later split that changes dependency cycles, initialization, or native ownership requires isolated semantic/runtime validation before integration.
- The current `iteration/package.ph` keeps protocols and adapters cohesive. Splitting protocol defaults from adapter implementations would create runtime edges in both directions under the present source import model; see [`iteration-split-incident.md`](iteration-split-incident.md).
