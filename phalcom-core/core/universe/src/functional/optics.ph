//! Reversible conversions and optional focus for closed sums.
from .lens import Lens
from universe.reflection.structural import Product

/// A reversible typed conversion. Round-trip laws are callback obligations.
data Iso<S, T> private(_ forward: (S) -> T, _ backward: (T) -> S)
impl<S, T> Iso<S, T> {
  @class new(forward: (S) -> T, backward: (T) -> S) -> Iso<S, T> { Iso<S, T>(forward, backward) }
  convert(_ source: S) -> T { self.forward.call(source) }
  restore(_ target: T) -> S { self.backward.call(target) }
  inverse -> Iso<T, S> { Iso<T, S>.new(forward: self.backward, backward: self.forward) }
  compose<U>(_ other: Iso<T, U>) -> Iso<S, U> {
    Iso<S, U>.new(forward: |source| { other.convert(self.convert(source)) }, backward: |target| { self.restore(other.restore(target)) })
  }
  modify(_ source: S, using f: (T) -> T) -> S { self.restore(f.call(self.convert(source))) }
  asLens -> Lens<S, T> {
    Lens<S, T>.new(get: self.forward, set: |_, value| { self.restore(value) })
  }
}

impl<S, T> Iso<S, T>
  where S |= Product, T |= Product, <S as Product>::Shape == <T as Product>::Shape
{
  /// Requires canonical reflected product-shape equivalence at the call site.
  /// This checks shapes, not callback round-trip laws or field-value equality.
  @class sameShape(forward: (S) -> T, backward: (T) -> S) -> Iso<S, T> {
    Iso<S, T>.new(forward: forward, backward: backward)
  }
}

/// A focus that can be absent. Replacement happens only for a present focus.
data Optional<S, T> private(_ read: (S) -> Option<T>, _ replace: (S, T) -> S)
impl<S, T> Optional<S, T> {
  @class new(get: (S) -> Option<T>, set: (S, T) -> S) -> Optional<S, T> { Optional<S, T>(get, set) }
  get(_ source: S) -> Option<T> { self.read.call(source) }
  set(_ source: S, to value: T) -> S {
    match self.get(source) {
      Option::Some(_) => self.replace.call(source, value)
      Option::None => source
    }
  }
  modify(_ source: S, using f: (T) -> T) -> S {
    match self.get(source) {
      Option::Some(value) => self.replace.call(source, f.call(value))
      Option::None => source
    }
  }
  compose<U>(_ other: Optional<T, U>) -> Optional<S, U> {
    Optional<S, U>.new(
      get: |source| { self.get(source).flatMap(|part| { other.get(part) }) },
      set: |source, value| { self.modify(source, using: |part| { other.set(part, to: value) }) }
    )
  }
}

/// Selects a case of a closed sum and constructs that case from its payload.
/// preview(review(t)) == Some(t) is an obligation of the callbacks.
data Prism<S, T> private(_ select: (S) -> Option<T>, _ inject: (T) -> S)
impl<S, T> Prism<S, T> {
  @class new(preview: (S) -> Option<T>, review: (T) -> S) -> Prism<S, T> { Prism<S, T>(preview, review) }
  preview(_ source: S) -> Option<T> { self.select.call(source) }
  review(_ value: T) -> S { self.inject.call(value) }
  modify(_ source: S, using f: (T) -> T) -> S {
    match self.preview(source) {
      Option::Some(value) => self.review(f.call(value))
      Option::None => source
    }
  }
  compose<U>(_ other: Prism<T, U>) -> Prism<S, U> {
    Prism<S, U>.new(
      preview: |source| { self.preview(source).flatMap(|part| { other.preview(part) }) },
      review: |value| { self.review(other.review(value)) }
    )
  }
  asOptional -> Optional<S, T> {
    Optional<S, T>.new(get: self.select, set: |_, value| { self.review(value) })
  }
}

export Iso, Optional, Prism
