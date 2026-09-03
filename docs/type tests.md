```ph
class Functor<F: Type -> Type> {
    map<A, B>(
	    _ value: F<A>,
	    _ f: (A) -> B
		) -> F<B> {}
}

class Applicative<F: Type -> Type> is Functor<F> {
    pure<A>(_ value: A) -> F<A> {
        ...
    }

    map2<A, B, C>(
        _ left: F<A>,
        _ right: F<B>,
        _ f: (A, B) -> C
    ) -> F<C> {}
}

class Monad<F<_>> : Applicative<F> {
    flatMap<A, B>(
        _ value: F<A>,
        _ f: (A) -> F<B>
    ) -> F<B> {}
}

class OptionMonad : Monad<Option> {
    ...
}

class ListMonad : Monad<List> {
    ...
}

class EitherMonad<E>
    : Monad<[X] =>> Either<E, X>>
{
    ...
}

class Algorithms {
    @class
    useMonad<F<_], A, B>(
        _ monad: Monad<F>,
        _ value: F<A>,
        _ f: (A) -> F<B>
    ) -> F<B> {
        monad.flatMap(value, f)
    }
}

let result = eitherMonad.map(
    Either::Right(42),
    |value| { value.toString() }
)
```
