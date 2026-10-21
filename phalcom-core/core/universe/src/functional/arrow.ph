//! Typed pipelines over products and choices, without erased intermediate values.
from .either import Either

/// A binary constructor family: contravariant input, covariant output mapping.
/// The declaration describes operations; it does not infer declaration variance
/// or prove parametricity or purity of supplied callbacks.
trait Profunctor<P: Type -> Type -> Type> {
  @class dimap<A, B, C, D>(_ value: P<A, B>, before: (C) -> A, after: (B) -> D) -> P<C, D>
}

data Arrow<A, B> private(_ transform: (A) -> B)
impl<A, B> Arrow<A, B> {
  type Input = A
  type Output = B
  @class new(_ transform: (A) -> B) -> Arrow<A, B> { Arrow<A, B>(transform) }
  @class identity() -> Arrow<A, B> where A == B { Arrow<A, A>.new(|value| { value }) }
  call(_ value: A) -> B { self.transform.call(value) }

  then<C>(_ next: Arrow<B, C>) -> Arrow<A, C> {
    Arrow<A, C>.new(|value| { next.call(self.call(value)) })
  }
  compose<C>(_ previous: Arrow<C, A>) -> Arrow<C, B> { previous.then(self) }
  dimap<C, D>(before: (C) -> A, after: (B) -> D) -> Arrow<C, D> {
    Arrow<C, D>.new(|value| { after.call(self.call(before.call(value))) })
  }
  /// Transforms the first component and preserves the second component type.
  first<C>() -> Arrow<Tuple<***(A, C)>, Tuple<***(B, C)>> {
    Arrow<Tuple<***(A, C)>, Tuple<***(B, C)>>.new(|pair| { (self.call(pair[0]), pair[1]) })
  }
  second<C>() -> Arrow<Tuple<***(C, A)>, Tuple<***(C, B)>> {
    Arrow<Tuple<***(C, A)>, Tuple<***(C, B)>>.new(|pair| { (pair[0], self.call(pair[1])) })
  }
  /// Runs independent component pipelines left to right on a typed product.
  split<C, D>(_ other: Arrow<C, D>) -> Arrow<Tuple<***(A, C)>, Tuple<***(B, D)>> {
    Arrow<Tuple<***(A, C)>, Tuple<***(B, D)>>.new(|pair| {
      let left = self.call(pair[0])
      let right = other.call(pair[1])
      (left, right)
    })
  }
  /// Passes the same input to two pipelines, executing them left to right.
  fanout<C>(_ other: Arrow<A, C>) -> Arrow<A, Tuple<***(B, C)>> {
    Arrow<A, Tuple<***(B, C)>>.new(|value| {
      let left = self.call(value)
      let right = other.call(value)
      (left, right)
    })
  }
  /// Runs exactly the pipeline selected by the sum variant, preserving the side.
  choose<C, D>(_ other: Arrow<C, D>) -> Arrow<Either<A, C>, Either<B, D>> {
    Arrow<Either<A, C>, Either<B, D>>.new(|choice| {
      match choice {
        Either::Left(value) => Either<B, D>::Left(self.call(value))
        Either::Right(value) => Either<B, D>::Right(other.call(value))
      }
    })
  }
}

class ArrowContext {}
impl Profunctor<Arrow> for ArrowContext {
  @class dimap<A, B, C, D>(_ value: Arrow<A, B>, before: (C) -> A, after: (B) -> D) -> Arrow<C, D> {
    value.dimap(before: before, after: after)
  }
}

export Arrow, ArrowContext, Profunctor
