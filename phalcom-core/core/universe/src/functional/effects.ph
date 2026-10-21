//! Generic algorithms over an explicitly selected effect family and dictionary.
from .higher_kinded import Functor, Applicative, Monad
from .chain import Chain
from universe.collections.iterator_laws import Iterable

class Effects {
  @class
  map<F: Type -> Type, D, A, B>(_ value: F<A>, using f: (A) -> B) -> F<B>
    where D |= Functor<F>
  { D.map(value, using: f) }

  @class
  lift2<F: Type -> Type, D, A, B, C>(_ left: F<A>, _ right: F<B>, using f: (A, B) -> C) -> F<C>
    where D |= Functor<F>, D |= Applicative<F>
  { D.map2(left, right, using: f) }

  @class
  apply<F: Type -> Type, D, A, B>(_ function: F<(A) -> B>, _ argument: F<A>) -> F<B>
    where D |= Functor<F>, D |= Applicative<F>
  { D.map2(function, argument, using: |f, value| { f.call(value) }) }

  @class
  flatten<F: Type -> Type, D, A>(_ nested: F<F<A>>) -> F<A>
    where D |= Functor<F>, D |= Applicative<F>, D |= Monad<F>
  { D.flatMap(nested, using: |inner| { inner }) }

  /// Takes a finite snapshot of the input spine, then builds a left-to-right
  /// monadic computation. f is invoked through flatMap, so a failed Option
  /// suppresses later callbacks. Deferred effect dictionaries may defer f.
  /// Source iteration completes eagerly; it is not an infinite-stream API.
  @class
  traverse<F: Type -> Type, D, I, A, B>(_ source: I, using f: (A) -> F<B>) -> F<Chain<B>>
    where D |= Functor<F>, D |= Applicative<F>, D |= Monad<F>,
          I |= Iterable, <I as Iterable>::Item == A
  {
    let snapshot = Chain<A>.fromIterable(source)
    let empty: Chain<B> = Chain<B>::Empty
    let mut accumulated = D.pure(empty)
    let mut remaining: Chain<A> = snapshot
    while not remaining.isEmpty {
      match remaining {
        Chain::Link(value, rest) => {
          // Capture this immutable value binding, never the mutable cursor.
          accumulated = D.flatMap(accumulated, using: |prefix| {
            D.map(f.call(value), using: |item| { prefix.prepend(item) })
          })
          remaining = rest
        }
        _ => {}
      }
    }
    D.map(accumulated, using: |reversed| { reversed.reverse })
  }

  @class
  sequence<F: Type -> Type, D, I, A>(_ source: I) -> F<Chain<A>>
    where D |= Functor<F>, D |= Applicative<F>, D |= Monad<F>,
          I |= Iterable, <I as Iterable>::Item == F<A>
  {
    let snapshot = Chain<F<A>>.fromIterable(source)
    let empty: Chain<A> = Chain<A>::Empty
    let mut accumulated = D.pure(empty)
    let mut remaining: Chain<F<A>> = snapshot
    while not remaining.isEmpty {
      match remaining {
        Chain::Link(effect, rest) => {
          accumulated = D.flatMap(accumulated, using: |prefix| {
            D.map(effect, using: |item| { prefix.prepend(item) })
          })
          remaining = rest
        }
        _ => {}
      }
    }
    D.map(accumulated, using: |reversed| { reversed.reverse })
  }
}

export Effects
