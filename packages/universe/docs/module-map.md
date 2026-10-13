# Current shadow module map

This is an inventory of the current shadow source layout. Module paths are derived from `src/` paths (`package.ph` is the containing package module); the initial legacy owners are retained in `identity-migration.md`. The scanner includes every unindented top-level declaration line matching zero or more `intrinsic`, `native`, `primitive`, or `type` prefixes followed by one of `data`, `enum`, `class`, `trait`, or `type`, then an identifier. It therefore includes intrinsic aliases/data facades such as `Unit`, `RecordFields`, `Record`, `EmptyRecord`, `TypePack`, `Tuple`, and structural declarations. It separately includes top-level `let`, `let mut`, `const`, `fn`, and `def` names. It does not parse Telos syntax, enumerate nested declarations/variants/members, infer generated declarations, or prove semantic identity. Source statements remain authoritative; no compiler validation was performed.

Recognized top-level type-like declarations: **364**. Recognized top-level value declarations: **77**.

| Current shadow module | Source file | Top-level type-like declarations | Top-level values |
|---|---|---|---|
| `universe.callable.closure` | `callable/closure.ph` | `Closure` | — |
| `universe.callable.family` | `callable/family.ph` | `Family`, `MethodFamily`, `BoundMethodFamily` | — |
| `universe.callable.function` | `callable/function.ph` | `Function` | — |
| `universe.callable.method` | `callable/method.ph` | `Method`, `BoundMethod` | — |
| `universe.callable` | `callable/package.ph` | — | — |
| `universe.collections.array` | `collections/array.ph` | `Array`, `ArrayIterator` | — |
| `universe.collections.bag` | `collections/bag.ph` | `Bag`, `BagIterator` | — |
| `universe.collections.bijection` | `collections/bijection.ph` | `Bijection`, `BijectionIterator` | — |
| `universe.collections.buffer` | `collections/buffer.ph` | `Buffer`, `BufferIterator` | — |
| `universe.collections.bytes` | `collections/bytes.ph` | `Bytes`, `BytesIterator`, `OpenMode`, `Path`, `Resource`, `UnflushedError`, `ConcurrentOperationError`, `StreamProtocolError`, `WriteZeroError`, `BytesReader`, `BytesWriter`, `BufferedWriter`, `BufferedReader` | — |
| `universe.collections.collection` | `collections/collection.ph` | `Collection` | — |
| `universe.collections.indexed_dict` | `collections/indexed_dict.ph` | `IndexedDict`, `IndexedDictIterator` | — |
| `universe.collections.interval` | `collections/interval.ph` | `Bound`, `Interval` | — |
| `universe.collections.iterator_laws` | `collections/iterator_laws.ph` | `Iterator`, `Iterable`, `MappedIterator`, `FilterIterator`, `FilterMapIterator`, `FlatMapIterator`, `FlattenIterator`, `ChainIterator`, `SkipIterator`, `SkipWhileIterator`, `TakeIterator`, `TakeWhileIterator`, `MapWhileIterator`, `InspectIterator`, `StrictZipIterator`, `IndexedIterator` | — |
| `universe.collections.list` | `collections/list.ph` | `List`, `ListIterator` | — |
| `universe.collections.map` | `collections/map.ph` | `Map`, `MapIterator`, `MapKeysView`, `MapValuesView`, `Entry`, `MapEntriesView`, `MapKeysIterator`, `MapValuesIterator`, `MapEntriesIterator` | — |
| `universe.collections.mutable_bag` | `collections/mutable_bag.ph` | `MutableBag`, `MutableBagIterator`, `MutableBagEntriesView`, `MutableBagEntriesIterator` | — |
| `universe.collections.ordered_set` | `collections/ordered_set.ph` | `OrderedSet`, `OrderedSetIterator` | — |
| `universe.collections.pack` | `collections/pack.ph` | `Pack`, `PositionalPack`, `LabeledPack`, `CompletePack`, `PositionalPackView`, `LabeledPackView`, `PositionalTypePackView`, `LabeledTypePackView` | — |
| `universe.collections` | `collections/package.ph` | — | — |
| `universe.collections.range` | `collections/range.ph` | `Range`, `RangeIterator` | — |
| `universe.collections.range_map` | `collections/range_map.ph` | `RangeMapNode`, `RangeMap`, `RangeMapIterator` | — |
| `universe.collections.record` | `collections/record.ph` | `RecordFields`, `FieldKey`, `Record`, `EmptyRecord` | — |
| `universe.collections.set` | `collections/set.ph` | `Set`, `SetIterator` | — |
| `universe.collections.slice` | `collections/slice.ph` | `Slice`, `SliceIterator` | — |
| `universe.collections.tuple` | `collections/tuple.ph` | `TypePack`, `LabeledTypePack`, `CompleteTypePack`, `Tuple`, `TupleIterator`, `TupleZipIterator`, `TupleZipView` | — |
| `universe.collections.vector` | `collections/vector.ph` | `Vector`, `VectorIterator` | — |
| `universe.concurrency.fiber` | `concurrency/fiber.ph` | `System`, `Fiber`, `FutureSubscription`, `Future`, `CompletionSource`, `Tracer`, `OffBehavior`, `Backoff` | — |
| `universe.concurrency` | `concurrency/package.ph` | — | — |
| `universe.concurrent` | `concurrent/package.ph` | — | — |
| `universe.errors.argument` | `errors/argument.ph` | `ArgumentError`, `KeyError`, `IndexError`, `SliceError`, `DuplicateKeyError`, `BijectionConflictError`, `InvalidIntervalError`, `RangeOverlapError` | — |
| `universe.errors.contracts` | `errors/contracts.ph` | `PreconditionError`, `PostconditionError`, `InvariantError` | — |
| `universe.errors.error` | `errors/error.ph` | `Error`, `MessageNotUnderstood`, `CannotYieldAcrossNativeFrame`, `UseAfterCloseError`, `UnwrapError`, `TimeoutError` | — |
| `universe.errors.indexing` | `errors/indexing.ph` | — | — |
| `universe.errors.iteration` | `errors/iteration.ph` | `ZipLengthMismatchError` | — |
| `universe.errors` | `errors/package.ph` | — | — |
| `universe.errors.unimplemented` | `errors/unimplemented.ph` | `Unimplemented` | — |
| `universe.errors.unsupported` | `errors/unsupported.ph` | `Unsupported` | `unsupported` |
| `universe.fs` | `fs/package.ph` | — | — |
| `universe.functional.algebra` | `functional/algebra.ph` | `Semigroup`, `Monoid`, `Additive`, `Multiplicative`, `AllTrue`, `AnyTrue`, `First`, `Last`, `Dual`, `Endo` | — |
| `universe.functional.arrow` | `functional/arrow.ph` | `Profunctor`, `Arrow`, `ArrowContext` | — |
| `universe.functional.chain` | `functional/chain.ph` | `Chain`, `Partition`, `ChainIterator`, `NonEmpty` | — |
| `universe.functional.effects` | `functional/effects.ph` | `Effects` | — |
| `universe.functional.either` | `functional/either.ph` | `Either` | — |
| `universe.functional.fold` | `functional/fold.ph` | `Fold`, `FoldFactory` | — |
| `universe.functional.folds` | `functional/folds.ph` | `Folds` | — |
| `universe.functional.higher_kinded` | `functional/higher_kinded.ph` | `Functor`, `Applicative`, `Monad`, `Identity`, `IdentityContext`, `OptionContext`, `ChainContext`, `NonEmptyContext` | — |
| `universe.functional.journal` | `functional/journal.ph` | `Journal` | — |
| `universe.functional.lens` | `functional/lens.ph` | `Lens` | — |
| `universe.functional.optics` | `functional/optics.ph` | `Iso`, `Optional`, `Prism` | — |
| `universe.functional` | `functional/package.ph` | — | — |
| `universe.functional.reader` | `functional/reader.ph` | `Reader` | — |
| `universe.functional.reader_result` | `functional/reader_result.ph` | `ReaderResult` | — |
| `universe.functional.state` | `functional/state.ph` | `StateStep`, `State` | — |
| `universe.functional.state_result` | `functional/state_result.ph` | `StateResult` | — |
| `universe.functional.validation` | `functional/validation.ph` | `Validation` | — |
| `universe.functional.writer` | `functional/writer.ph` | `Writer` | — |
| `universe.io` | `io/package.ph` | — | — |
| `universe.json` | `json/package.ph` | — | `parse`, `stringify` |
| `universe.math.complex.kernel` | `math/complex/kernel.ph` | `ComplexMathKernel` | — |
| `universe.math.complex` | `math/complex/package.ph` | — | `sqrt`, `exp`, `log`, `log1p`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`, `powInt`, `pow` |
| `universe.math.float_kernel` | `math/float_kernel.ph` | `FloatMathKernel`, `FloatAngleHelpers` | — |
| `universe.math.integer` | `math/integer.ph` | `IntegerMath` | — |
| `universe.math.numerical_errors` | `math/numerical_errors.ph` | `NumericalRangeError`, `NumericalEvaluationError` | — |
| `universe.math` | `math/package.ph` | — | `pi`, `tau`, `e`, `ln2`, `ln10`, `sqrt2`, `sqrt`, `cbrt`, `exp`, `exp2`, `expm1`, `ln`, `ln1p`, `log`, `log2`, `log10`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`, `hypot`, `degrees`, `radians`, `logSumExp`, `sum`, `mean`, `meanOption`, `logAddExp`, `log1mexp`, `factorial`, `choose`, `fallingFactorial`, `risingFactorial`, `gcd`, `lcm`, `isqrt` |
| `universe.math.reduction` | `math/reduction.ph` | `FloatReduction` | — |
| `universe.net` | `net/package.ph` | — | — |
| `universe.numbers.algebra` | `numbers/algebra.ph` | `I`, `J`, `K`, `Basis`, `RealEmbeddable`, `BasisRealizable` | `i`, `j`, `k` |
| `universe.numbers.capabilities` | `numbers/capabilities.ph` | `Addable`, `Subtractable`, `Multipliable`, `Negatable`, `IntegerPower`, `Zero`, `One` | — |
| `universe.numbers.complex` | `numbers/complex.ph` | `Complex`, `ComplexDivision` | — |
| `universe.numbers.float_complex` | `numbers/float_complex.ph` | `FloatComplex` | — |
| `universe.numbers` | `numbers/package.ph` | — | — |
| `universe.numbers.quaternion` | `numbers/quaternion.ph` | `Quaternion` | — |
| `universe.numbers.rational` | `numbers/rational.ph` | `Rational` | — |
| `universe.object.behavior` | `object/behavior.ph` | `Behavior` | — |
| `universe.object.class` | `object/class.ph` | `Class` | — |
| `universe.object.ellipsis` | `object/ellipsis.ph` | `Ellipsis` | `ellipsis` |
| `universe.object.metaclass` | `object/metaclass.ph` | `Metaclass` | — |
| `universe.object.object` | `object/object.ph` | `Object` | — |
| `universe.object.ordering` | `object/ordering.ph` | `Ordering` | — |
| `universe.object` | `object/package.ph` | — | — |
| `universe.option` | `option.ph` | `Option` | — |
| `universe` | `package.ph` | — | — |
| `universe.path` | `path/package.ph` | — | — |
| `universe.process` | `process/package.ph` | — | — |
| `universe.random.bernoulli` | `random/bernoulli.ph` | `Bernoulli` | — |
| `universe.random.distribution` | `random/distribution.ph` | `Distribution` | — |
| `universe.random.generator` | `random/generator.ph` | `Generator` | — |
| `universe.random` | `random/package.ph` | — | — |
| `universe.random.secure` | `random/secure.ph` | `EntropyError`, `SecureRandom` | — |
| `universe.random.uniform_int` | `random/uniform_int.ph` | `UniformInt` | — |
| `universe.random.xoshiro256pp` | `random/xoshiro256pp.ph` | `Xoshiro256PlusPlus` | `WORD_MODULUS`, `WORD_MASK`, `GAMMA`, `MUL1`, `MUL2` |
| `universe.result` | `result.ph` | `Result` | — |
| `universe.reflection.attribute` | `reflection/attribute.ph` | `Attribute` | — |
| `universe.reflection.child-module-table` | `reflection/child-module-table.ph` | `ChildModuleTable` | — |
| `universe.reflection.export-kind` | `reflection/export-kind.ph` | `ExportKind` | — |
| `universe.reflection.export-table` | `reflection/export-table.ph` | `ExportTable` | — |
| `universe.reflection.export` | `reflection/export.ph` | `Export` | — |
| `universe.reflection.implementation` | `reflection/implementation.ph` | — | — |
| `universe.reflection.message` | `reflection/message.ph` | `Message` | — |
| `universe.reflection.module-dependency` | `reflection/module-dependency.ph` | `ModuleDependency` | — |
| `universe.reflection.module-identity` | `reflection/module-identity.ph` | `ModuleIdentity` | — |
| `universe.reflection.module` | `reflection/module.ph` | `Module` | — |
| `universe.reflection.package-author` | `reflection/package-author.ph` | `PackageAuthor` | — |
| `universe.reflection.package-identity` | `reflection/package-identity.ph` | `PackageIdentity` | — |
| `universe.reflection.package-info` | `reflection/package-info.ph` | `PackageInfo` | — |
| `universe.reflection.package-object` | `reflection/package-object.ph` | `Package` | — |
| `universe.reflection.package-requirement` | `reflection/package-requirement.ph` | `PackageRequirement` | — |
| `universe.reflection` | `reflection/package.ph` | — | — |
| `universe.reflection.project-identity` | `reflection/project-identity.ph` | `ProjectIdentity` | — |
| `universe.reflection.project-manifest` | `reflection/project-manifest.ph` | `ProjectManifest` | — |
| `universe.reflection.project` | `reflection/project.ph` | `Project` | — |
| `universe.reflection.resolved-project-dependency` | `reflection/resolved-project-dependency.ph` | `ResolvedProjectDependency` | — |
| `universe.reflection.selector` | `reflection/selector.ph` | `Selector`, `SelectorPattern` | — |
| `universe.reflection.structural` | `reflection/structural.ph` | `Product`, `OrderedProduct`, `UnorderedProduct`, `ProductKind`, `ProductShape`, `OrderedTail`, `PositionalTail`, `LabeledTail`, `OrderedShape`, `Presence`, `FieldInfo`, `ProductField`, `FixedFields`, `UnorderedTail`, `UnorderedShape`, `FieldLookup`, `UnorderedShapeView`, `Sum`, `Variant`, `Variants`, `VariantNames` | — |
| `universe.reflection.typing.context` | `reflection/typing/context.ph` | `TypingContext`, `Typing` | — |
| `universe.reflection.typing.evidence` | `reflection/typing/evidence.ph` | `RelationEvidence`, `RelationFailure`, `DynamicBoundary`, `ReflectionCapability` | — |
| `universe.reflection.typing.generic-signature` | `reflection/typing/generic-signature.ph` | `GenericSignature`, `GenericConstraint` | — |
| `universe.reflection.typing.kind` | `reflection/typing/kind.ph` | `KindDescriptor`, `AtomicKind`, `FunctionKind`, `Type` | — |
| `universe.reflection.typing` | `reflection/typing/package.ph` | — | — |
| `universe.reflection.typing.result` | `reflection/typing/result.ph` | `TypingResult`, `TypingKnown`, `TypingUnknown`, `TypingInvalid`, `TypingUnavailable`, `TypingCancelled`, `TypingBudgetExceeded`, `TypingInternalFailure`, `TypeRelationResult`, `RelationSatisfied`, `RelationRejected`, `RelationDynamicBoundary`, `RelationBlocked`, `RelationCancelled`, `RelationBudgetExceeded`, `RelationInternalFailure`, `MemberLookupResult`, `MemberFound`, `MemberMissing`, `MemberDynamicBoundary`, `MemberBlocked`, `MemberCancelled`, `MemberBudgetExceeded`, `MemberInternalFailure` | — |
| `universe.reflection.typing.signature` | `reflection/typing/signature.ph` | `CallableSignature`, `CallableParameter`, `FieldSignature` | — |
| `universe.reflection.typing.type-descriptor` | `reflection/typing/type-descriptor.ph` | `TypeDescriptor`, `AppliedType`, `UnionType`, `TupleType`, `RecordType`, `CallableType`, `TypeLambda`, `SpecialType`, `SelfType` | — |
| `universe.reflection.typing.type-parameter` | `reflection/typing/type-parameter.ph` | `TypeParameter` | — |
| `universe.reflection.typing.type-use` | `reflection/typing/type-use.ph` | `TypeUse` | — |
| `universe.reflection.uri` | `reflection/uri.ph` | `Uri` | — |
| `universe.regex` | `regex/package.ph` | — | — |
| `universe.scalar.bool` | `scalar/bool.ph` | `Bool`, `True`, `False` | — |
| `universe.scalar.char` | `scalar/char.ph` | `Char` | — |
| `universe.scalar.float` | `scalar/float.ph` | `Float` | — |
| `universe.scalar.int` | `scalar/int.ph` | `Int` | — |
| `universe.scalar.nil` | `scalar/nil.ph` | `Nil` | — |
| `universe.scalar.number` | `scalar/number.ph` | `Number` | — |
| `universe.scalar` | `scalar/package.ph` | — | — |
| `universe.scalar.string` | `scalar/string.ph` | `String`, `StringChars`, `StringCharIterator`, `StringByteSequence`, `StringByteIterator`, `StringCodePointSequence`, `StringCodePointIterator` | — |
| `universe.scalar.string_index` | `scalar/string_index.ph` | `StringIndex` | — |
| `universe.scalar.symbol` | `scalar/symbol.ph` | `Symbol` | — |
| `universe.scalar.unit` | `scalar/unit.ph` | `Unit` | — |
| `universe.testing` | `testing/package.ph` | — | — |
| `universe.text.builder` | `text/builder.ph` | `StringBuilder` | — |
| `universe.text.cursor` | `text/cursor.ph` | `TextCursor` | — |
| `universe.text.errors` | `text/errors.ph` | `Utf8Error`, `IntParseError`, `FloatParseError`, `BoolParseError` | — |
| `universe.text.line_map` | `text/line_map.ph` | `SourceLocation`, `LineMap` | — |
| `universe.text.lines` | `text/lines.ph` | — | — |
| `universe.text` | `text/package.ph` | — | — |
| `universe.text.patterns` | `text/patterns.ph` | `TextPattern` | — |
| `universe.text.split` | `text/split.ph` | — | — |
| `universe.text.substring` | `text/substring.ph` | `Substring`, `SubstringChars`, `SubstringCharIterator`, `SubstringBytes`, `SubstringByteIterator`, `Split`, `SplitIterator`, `SplitWhitespace`, `SplitWhitespaceIterator`, `Lines`, `LinesIterator` | — |
| `universe.text.traits` | `text/traits.ph` | `Text` | — |
| `universe.text.writer` | `text/writer.ph` | `TextWriter` | — |
| `universe.time.calendar.date` | `time/calendar/date.ph` | `DateOverflow`, `Date` | — |
| `universe.time.calendar.datetime` | `time/calendar/datetime.ph` | `DateTime`, `LocalTimeDisambiguation`, `ZonedDateTime`, `LocalTimeResolution` | — |
| `universe.time.calendar.month` | `time/calendar/month.ph` | `Gregorian`, `Month` | — |
| `universe.time.calendar` | `time/calendar/package.ph` | — | — |
| `universe.time.calendar.partial` | `time/calendar/partial.ph` | `YearMonth`, `MonthDay` | — |
| `universe.time.calendar.period` | `time/calendar/period.ph` | `Period` | — |
| `universe.time.calendar.time` | `time/calendar/time.ph` | `Time` | — |
| `universe.time.calendar.weekday` | `time/calendar/weekday.ph` | `Weekday` | — |
| `universe.time.clock` | `time/clock.ph` | `Instant`, `Clock`, `SystemClock` | — |
| `universe.time.duration` | `time/duration.ph` | `Duration`, `Measurement` | — |
| `universe.time.offset` | `time/offset.ph` | `UtcOffset` | — |
| `universe.time` | `time/package.ph` | — | `clock` |
| `universe.time.resolution` | `time/resolution.ph` | — | — |
| `universe.time.testing.clocks` | `time/testing/clocks.ph` | `ManualClock`, `ManualWallClock` | — |
| `universe.time.testing` | `time/testing/package.ph` | — | — |
| `universe.time.timestamp` | `time/timestamp.ph` | `WallClock`, `SystemWallClock`, `Timestamp` | `wallClock` |
| `universe.time.timezone` | `time/timezone.ph` | `TimeZone` | — |
| `universe.time.zone_transition` | `time/zone_transition.ph` | `ZoneTransition` | — |
| `universe.time.zoned` | `time/zoned.ph` | — | — |

## Authored interface and import statements

This table records top-level `import`, `from`, `export`, and `expose` statements from the current shadow files, with whitespace normalized and import paths left as authored. The resolved import graph is in `dependency-graph.md`.

| Module | Direct source statements |
|---|---|
| `universe.callable` | `expose .function`; `expose .closure`; `expose .method`; `expose .family` |
| `universe.collections.array` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable` |
| `universe.collections.bag` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable`; `export Bag, BagIterator` |
| `universe.collections.bijection` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable`; `export Bijection, BijectionIterator` |
| `universe.collections.buffer` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable`; `export Buffer` |
| `universe.collections.bytes` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable` |
| `universe.collections.indexed_dict` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable` |
| `universe.collections.interval` | `export Bound, Interval` |
| `universe.collections.iterator_laws` | `from universe.text.builder import StringBuilder`; `export Iterator, Iterable, MappedIterator, FilterIterator, FilterMapIterator, FlatMapIterator, FlattenIterator, ChainIterator, SkipIterator, SkipWhileIterator, TakeIterator, TakeWhileIterator, MapWhileIterator, InspectIterator, StrictZipIterator, IndexedIterator` |
| `universe.collections.list` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable`; `from universe.text.builder import StringBuilder` |
| `universe.collections.map` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable` |
| `universe.collections.mutable_bag` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable` |
| `universe.collections.ordered_set` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable` |
| `universe.collections.pack` | `from universe.collections.tuple import CompleteTypePack`; `export PositionalPackView`; `export LabeledPackView`; `export PositionalTypePackView`; `export LabeledTypePackView`; `export Pack, PositionalPack, LabeledPack, CompletePack` |
| `universe.collections` | `expose .collection`; `expose .iterator_laws`; `expose .list`; `expose .array`; `expose .vector`; `expose .slice`; `expose .buffer`; `expose .map`; `expose .set`; `expose .mutable_bag`; `expose .indexed_dict`; `expose .ordered_set`; `expose .bag`; `expose .bijection`; `expose .tuple`; `expose .pack`; `expose .record`; `expose .range`; `expose .bytes`; `expose .interval`; `expose .range_map` |
| `universe.collections.range` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable, TakeIterator` |
| `universe.collections.range_map` | `from universe.collections.interval import Bound, Interval`; `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable`; `export RangeMap, RangeMapIterator` |
| `universe.collections.record` | `export FieldKey`; `export RecordFields` |
| `universe.collections.set` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable` |
| `universe.collections.slice` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable`; `export Slice` |
| `universe.collections.tuple` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable`; `export CompleteTypePack` |
| `universe.collections.vector` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable` |
| `universe.concurrency.fiber` | `export Fiber, Future, FutureSubscription, CompletionSource, Tracer, OffBehavior, Backoff` |
| `universe.concurrency` | `expose .fiber` |
| `universe.errors.argument` | `export ArgumentError, KeyError, IndexError, SliceError, DuplicateKeyError, BijectionConflictError, InvalidIntervalError, RangeOverlapError` |
| `universe.errors` | `expose .error`; `expose .argument`; `expose .iteration`; `expose .indexing`; `expose .contracts`; `expose .unsupported`; `expose .unimplemented`; `from .argument import ArgumentError, KeyError, IndexError, SliceError, DuplicateKeyError, BijectionConflictError, InvalidIntervalError, RangeOverlapError`; `export ArgumentError, KeyError, IndexError, SliceError, DuplicateKeyError, BijectionConflictError, InvalidIntervalError, RangeOverlapError` |
| `universe.errors.unsupported` | `export unsupported` |
| `universe.functional.algebra` | `from .chain import Chain, NonEmpty`; `from universe.numbers.capabilities import Addable, Multipliable, Zero, One`; `export Semigroup, Monoid, Additive, Multiplicative, AllTrue, AnyTrue, First, Last, Dual, Endo` |
| `universe.functional.arrow` | `from .either import Either`; `export Arrow, ArrowContext, Profunctor` |
| `universe.functional.chain` | `from universe.collections.iterator_laws import Iterator, Iterable`; `export Chain, ChainIterator, NonEmpty, Partition` |
| `universe.functional.effects` | `from .higher_kinded import Functor, Applicative, Monad`; `from .chain import Chain`; `from universe.collections.iterator_laws import Iterable`; `export Effects` |
| `universe.functional.either` | `export Either` |
| `universe.functional.fold` | `from .chain import Chain`; `from universe.collections.iterator_laws import Iterable`; `from universe.numbers.capabilities import Addable, Zero`; `export Fold, FoldFactory` |
| `universe.functional.folds` | `from .algebra import Semigroup, Monoid`; `from .chain import Chain, NonEmpty`; `from .validation import Validation`; `from universe.collections.iterator_laws import Iterable`; `export Folds` |
| `universe.functional.higher_kinded` | `from .chain import Chain, NonEmpty`; `export Functor, Applicative, Monad, Identity, IdentityContext, OptionContext, ChainContext, NonEmptyContext` |
| `universe.functional.journal` | `from .chain import Chain`; `from .algebra import Semigroup, Monoid`; `export Journal` |
| `universe.functional.lens` | `export Lens` |
| `universe.functional.optics` | `from .lens import Lens`; `from universe.reflection.structural import Product`; `export Iso, Optional, Prism` |
| `universe.functional` | `expose .state`; `expose .reader`; `expose .validation`; `expose .lens`; `expose .chain`; `expose .algebra`; `expose .writer`; `expose .journal`; `expose .state_result`; `expose .reader_result`; `expose .higher_kinded`; `expose .effects`; `expose .folds`; `expose .fold`; `expose .optics`; `expose .either`; `expose .arrow`; `from .state import State, StateStep`; `from .reader import Reader`; `from .validation import Validation`; `from .lens import Lens`; `from .chain import Chain, ChainIterator, NonEmpty, Partition`; `from .algebra import Semigroup, Monoid, Additive, Multiplicative, AllTrue, AnyTrue, First, Last, Dual, Endo`; `from .writer import Writer`; `from .journal import Journal`; `from .state_result import StateResult`; `from .reader_result import ReaderResult`; `from .higher_kinded import Functor, Applicative, Monad, Identity, IdentityContext, OptionContext, ChainContext, NonEmptyContext`; `from .effects import Effects`; `from .folds import Folds`; `from .fold import Fold, FoldFactory`; `from .optics import Iso, Optional, Prism`; `from .either import Either`; `from .arrow import Arrow, ArrowContext, Profunctor`; `export State, StateStep`; `export Reader`; `export Validation`; `export Lens`; `export Chain, ChainIterator, NonEmpty, Partition`; `export Semigroup, Monoid, Additive, Multiplicative, AllTrue, AnyTrue, First, Last, Dual, Endo`; `export Writer`; `export Journal`; `export StateResult`; `export ReaderResult`; `export Functor, Applicative, Monad, Identity, IdentityContext, OptionContext, ChainContext, NonEmptyContext`; `export Effects`; `export Folds`; `export Fold, FoldFactory`; `export Iso, Optional, Prism`; `export Either, Arrow, ArrowContext, Profunctor` |
| `universe.functional.reader` | `export Reader` |
| `universe.functional.reader_result` | `from .reader import Reader`; `export ReaderResult` |
| `universe.functional.state` | `from .lens import Lens`; `export State, StateStep` |
| `universe.functional.state_result` | `from .state import State, StateStep`; `from .lens import Lens`; `export StateResult` |
| `universe.functional.validation` | `from .algebra import Semigroup`; `export Validation` |
| `universe.functional.writer` | `from .algebra import Semigroup, Monoid`; `export Writer` |
| `universe.json` | `export parse`; `export stringify` |
| `universe.math.complex.kernel` | `from ...numbers.complex import Complex`; `from ...numbers.float_complex import FloatComplex as H`; `from ..float_kernel import FloatMathKernel as F`; `export ComplexMathKernel` |
| `universe.math.complex` | `from .kernel import ComplexMathKernel as K`; `export sqrt, exp, log, log1p, sin, cos, tan, asin, acos, atan, sinh, cosh, tanh, asinh, acosh, atanh, powInt, pow` |
| `universe.math.float_kernel` | `export FloatMathKernel, FloatAngleHelpers` |
| `universe.math.integer` | `export IntegerMath` |
| `universe.math.numerical_errors` | `export NumericalRangeError, NumericalEvaluationError` |
| `universe.math` | `expose .complex`; `from .float_kernel import FloatMathKernel`; `from .float_kernel import FloatAngleHelpers`; `from .numerical_errors import NumericalRangeError, NumericalEvaluationError`; `from .integer import IntegerMath`; `from .reduction import FloatReduction`; `export ( pi, tau, e, ln2, ln10, sqrt2, sqrt, cbrt, exp, exp2, expm1, ln, ln1p, log, log2, log10, sin, cos, tan, asin, acos, atan, atan2, degrees, radians, sinh, cosh, tanh, asinh, acosh, atanh, hypot, gcd, lcm, isqrt, sum, mean, meanOption, logSumExp, logAddExp, log1mexp, factorial, choose, fallingFactorial, risingFactorial, NumericalRangeError, NumericalEvaluationError )` |
| `universe.math.reduction` | `from .float_kernel import FloatMathKernel`; `from .numerical_errors import NumericalRangeError`; `from universe.collections.iterator_laws import Iterable`; `export FloatReduction` |
| `universe.numbers.algebra` | `export I, J, K, Basis, i, j, k, RealEmbeddable, BasisRealizable` |
| `universe.numbers.capabilities` | `export Addable, Subtractable, Multipliable, Negatable, IntegerPower, Zero, One` |
| `universe.numbers.complex` | `from .capabilities import Addable, Subtractable, Multipliable, Negatable, IntegerPower, Zero, One`; `from .algebra import Basis, I, BasisRealizable, RealEmbeddable`; `from .rational import Rational`; `from .float_complex import FloatComplex as FC`; `from ..math.float_kernel import FloatMathKernel as F`; `export Complex, ComplexDivision` |
| `universe.numbers.float_complex` | `from ..math.float_kernel import FloatMathKernel as F`; `export FloatComplex` |
| `universe.numbers` | `from .capabilities import Addable, Subtractable, Multipliable, Negatable, IntegerPower, Zero, One`; `from .algebra import I, J, K, Basis, i, j, k, BasisRealizable, RealEmbeddable`; `from .complex import Complex, ComplexDivision`; `from .quaternion import Quaternion`; `from .rational import Rational`; `export Rational, Complex, ComplexDivision, Quaternion, I, J, K, Basis, i, j, k, Addable, Subtractable, Multipliable, Negatable, IntegerPower, Zero, One, BasisRealizable, RealEmbeddable` |
| `universe.numbers.quaternion` | `from .capabilities import Addable, Subtractable, Multipliable, Negatable, Zero, One`; `from .algebra import Basis, I, J, K, BasisRealizable, RealEmbeddable`; `from .complex import Complex`; `export Quaternion` |
| `universe.numbers.rational` | `from .capabilities import Addable, Subtractable, Multipliable, Negatable, IntegerPower, Zero, One`; `from ..math.integer import IntegerMath`; `export Rational` |
| `universe.object.ellipsis` | `export Ellipsis, ellipsis` |
| `universe.object.ordering` | `export Ordering` |
| `universe.object` | `expose .object`; `expose .behavior`; `expose .class`; `expose .metaclass`; `expose .ellipsis`; `expose .ordering` |
| `universe.option` | `export Option` |
| `universe` | `expose .object`; `expose .scalar`; `expose .errors`; `expose .result`; `expose .callable`; `expose .option`; `expose .functional`; `expose .time`; `expose .concurrency`; `expose .collections`; `expose .reflection`; `expose .io`; `expose .fs`; `expose .path`; `expose .text`; `expose .regex`; `expose .json`; `expose .math`; `expose .numbers`; `expose .random`; `expose .process`; `expose .net`; `expose .concurrent`; `expose .testing`; `import .object`; `import .scalar`; `from .scalar import StringIndex`; `import .errors`; `import .result`; `from .errors import ( ArgumentError, KeyError, IndexError, SliceError, DuplicateKeyError )`; `import .callable`; `import .option`; `import .functional`; `import .time`; `from .time import ( Duration, Timestamp )`; `import .concurrency`; `import .collections`; `import .reflection`; `import .io`; `import .fs`; `import .path`; `import .text`; `from .text import ( Substring, Text, TextPattern, Split, SplitWhitespace, Lines, TextWriter, StringBuilder, TextCursor, SourceLocation, LineMap, Utf8Error, IntParseError, FloatParseError, BoolParseError )`; `import .regex`; `import .json`; `import .math`; `import .numbers`; `import .random`; `import .process`; `import .net`; `import .concurrent`; `import .testing`; `export object`; `export scalar`; `export StringIndex`; `export Substring`; `export Text`; `export TextPattern`; `export Split`; `export SplitWhitespace`; `export Lines`; `export TextWriter`; `export StringBuilder`; `export TextCursor`; `export SourceLocation`; `export LineMap`; `export Utf8Error`; `export IntParseError`; `export FloatParseError`; `export BoolParseError`; `export errors`; `export result`; `export ( ArgumentError, KeyError, IndexError, SliceError, DuplicateKeyError )`; `export callable`; `export option`; `export functional`; `export ( Duration, Timestamp )`; `export time`; `export concurrency`; `export collections`; `export reflection`; `export io`; `export fs`; `export path`; `export text`; `export regex`; `export json`; `export math`; `export numbers`; `export random`; `export process`; `export net`; `export concurrent`; `export testing` |
| `universe.random.bernoulli` | `from .generator import Generator`; `from .distribution import Distribution`; `export Bernoulli` |
| `universe.random.distribution` | `from .generator import Generator`; `export Distribution` |
| `universe.random.generator` | `export Generator` |
| `universe.random` | `from .generator import Generator`; `from .xoshiro256pp import Xoshiro256PlusPlus`; `from .distribution import Distribution`; `from .uniform_int import UniformInt`; `from .bernoulli import Bernoulli`; `from .secure import SecureRandom, EntropyError`; `export Generator, Xoshiro256PlusPlus, Distribution, UniformInt, Bernoulli, SecureRandom, EntropyError` |
| `universe.random.secure` | `export SecureRandom, EntropyError` |
| `universe.random.uniform_int` | `from .generator import Generator`; `from .distribution import Distribution`; `export UniformInt` |
| `universe.random.xoshiro256pp` | `from .generator import Generator`; `export Xoshiro256PlusPlus` |
| `universe.reflection.message` | `export Message` |
| `universe.reflection` | `expose .module`; `expose .package_object`; `expose .project`; `expose .project_manifest`; `expose .package_info`; `expose .package_author`; `expose .package_requirement`; `expose .resolved_project_dependency`; `expose .module_dependency`; `expose .export_table`; `expose .export`; `expose .export_kind`; `expose .child_module_table`; `expose .module_identity`; `expose .package_identity`; `expose .project_identity`; `expose .uri`; `expose .selector`; `expose .message`; `expose .attribute`; `expose .implementation`; `expose .typing`; `expose .structural` |
| `universe.reflection.selector` | `export Selector`; `export SelectorPattern` |
| `universe.reflection.structural` | `from universe.collections.pack import PositionalPack, LabeledPack, CompletePack`; `from universe.collections.record import FieldKey`; `from universe.reflection.typing.kind import Type`; `export Product, OrderedProduct, UnorderedProduct, Sum`; `export ProductKind, ProductShape, OrderedTail, PositionalTail, LabeledTail, OrderedShape`; `export Presence, FieldInfo, ProductField, FixedFields, UnorderedTail, UnorderedShape, FieldLookup, UnorderedShapeView`; `export Variant, Variants, VariantNames` |
| `universe.reflection.typing.kind` | `export KindDescriptor, AtomicKind, FunctionKind, Type` |
| `universe.reflection.typing` | `expose .kind`; `expose .type_descriptor`; `expose .type_parameter`; `expose .generic_signature`; `expose .signature`; `expose .type_use`; `expose .result`; `expose .evidence`; `expose .context` |
| `universe.scalar.bool` | `from universe.text.errors import BoolParseError` |
| `universe.scalar.char` | `from universe.text.substring import Substring` |
| `universe.scalar.float` | `from .number import Number`; `from universe.text.errors import FloatParseError`; `export Float` |
| `universe.scalar.int` | `from .number import Number`; `from universe.text.errors import IntParseError`; `export Int` |
| `universe.scalar.number` | `export Number` |
| `universe.scalar` | `expose .number`; `expose .int`; `expose .float`; `expose .unit`; `expose .nil`; `expose .string`; `expose .string_index`; `expose .char`; `expose .bool`; `expose .symbol`; `from .string_index import StringIndex`; `from .number import Number`; `from .int import Int`; `from .float import Float`; `from .unit import Unit`; `export StringIndex`; `export Number`; `export Int`; `export Float`; `export Unit` |
| `universe.result` | `export Result` |
| `universe.scalar.string` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable`; `from universe.scalar.string_index import StringIndex`; `from universe.text.errors import Utf8Error`; `from universe.text.substring import ( Substring, Split, SplitWhitespace, Lines )`; `export ( StringChars, StringCharIterator, StringByteSequence, StringByteIterator, StringCodePointSequence, StringCodePointIterator )` |
| `universe.scalar.string_index` | `export StringIndex` |
| `universe.text.builder` | `from universe.text.writer import TextWriter`; `export StringBuilder` |
| `universe.text.cursor` | `from universe.scalar.string_index import StringIndex`; `from universe.text.substring import Substring`; `export TextCursor` |
| `universe.text.errors` | `export ( Utf8Error, IntParseError, FloatParseError, BoolParseError )` |
| `universe.text.line_map` | `from universe.scalar.string_index import StringIndex`; `from universe.text.substring import Substring`; `export ( SourceLocation, LineMap )` |
| `universe.text.lines` | `from .substring import ( Lines, LinesIterator )`; `export ( Lines, LinesIterator )` |
| `universe.text` | `expose .substring`; `expose .traits`; `expose .patterns`; `expose .split`; `expose .lines`; `expose .writer`; `expose .builder`; `expose .cursor`; `expose .line_map`; `expose .errors`; `from .substring import ( Substring, SubstringChars, SubstringCharIterator, SubstringBytes, SubstringByteIterator )`; `from .traits import Text`; `from .patterns import TextPattern`; `from .split import ( Split, SplitIterator, SplitWhitespace, SplitWhitespaceIterator )`; `from .lines import ( Lines, LinesIterator )`; `from .writer import TextWriter`; `from .builder import StringBuilder`; `from .cursor import TextCursor`; `from .line_map import ( SourceLocation, LineMap )`; `from .errors import ( Utf8Error, IntParseError, FloatParseError, BoolParseError )`; `export ( Substring, SubstringChars, SubstringCharIterator, SubstringBytes, SubstringByteIterator, Text, TextPattern, Split, SplitIterator, SplitWhitespace, SplitWhitespaceIterator, Lines, LinesIterator, TextWriter, StringBuilder, TextCursor, SourceLocation, LineMap, Utf8Error, IntParseError, FloatParseError, BoolParseError )` |
| `universe.text.patterns` | `from universe.text.substring import Substring`; `export TextPattern` |
| `universe.text.split` | `from .substring import ( Split, SplitIterator, SplitWhitespace, SplitWhitespaceIterator )`; `export ( Split, SplitIterator, SplitWhitespace, SplitWhitespaceIterator )` |
| `universe.text.substring` | `from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable`; `from universe.scalar.string_index import StringIndex`; `export ( Substring, SubstringChars, SubstringCharIterator, SubstringBytes, SubstringByteIterator, Split, SplitIterator, SplitWhitespace, SplitWhitespaceIterator, Lines, LinesIterator )` |
| `universe.text.traits` | `from universe.scalar.string import ( StringChars, StringByteSequence )`; `from universe.text.substring import ( Substring, SubstringChars, SubstringBytes )`; `export Text` |
| `universe.text.writer` | `export TextWriter` |
| `universe.time.calendar.date` | `from universe.time.calendar.month import Month, Gregorian`; `from universe.time.calendar.period import Period`; `from universe.time.calendar.weekday import Weekday`; `from universe.time.timezone import TimeZone`; `from universe.time.timestamp import Timestamp`; `export Date, DateOverflow` |
| `universe.time.calendar.datetime` | `from universe.time.calendar.date import Date, DateOverflow`; `from universe.time.calendar.period import Period`; `from universe.time.calendar.time import Time`; `from universe.time.calendar.month import Month`; `from universe.time.offset import UtcOffset`; `from universe.time.timezone import TimeZone`; `from universe.time.timestamp import Timestamp`; `from universe.time.duration import Duration`; `export DateTime, ZonedDateTime, LocalTimeDisambiguation, LocalTimeResolution` |
| `universe.time.calendar.month` | `export Month, Gregorian` |
| `universe.time.calendar` | `expose .month`; `expose .weekday`; `expose .period`; `expose .date`; `expose .time`; `expose .datetime`; `expose .partial`; `from universe.time.calendar.month import Month`; `from universe.time.calendar.weekday import Weekday`; `from universe.time.calendar.period import Period`; `from universe.time.calendar.date import ( Date, DateOverflow )`; `from universe.time.calendar.time import Time`; `from universe.time.calendar.datetime import DateTime`; `from universe.time.calendar.partial import ( YearMonth, MonthDay )`; `export ( Month, Weekday, Period, Date, DateOverflow, Time, DateTime, YearMonth, MonthDay )` |
| `universe.time.calendar.partial` | `from universe.time.calendar.date import Date`; `from universe.time.calendar.month import Month, Gregorian`; `export YearMonth, MonthDay` |
| `universe.time.calendar.period` | `export Period` |
| `universe.time.calendar.time` | `export Time` |
| `universe.time.calendar.weekday` | `export Weekday` |
| `universe.time.clock` | `from universe.time.duration import Duration, Measurement`; `export Clock, Instant, SystemClock` |
| `universe.time.duration` | `export Duration, Measurement` |
| `universe.time.offset` | `export UtcOffset` |
| `universe.time` | `expose .clock`; `expose .duration`; `expose .timestamp`; `expose .offset`; `expose .calendar`; `expose .timezone`; `expose .zone_transition`; `expose .zoned`; `expose .resolution`; `expose .testing`; `from universe.time.clock import ( Clock, Instant, SystemClock )`; `from universe.time.duration import ( Duration, Measurement )`; `from universe.time.timestamp import ( Timestamp, WallClock, SystemWallClock, wallClock )`; `from universe.time.offset import UtcOffset`; `from universe.time.timezone import TimeZone`; `from universe.time.zone_transition import ZoneTransition`; `from universe.time.zoned import ZonedDateTime`; `from universe.time.resolution import LocalTimeDisambiguation, LocalTimeResolution`; `from universe.time.calendar.month import Month`; `from universe.time.calendar.weekday import Weekday`; `from universe.time.calendar.period import Period`; `from universe.time.calendar.date import ( Date, DateOverflow )`; `from universe.time.calendar.time import Time`; `from universe.time.calendar.datetime import DateTime`; `from universe.time.calendar.partial import ( YearMonth, MonthDay )`; `export ( Clock, Instant, SystemClock, Duration, Measurement, Timestamp, WallClock, SystemWallClock, UtcOffset, TimeZone, ZoneTransition, ZonedDateTime, LocalTimeDisambiguation, LocalTimeResolution, Month, Weekday, Period, Date, DateOverflow, Time, DateTime, YearMonth, MonthDay, clock, wallClock )` |
| `universe.time.resolution` | `from universe.time.calendar.datetime import LocalTimeDisambiguation, LocalTimeResolution`; `export LocalTimeDisambiguation, LocalTimeResolution` |
| `universe.time.testing.clocks` | `from universe.time.clock import Clock, Instant`; `from universe.time.duration import Duration`; `from universe.time.timestamp import Timestamp`; `from universe.time.timestamp import WallClock`; `export ManualClock, ManualWallClock` |
| `universe.time.testing` | `expose .clocks` |
| `universe.time.timestamp` | `from universe.time.duration import Duration`; `export Timestamp, WallClock, SystemWallClock, wallClock` |
| `universe.time.timezone` | `from universe.time.offset import UtcOffset`; `from universe.time.duration import Duration`; `from universe.time.timestamp import Timestamp`; `from universe.time.zone_transition import ZoneTransition`; `export TimeZone` |
| `universe.time.zone_transition` | `from universe.time.timestamp import Timestamp`; `from universe.time.offset import UtcOffset`; `export ZoneTransition` |
| `universe.time.zoned` | `from universe.time.calendar.datetime import ZonedDateTime`; `export ZonedDateTime` |

## Scalar type source references

This lexical scan lists modules containing each token as a whole identifier in `.ph` files, including declaration owners and import/facade text. It is a source-dependent inventory, not a semantic reference index. Most uses remain unqualified prelude names; no shadow module imports `Int` or `Float` from the old number module. The only explicit `.number` imports added here are `Number` imports from `int.ph` and `float.ph`.

| Name | Modules containing the identifier |
|---|---|
| `Number` (12 modules) | `universe.collections.array`, `universe.collections.bytes`, `universe.collections.iterator_laws`, `universe.collections.list`, `universe.collections.range`, `universe.scalar`, `universe.scalar.float`, `universe.scalar.int`, `universe.scalar.number`, `universe.scalar.string`, `universe.scalar.string_index`, `universe.text.substring` |
| `Int` (72 modules) | `universe.callable.closure`, `universe.callable.family`, `universe.callable.function`, `universe.callable.method`, `universe.collections.array`, `universe.collections.bag`, `universe.collections.bijection`, `universe.collections.buffer`, `universe.collections.bytes`, `universe.collections.indexed_dict`, `universe.collections.interval`, `universe.collections.iterator_laws`, `universe.collections.list`, `universe.collections.map`, `universe.collections.mutable_bag`, `universe.collections.ordered_set`, `universe.collections.pack`, `universe.collections.range_map`, `universe.collections.record`, `universe.collections.set`, `universe.collections.slice`, `universe.collections.tuple`, `universe.collections.vector`, `universe.concurrency.fiber`, `universe.functional.chain`, `universe.functional.fold`, `universe.functional.journal`, `universe.math`, `universe.math.complex.kernel`, `universe.math.integer`, `universe.numbers.capabilities`, `universe.numbers.complex`, `universe.numbers.float_complex`, `universe.numbers.rational`, `universe.object.object`, `universe.option`, `universe.random.generator`, `universe.random.secure`, `universe.random.uniform_int`, `universe.random.xoshiro256pp`, `universe.reflection.child-module-table`, `universe.reflection.export-table`, `universe.reflection.selector`, `universe.scalar`, `universe.scalar.bool`, `universe.scalar.char`, `universe.scalar.float`, `universe.scalar.int`, `universe.scalar.number`, `universe.scalar.string`, `universe.scalar.string_index`, `universe.scalar.symbol`, `universe.text.builder`, `universe.text.cursor`, `universe.text.errors`, `universe.text.line_map`, `universe.text.patterns`, `universe.text.substring`, `universe.text.traits`, `universe.time.calendar.date`, `universe.time.calendar.datetime`, `universe.time.calendar.month`, `universe.time.calendar.partial`, `universe.time.calendar.period`, `universe.time.calendar.time`, `universe.time.calendar.weekday`, `universe.time.clock`, `universe.time.duration`, `universe.time.offset`, `universe.time.testing.clocks`, `universe.time.timestamp`, `universe.time.timezone` |
| `Float` (19 modules) | `universe.collections.indexed_dict`, `universe.collections.mutable_bag`, `universe.collections.ordered_set`, `universe.math`, `universe.math.complex`, `universe.math.complex.kernel`, `universe.math.float_kernel`, `universe.math.reduction`, `universe.numbers.capabilities`, `universe.numbers.complex`, `universe.numbers.float_complex`, `universe.numbers.rational`, `universe.random.bernoulli`, `universe.random.generator`, `universe.scalar`, `universe.scalar.float`, `universe.scalar.number`, `universe.time.duration`, `universe.time.timestamp` |
