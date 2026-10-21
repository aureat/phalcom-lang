//! Conformance-checked folds and fail-fast or accumulating traversal.
from .algebra import Semigroup, Monoid
from .chain import Chain, NonEmpty
from .validation import Validation
from universe.collections.iterator_laws import Iterable

class Folds {
  @class
  foldMap<I, M>(_ source: I, using f: (<I as Iterable>::Item) -> M) -> M
    where I |= Iterable, M |= Semigroup, M |= Monoid
  {
    let mut result = M.empty
    let iterator = source.iterator
    while let Option::Some(value) = iterator.next {
      result = result.combine(f.call(value))
    }
    result
  }

  @class
  combineAll<I, M>(_ source: I) -> M
    where I |= Iterable, <I as Iterable>::Item == M, M |= Semigroup, M |= Monoid
  { Folds.foldMap(source, using: |value| { value }) }

  /// Stops at the first Error; neither f nor next is called for later inputs.
  /// Empty input succeeds with an empty persistent sequence.
  @class
  traverseResult<I, T, E>(_ source: I, using f: (<I as Iterable>::Item) -> Result<T, E>) -> Result<Chain<T>, E>
    where I |= Iterable
  {
    let mut result: Chain<T> = Chain<T>::Empty
    let iterator = source.iterator
    while let Option::Some(value) = iterator.next {
      match f.call(value) {
        Result::Ok(item) => { result = result.prepend(item) }
        Result::Error(error) => { return Result<Chain<T>, E>::Error(error) }
      }
    }
    Result<Chain<T>, E>::Ok(result.reverse)
  }

  @class
  traverseOption<I, T>(_ source: I, using f: (<I as Iterable>::Item) -> Option<T>) -> Option<Chain<T>>
    where I |= Iterable
  {
    let mut result: Chain<T> = Chain<T>::Empty
    let iterator = source.iterator
    while let Option::Some(value) = iterator.next {
      match f.call(value) {
        Option::Some(item) => { result = result.prepend(item) }
        Option::None => { return Option<Chain<T>>::None }
      }
    }
    Option<Chain<T>>::Some(result.reverse)
  }

  /// Visits every input and accumulates all errors in order. No Semigroup
  /// constraint is needed on E; a NonEmpty<E> preserves the failure invariant.
  @class
  validateAll<I, T, E>(_ source: I, using f: (<I as Iterable>::Item) -> Validation<T, E>) -> Validation<Chain<T>, NonEmpty<E>>
    where I |= Iterable
  {
    let mut values: Chain<T> = Chain<T>::Empty
    let mut errors: Chain<E> = Chain<E>::Empty
    let iterator = source.iterator
    while let Option::Some(value) = iterator.next {
      match f.call(value) {
        Validation::Valid(item) => { values = values.prepend(item) }
        Validation::Invalid(error) => { errors = errors.prepend(error) }
      }
    }
    match NonEmpty<E>.fromChain(errors.reverse) {
      Option::Some(nonempty) => Validation<Chain<T>, NonEmpty<E>>::Invalid(nonempty)
      Option::None => Validation<Chain<T>, NonEmpty<E>>::Valid(values.reverse)
    }
  }
}

export Folds
