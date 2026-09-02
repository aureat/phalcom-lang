enum Either<L, R> {
    @variant
    Left(_ value: L)

    @variant
    Right(_ value: R)

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

    flatMap<R2>(_ f: (R) -> Either<L, R2>) -> Either<L, R2> {
        match self {
            Left(value) => Either::Left(value)
            Right(value) => f.call(value)
        }
    }
}

class Box<T> {}

class Functor<F: Type -> Type> {
    map<A, B>(
        _ value: F<A>,
        _ f: (A) -> B
    ) -> F<B> {
        throw Error.new("Functor.map is a contract stub")
    }
}

class Applicative<F: Type -> Type> is Functor<F> {
    pure<A>(_ value: A) -> F<A> {
        throw Error.new("Applicative.pure is a contract stub")
    }

    map2<A, B, C>(
        _ left: F<A>,
        _ right: F<B>,
        _ f: (A, B) -> C
    ) -> F<C> {
        throw Error.new("Applicative.map2 is a contract stub")
    }
}

class Monad<F: Type -> Type> is Applicative<F> {
    flatMap<A, B>(
        _ value: F<A>,
        _ f: (A) -> F<B>
    ) -> F<B> {
        throw Error.new("Monad.flatMap is a contract stub")
    }
}

class BoxMonad is Monad<Box> {}

// Pure inheritance probe: deliberately no overrides. Calls through this class
// must resolve to Functor / Applicative / Monad and specialize their F.
class ContractEitherMonad<E> is Monad<<X> =>> Either<E, X>> {}
class StringContractEitherMonad is ContractEitherMonad<String> {}

// Executable specialization: concrete operations are supplied for VM tests.
class EitherMonad<E> is Monad<<X> =>> Either<E, X>> {
    map<A, B>(
        _ value: Either<E, A>,
        _ f: (A) -> B
    ) -> Either<E, B> {
        value.map(f)
    }

    pure<A>(_ value: A) -> Either<E, A> {
        Either::Right(value)
    }

    map2<A, B, C>(
        _ left: Either<E, A>,
        _ right: Either<E, B>,
        _ f: (A, B) -> C
    ) -> Either<E, C> {
        left.flatMap(|leftValue| {
            right.map(|rightValue| {
                f.call(leftValue, rightValue)
            })
        })
    }

    flatMap<A, B>(
        _ value: Either<E, A>,
        _ f: (A) -> Either<E, B>
    ) -> Either<E, B> {
        value.flatMap(f)
    }
}

class StringEitherMonad is EitherMonad<String> {}

class MonadAlgorithms {
    @class
    bind<F: Type -> Type, A, B>(
        _ monad: Monad<F>,
        _ value: F<A>,
        _ next: (A) -> F<B>
    ) -> F<B> {
        monad.flatMap(value, next)
    }

    @class
    sequenceSeed<F: Type -> Type, A>(
        _ monad: Monad<F>,
        _ values: List<F<A>>,
        _ initial: F<List<A>>
    ) -> F<List<A>> {
        initial
    }

    @class
    constructorIdentity<F: Type -> Type, A>(
        _ value: F<A>
    ) -> F<A> {
        value
    }

    @class
    kleisli<F: Type -> Type, A, B, C>(
        _ monad: Monad<F>,
        _ first: (A) -> F<B>,
        _ second: (B) -> F<C>
    ) -> (A) -> F<C> {
        |value| {
            monad.flatMap(first.call(value), second)
        }
    }

    @class
    traverse<F: Type -> Type, A, B>(
        _ monad: Monad<F>,
        _ values: List<A>,
        _ transform: (A) -> F<B>
    ) -> F<List<B>> {
        let empty: List<B> = []
        let state = monad.pure(empty)
        let index = 0
        while (index < values.size) {
            let value = values[index]
            state = monad.flatMap(state, |items| {
                monad.map(transform.call(value), |mapped| {
                    items.append(mapped)
                    items
                })
            })
            index = index + 1
        }
        state
    }
}
