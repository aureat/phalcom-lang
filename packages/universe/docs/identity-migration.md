# Declaration identity migration ledger

The two source trees are alternative layouts of the same conceptual builtin project. Both manifests retain `name = "universe"` and `namespace = "universe"`; no second builtin project identity was introduced. Those manifest fields do not make standalone filesystem loading equivalent to the production builtin identity: loading the copied manifest through `ProjectUniverse` would be filesystem-backed with `Resolved` identity. Production continues to use the legacy source provider with `ProjectIdentity::Universe`. The Phase S3 scalar and foundational-type increments reorganize the shadow owners of `Int`, `Float`, `Option`, `Result`, and `Unit`; S4 moves the iterator protocols and adapters together to the cohesive shadow source module `universe.collections.iteration`; the S5 source-only relocations place `Backoff` in `universe.concurrency.backoff`, `CompletionSource` in `universe.concurrency.completion_source`, `Tracer` in `universe.diagnostics.tracer`, and `OffBehavior` in `universe.features.off_behavior`. The finer protocol/adapter ownership proposed by the target architecture remains deferred, as recorded in `docs/specs/universe.md` and `migration-status.md`. Production source/native/prelude mappings remain on legacy paths, and no compiler or runtime cutover has occurred.

## Conservative source declaration index

The index is a source scan, not a parser/compiler identity result. It includes every top-level unindented line matching zero or more `intrinsic`, `native`, `primitive`, or `type` prefixes, followed by one of `data`, `enum`, `class`, `trait`, or `type`, followed by an identifier. It includes all **364** recognized declarations, including intrinsic type aliases and type-only/runtime intrinsic data declarations. It does not parse declarations, enumerate nested members/variants, include values, validate native associations, or prove the resolved semantic identity. Every recognized declaration row is listed here with its baseline owner and current shadow owner, and in the current-layout `module-map.md`; the scan count is not a claim that all Telos declarations/forms are accounted for.

| Declaration | Legacy source module | Legacy source file | Current shadow source module | Current shadow source file | Form | Migration state |
|---|---|---|---|---|---|---|
| `Closure` | `universe.callable.closure` | `callable/closure.ph` | `universe.callable.closure` | `callable/closure.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `Family` | `universe.callable.family` | `callable/family.ph` | `universe.callable.family` | `callable/family.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `MethodFamily` | `universe.callable.family` | `callable/family.ph` | `universe.callable.family` | `callable/family.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `BoundMethodFamily` | `universe.callable.family` | `callable/family.ph` | `universe.callable.family` | `callable/family.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `Function` | `universe.callable.function` | `callable/function.ph` | `universe.callable.function` | `callable/function.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `Method` | `universe.callable.method` | `callable/method.ph` | `universe.callable.method` | `callable/method.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `BoundMethod` | `universe.callable.method` | `callable/method.ph` | `universe.callable.method` | `callable/method.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `Array` | `universe.collections.array` | `collections/array.ph` | `universe.collections.array` | `collections/array.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `ArrayIterator` | `universe.collections.array` | `collections/array.ph` | `universe.collections.array` | `collections/array.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Bag` | `universe.collections.bag` | `collections/bag.ph` | `universe.collections.bag` | `collections/bag.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `BagIterator` | `universe.collections.bag` | `collections/bag.ph` | `universe.collections.bag` | `collections/bag.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Bijection` | `universe.collections.bijection` | `collections/bijection.ph` | `universe.collections.bijection` | `collections/bijection.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `BijectionIterator` | `universe.collections.bijection` | `collections/bijection.ph` | `universe.collections.bijection` | `collections/bijection.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Buffer` | `universe.collections.buffer` | `collections/buffer.ph` | `universe.collections.buffer` | `collections/buffer.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `BufferIterator` | `universe.collections.buffer` | `collections/buffer.ph` | `universe.collections.buffer` | `collections/buffer.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Bytes` | `universe.collections.bytes` | `collections/bytes.ph` | `universe.collections.bytes` | `collections/bytes.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `BytesIterator` | `universe.collections.bytes` | `collections/bytes.ph` | `universe.collections.bytes` | `collections/bytes.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `OpenMode` | `universe.collections.bytes` | `collections/bytes.ph` | `universe.collections.bytes` | `collections/bytes.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Path` | `universe.collections.bytes` | `collections/bytes.ph` | `universe.collections.bytes` | `collections/bytes.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Resource` | `universe.collections.bytes` | `collections/bytes.ph` | `universe.collections.bytes` | `collections/bytes.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `UnflushedError` | `universe.collections.bytes` | `collections/bytes.ph` | `universe.collections.bytes` | `collections/bytes.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `ConcurrentOperationError` | `universe.collections.bytes` | `collections/bytes.ph` | `universe.collections.bytes` | `collections/bytes.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `StreamProtocolError` | `universe.collections.bytes` | `collections/bytes.ph` | `universe.collections.bytes` | `collections/bytes.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `WriteZeroError` | `universe.collections.bytes` | `collections/bytes.ph` | `universe.collections.bytes` | `collections/bytes.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `BytesReader` | `universe.collections.bytes` | `collections/bytes.ph` | `universe.collections.bytes` | `collections/bytes.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `BytesWriter` | `universe.collections.bytes` | `collections/bytes.ph` | `universe.collections.bytes` | `collections/bytes.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `BufferedWriter` | `universe.collections.bytes` | `collections/bytes.ph` | `universe.collections.bytes` | `collections/bytes.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `BufferedReader` | `universe.collections.bytes` | `collections/bytes.ph` | `universe.collections.bytes` | `collections/bytes.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Collection` | `universe.collections.collection` | `collections/collection.ph` | `universe.collections.collection` | `collections/collection.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `IndexedDict` | `universe.collections.indexed_dict` | `collections/indexed_dict.ph` | `universe.collections.indexed_dict` | `collections/indexed_dict.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `IndexedDictIterator` | `universe.collections.indexed_dict` | `collections/indexed_dict.ph` | `universe.collections.indexed_dict` | `collections/indexed_dict.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Bound` | `universe.collections.interval` | `collections/interval.ph` | `universe.collections.interval` | `collections/interval.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Interval` | `universe.collections.interval` | `collections/interval.ph` | `universe.collections.interval` | `collections/interval.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Iterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `trait` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `Iterable` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `trait` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `MappedIterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `FilterIterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `FilterMapIterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `FlatMapIterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `FlattenIterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `ChainIterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `SkipIterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `SkipWhileIterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `TakeIterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `TakeWhileIterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `MapWhileIterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `InspectIterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `StrictZipIterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `IndexedIterator` | `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `universe.collections.iteration` | `collections/iteration/package.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `List` | `universe.collections.list` | `collections/list.ph` | `universe.collections.list` | `collections/list.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `ListIterator` | `universe.collections.list` | `collections/list.ph` | `universe.collections.list` | `collections/list.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Map` | `universe.collections.map` | `collections/map.ph` | `universe.collections.map` | `collections/map.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `MapIterator` | `universe.collections.map` | `collections/map.ph` | `universe.collections.map` | `collections/map.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MapKeysView` | `universe.collections.map` | `collections/map.ph` | `universe.collections.map` | `collections/map.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MapValuesView` | `universe.collections.map` | `collections/map.ph` | `universe.collections.map` | `collections/map.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Entry` | `universe.collections.map` | `collections/map.ph` | `universe.collections.map` | `collections/map.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MapEntriesView` | `universe.collections.map` | `collections/map.ph` | `universe.collections.map` | `collections/map.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MapKeysIterator` | `universe.collections.map` | `collections/map.ph` | `universe.collections.map` | `collections/map.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MapValuesIterator` | `universe.collections.map` | `collections/map.ph` | `universe.collections.map` | `collections/map.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MapEntriesIterator` | `universe.collections.map` | `collections/map.ph` | `universe.collections.map` | `collections/map.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MutableBag` | `universe.collections.mutable_bag` | `collections/mutable_bag.ph` | `universe.collections.mutable_bag` | `collections/mutable_bag.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `MutableBagIterator` | `universe.collections.mutable_bag` | `collections/mutable_bag.ph` | `universe.collections.mutable_bag` | `collections/mutable_bag.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MutableBagEntriesView` | `universe.collections.mutable_bag` | `collections/mutable_bag.ph` | `universe.collections.mutable_bag` | `collections/mutable_bag.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MutableBagEntriesIterator` | `universe.collections.mutable_bag` | `collections/mutable_bag.ph` | `universe.collections.mutable_bag` | `collections/mutable_bag.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `OrderedSet` | `universe.collections.ordered_set` | `collections/ordered_set.ph` | `universe.collections.ordered_set` | `collections/ordered_set.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `OrderedSetIterator` | `universe.collections.ordered_set` | `collections/ordered_set.ph` | `universe.collections.ordered_set` | `collections/ordered_set.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Pack` | `universe.collections.pack` | `collections/pack.ph` | `universe.collections.pack` | `collections/pack.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `PositionalPack` | `universe.collections.pack` | `collections/pack.ph` | `universe.collections.pack` | `collections/pack.ph` | `type` | `LEGACY_PATH_UNCHANGED` |
| `LabeledPack` | `universe.collections.pack` | `collections/pack.ph` | `universe.collections.pack` | `collections/pack.ph` | `type` | `LEGACY_PATH_UNCHANGED` |
| `CompletePack` | `universe.collections.pack` | `collections/pack.ph` | `universe.collections.pack` | `collections/pack.ph` | `type` | `LEGACY_PATH_UNCHANGED` |
| `PositionalPackView` | `universe.collections.pack` | `collections/pack.ph` | `universe.collections.pack` | `collections/pack.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `LabeledPackView` | `universe.collections.pack` | `collections/pack.ph` | `universe.collections.pack` | `collections/pack.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `PositionalTypePackView` | `universe.collections.pack` | `collections/pack.ph` | `universe.collections.pack` | `collections/pack.ph` | `type` | `LEGACY_PATH_UNCHANGED` |
| `LabeledTypePackView` | `universe.collections.pack` | `collections/pack.ph` | `universe.collections.pack` | `collections/pack.ph` | `type` | `LEGACY_PATH_UNCHANGED` |
| `Range` | `universe.collections.range` | `collections/range.ph` | `universe.collections.range` | `collections/range.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `RangeIterator` | `universe.collections.range` | `collections/range.ph` | `universe.collections.range` | `collections/range.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RangeMapNode` | `universe.collections.range_map` | `collections/range_map.ph` | `universe.collections.range_map` | `collections/range_map.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RangeMap` | `universe.collections.range_map` | `collections/range_map.ph` | `universe.collections.range_map` | `collections/range_map.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RangeMapIterator` | `universe.collections.range_map` | `collections/range_map.ph` | `universe.collections.range_map` | `collections/range_map.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RecordFields` | `universe.collections.record` | `collections/record.ph` | `universe.collections.record` | `collections/record.ph` | `intrinsic type data` | `LEGACY_PATH_UNCHANGED` |
| `FieldKey` | `universe.collections.record` | `collections/record.ph` | `universe.collections.record` | `collections/record.ph` | `type` | `LEGACY_PATH_UNCHANGED` |
| `Record` | `universe.collections.record` | `collections/record.ph` | `universe.collections.record` | `collections/record.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `EmptyRecord` | `universe.collections.record` | `collections/record.ph` | `universe.collections.record` | `collections/record.ph` | `intrinsic type` | `LEGACY_PATH_UNCHANGED` |
| `Set` | `universe.collections.set` | `collections/set.ph` | `universe.collections.set` | `collections/set.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `SetIterator` | `universe.collections.set` | `collections/set.ph` | `universe.collections.set` | `collections/set.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Slice` | `universe.collections.slice` | `collections/slice.ph` | `universe.collections.slice` | `collections/slice.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `SliceIterator` | `universe.collections.slice` | `collections/slice.ph` | `universe.collections.slice` | `collections/slice.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TypePack` | `universe.collections.tuple` | `collections/tuple.ph` | `universe.collections.tuple` | `collections/tuple.ph` | `intrinsic type data` | `LEGACY_PATH_UNCHANGED` |
| `LabeledTypePack` | `universe.collections.tuple` | `collections/tuple.ph` | `universe.collections.tuple` | `collections/tuple.ph` | `intrinsic type data` | `LEGACY_PATH_UNCHANGED` |
| `CompleteTypePack` | `universe.collections.tuple` | `collections/tuple.ph` | `universe.collections.tuple` | `collections/tuple.ph` | `intrinsic type data` | `LEGACY_PATH_UNCHANGED` |
| `Tuple` | `universe.collections.tuple` | `collections/tuple.ph` | `universe.collections.tuple` | `collections/tuple.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `TupleIterator` | `universe.collections.tuple` | `collections/tuple.ph` | `universe.collections.tuple` | `collections/tuple.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TupleZipIterator` | `universe.collections.tuple` | `collections/tuple.ph` | `universe.collections.tuple` | `collections/tuple.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TupleZipView` | `universe.collections.tuple` | `collections/tuple.ph` | `universe.collections.tuple` | `collections/tuple.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Vector` | `universe.collections.vector` | `collections/vector.ph` | `universe.collections.vector` | `collections/vector.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `VectorIterator` | `universe.collections.vector` | `collections/vector.ph` | `universe.collections.vector` | `collections/vector.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `System` | `universe.concurrency.fiber` | `concurrency/fiber.ph` | `universe.concurrency.fiber` | `concurrency/fiber.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `Fiber` | `universe.concurrency.fiber` | `concurrency/fiber.ph` | `universe.concurrency.fiber` | `concurrency/fiber.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `FutureSubscription` | `universe.concurrency.fiber` | `concurrency/fiber.ph` | `universe.concurrency.fiber` | `concurrency/fiber.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Future` | `universe.concurrency.fiber` | `concurrency/fiber.ph` | `universe.concurrency.fiber` | `concurrency/fiber.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `CompletionSource` | `universe.concurrency.fiber` | `concurrency/fiber.ph` | `universe.concurrency.completion_source` | `concurrency/completion_source.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `Tracer` | `universe.concurrency.fiber` | `concurrency/fiber.ph` | `universe.diagnostics.tracer` | `diagnostics/tracer.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `OffBehavior` | `universe.concurrency.fiber` | `concurrency/fiber.ph` | `universe.features.off_behavior` | `features/off_behavior.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `Backoff` | `universe.concurrency.fiber` | `concurrency/fiber.ph` | `universe.concurrency.backoff` | `concurrency/backoff.ph` | `class` | `MOVED_TO_SHADOW_MODULE; PRODUCTION_LEGACY` |
| `ArgumentError` | `universe.errors.argument` | `errors/argument.ph` | `universe.errors.argument` | `errors/argument.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `KeyError` | `universe.errors.argument` | `errors/argument.ph` | `universe.errors.argument` | `errors/argument.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `IndexError` | `universe.errors.argument` | `errors/argument.ph` | `universe.errors.argument` | `errors/argument.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `SliceError` | `universe.errors.argument` | `errors/argument.ph` | `universe.errors.argument` | `errors/argument.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `DuplicateKeyError` | `universe.errors.argument` | `errors/argument.ph` | `universe.errors.argument` | `errors/argument.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `BijectionConflictError` | `universe.errors.argument` | `errors/argument.ph` | `universe.errors.argument` | `errors/argument.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `InvalidIntervalError` | `universe.errors.argument` | `errors/argument.ph` | `universe.errors.argument` | `errors/argument.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RangeOverlapError` | `universe.errors.argument` | `errors/argument.ph` | `universe.errors.argument` | `errors/argument.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `PreconditionError` | `universe.errors.contracts` | `errors/contracts.ph` | `universe.errors.contracts` | `errors/contracts.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `PostconditionError` | `universe.errors.contracts` | `errors/contracts.ph` | `universe.errors.contracts` | `errors/contracts.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `InvariantError` | `universe.errors.contracts` | `errors/contracts.ph` | `universe.errors.contracts` | `errors/contracts.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Error` | `universe.errors.error` | `errors/error.ph` | `universe.errors.error` | `errors/error.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `MessageNotUnderstood` | `universe.errors.error` | `errors/error.ph` | `universe.errors.error` | `errors/error.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `CannotYieldAcrossNativeFrame` | `universe.errors.error` | `errors/error.ph` | `universe.errors.error` | `errors/error.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `UseAfterCloseError` | `universe.errors.error` | `errors/error.ph` | `universe.errors.error` | `errors/error.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `UnwrapError` | `universe.errors.error` | `errors/error.ph` | `universe.errors.error` | `errors/error.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TimeoutError` | `universe.errors.error` | `errors/error.ph` | `universe.errors.error` | `errors/error.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `ZipLengthMismatchError` | `universe.errors.iteration` | `errors/iteration.ph` | `universe.errors.iteration` | `errors/iteration.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Result` | `universe.errors.result` | `errors/result.ph` | `universe.result` | `result.ph` | `native data` | `MOVED_TO_SHADOW_MODULE; NORMATIVE_PATH_UPDATED; PRODUCTION_LEGACY` |
| `Unimplemented` | `universe.errors.unimplemented` | `errors/unimplemented.ph` | `universe.errors.unimplemented` | `errors/unimplemented.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Unsupported` | `universe.errors.unsupported` | `errors/unsupported.ph` | `universe.errors.unsupported` | `errors/unsupported.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Semigroup` | `universe.functional.algebra` | `functional/algebra.ph` | `universe.functional.algebra` | `functional/algebra.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Monoid` | `universe.functional.algebra` | `functional/algebra.ph` | `universe.functional.algebra` | `functional/algebra.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Additive` | `universe.functional.algebra` | `functional/algebra.ph` | `universe.functional.algebra` | `functional/algebra.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Multiplicative` | `universe.functional.algebra` | `functional/algebra.ph` | `universe.functional.algebra` | `functional/algebra.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `AllTrue` | `universe.functional.algebra` | `functional/algebra.ph` | `universe.functional.algebra` | `functional/algebra.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `AnyTrue` | `universe.functional.algebra` | `functional/algebra.ph` | `universe.functional.algebra` | `functional/algebra.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `First` | `universe.functional.algebra` | `functional/algebra.ph` | `universe.functional.algebra` | `functional/algebra.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Last` | `universe.functional.algebra` | `functional/algebra.ph` | `universe.functional.algebra` | `functional/algebra.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Dual` | `universe.functional.algebra` | `functional/algebra.ph` | `universe.functional.algebra` | `functional/algebra.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Endo` | `universe.functional.algebra` | `functional/algebra.ph` | `universe.functional.algebra` | `functional/algebra.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Profunctor` | `universe.functional.arrow` | `functional/arrow.ph` | `universe.functional.arrow` | `functional/arrow.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Arrow` | `universe.functional.arrow` | `functional/arrow.ph` | `universe.functional.arrow` | `functional/arrow.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `ArrowContext` | `universe.functional.arrow` | `functional/arrow.ph` | `universe.functional.arrow` | `functional/arrow.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Chain` | `universe.functional.chain` | `functional/chain.ph` | `universe.functional.chain` | `functional/chain.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Partition` | `universe.functional.chain` | `functional/chain.ph` | `universe.functional.chain` | `functional/chain.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `ChainIterator` | `universe.functional.chain` | `functional/chain.ph` | `universe.functional.chain` | `functional/chain.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `NonEmpty` | `universe.functional.chain` | `functional/chain.ph` | `universe.functional.chain` | `functional/chain.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Effects` | `universe.functional.effects` | `functional/effects.ph` | `universe.functional.effects` | `functional/effects.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Either` | `universe.functional.either` | `functional/either.ph` | `universe.functional.either` | `functional/either.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Fold` | `universe.functional.fold` | `functional/fold.ph` | `universe.functional.fold` | `functional/fold.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `FoldFactory` | `universe.functional.fold` | `functional/fold.ph` | `universe.functional.fold` | `functional/fold.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Folds` | `universe.functional.folds` | `functional/folds.ph` | `universe.functional.folds` | `functional/folds.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Functor` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Applicative` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Monad` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Identity` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `IdentityContext` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `OptionContext` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `ChainContext` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `NonEmptyContext` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Journal` | `universe.functional.journal` | `functional/journal.ph` | `universe.functional.journal` | `functional/journal.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Lens` | `universe.functional.lens` | `functional/lens.ph` | `universe.functional.lens` | `functional/lens.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Iso` | `universe.functional.optics` | `functional/optics.ph` | `universe.functional.optics` | `functional/optics.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Optional` | `universe.functional.optics` | `functional/optics.ph` | `universe.functional.optics` | `functional/optics.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Prism` | `universe.functional.optics` | `functional/optics.ph` | `universe.functional.optics` | `functional/optics.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Reader` | `universe.functional.reader` | `functional/reader.ph` | `universe.functional.reader` | `functional/reader.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `ReaderResult` | `universe.functional.reader_result` | `functional/reader_result.ph` | `universe.functional.reader_result` | `functional/reader_result.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `StateStep` | `universe.functional.state` | `functional/state.ph` | `universe.functional.state` | `functional/state.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `State` | `universe.functional.state` | `functional/state.ph` | `universe.functional.state` | `functional/state.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `StateResult` | `universe.functional.state_result` | `functional/state_result.ph` | `universe.functional.state_result` | `functional/state_result.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Validation` | `universe.functional.validation` | `functional/validation.ph` | `universe.functional.validation` | `functional/validation.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Writer` | `universe.functional.writer` | `functional/writer.ph` | `universe.functional.writer` | `functional/writer.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `ComplexMathKernel` | `universe.math.complex.kernel` | `math/complex/kernel.ph` | `universe.math.complex.kernel` | `math/complex/kernel.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `FloatMathKernel` | `universe.math.float_kernel` | `math/float_kernel.ph` | `universe.math.float_kernel` | `math/float_kernel.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `FloatAngleHelpers` | `universe.math.float_kernel` | `math/float_kernel.ph` | `universe.math.float_kernel` | `math/float_kernel.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `IntegerMath` | `universe.math.integer` | `math/integer.ph` | `universe.math.integer` | `math/integer.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `NumericalRangeError` | `universe.math.numerical_errors` | `math/numerical_errors.ph` | `universe.math.numerical_errors` | `math/numerical_errors.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `NumericalEvaluationError` | `universe.math.numerical_errors` | `math/numerical_errors.ph` | `universe.math.numerical_errors` | `math/numerical_errors.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `FloatReduction` | `universe.math.reduction` | `math/reduction.ph` | `universe.math.reduction` | `math/reduction.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `I` | `universe.numbers.algebra` | `numbers/algebra.ph` | `universe.numbers.algebra` | `numbers/algebra.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `J` | `universe.numbers.algebra` | `numbers/algebra.ph` | `universe.numbers.algebra` | `numbers/algebra.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `K` | `universe.numbers.algebra` | `numbers/algebra.ph` | `universe.numbers.algebra` | `numbers/algebra.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Basis` | `universe.numbers.algebra` | `numbers/algebra.ph` | `universe.numbers.algebra` | `numbers/algebra.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `RealEmbeddable` | `universe.numbers.algebra` | `numbers/algebra.ph` | `universe.numbers.algebra` | `numbers/algebra.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `BasisRealizable` | `universe.numbers.algebra` | `numbers/algebra.ph` | `universe.numbers.algebra` | `numbers/algebra.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Addable` | `universe.numbers.capabilities` | `numbers/capabilities.ph` | `universe.numbers.capabilities` | `numbers/capabilities.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Subtractable` | `universe.numbers.capabilities` | `numbers/capabilities.ph` | `universe.numbers.capabilities` | `numbers/capabilities.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Multipliable` | `universe.numbers.capabilities` | `numbers/capabilities.ph` | `universe.numbers.capabilities` | `numbers/capabilities.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Negatable` | `universe.numbers.capabilities` | `numbers/capabilities.ph` | `universe.numbers.capabilities` | `numbers/capabilities.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `IntegerPower` | `universe.numbers.capabilities` | `numbers/capabilities.ph` | `universe.numbers.capabilities` | `numbers/capabilities.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Zero` | `universe.numbers.capabilities` | `numbers/capabilities.ph` | `universe.numbers.capabilities` | `numbers/capabilities.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `One` | `universe.numbers.capabilities` | `numbers/capabilities.ph` | `universe.numbers.capabilities` | `numbers/capabilities.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Complex` | `universe.numbers.complex` | `numbers/complex.ph` | `universe.numbers.complex` | `numbers/complex.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `ComplexDivision` | `universe.numbers.complex` | `numbers/complex.ph` | `universe.numbers.complex` | `numbers/complex.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `FloatComplex` | `universe.numbers.float_complex` | `numbers/float_complex.ph` | `universe.numbers.float_complex` | `numbers/float_complex.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Quaternion` | `universe.numbers.quaternion` | `numbers/quaternion.ph` | `universe.numbers.quaternion` | `numbers/quaternion.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Rational` | `universe.numbers.rational` | `numbers/rational.ph` | `universe.numbers.rational` | `numbers/rational.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Behavior` | `universe.object.behavior` | `object/behavior.ph` | `universe.object.behavior` | `object/behavior.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `Class` | `universe.object.class` | `object/class.ph` | `universe.object.class` | `object/class.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `Ellipsis` | `universe.object.ellipsis` | `object/ellipsis.ph` | `universe.object.ellipsis` | `object/ellipsis.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Metaclass` | `universe.object.metaclass` | `object/metaclass.ph` | `universe.object.metaclass` | `object/metaclass.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `Object` | `universe.object.object` | `object/object.ph` | `universe.object.object` | `object/object.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `Ordering` | `universe.object.ordering` | `object/ordering.ph` | `universe.object.ordering` | `object/ordering.ph` | `native data` | `LEGACY_PATH_UNCHANGED` |
| `Option` | `universe.option.option` | `option/option.ph` | `universe.option` | `option.ph` | `native data` | `MOVED_TO_SHADOW_MODULE; NORMATIVE_PATH_UPDATED; PRODUCTION_LEGACY` |
| `Unit` | `universe.option.unit` | `option/unit.ph` | `universe.scalar.unit` | `scalar/unit.ph` | `intrinsic type` | `MOVED_TO_SHADOW_MODULE; NORMATIVE_PATH_UPDATED; PRODUCTION_LEGACY` |
| `Bernoulli` | `universe.random.bernoulli` | `random/bernoulli.ph` | `universe.random.bernoulli` | `random/bernoulli.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Distribution` | `universe.random.distribution` | `random/distribution.ph` | `universe.random.distribution` | `random/distribution.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Generator` | `universe.random.generator` | `random/generator.ph` | `universe.random.generator` | `random/generator.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `EntropyError` | `universe.random.secure` | `random/secure.ph` | `universe.random.secure` | `random/secure.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `SecureRandom` | `universe.random.secure` | `random/secure.ph` | `universe.random.secure` | `random/secure.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `UniformInt` | `universe.random.uniform_int` | `random/uniform_int.ph` | `universe.random.uniform_int` | `random/uniform_int.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Xoshiro256PlusPlus` | `universe.random.xoshiro256pp` | `random/xoshiro256pp.ph` | `universe.random.xoshiro256pp` | `random/xoshiro256pp.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Attribute` | `universe.reflection.attribute` | `reflection/attribute.ph` | `universe.reflection.attribute` | `reflection/attribute.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `ChildModuleTable` | `universe.reflection.child-module-table` | `reflection/child-module-table.ph` | `universe.reflection.child-module-table` | `reflection/child-module-table.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `ExportKind` | `universe.reflection.export-kind` | `reflection/export-kind.ph` | `universe.reflection.export-kind` | `reflection/export-kind.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `ExportTable` | `universe.reflection.export-table` | `reflection/export-table.ph` | `universe.reflection.export-table` | `reflection/export-table.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `Export` | `universe.reflection.export` | `reflection/export.ph` | `universe.reflection.export` | `reflection/export.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `Message` | `universe.reflection.message` | `reflection/message.ph` | `universe.reflection.message` | `reflection/message.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `ModuleDependency` | `universe.reflection.module-dependency` | `reflection/module-dependency.ph` | `universe.reflection.module-dependency` | `reflection/module-dependency.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `ModuleIdentity` | `universe.reflection.module-identity` | `reflection/module-identity.ph` | `universe.reflection.module-identity` | `reflection/module-identity.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `Module` | `universe.reflection.module` | `reflection/module.ph` | `universe.reflection.module` | `reflection/module.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `PackageAuthor` | `universe.reflection.package-author` | `reflection/package-author.ph` | `universe.reflection.package-author` | `reflection/package-author.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `PackageIdentity` | `universe.reflection.package-identity` | `reflection/package-identity.ph` | `universe.reflection.package-identity` | `reflection/package-identity.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `PackageInfo` | `universe.reflection.package-info` | `reflection/package-info.ph` | `universe.reflection.package-info` | `reflection/package-info.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `Package` | `universe.reflection.package-object` | `reflection/package-object.ph` | `universe.reflection.package-object` | `reflection/package-object.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `PackageRequirement` | `universe.reflection.package-requirement` | `reflection/package-requirement.ph` | `universe.reflection.package-requirement` | `reflection/package-requirement.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `ProjectIdentity` | `universe.reflection.project-identity` | `reflection/project-identity.ph` | `universe.reflection.project-identity` | `reflection/project-identity.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `ProjectManifest` | `universe.reflection.project-manifest` | `reflection/project-manifest.ph` | `universe.reflection.project-manifest` | `reflection/project-manifest.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `Project` | `universe.reflection.project` | `reflection/project.ph` | `universe.reflection.project` | `reflection/project.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `ResolvedProjectDependency` | `universe.reflection.resolved-project-dependency` | `reflection/resolved-project-dependency.ph` | `universe.reflection.resolved-project-dependency` | `reflection/resolved-project-dependency.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `Selector` | `universe.reflection.selector` | `reflection/selector.ph` | `universe.reflection.selector` | `reflection/selector.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `SelectorPattern` | `universe.reflection.selector` | `reflection/selector.ph` | `universe.reflection.selector` | `reflection/selector.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `Product` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `OrderedProduct` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `UnorderedProduct` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `ProductKind` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `ProductShape` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `OrderedTail` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `PositionalTail` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `type` | `LEGACY_PATH_UNCHANGED` |
| `LabeledTail` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `type` | `LEGACY_PATH_UNCHANGED` |
| `OrderedShape` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `Presence` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `FieldInfo` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `ProductField` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `FixedFields` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `type` | `LEGACY_PATH_UNCHANGED` |
| `UnorderedTail` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `UnorderedShape` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `FieldLookup` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `UnorderedShapeView` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `Sum` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Variant` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `intrinsic data` | `LEGACY_PATH_UNCHANGED` |
| `Variants` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `type` | `LEGACY_PATH_UNCHANGED` |
| `VariantNames` | `universe.reflection.structural` | `reflection/structural.ph` | `universe.reflection.structural` | `reflection/structural.ph` | `type` | `LEGACY_PATH_UNCHANGED` |
| `TypingContext` | `universe.reflection.typing.context` | `reflection/typing/context.ph` | `universe.reflection.typing.context` | `reflection/typing/context.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Typing` | `universe.reflection.typing.context` | `reflection/typing/context.ph` | `universe.reflection.typing.context` | `reflection/typing/context.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RelationEvidence` | `universe.reflection.typing.evidence` | `reflection/typing/evidence.ph` | `universe.reflection.typing.evidence` | `reflection/typing/evidence.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RelationFailure` | `universe.reflection.typing.evidence` | `reflection/typing/evidence.ph` | `universe.reflection.typing.evidence` | `reflection/typing/evidence.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `DynamicBoundary` | `universe.reflection.typing.evidence` | `reflection/typing/evidence.ph` | `universe.reflection.typing.evidence` | `reflection/typing/evidence.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `ReflectionCapability` | `universe.reflection.typing.evidence` | `reflection/typing/evidence.ph` | `universe.reflection.typing.evidence` | `reflection/typing/evidence.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `GenericSignature` | `universe.reflection.typing.generic-signature` | `reflection/typing/generic-signature.ph` | `universe.reflection.typing.generic-signature` | `reflection/typing/generic-signature.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `GenericConstraint` | `universe.reflection.typing.generic-signature` | `reflection/typing/generic-signature.ph` | `universe.reflection.typing.generic-signature` | `reflection/typing/generic-signature.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `KindDescriptor` | `universe.reflection.typing.kind` | `reflection/typing/kind.ph` | `universe.reflection.typing.kind` | `reflection/typing/kind.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `AtomicKind` | `universe.reflection.typing.kind` | `reflection/typing/kind.ph` | `universe.reflection.typing.kind` | `reflection/typing/kind.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `FunctionKind` | `universe.reflection.typing.kind` | `reflection/typing/kind.ph` | `universe.reflection.typing.kind` | `reflection/typing/kind.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Type` | `universe.reflection.typing.kind` | `reflection/typing/kind.ph` | `universe.reflection.typing.kind` | `reflection/typing/kind.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TypingResult` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TypingKnown` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TypingUnknown` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TypingInvalid` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TypingUnavailable` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TypingCancelled` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TypingBudgetExceeded` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TypingInternalFailure` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TypeRelationResult` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RelationSatisfied` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RelationRejected` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RelationDynamicBoundary` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RelationBlocked` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RelationCancelled` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RelationBudgetExceeded` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RelationInternalFailure` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MemberLookupResult` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MemberFound` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MemberMissing` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MemberDynamicBoundary` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MemberBlocked` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MemberCancelled` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MemberBudgetExceeded` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `MemberInternalFailure` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `universe.reflection.typing.result` | `reflection/typing/result.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `CallableSignature` | `universe.reflection.typing.signature` | `reflection/typing/signature.ph` | `universe.reflection.typing.signature` | `reflection/typing/signature.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `CallableParameter` | `universe.reflection.typing.signature` | `reflection/typing/signature.ph` | `universe.reflection.typing.signature` | `reflection/typing/signature.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `FieldSignature` | `universe.reflection.typing.signature` | `reflection/typing/signature.ph` | `universe.reflection.typing.signature` | `reflection/typing/signature.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TypeDescriptor` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `AppliedType` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `UnionType` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TupleType` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `RecordType` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `CallableType` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TypeLambda` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `SpecialType` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `SelfType` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TypeParameter` | `universe.reflection.typing.type-parameter` | `reflection/typing/type-parameter.ph` | `universe.reflection.typing.type-parameter` | `reflection/typing/type-parameter.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TypeUse` | `universe.reflection.typing.type-use` | `reflection/typing/type-use.ph` | `universe.reflection.typing.type-use` | `reflection/typing/type-use.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Uri` | `universe.reflection.uri` | `reflection/uri.ph` | `universe.reflection.uri` | `reflection/uri.ph` | `native class` | `LEGACY_PATH_UNCHANGED` |
| `Bool` | `universe.scalar.bool` | `scalar/bool.ph` | `universe.scalar.bool` | `scalar/bool.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `True` | `universe.scalar.bool` | `scalar/bool.ph` | `universe.scalar.bool` | `scalar/bool.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `False` | `universe.scalar.bool` | `scalar/bool.ph` | `universe.scalar.bool` | `scalar/bool.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `Char` | `universe.scalar.char` | `scalar/char.ph` | `universe.scalar.char` | `scalar/char.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `Nil` | `universe.scalar.nil` | `scalar/nil.ph` | `universe.scalar.nil` | `scalar/nil.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `Number` | `universe.scalar.number` | `scalar/number.ph` | `universe.scalar.number` | `scalar/number.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `Int` | `universe.scalar.number` | `scalar/number.ph` | `universe.scalar.int` | `scalar/int.ph` | `native primitive class` | `MOVED_TO_SHADOW_MODULE; NORMATIVE_PATH_UPDATED; PRODUCTION_LEGACY` |
| `Float` | `universe.scalar.number` | `scalar/number.ph` | `universe.scalar.float` | `scalar/float.ph` | `native primitive class` | `MOVED_TO_SHADOW_MODULE; NORMATIVE_PATH_UPDATED; PRODUCTION_LEGACY` |
| `String` | `universe.scalar.string` | `scalar/string.ph` | `universe.scalar.string` | `scalar/string.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `StringChars` | `universe.scalar.string` | `scalar/string.ph` | `universe.scalar.string` | `scalar/string.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `StringCharIterator` | `universe.scalar.string` | `scalar/string.ph` | `universe.scalar.string` | `scalar/string.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `StringByteSequence` | `universe.scalar.string` | `scalar/string.ph` | `universe.scalar.string` | `scalar/string.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `StringByteIterator` | `universe.scalar.string` | `scalar/string.ph` | `universe.scalar.string` | `scalar/string.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `StringCodePointSequence` | `universe.scalar.string` | `scalar/string.ph` | `universe.scalar.string` | `scalar/string.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `StringCodePointIterator` | `universe.scalar.string` | `scalar/string.ph` | `universe.scalar.string` | `scalar/string.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `StringIndex` | `universe.scalar.string_index` | `scalar/string_index.ph` | `universe.scalar.string_index` | `scalar/string_index.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Symbol` | `universe.scalar.symbol` | `scalar/symbol.ph` | `universe.scalar.symbol` | `scalar/symbol.ph` | `native primitive class` | `LEGACY_PATH_UNCHANGED` |
| `StringBuilder` | `universe.text.builder` | `text/builder.ph` | `universe.text.builder` | `text/builder.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TextCursor` | `universe.text.cursor` | `text/cursor.ph` | `universe.text.cursor` | `text/cursor.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Utf8Error` | `universe.text.errors` | `text/errors.ph` | `universe.text.errors` | `text/errors.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `IntParseError` | `universe.text.errors` | `text/errors.ph` | `universe.text.errors` | `text/errors.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `FloatParseError` | `universe.text.errors` | `text/errors.ph` | `universe.text.errors` | `text/errors.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `BoolParseError` | `universe.text.errors` | `text/errors.ph` | `universe.text.errors` | `text/errors.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `SourceLocation` | `universe.text.line_map` | `text/line_map.ph` | `universe.text.line_map` | `text/line_map.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `LineMap` | `universe.text.line_map` | `text/line_map.ph` | `universe.text.line_map` | `text/line_map.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `TextPattern` | `universe.text.patterns` | `text/patterns.ph` | `universe.text.patterns` | `text/patterns.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `Substring` | `universe.text.substring` | `text/substring.ph` | `universe.text.substring` | `text/substring.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `SubstringChars` | `universe.text.substring` | `text/substring.ph` | `universe.text.substring` | `text/substring.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `SubstringCharIterator` | `universe.text.substring` | `text/substring.ph` | `universe.text.substring` | `text/substring.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `SubstringBytes` | `universe.text.substring` | `text/substring.ph` | `universe.text.substring` | `text/substring.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `SubstringByteIterator` | `universe.text.substring` | `text/substring.ph` | `universe.text.substring` | `text/substring.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Split` | `universe.text.substring` | `text/substring.ph` | `universe.text.substring` | `text/substring.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `SplitIterator` | `universe.text.substring` | `text/substring.ph` | `universe.text.substring` | `text/substring.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `SplitWhitespace` | `universe.text.substring` | `text/substring.ph` | `universe.text.substring` | `text/substring.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `SplitWhitespaceIterator` | `universe.text.substring` | `text/substring.ph` | `universe.text.substring` | `text/substring.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Lines` | `universe.text.substring` | `text/substring.ph` | `universe.text.substring` | `text/substring.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `LinesIterator` | `universe.text.substring` | `text/substring.ph` | `universe.text.substring` | `text/substring.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Text` | `universe.text.traits` | `text/traits.ph` | `universe.text.traits` | `text/traits.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `TextWriter` | `universe.text.writer` | `text/writer.ph` | `universe.text.writer` | `text/writer.ph` | `trait` | `LEGACY_PATH_UNCHANGED` |
| `DateOverflow` | `universe.time.calendar.date` | `time/calendar/date.ph` | `universe.time.calendar.date` | `time/calendar/date.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Date` | `universe.time.calendar.date` | `time/calendar/date.ph` | `universe.time.calendar.date` | `time/calendar/date.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `DateTime` | `universe.time.calendar.datetime` | `time/calendar/datetime.ph` | `universe.time.calendar.datetime` | `time/calendar/datetime.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `LocalTimeDisambiguation` | `universe.time.calendar.datetime` | `time/calendar/datetime.ph` | `universe.time.calendar.datetime` | `time/calendar/datetime.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `ZonedDateTime` | `universe.time.calendar.datetime` | `time/calendar/datetime.ph` | `universe.time.calendar.datetime` | `time/calendar/datetime.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `LocalTimeResolution` | `universe.time.calendar.datetime` | `time/calendar/datetime.ph` | `universe.time.calendar.datetime` | `time/calendar/datetime.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Gregorian` | `universe.time.calendar.month` | `time/calendar/month.ph` | `universe.time.calendar.month` | `time/calendar/month.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Month` | `universe.time.calendar.month` | `time/calendar/month.ph` | `universe.time.calendar.month` | `time/calendar/month.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `YearMonth` | `universe.time.calendar.partial` | `time/calendar/partial.ph` | `universe.time.calendar.partial` | `time/calendar/partial.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `MonthDay` | `universe.time.calendar.partial` | `time/calendar/partial.ph` | `universe.time.calendar.partial` | `time/calendar/partial.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Period` | `universe.time.calendar.period` | `time/calendar/period.ph` | `universe.time.calendar.period` | `time/calendar/period.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Time` | `universe.time.calendar.time` | `time/calendar/time.ph` | `universe.time.calendar.time` | `time/calendar/time.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Weekday` | `universe.time.calendar.weekday` | `time/calendar/weekday.ph` | `universe.time.calendar.weekday` | `time/calendar/weekday.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Instant` | `universe.time.clock` | `time/clock.ph` | `universe.time.clock` | `time/clock.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Clock` | `universe.time.clock` | `time/clock.ph` | `universe.time.clock` | `time/clock.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `SystemClock` | `universe.time.clock` | `time/clock.ph` | `universe.time.clock` | `time/clock.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Duration` | `universe.time.duration` | `time/duration.ph` | `universe.time.duration` | `time/duration.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `Measurement` | `universe.time.duration` | `time/duration.ph` | `universe.time.duration` | `time/duration.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `UtcOffset` | `universe.time.offset` | `time/offset.ph` | `universe.time.offset` | `time/offset.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `ManualClock` | `universe.time.testing.clocks` | `time/testing/clocks.ph` | `universe.time.testing.clocks` | `time/testing/clocks.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `ManualWallClock` | `universe.time.testing.clocks` | `time/testing/clocks.ph` | `universe.time.testing.clocks` | `time/testing/clocks.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `WallClock` | `universe.time.timestamp` | `time/timestamp.ph` | `universe.time.timestamp` | `time/timestamp.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `SystemWallClock` | `universe.time.timestamp` | `time/timestamp.ph` | `universe.time.timestamp` | `time/timestamp.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `Timestamp` | `universe.time.timestamp` | `time/timestamp.ph` | `universe.time.timestamp` | `time/timestamp.ph` | `data` | `LEGACY_PATH_UNCHANGED` |
| `TimeZone` | `universe.time.timezone` | `time/timezone.ph` | `universe.time.timezone` | `time/timezone.ph` | `class` | `LEGACY_PATH_UNCHANGED` |
| `ZoneTransition` | `universe.time.zone_transition` | `time/zone_transition.ph` | `universe.time.zone_transition` | `time/zone_transition.ph` | `data` | `LEGACY_PATH_UNCHANGED` |

## Current legacy native-association metadata

The table is derived from `UniverseKey::source_path()` and `UniverseKey::name()` together with `UNIVERSE_BINDINGS` in `phalcom-native-meta/src/universe.rs`. The production source declaration catalog in `phalcom-modules/src/builtin_interface.rs` resolves that key to the declaration with that name in the indicated legacy module. These are current production inputs, not the target owners in the amended normative spec; production still needs a later coordinated migration. The catalog was not built or semantically checked for this shadow copy. `UNIVERSE_BINDINGS.exported` and `.prelude` are metadata policy fields, not proof of source-authored export or effective prelude presence. The table includes all **78** `UniverseKey` variants; **72** have a `UNIVERSE_BINDINGS` row and **6** are key/path entries without one.

| UniverseKey | Key/source declaration name | Legacy source module path | Binding kind | Metadata exported | Metadata prelude | Binding row |
|---|---|---|---|---|---|---|
| `Object` | `Object` | `universe.object.object` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Behavior` | `Behavior` | `universe.object.behavior` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `Class` | `Class` | `universe.object.class` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Metaclass` | `Metaclass` | `universe.object.metaclass` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `Number` | `Number` | `universe.scalar.number` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Int` | `Int` | `universe.scalar.number` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Float` | `Float` | `universe.scalar.number` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `String` | `String` | `universe.scalar.string` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Char` | `Char` | `universe.scalar.char` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Nil` | `Nil` | `universe.scalar.nil` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `Bool` | `Bool` | `universe.scalar.bool` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `True` | `True` | `universe.scalar.bool` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `False` | `False` | `universe.scalar.bool` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `Symbol` | `Symbol` | `universe.scalar.symbol` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Selector` | `Selector` | `universe.reflection.selector` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `SelectorPattern` | `SelectorPattern` | `universe.reflection.selector` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Function` | `Function` | `universe.callable.function` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Closure` | `Closure` | `universe.callable.closure` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `BoundMethod` | `BoundMethod` | `universe.callable.method` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `Method` | `Method` | `universe.callable.method` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `MethodFamily` | `MethodFamily` | `universe.callable.family` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `BoundMethodFamily` | `BoundMethodFamily` | `universe.callable.family` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `Family` | `Family` | `universe.callable.family` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `Option` | `Option` | `universe.option.option` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Result` | `Result` | `universe.errors.result` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Ordering` | `Ordering` | `universe.object.ordering` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Some` | `Some` | `universe.option.option` | `RuntimeSupportClass` | `false` | `false` | `UNIVERSE_BINDINGS` |
| `None` | `None` | `universe.option.option` | `RuntimeSupportClass` | `false` | `false` | `UNIVERSE_BINDINGS` |
| `EmptyRecord` | `EmptyRecord` | `universe.collections.record` | — | — | — | no `UNIVERSE_BINDINGS` row |
| `Collection` | `Collection` | `universe.collections.collection` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `List` | `List` | `universe.collections.list` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Array` | `Array` | `universe.collections.array` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Vector` | `Vector` | `universe.collections.vector` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Slice` | `Slice` | `universe.collections.slice` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Buffer` | `Buffer` | `universe.collections.buffer` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Map` | `Map` | `universe.collections.map` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Set` | `Set` | `universe.collections.set` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `MutableBag` | `MutableBag` | `universe.collections.mutable_bag` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `IndexedDict` | `IndexedDict` | `universe.collections.indexed_dict` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `OrderedSet` | `OrderedSet` | `universe.collections.ordered_set` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Bag` | `Bag` | `universe.collections.bag` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Bijection` | `Bijection` | `universe.collections.bijection` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Tuple` | `Tuple` | `universe.collections.tuple` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Pack` | `Pack` | `universe.collections.pack` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Record` | `Record` | `universe.collections.record` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Range` | `Range` | `universe.collections.range` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Bytes` | `Bytes` | `universe.collections.bytes` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Module` | `Module` | `universe.reflection.module` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `Package` | `Package` | `universe.reflection.package_object` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `Project` | `Project` | `universe.reflection.project` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `System` | `System` | `universe.concurrency.fiber` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `SystemClock` | `SystemClock` | `universe.time.clock` | — | — | — | no `UNIVERSE_BINDINGS` row |
| `SystemWallClock` | `SystemWallClock` | `universe.time.timestamp` | — | — | — | no `UNIVERSE_BINDINGS` row |
| `TimeZone` | `TimeZone` | `universe.time.timezone` | — | — | — | no `UNIVERSE_BINDINGS` row |
| `FloatMathKernel` | `FloatMathKernel` | `universe.math.float_kernel` | — | — | — | no `UNIVERSE_BINDINGS` row |
| `SecureRandom` | `SecureRandom` | `universe.random.secure` | — | — | — | no `UNIVERSE_BINDINGS` row |
| `Message` | `Message` | `universe.reflection.message` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `Attribute` | `Attribute` | `universe.reflection.attribute` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Error` | `Error` | `universe.errors.error` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `MessageNotUnderstood` | `MessageNotUnderstood` | `universe.errors.error` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `CannotYieldAcrossNativeFrame` | `CannotYieldAcrossNativeFrame` | `universe.errors.error` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `UseAfterCloseError` | `UseAfterCloseError` | `universe.errors.error` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `Fiber` | `Fiber` | `universe.concurrency.fiber` | `Class` | `true` | `true` | `UNIVERSE_BINDINGS` |
| `Resource` | `Resource` | `universe.collections.bytes` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `ProjectManifest` | `ProjectManifest` | `universe.reflection.project_manifest` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `PackageInfo` | `PackageInfo` | `universe.reflection.package_info` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `PackageAuthor` | `PackageAuthor` | `universe.reflection.package_author` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `PackageRequirement` | `PackageRequirement` | `universe.reflection.package_requirement` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `ResolvedProjectDependency` | `ResolvedProjectDependency` | `universe.reflection.resolved_project_dependency` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `ModuleDependency` | `ModuleDependency` | `universe.reflection.module_dependency` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `ExportTable` | `ExportTable` | `universe.reflection.export_table` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `Export` | `Export` | `universe.reflection.export` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `ExportKind` | `ExportKind` | `universe.reflection.export_kind` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `ChildModuleTable` | `ChildModuleTable` | `universe.reflection.child_module_table` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `ModuleIdentity` | `ModuleIdentity` | `universe.reflection.module_identity` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `PackageIdentity` | `PackageIdentity` | `universe.reflection.package_identity` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `ProjectIdentity` | `ProjectIdentity` | `universe.reflection.project_identity` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |
| `Uri` | `Uri` | `universe.reflection.uri` | `Class` | `true` | `false` | `UNIVERSE_BINDINGS` |

## Current legacy prelude-membership policy

`PreludeTypeMap::canonical_universe()` in `phalcom-semantic/src/prelude.rs` considers `UNIVERSE_BINDINGS` entries with `prelude: true`, adding a name only when the production legacy source declaration catalog resolves the key. It then adds the hardcoded entries below. `Unit` is additionally asserted to exist in the production source catalog; the other hardcoded entries are inserted directly. This table records current production legacy policy inputs and owner paths; it does not describe a connected shadow prelude or prove compiler results.

Native metadata rows flagged `prelude: true`: **37**. Hardcoded source-owned additions in `prelude.rs`: **9**. Compiler-provided `core.proof` additions: **3**. Candidate policy names before any source-catalog lookup omissions: **49**.

| Prelude name | Current owner module | Source of membership |
|---|---|---|
| `Object` | `universe.object.object` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Class` | `universe.object.class` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Number` | `universe.scalar.number` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Int` | `universe.scalar.number` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Float` | `universe.scalar.number` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `String` | `universe.scalar.string` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Char` | `universe.scalar.char` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Bool` | `universe.scalar.bool` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Symbol` | `universe.scalar.symbol` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Selector` | `universe.reflection.selector` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `SelectorPattern` | `universe.reflection.selector` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Function` | `universe.callable.function` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Option` | `universe.option.option` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Result` | `universe.errors.result` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Ordering` | `universe.object.ordering` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Collection` | `universe.collections.collection` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `List` | `universe.collections.list` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Array` | `universe.collections.array` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Vector` | `universe.collections.vector` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Slice` | `universe.collections.slice` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Buffer` | `universe.collections.buffer` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Map` | `universe.collections.map` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Set` | `universe.collections.set` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `MutableBag` | `universe.collections.mutable_bag` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `IndexedDict` | `universe.collections.indexed_dict` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `OrderedSet` | `universe.collections.ordered_set` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Bag` | `universe.collections.bag` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Bijection` | `universe.collections.bijection` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Tuple` | `universe.collections.tuple` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Pack` | `universe.collections.pack` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Record` | `universe.collections.record` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Range` | `universe.collections.range` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Bytes` | `universe.collections.bytes` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `System` | `universe.concurrency.fiber` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Attribute` | `universe.reflection.attribute` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Error` | `universe.errors.error` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Fiber` | `universe.concurrency.fiber` | `UNIVERSE_BINDINGS.prelude = true` (source-catalog lookup required) |
| `Unit` | `universe.option.unit` | hardcoded; source-catalog presence asserted |
| `Duration` | `universe.time.duration` | hardcoded source-owned addition |
| `Timestamp` | `universe.time.timestamp` | hardcoded source-owned addition |
| `DuplicateKeyError` | `universe.errors.argument` | hardcoded source-owned addition |
| `InvalidIntervalError` | `universe.errors.argument` | hardcoded source-owned addition |
| `RangeOverlapError` | `universe.errors.argument` | hardcoded source-owned addition |
| `Bound` | `universe.collections.interval` | hardcoded source-owned addition |
| `Interval` | `universe.collections.interval` | hardcoded source-owned addition |
| `RangeMap` | `universe.collections.range_map` | hardcoded source-owned addition |
| `Equivalent` | `universe.core.proof` | hardcoded compiler-provided core-surface entry; not package-authored |
| `Subtype` | `universe.core.proof` | hardcoded compiler-provided core-surface entry; not package-authored |
| `Conforms` | `universe.core.proof` | hardcoded compiler-provided core-surface entry; not package-authored |

## S3 scalar relocation review

| Declaration | Legacy owner → shadow source owner | Current production metadata | Source dependents | Bootstrap and compatibility review |
|---|---|---|---|---|
| `Number` | `universe.scalar.number` → `universe.scalar.number` | `UniverseKey::Number`; legacy source path `scalar.number`; binding metadata says exported class and prelude member | Lexical occurrences in 12 modules; complete module list is in `module-map.md` under “Scalar type source references.” | Runtime class slot and relation remain keyed by `UniverseKey::Number`; metadata relation is `Number <: Object`. Declaration body is unchanged. Source module now authors `export Number`; scalar facade re-exports it. |
| `Int` | `universe.scalar.number` → `universe.scalar.int` | `UniverseKey::Int`; production source path remains `scalar.number`; binding metadata says exported class and prelude member | Lexical occurrences in 72 modules; complete module list is in `module-map.md` under “Scalar type source references.” No shadow module imports `Int` or `Float` from the old path; the two new `.number` imports in the target modules import `Number`. | Runtime class slot remains keyed by `UniverseKey::Int`; metadata relation is `Int <: Number`. Name, declaration form, superclass, members, and method bodies are preserved. Source module now authors `export Int`; scalar facade re-exports it. |
| `Float` | `universe.scalar.number` → `universe.scalar.float` | `UniverseKey::Float`; production source path remains `scalar.number`; binding metadata says exported class and prelude member | Lexical occurrences in 19 modules; complete module list is in `module-map.md` under “Scalar type source references.” No shadow module imports `Int` or `Float` from the old path; the two new `.number` imports in the target modules import `Number`. | Runtime class slot remains keyed by `UniverseKey::Float`; metadata relation is `Float <: Number`. Name, declaration form, superclass, members, and method bodies are preserved. Source module now authors `export Float`; scalar facade re-exports it. |

The current native keys, legacy `source_path()` values, prelude policy, runtime class slots, and class relations still describe the production tree. They have not been retargeted to the shadow modules. Source references keep the same unqualified names; their intended shadow binding therefore requires an isolated prelude/catalog that maps those names to the shadow owners. The scalar split had no compiler or runtime validation. Its then-current spec boundary was superseded by the Phase S3 foundational-type amendment below.

## S3 foundational owner relocation review

| Declaration | Legacy owner → shadow source owner | Normative canonical owner | Current production association | Source/dependency note |
|---|---|---|---|---|
| `Option` | `universe.option.option` → `universe.option` | `universe.option::Option` | `UniverseKey::Option` and prelude metadata still name `universe.option.option` | Body preserved byte-for-byte; unqualified `Result` references remain prelude-bound in the shadow source. |
| `Result` | `universe.errors.result` → `universe.result` | `universe.result::Result` | `UniverseKey::Result` and prelude metadata still name `universe.errors.result` | Body preserved byte-for-byte; unqualified `Option` references remain prelude-bound in the shadow source. |
| `Unit` | `universe.option.unit` → `universe.scalar.unit` | `universe.scalar.unit::Unit` | No `UniverseKey`; production prelude hardcodes `universe.option.unit` and asserts source-catalog presence | Alias body preserved byte-for-byte; scalar facade now re-exports it. |

The normative spec now agrees with the target source owners for `Int`, `Float`, `Option`, `Result`, and `Unit`. Production native/source anchors, prelude construction, and runtime class attachments have not moved. The option package file was removed because `src/option.ph` and `src/option/package.ph` resolve ambiguously to the same module path. No explicit reciprocal imports were added between Option and Result; their references remain prelude-visible names. Source structure and body preservation were checked, but shadow semantic resolution and runtime initialization remain unverified.

## Required fields before a declaration moves

Before relocating a declaration, extend its ledger entry to record and review its native key/association, prelude membership, public export status, source dependents, bootstrap role, and compatibility requirements, in addition to its old and proposed canonical source module. These fields are not complete for the current baseline inventory.

## Identity constraints

- Preserve conceptual `ProjectIdentity::Universe`; do not create a shadow project identity.
- Imports and re-exports must resolve to declaration owners rather than create nominal declarations.
- Amend the owning normative specification with any canonical module move as part of the source-owner change.
- Keep production native associations, prelude ownership, and runtime identity aligned with the amended specification before integration or cutover; dedicated verification remains required.
