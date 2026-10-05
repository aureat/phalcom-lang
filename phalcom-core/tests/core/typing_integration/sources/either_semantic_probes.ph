class EitherGenericProbe {
    @class
    preserve<L, R>(_ value: Either<L, R>) -> Either<L, R> {
        value
    }

    @class
    lift<L, A, B>(
        _ value: Either<L, A>,
        _ transform: (A) -> B
    ) -> Either<L, B> {
        value.map(transform)
    }

    @class
    flatten<L, R>(
        _ value: Either<L, Either<L, R>>
    ) -> Either<L, R> {
        value.flatMap(|inner| {
            inner
        })
    }

    @class
    merge<T>(_ value: Either<T, T>) -> T {
        value.fold(
            left: |v| { v },
            right: |v| { v }
        )
    }
}

class EitherInferenceProbe {
    @class
    contextualLeft() {
        let mut contextualLeft: Either<String, Int> = Either::Left("failure")
    }

    @class
    contextualRight() {
        let mut contextualRight: Either<String, Int> = Either::Right(42)
    }

    @class
    canonicalPaths() {
        let mut fromLeft: Either<String, Int> = Either::Left("left")
        let mut fromRight: Either<String, Int> = Either::Right(42)
    }

    @class
    mapInference() {
        let mut source: Either<String, Int> = Either::Right(42)
        let mut mapped = source.map(|value| {
            value > 0
        })
    }

    @class
    mapLeftInference() {
        let mut source: Either<String, Int> = Either::Right(42)
        let mut mappedLeft = source.mapLeft(|value| {
            value == "failure"
        })
    }

    @class
    bimapInference() {
        let mut source: Either<String, Int> = Either::Right(42)
        let mut bimapped = source.bimap(
            left: |value| { value == "failure" },
            right: |value| { value > 0 }
        )
    }

    @class
    swapInference() {
        let mut source: Either<String, Int> = Either::Right(42)
        let mut swapped = source.swap
    }

    @class
    orElseInference() {
        let mut source: Either<String, Int> = Either::Left("failure")
        let mut replacement: Either<Bool, Int> = Either::Right(100)
        let mut recovered = source.orElse(replacement)
    }

    @class
    chainedInference() {
        let mut initial: Either<String, Int> = Either::Right(41)
        let mut mapped = initial.map(|value| { value == 41 })
        let mut leftMapped = mapped.mapLeft(|value| { 100 })
        let mut swapped = leftMapped.swap
    }
}

class EitherHigherOrderProbe {
    @class
    inferAcrossArguments() {
        let mut source: Either<String, Int> = Either::Right(41)
        let mut lifted = EitherGenericProbe.lift(
            source,
            |value| { value == 41 }
        )
    }

    @class
    freshCalls() {
        let mut firstInput: Either<String, Int> = Either::Right(41)
        let mut first = EitherGenericProbe.lift(
            firstInput,
            |value| { value == 41 }
        )

        let mut secondInput: Either<Int, Bool> = Either::Right(true)
        let mut second = EitherGenericProbe.lift(
            secondInput,
            |value| { "second" }
        )
    }
}

class EitherNestedProbe {
    @class
    flattenNested() {
        let mut inner: Either<String, Int> = Either::Right(73)
        let mut outer: Either<String, Either<String, Int>> = Either::Right(inner)
        let mut flattened = EitherGenericProbe.flatten(outer)
    }

    @class
    repeatedVariable() {
        let mut left: Either<Int, Int> = Either::Left(10)
        let mut right: Either<Int, Int> = Either::Right(20)
        let mut leftValue = EitherGenericProbe.merge(left)
        let mut rightValue = EitherGenericProbe.merge(right)
    }
}
