//! Explicit effect dictionaries indexed by a unary type-level callable domain.
from .chain import Chain, NonEmpty

/// F is a constructor domain, not a runtime value or a proper-type parameter.
/// The dictionary's class-side operation preserves F while changing payloads.
trait Functor<F: Type -> Type> {
  @class map<A, B>(_ value: F<A>, using f: (A) -> B) -> F<B>
}

trait Applicative<F: Type -> Type> where Self |= Functor<F> {
  @class pure<A>(_ value: A) -> F<A>
  @class map2<A, B, C>(_ left: F<A>, _ right: F<B>, using f: (A, B) -> C) -> F<C>
}

trait Monad<F: Type -> Type> where Self |= Functor<F>, Self |= Applicative<F> {
  @class flatMap<A, B>(_ value: F<A>, using f: (A) -> F<B>) -> F<B>
}

/// An effect with exactly one value and no additional context.
data Identity<T>(_ value: T)
impl<T> Identity<T> {
  map<U>(_ f: (T) -> U) -> Identity<U> { Identity<U>(f.call(self.value)) }
  flatMap<U>(_ f: (T) -> Identity<U>) -> Identity<U> { f.call(self.value) }
}

class IdentityContext {}
impl Functor<Identity> for IdentityContext {
  @class map<A, B>(_ value: Identity<A>, using f: (A) -> B) -> Identity<B> { value.map(f) }
}
impl Applicative<Identity> for IdentityContext {
  @class pure<A>(_ value: A) -> Identity<A> { Identity<A>(value) }
  @class map2<A, B, C>(_ left: Identity<A>, _ right: Identity<B>, using f: (A, B) -> C) -> Identity<C> {
    Identity<C>(f.call(left.value, right.value))
  }
}
impl Monad<Identity> for IdentityContext {
  @class flatMap<A, B>(_ value: Identity<A>, using f: (A) -> Identity<B>) -> Identity<B> { value.flatMap(f) }
}

class OptionContext {}
impl Functor<Option> for OptionContext {
  @class map<A, B>(_ value: Option<A>, using f: (A) -> B) -> Option<B> { value.map(f) }
}
impl Applicative<Option> for OptionContext {
  @class pure<A>(_ value: A) -> Option<A> { Option<A>::Some(value) }
  @class map2<A, B, C>(_ left: Option<A>, _ right: Option<B>, using f: (A, B) -> C) -> Option<C> {
    left.zipWith(right, combine: f)
  }
}
impl Monad<Option> for OptionContext {
  @class flatMap<A, B>(_ value: Option<A>, using f: (A) -> Option<B>) -> Option<B> { value.flatMap(f) }
}

/// Cartesian-product applicative and order-preserving sequence monad.
class ChainContext {}
impl Functor<Chain> for ChainContext {
  @class map<A, B>(_ value: Chain<A>, using f: (A) -> B) -> Chain<B> { value.map(f) }
}
impl Applicative<Chain> for ChainContext {
  @class
  pure<A>(_ value: A) -> Chain<A> {
    let empty: Chain<A> = Chain<A>::Empty
    empty.prepend(value)
  }
  @class map2<A, B, C>(_ left: Chain<A>, _ right: Chain<B>, using f: (A, B) -> C) -> Chain<C> {
    left.flatMap(|a| { right.map(|b| { f.call(a, b) }) })
  }
}
impl Monad<Chain> for ChainContext {
  @class flatMap<A, B>(_ value: Chain<A>, using f: (A) -> Chain<B>) -> Chain<B> { value.flatMap(f) }
}

class NonEmptyContext {}
impl Functor<NonEmpty> for NonEmptyContext {
  @class map<A, B>(_ value: NonEmpty<A>, using f: (A) -> B) -> NonEmpty<B> { value.map(f) }
}
impl Applicative<NonEmpty> for NonEmptyContext {
  @class pure<A>(_ value: A) -> NonEmpty<A> { NonEmpty<A>.one(value) }
  @class map2<A, B, C>(_ left: NonEmpty<A>, _ right: NonEmpty<B>, using f: (A, B) -> C) -> NonEmpty<C> {
    left.flatMap(|a| { right.map(|b| { f.call(a, b) }) })
  }
}
impl Monad<NonEmpty> for NonEmptyContext {
  @class flatMap<A, B>(_ value: NonEmpty<A>, using f: (A) -> NonEmpty<B>) -> NonEmpty<B> { value.flatMap(f) }
}

export Functor, Applicative, Monad, Identity, IdentityContext, OptionContext, ChainContext, NonEmptyContext
