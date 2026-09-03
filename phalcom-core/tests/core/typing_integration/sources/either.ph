enum Either<L, R> {

    @variant
    Left(_ value: L)

    @variant
    Right(_ value: R)

    isLeft -> Bool {
        match self {
            Left(_) => true
            Right(_) => false
        }
    }

    isRight -> Bool {
        match self {
            Left(_) => false
            Right(_) => true
        }
    }

    fold<T>(
        left: (L) -> T,
        right: (R) -> T
    ) -> T {
        match self {
            Left(value) => left.call(value)
            Right(value) => right.call(value)
        }
    }

    map<R2>(_ f: (R) -> R2) -> Either<L, R2> {
        match self {
            Left(value) => Either::Left(value)
            Right(value) => Either::Right(f.call(value))
        }
    }

    mapLeft<L2>(_ f: (L) -> L2) -> Either<L2, R> {
        match self {
            Left(value) => Either::Left(f.call(value))
            Right(value) => Either::Right(value)
        }
    }

    bimap<L2, R2>(
        left: (L) -> L2,
        right: (R) -> R2
    ) -> Either<L2, R2> {
        match self {
            Left(value) => Either::Left(left.call(value))
            Right(value) => Either::Right(right.call(value))
        }
    }

    flatMap<R2>(_ f: (R) -> Either<L, R2>) -> Either<L, R2> {
        match self {
            Left(value) => Either::Left(value)
            Right(value) => f.call(value)
        }
    }

    orElse<L2>(_ other: Either<L2, R>) -> Either<L2, R> {
        match self {
            Left(_) => other
            Right(value) => Either::Right(value)
        }
    }

    recover(_ f: (L) -> R) -> R {
        match self {
            Left(value) => f.call(value)
            Right(value) => value
        }
    }

    getOrElse(_ fallback: R) -> R {
        match self {
            Left(_) => fallback
            Right(value) => value
        }
    }

    swap -> Either<R, L> {
        match self {
            Left(value) => Either::Right(value)
            Right(value) => Either::Left(value)
        }
    }

    zip<R2>(_ other: Either<L, R2>) -> Either<L, (R, R2)> {
        match self {
            Left(value) => Either::Left(value)
            Right(value) => match other {
                Left(otherValue) => Either::Left(otherValue)
                Right(otherValue) => Either::Right((value, otherValue))
            }
        }
    }
}
