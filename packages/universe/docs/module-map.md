# Initial module map

This is an inventory of the copied legacy source layout, not the target reorganized layout. Module paths are derived from `src/` paths (`package.ph` is the containing package module). The scanner includes every unindented top-level declaration line matching zero or more `intrinsic`, `native`, `primitive`, or `type` prefixes followed by one of `data`, `enum`, `class`, `trait`, or `type`, then an identifier. It therefore includes intrinsic aliases/data facades such as `Unit`, `RecordFields`, `Record`, `EmptyRecord`, `TypePack`, `Tuple`, and structural declarations. It separately includes top-level `let`, `let mut`, `const`, `fn`, and `def` names. It does not parse Telos syntax, enumerate nested declarations/variants/members, infer generated declarations, or prove semantic identity. Source statements remain authoritative; no compiler validation was performed.

Recognized top-level type-like declarations: **364**. Recognized top-level value declarations: **77**.

| Baseline module | Source file | Top-level type-like declarations | Top-level values |
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
| `universe.errors.result` | `errors/result.ph` | `Result` | — |
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
| `universe.option.option` | `option/option.ph` | `Option` | — |
| `universe.option` | `option/package.ph` | — | — |
| `universe.option.unit` | `option/unit.ph` | `Unit` | — |
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
| `universe.scalar.nil` | `scalar/nil.ph` | `Nil` | — |
| `universe.scalar.number` | `scalar/number.ph` | `Number`, `Int`, `Float` | — |
| `universe.scalar` | `scalar/package.ph` | — | — |
| `universe.scalar.string` | `scalar/string.ph` | `String`, `StringChars`, `StringCharIterator`, `StringByteSequence`, `StringByteIterator`, `StringCodePointSequence`, `StringCodePointIterator` | — |
| `universe.scalar.string_index` | `scalar/string_index.ph` | `StringIndex` | — |
| `universe.scalar.symbol` | `scalar/symbol.ph` | `Symbol` | — |
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

| Module | Direct source statements |
|---|---|
| Module | Direct source statements |
|---|---|
| Module | Direct source statements |
|---|---|
| Module | Direct source statements |
|---|---|
| Module | Direct source statements |
|---|---|
| `universe.callable` | expose: `.function`, `.closure`, `.method`, `.family` |
| `universe.collections.array` | imports: `universe.collections.iterator_laws` |
| `universe.collections.bag` | export: `Bag, BagIterator`; imports: `universe.collections.iterator_laws` |
| `universe.collections.bijection` | export: `Bijection, BijectionIterator`; imports: `universe.collections.iterator_laws` |
| `universe.collections.buffer` | export: `Buffer`; imports: `universe.collections.iterator_laws` |
| `universe.collections.bytes` | imports: `universe.collections.iterator_laws` |
| `universe.collections.indexed_dict` | imports: `universe.collections.iterator_laws` |
| `universe.collections.interval` | export: `Bound, Interval` |
| `universe.collections.iterator_laws` | export: `Iterator, Iterable, MappedIterator, FilterIterator, FilterMapIterator, FlatMapIterator, FlattenIterator, ChainIterator, SkipIterator, SkipWhileIterator, TakeIterator, TakeWhileIterator, MapWhileIterator, InspectIterator, StrictZipIterator, IndexedIterator`; imports: `universe.text.builder` |
| `universe.collections.list` | imports: `universe.collections.iterator_laws`, `universe.text.builder` |
| `universe.collections.map` | imports: `universe.collections.iterator_laws` |
| `universe.collections.mutable_bag` | imports: `universe.collections.iterator_laws` |
| `universe.collections.ordered_set` | imports: `universe.collections.iterator_laws` |
| `universe.collections.pack` | export: `PositionalPackView`, `LabeledPackView`, `PositionalTypePackView`, `LabeledTypePackView`, `Pack, PositionalPack, LabeledPack, CompletePack`; imports: `universe.collections.tuple` |
| `universe.collections` | expose: `.collection`, `.iterator_laws`, `.list`, `.array`, `.vector`, `.slice`, `.buffer`, `.map`, `.set`, `.mutable_bag`, `.indexed_dict`, `.ordered_set`, `.bag`, `.bijection`, `.tuple`, `.pack`, `.record`, `.range`, `.bytes`, `.interval`, `.range_map` |
| `universe.collections.range` | imports: `universe.collections.iterator_laws` |
| `universe.collections.range_map` | export: `RangeMap, RangeMapIterator`; imports: `universe.collections.interval`, `universe.collections.iterator_laws` |
| `universe.collections.record` | export: `FieldKey`, `RecordFields` |
| `universe.collections.set` | imports: `universe.collections.iterator_laws` |
| `universe.collections.slice` | export: `Slice`; imports: `universe.collections.iterator_laws` |
| `universe.collections.tuple` | export: `CompleteTypePack`; imports: `universe.collections.iterator_laws` |
| `universe.collections.vector` | imports: `universe.collections.iterator_laws` |
| `universe.concurrency.fiber` | export: `Fiber, Future, FutureSubscription, CompletionSource, Tracer, OffBehavior, Backoff` |
| `universe.concurrency` | expose: `.fiber` |
| `universe.errors.argument` | export: `ArgumentError, KeyError, IndexError, SliceError, DuplicateKeyError, BijectionConflictError, InvalidIntervalError, RangeOverlapError` |
| `universe.errors` | expose: `.error`, `.result`, `.argument`, `.iteration`, `.indexing`, `.contracts`, `.unsupported`, `.unimplemented`; export: `ArgumentError, KeyError, IndexError, SliceError, DuplicateKeyError, BijectionConflictError, InvalidIntervalError, RangeOverlapError`; imports: `universe.argument` |
| `universe.errors.unsupported` | export: `unsupported` |
| `universe.functional.algebra` | export: `Semigroup, Monoid, Additive, Multiplicative, AllTrue, AnyTrue, First, Last, Dual, Endo`; imports: `universe.chain`, `universe.numbers.capabilities` |
| `universe.functional.arrow` | export: `Arrow, ArrowContext, Profunctor`; imports: `universe.either` |
| `universe.functional.chain` | export: `Chain, ChainIterator, NonEmpty, Partition`; imports: `universe.collections.iterator_laws` |
| `universe.functional.effects` | export: `Effects`; imports: `universe.chain`, `universe.collections.iterator_laws`, `universe.higher_kinded` |
| `universe.functional.either` | export: `Either` |
| `universe.functional.fold` | export: `Fold, FoldFactory`; imports: `universe.chain`, `universe.collections.iterator_laws`, `universe.numbers.capabilities` |
| `universe.functional.folds` | export: `Folds`; imports: `universe.algebra`, `universe.chain`, `universe.collections.iterator_laws`, `universe.validation` |
| `universe.functional.higher_kinded` | export: `Functor, Applicative, Monad, Identity, IdentityContext, OptionContext, ChainContext, NonEmptyContext`; imports: `universe.chain` |
| `universe.functional.journal` | export: `Journal`; imports: `universe.algebra`, `universe.chain` |
| `universe.functional.lens` | export: `Lens` |
| `universe.functional.optics` | export: `Iso, Optional, Prism`; imports: `universe.lens`, `universe.reflection.structural` |
| `universe.functional` | expose: `.state`, `.reader`, `.validation`, `.lens`, `.chain`, `.algebra`, `.writer`, `.journal`, `.state_result`, `.reader_result`, `.higher_kinded`, `.effects`, `.folds`, `.fold`, `.optics`, `.either`, `.arrow`; export: `State, StateStep`, `Reader`, `Validation`, `Lens`, `Chain, ChainIterator, NonEmpty, Partition`, `Semigroup, Monoid, Additive, Multiplicative, AllTrue, AnyTrue, First, Last, Dual, Endo`, `Writer`, `Journal`, `StateResult`, `ReaderResult`, `Functor, Applicative, Monad, Identity, IdentityContext, OptionContext, ChainContext, NonEmptyContext`, `Effects`, `Folds`, `Fold, FoldFactory`, `Iso, Optional, Prism`, `Either, Arrow, ArrowContext, Profunctor`; imports: `universe.algebra`, `universe.arrow`, `universe.chain`, `universe.effects`, `universe.either`, `universe.fold`, `universe.folds`, `universe.higher_kinded`, `universe.journal`, `universe.lens`, `universe.optics`, `universe.reader`, `universe.reader_result`, `universe.state`, `universe.state_result`, `universe.validation`, `universe.writer` |
| `universe.functional.reader` | export: `Reader` |
| `universe.functional.reader_result` | export: `ReaderResult`; imports: `universe.reader` |
| `universe.functional.state` | export: `State, StateStep`; imports: `universe.lens` |
| `universe.functional.state_result` | export: `StateResult`; imports: `universe.lens`, `universe.state` |
| `universe.functional.validation` | export: `Validation`; imports: `universe.algebra` |
| `universe.functional.writer` | export: `Writer`; imports: `universe.algebra` |
| `universe.json` | export: `parse`, `stringify` |
| `universe.math.complex.kernel` | export: `ComplexMathKernel`; imports: `universe.float_kernel`, `universe.numbers.complex`, `universe.numbers.float_complex` |
| `universe.math.complex` | export: `sqrt, exp, log, log1p, sin, cos, tan, asin, acos, atan, sinh, cosh, tanh, asinh, acosh, atanh, powInt, pow`; imports: `universe.kernel` |
| `universe.math.float_kernel` | export: `FloatMathKernel, FloatAngleHelpers` |
| `universe.math.integer` | export: `IntegerMath` |
| `universe.math.numerical_errors` | export: `NumericalRangeError, NumericalEvaluationError` |
| `universe.math` | expose: `.complex`; export: `(`; imports: `universe.float_kernel`, `universe.integer`, `universe.numerical_errors`, `universe.reduction` |
| `universe.math.reduction` | export: `FloatReduction`; imports: `universe.collections.iterator_laws`, `universe.float_kernel`, `universe.numerical_errors` |
| `universe.numbers.algebra` | export: `I, J, K, Basis, i, j, k, RealEmbeddable, BasisRealizable` |
| `universe.numbers.capabilities` | export: `Addable, Subtractable, Multipliable, Negatable, IntegerPower, Zero, One` |
| `universe.numbers.complex` | export: `Complex, ComplexDivision`; imports: `universe.algebra`, `universe.capabilities`, `universe.float_complex`, `universe.math.float_kernel`, `universe.rational` |
| `universe.numbers.float_complex` | export: `FloatComplex`; imports: `universe.math.float_kernel` |
| `universe.numbers` | export: `Rational, Complex, ComplexDivision, Quaternion, I, J, K, Basis, i, j, k, Addable, Subtractable, Multipliable, Negatable, IntegerPower, Zero, One, BasisRealizable, RealEmbeddable`; imports: `universe.algebra`, `universe.capabilities`, `universe.complex`, `universe.quaternion`, `universe.rational` |
| `universe.numbers.quaternion` | export: `Quaternion`; imports: `universe.algebra`, `universe.capabilities`, `universe.complex` |
| `universe.numbers.rational` | export: `Rational`; imports: `universe.capabilities`, `universe.math.integer` |
| `universe.object.ellipsis` | export: `Ellipsis, ellipsis` |
| `universe.object.ordering` | export: `Ordering` |
| `universe.object` | expose: `.object`, `.behavior`, `.class`, `.metaclass`, `.ellipsis`, `.ordering` |
| `universe.option` | expose: `.option`, `.unit` |
| `universe` | expose: `.object`, `.scalar`, `.errors`, `.callable`, `.option`, `.functional`, `.time`, `.concurrency`, `.collections`, `.reflection`, `.io`, `.fs`, `.path`, `.text`, `.regex`, `.json`, `.math`, `.numbers`, `.random`, `.process`, `.net`, `.concurrent`, `.testing`; export: `object`, `scalar`, `StringIndex`, `Substring`, `Text`, `TextPattern`, `Split`, `SplitWhitespace`, `Lines`, `TextWriter`, `StringBuilder`, `TextCursor`, `SourceLocation`, `LineMap`, `Utf8Error`, `IntParseError`, `FloatParseError`, `BoolParseError`, `errors`, `(`, `callable`, `option`, `functional`, `(`, `time`, `concurrency`, `collections`, `reflection`, `io`, `fs`, `path`, `text`, `regex`, `json`, `math`, `numbers`, `random`, `process`, `net`, `concurrent`, `testing`; imports: `universe.callable`, `universe.collections`, `universe.concurrency`, `universe.concurrent`, `universe.errors`, `universe.fs`, `universe.functional`, `universe.io`, `universe.json`, `universe.math`, `universe.net`, `universe.numbers`, `universe.object`, `universe.option`, `universe.path`, `universe.process`, `universe.random`, `universe.reflection`, `universe.regex`, `universe.scalar`, `universe.testing`, `universe.text`, `universe.time` |
| `universe.random.bernoulli` | export: `Bernoulli`; imports: `universe.distribution`, `universe.generator` |
| `universe.random.distribution` | export: `Distribution`; imports: `universe.generator` |
| `universe.random.generator` | export: `Generator` |
| `universe.random` | export: `Generator, Xoshiro256PlusPlus, Distribution, UniformInt, Bernoulli, SecureRandom, EntropyError`; imports: `universe.bernoulli`, `universe.distribution`, `universe.generator`, `universe.secure`, `universe.uniform_int`, `universe.xoshiro256pp` |
| `universe.random.secure` | export: `SecureRandom, EntropyError` |
| `universe.random.uniform_int` | export: `UniformInt`; imports: `universe.distribution`, `universe.generator` |
| `universe.random.xoshiro256pp` | export: `Xoshiro256PlusPlus`; imports: `universe.generator` |
| `universe.reflection.message` | export: `Message` |
| `universe.reflection` | expose: `.module`, `.package_object`, `.project`, `.project_manifest`, `.package_info`, `.package_author`, `.package_requirement`, `.resolved_project_dependency`, `.module_dependency`, `.export_table`, `.export`, `.export_kind`, `.child_module_table`, `.module_identity`, `.package_identity`, `.project_identity`, `.uri`, `.selector`, `.message`, `.attribute`, `.implementation`, `.typing`, `.structural` |
| `universe.reflection.selector` | export: `Selector`, `SelectorPattern` |
| `universe.reflection.structural` | export: `Product, OrderedProduct, UnorderedProduct, Sum`, `ProductKind, ProductShape, OrderedTail, PositionalTail, LabeledTail, OrderedShape`, `Presence, FieldInfo, ProductField, FixedFields, UnorderedTail, UnorderedShape, FieldLookup, UnorderedShapeView`, `Variant, Variants, VariantNames`; imports: `universe.collections.pack`, `universe.collections.record`, `universe.reflection.typing.kind` |
| `universe.reflection.typing.kind` | export: `KindDescriptor, AtomicKind, FunctionKind, Type` |
| `universe.reflection.typing` | expose: `.kind`, `.type_descriptor`, `.type_parameter`, `.generic_signature`, `.signature`, `.type_use`, `.result`, `.evidence`, `.context` |
| `universe.scalar.bool` | imports: `universe.text.errors` |
| `universe.scalar.char` | imports: `universe.text.substring` |
| `universe.scalar.number` | imports: `universe.text.errors` |
| `universe.scalar` | expose: `.number`, `.nil`, `.string`, `.string_index`, `.char`, `.bool`, `.symbol`; export: `StringIndex`; imports: `universe.string_index` |
| `universe.scalar.string` | export: `(`; imports: `universe.collections.iterator_laws`, `universe.scalar.string_index`, `universe.text.errors`, `universe.text.substring` |
| `universe.scalar.string_index` | export: `StringIndex` |
| `universe.text.builder` | export: `StringBuilder`; imports: `universe.text.writer` |
| `universe.text.cursor` | export: `TextCursor`; imports: `universe.scalar.string_index`, `universe.text.substring` |
| `universe.text.errors` | export: `(` |
| `universe.text.line_map` | export: `(`; imports: `universe.scalar.string_index`, `universe.text.substring` |
| `universe.text.lines` | export: `(`; imports: `universe.substring` |
| `universe.text` | expose: `.substring`, `.traits`, `.patterns`, `.split`, `.lines`, `.writer`, `.builder`, `.cursor`, `.line_map`, `.errors`; export: `(`; imports: `universe.builder`, `universe.cursor`, `universe.errors`, `universe.line_map`, `universe.lines`, `universe.patterns`, `universe.split`, `universe.substring`, `universe.traits`, `universe.writer` |
| `universe.text.patterns` | export: `TextPattern`; imports: `universe.text.substring` |
| `universe.text.split` | export: `(`; imports: `universe.substring` |
| `universe.text.substring` | export: `(`; imports: `universe.collections.iterator_laws`, `universe.scalar.string_index` |
| `universe.text.traits` | export: `Text`; imports: `universe.scalar.string`, `universe.text.substring` |
| `universe.text.writer` | export: `TextWriter` |
| `universe.time.calendar.date` | export: `Date, DateOverflow`; imports: `universe.time.calendar.month`, `universe.time.calendar.period`, `universe.time.calendar.weekday`, `universe.time.timestamp`, `universe.time.timezone` |
| `universe.time.calendar.datetime` | export: `DateTime, ZonedDateTime, LocalTimeDisambiguation, LocalTimeResolution`; imports: `universe.time.calendar.date`, `universe.time.calendar.month`, `universe.time.calendar.period`, `universe.time.calendar.time`, `universe.time.duration`, `universe.time.offset`, `universe.time.timestamp`, `universe.time.timezone` |
| `universe.time.calendar.month` | export: `Month, Gregorian` |
| `universe.time.calendar` | expose: `.month`, `.weekday`, `.period`, `.date`, `.time`, `.datetime`, `.partial`; export: `(`; imports: `universe.time.calendar.date`, `universe.time.calendar.datetime`, `universe.time.calendar.month`, `universe.time.calendar.partial`, `universe.time.calendar.period`, `universe.time.calendar.time`, `universe.time.calendar.weekday` |
| `universe.time.calendar.partial` | export: `YearMonth, MonthDay`; imports: `universe.time.calendar.date`, `universe.time.calendar.month` |
| `universe.time.calendar.period` | export: `Period` |
| `universe.time.calendar.time` | export: `Time` |
| `universe.time.calendar.weekday` | export: `Weekday` |
| `universe.time.clock` | export: `Clock, Instant, SystemClock`; imports: `universe.time.duration` |
| `universe.time.duration` | export: `Duration, Measurement` |
| `universe.time.offset` | export: `UtcOffset` |
| `universe.time` | expose: `.clock`, `.duration`, `.timestamp`, `.offset`, `.calendar`, `.timezone`, `.zone_transition`, `.zoned`, `.resolution`, `.testing`; export: `(`; imports: `universe.time.calendar.date`, `universe.time.calendar.datetime`, `universe.time.calendar.month`, `universe.time.calendar.partial`, `universe.time.calendar.period`, `universe.time.calendar.time`, `universe.time.calendar.weekday`, `universe.time.clock`, `universe.time.duration`, `universe.time.offset`, `universe.time.resolution`, `universe.time.timestamp`, `universe.time.timezone`, `universe.time.zone_transition`, `universe.time.zoned` |
| `universe.time.resolution` | export: `LocalTimeDisambiguation, LocalTimeResolution`; imports: `universe.time.calendar.datetime` |
| `universe.time.testing.clocks` | export: `ManualClock, ManualWallClock`; imports: `universe.time.clock`, `universe.time.duration`, `universe.time.timestamp` |
| `universe.time.testing` | expose: `.clocks` |
| `universe.time.timestamp` | export: `Timestamp, WallClock, SystemWallClock, wallClock`; imports: `universe.time.duration` |
| `universe.time.timezone` | export: `TimeZone`; imports: `universe.time.duration`, `universe.time.offset`, `universe.time.timestamp`, `universe.time.zone_transition` |
| `universe.time.zone_transition` | export: `ZoneTransition`; imports: `universe.time.offset`, `universe.time.timestamp` |
| `universe.time.zoned` | export: `ZonedDateTime`; imports: `universe.time.calendar.datetime` |
