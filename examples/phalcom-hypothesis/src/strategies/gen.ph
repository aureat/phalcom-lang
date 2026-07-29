// Public strategy-construction façade.

import Strategy from "strategies/strategy"
import combinators from "strategies/combinators"
import primitives from "strategies/primitives"
import collections from "strategies/collections"
import composite from "strategies/composite"

class Gen {
  @class
  int -> Strategy<Int> => primitives._IntStrategy.unbounded()

  @class
  int(min: Int, max: Int) -> Strategy<Int> {
    return primitives._IntStrategy.bounded(min: min, max: max)
  }

  @class
  bool -> Strategy<Bool> => primitives._BoolStrategy.new()

  @class
  float -> Strategy<Float> {
    return primitives._FloatStrategy.new(
      min: -1000000.0,
      max: 1000000.0
    )
  }

  @class
  float(min: Float, max: Float) -> Strategy<Float> {
    return primitives._FloatStrategy.new(min: min, max: max)
  }

  @class
  bytes -> Strategy<Bytes> {
    return primitives._BytesStrategy.new(minSize: 0, maxSize: 64)
  }

  @class
  bytes(minSize: Int, maxSize: Int) -> Strategy<Bytes> {
    return primitives._BytesStrategy.new(
      minSize: minSize,
      maxSize: maxSize
    )
  }

  @class
  text -> Strategy<String> {
    return primitives._TextStrategy.new(
      alphabet: primitives._IntStrategy.bounded(min: 32, max: 126),
      minSize: 0,
      maxSize: 64
    )
  }

  @class
  text(
    alphabet: Strategy<Int>,
    minSize: Int,
    maxSize: Int
  ) -> Strategy<String> {
    return primitives._TextStrategy.new(
      alphabet: alphabet,
      minSize: minSize,
      maxSize: maxSize
    )
  }

  @class
  just<T>(value: T) -> Strategy<T> {
    return combinators._JustStrategy.new(value: value)
  }

  @class
  sampledFrom<T>(values: List<T>) -> Strategy<T> {
    return primitives._SampledFromStrategy.new(values: values)
  }

  @class
  oneOf<T>(*strategies: Strategy<T>) -> Strategy<T> {
    return combinators._OneOfStrategy.new(strategies: strategies)
  }

  @class
  option<T>(value: Strategy<T>) -> Strategy<Option<T>> {
    return collections._OptionStrategy.new(value: value)
  }

  @class
  result<T, E>(
    ok: Strategy<T>,
    error: Strategy<E>
  ) -> Strategy<Result<T, E>> {
    return collections._ResultStrategy.new(ok: ok, error: error)
  }

  @class
  list<T>(of: Strategy<T>) -> Strategy<List<T>> {
    return collections._ListStrategy.new(
      elements: of,
      minSize: 0,
      maxSize: 20
    )
  }

  @class
  list<T>(
    of: Strategy<T>,
    minSize: Int,
    maxSize: Int
  ) -> Strategy<List<T>> {
    return collections._ListStrategy.new(
      elements: of,
      minSize: minSize,
      maxSize: maxSize
    )
  }

  @class
  set<T>(of: Strategy<T>) -> Strategy<Set<T>> {
    return collections._SetStrategy.new(
      elements: of,
      minSize: 0,
      maxSize: 20
    )
  }

  @class
  set<T>(
    of: Strategy<T>,
    minSize: Int,
    maxSize: Int
  ) -> Strategy<Set<T>> {
    return collections._SetStrategy.new(
      elements: of,
      minSize: minSize,
      maxSize: maxSize
    )
  }

  @class
  map<K, V>(
    keys: Strategy<K>,
    values: Strategy<V>
  ) -> Strategy<Map<K, V>> {
    return collections._MapStrategy.new(
      keys: keys,
      values: values,
      minSize: 0,
      maxSize: 20
    )
  }

  @class
  map<K, V>(
    keys: Strategy<K>,
    values: Strategy<V>,
    minSize: Int,
    maxSize: Int
  ) -> Strategy<Map<K, V>> {
    return collections._MapStrategy.new(
      keys: keys,
      values: values,
      minSize: minSize,
      maxSize: maxSize
    )
  }

  @class
  tuple(*strategies: Strategy<Any>) -> Strategy<Tuple> {
    return collections._TupleStrategy.new(elements: strategies)
  }

  @class
  build<T>(builder: [composite.Draw] -> T) -> Strategy<T> {
    return composite._BuildStrategy.new(builder: builder)
  }

  @class
  deferred<T>(factory: [] -> Strategy<T>) -> Strategy<T> {
    return composite._DeferredStrategy.new(factory: factory)
  }

  @class
  recursive<T>(
    base: Strategy<T>,
    extend: [Strategy<T>] -> Strategy<T>
  ) -> Strategy<T> {
    return composite._RecursiveStrategy.new(base: base, extend: extend)
  }
}
