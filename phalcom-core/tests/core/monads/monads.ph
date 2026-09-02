enum Either<L, R> {
    @variant
    Left(_ value: L)

    @variant
    Right(_ value: R)
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

class EitherMonad<E> is Monad<<X> =>> Either<E, X>> {}

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
}
