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

```ph
const stack = Stack.new()

stack.fold(initial: 0)
	using: |acc, value| acc + value
	
stack
	..push(10)
	..push(20)
	..push(30)
	

```

```ph
// ============================================================
//  Generic mutable Stack<T>
//
//  Representation:
//      - bottom is _items[0]
//      - top is _items[_items.size - 1]
//
//  Mutation:
//      push / pop / clear mutate self.
//
//  Value-producing operations:
//      peek / fold / + do not mutate self.
// ============================================================

class Stack<T> {
  const _items: List<T>

  @constructor
  new(_ items: List<T>) {
    _items = items
  }

  // ----------------------------------------------------------
  // Construction
  // ----------------------------------------------------------

  @class
  empty -> Stack<T> {
    Stack<T>.new(items: List<T>.new())
  }

  // Defensive construction: do not retain the caller's mutable List.
  @class
  from(_ items: List<T>) -> Stack<T> {
    const copy: List<T> = List<T>.new()

    items.each |item: T| {
      copy.append(item)
    }

    Stack<T>.new(items: copy)
  }

  // ----------------------------------------------------------
  // Queries
  // ----------------------------------------------------------

  size -> Int {
    _items.size
  }

  isEmpty -> Bool {
    _items.isEmpty
  }

  isNotEmpty -> Bool {
    not _items.isEmpty
  }

  // Non-mutating partial access.
  peek -> Option<T> {
    if (_items.isEmpty) {
      return None
    }

    Some.new(_items.at(_items.size - 1))
  }

  // ----------------------------------------------------------
  // Mutation
  // ----------------------------------------------------------

  push(_ value: T) -> Stack<T> {
    _items.append(value)
    self
  }

  // Assumes the kernel List removal operation is `removeAt`.
  // If List exposes `pop` instead, this becomes even simpler.
  pop -> Option<T> {
    if (_items.isEmpty) {
      return None
    }

    const index: Int = _items.size - 1
    const value: T = _items.at(index)

    _items.removeAt(index)

    Some.new(value)
  }

  clear -> Stack<T> {
    while (not _items.isEmpty) {
      _items.removeAt(_items.size - 1)
    }

    self
  }

  // ----------------------------------------------------------
  // Higher-order operations
  // ----------------------------------------------------------

  each(_ f: (T) -> Dynamic) -> Stack<T> {
    _items.each(f)
    self
  }

  fold<A>(
    initial seed: A,
    using f: (A, T) -> A
  ) -> A {
    _items.fold(initial: seed, using: f)
  }

  // ----------------------------------------------------------
  // Conversion
  // ----------------------------------------------------------

  // Never expose _items itself. That would allow mutation of the
  // stack without going through Stack's API.
  toList -> List<T> {
    const copy: List<T> = List<T>.new()

    _items.each |item: T| {
      copy.append(item)
    }

    copy
  }

  // ----------------------------------------------------------
  // Operators
  // ----------------------------------------------------------

  // s + t:
  //
  //     s = [1, 2]       top = 2
  //     t = [3, 4]       top = 4
  //
  // result = [1, 2, 3, 4]
  // resulting top = 4
  //
  // Neither operand is mutated.
  +(_ other: Stack<T>) -> Stack<T> {
    const out: List<T> = List<T>.new()

    _items.each |item: T| {
      out.append(item)
    }

    other.each |item: T| {
      out.append(item)
    }

    Stack<T>.new(items: out)
  }

  ==(_ other: Stack<T>) -> Bool {
    if (self.size != other.size) {
      return false
    }

    const rhs: List<T> = other.toList
    let i: Int = 0

    while (i < self.size) {
      if (_items.at(i) != rhs.at(i)) {
        return false
      }

      i = i + 1
    }

    true
  }

  // ----------------------------------------------------------
  // Representation
  // ----------------------------------------------------------

  toString -> String {
    "Stack(size: \(self.size))"
  }
}


// ============================================================
//  Typed usage
// ============================================================

const s: Stack<Int> = Stack<Int>.empty

s.push(10)
 .push(20)
 .push(30)

System.print(s.toString)                 // Stack(size: 3)
System.print("empty? \(s.isEmpty)")      // empty? false


// ------------------------------------------------------------
// Option<T>
// ------------------------------------------------------------

s.peek.ifSome |value: Int| {
  System.print("top is \(value)")
}

Stack<Int>.empty.peek.ifNone || {
  System.print("nothing to peek")
}


// ------------------------------------------------------------
// Generic fold<A>
//
// T = Int
// A = Int
// f = (Int, Int) -> Int
// ------------------------------------------------------------

const sum: Int = s.fold(
  initial: 0,
  using: |acc: Int, value: Int| -> Int {
    acc + value
  }
)

System.print("sum = \(sum)")             // sum = 60


// The accumulator does not need to have the stack's element type:
//
// T = Int
// A = String
//
const description: String = s.fold(
  initial: "",
  using: |out: String, value: Int| -> String {
    "\(out)\(value) "
  }
)

System.print(description)                // 10 20 30


// ------------------------------------------------------------
// pop
// ------------------------------------------------------------

const popped: Option<Int> = s.pop

popped.ifSome |value: Int| {
  System.print("popped \(value)")
}

System.print("size after pop: \(s.size)") // 2


// ------------------------------------------------------------
// Concatenation
// ------------------------------------------------------------

const t: Stack<Int> = Stack<Int>.empty
  .push(40)
  .push(50)

const combined: Stack<Int> = s + t

System.print("combined \(combined.size)") // 4


// ------------------------------------------------------------
// Equality
// ------------------------------------------------------------

const a: Stack<Int> = Stack<Int>.empty
  .push(1)
  .push(2)

const b: Stack<Int> = Stack<Int>.empty
  .push(1)
  .push(2)
	
Stack.empty
Stack.flattened
Stack.sorted by: |x, y| x <=> y
Stack.sort
	by: |x, y| x <=> y
Stack.reverse
	with: |x, y| x > y
Stack.reversed
Stack.shuffled

System.print("a == b: \(a == b)")         // true


// ------------------------------------------------------------
// Introspection
// ------------------------------------------------------------

System.print("class of s : \(s.class)")
System.print("s isA Stack : \(s.isA(Stack))")
System.print("s isA Object: \(s.isA(Object))")
```

```ph
performNetworkRequest(on: "https://api.com")
	onSuccess: |response| {
		print("Success: \(response)")
	}
	onFailure: |response| {
		print("Failure: \(response)")
	}
	
swap(a, with: b)
```

```ph
@native
class List<T> is Iterable {  
  @class @native new() -> List  
  @internal @native _$length -> Int  
  @internal @native _$at(_ index: Int) -> Dynamic  
  @internal @native _$set(_ index: Int, _ value: Dynamic) -> Dynamic  
  @internal @native _$push(_ value: Dynamic) -> Dynamic  
  @internal @native _$replaceSlice(_ start: Int, _ end: Int, _ replacement: List) -> Dynamic  
	
  @native
  toString -> String  
	
  @native
  size -> Int
  
  first -> Option<T> {
    if size == 0 { 
	    return Option::None 
		} 
    return Option::Some(_$at(0))  
  }
  
  last -> Option<T> {  
    if size == 0 { 
	    return Option::None 
		}
    return Option::Some(at(size - 1))
  }
  
  at(_ i) { _$at(i) }  
  
  get(_ index) -> Option<T> { 
		let raw = at(index), 
				len = size, 
				i = index
    if i < 0 
	    then i = len + 1
    if i >= 0 and i < len   
      then return Option::Some(raw)  
    return Option::None  
  }  
  
  @private  
  sliceByRange(_ range) {  
    return range._$sliceBounds(self.size).match(  
      ok: |bounds| {  
        let start = bounds[0]  
        let end = bounds[1]  
        // C.3's consumer-local rule: a reversed normalized interval selects  
        // no ascending elements. Range itself gains no descending semantics.  
        if start > end then end = start
        let result = List.new(), i = start
        while i < end {  
          result._$push(self._$at(i))  
          i = i + 1  
        } 
        result  
      },  
      err: |error| { error.raise() }  
    )  
  }  
  
  [_ index] {  
    if (index.is(Range)) { return self.sliceByRange(index) }  
    let raw = self._$at(index)  
    let len = self.size  
    let i = index  
    if (i < 0) { i = len + i }  
    if (i >= 0 and i < len) {  
      return raw  
    }  
    throw IndexError.new("List index out of range")  
  }  
  
  [_ index, default] {  
    let raw = self._$at(index)  
    let len = self.size  
    let i = index  
    if (i < 0) { i = len + i }  
    if (i >= 0 and i < len) {  
      return raw  
    }  
    return default  
  }  
  
  get(_ index, orElse) {  
    let raw = self._$at(index)  
    let len = self.size  
    let i = index  
    if (i < 0) { i = len + i }  
    if (i >= 0 and i < len) {  
      return raw  
    }  
    return orElse.call(index)  
  }  
  
  append(_ value) {  
    self._$push(value)  
    return ()  
  }  
  
  prepend(_ value) {  
    let oneElementList = [value]  
    self._$replaceSlice(0, 0, oneElementList)  
    return ()  
  }  
  
  clear {  
    let emptyList = []  
    self._$replaceSlice(0, self.size, emptyList)  
    return ()  
  }  
  
  insert(_ value: T, at index: Int) -> Result<Unit, IndexError> {  
    let n = self.size  
    let p = index  
    if p < 0 {  
        p = n + p  
    }  
    if p < 0 or p > n {  
      return Result::Error(  
          IndexError("List#insert: index out of bounds")  
      )  
    }  
    let oneElementList = [value]  
    _$replaceSlice(p, p, oneElementList)  
    Result::Ok(())  
  }  
  
  remove(at index) {  
    let n = self.size  
    let p = index  
    if (p < 0) { p = n + p }  
    if (p < 0 or p >= n) {  
      return Result::Error(IndexError.new("List#remove: index out of bounds"))  
    }  
    let captured = self._$at(p)  
    let emptyList = []  
    self._$replaceSlice(p, p + 1, emptyList)  
    return Result::Ok(captured)  
  }  
  
  popFirst {  
    let n = self.size  
    if (n == 0) { return None }  
    let captured = self._$at(0)  
    let emptyList = []  
    self._$replaceSlice(0, 1, emptyList)  
    return Some(captured)  
  }  
  
  popLast {  
    let n = self.size  
    if (n == 0) { return None }  
    let captured = self._$at(n - 1)  
    let emptyList = []  
    self._$replaceSlice(n - 1, n, emptyList)  
    return Some(captured)  
  }  
  
  removeAll(where predicate) {  
    let retained = List.new()  
    let count = 0  
    for x in self {  
      if (predicate.call(x)) {  
        count = count + 1  
      } else {  
        retained._$push(x)  
      }  
    }  
    self._$replaceSlice(0, self.size, retained)  
    return count  
  }  
  
  swap(first a, second b) {  
    let n = self.size  
    let idxA = a  
    if (idxA < 0) { idxA = n + idxA }  
    if (idxA < 0 or idxA >= n) {  
      return Result::Error(IndexError.new("List#swap: first index out of bounds"))  
    }  
    let idxB = b  
    if (idxB < 0) { idxB = n + idxB }  
    if (idxB < 0 or idxB >= n) {  
      return Result::Error(IndexError.new("List#swap: second index out of bounds"))  
    }  
    if (idxA == idxB) {  
      return Result::Ok(())  
    }  
    let valA = self._$at(idxA)  
    let valB = self._$at(idxB)  
    self._$set(idxA, valB)  
    self._$set(idxB, valA)  
    return Result::Ok(())  
  }  
  
  // U-STD item 4 (U-ITER-FIX plan §"Not in this unit", DEC-ITER-A resolved):  
  // drives the cursor protocol (`iterate(_)`/`iteratorValue(_)`, ADR-0035 §1)  
  // rather than a raw `size`/`at(_)` index walk. `for x in self` compiles  
  // to the same `Invoke`-only `iterate`/`iteratorValue`/`isSome` loop as any  
  // user iterable (spec §3.1) — no `block_call`, no index math — so `each`  
  // (and everything below built over it: `map`/`filter`/`reduce`/`includes`)  
  // is protocol-driven behavior-preservingly.  
  // Given a live cursor, yields the element there (ADR-0035 §1,  
  // iteration.md §1). Only ever called with an in-range index, so it defers to  
  // `at(_)` directly.  
  iteratorValue(_ cursor) { self.at(cursor) }  
  
  // U-STD (DEFERRED.md #18): the public `.ph` wrapper over `_$set(_,_)`  
  // floor primitive — writes `put` at index `i` and returns `self` so writes  
  // chain (mirrors `add`). Selector `at(_,put)` matches `_$set`'s 2 args;  
  // the labeled parameter is named `put` (label == name, parser convention).  
  at(_ i, put) {  
    let len = self.size  
    let norm = i  
    if (norm < 0) { norm = len + norm }  
    if (norm < 0 or norm >= len) {  
      throw IndexError.new("Expected an in-range index, got an out-of-range Number")  
    }  
    self._$set(i, put)  
    return self  
  }  
  
  // C.3 deliberately accepts only a finite List replacement source. General  
  // Iterable replacement waits for Spec E's boundedness and re-entrancy rules.  
  replace(_ range, with replacements) {  
    if (not range.is(Range)) {  
      return Result::Error(SliceError.new("List#replace: first argument must be a Range"))  
    }  
    if (not replacements.is(List)) {  
      return Result::Error(SliceError.new("List#replace: replacement must be a List"))  
    }  
    return range._$sliceBounds(self.size).match(  
      ok: |bounds| {  
        let start = bounds[0]  
        let end = bounds[1]  
        if (start > end) { end = start }  
        self._$replaceSlice(start, end, replacements)  
        Result::Ok(())  
      },  
      err: |error| { Result::Error(error) }  
    )  
  }  
  
  // U-INDEX (ADR-0060): `[]` is its own dedicated, user-overridable  
  // selector — not `at`'s call-site sugar — so `List` must opt in  
  // explicitly with a thin delegation, same as any other collection  
  // author would. `xs[i]` sends `[_]`; `xs[i] = v` sends `[_]=(put)`.  
  @raises IndexError  
  [_ i]=(put val) {  
    if i is Range {  
        return self.replace(i, with: val).unwrap  
    }  
    return self.at(i, put: val)  
  }  
  
  // U-CORE-5 (decisions.md Q5, R-INV-5.3 E1-E5): structural equality —  
  // element-wise, order-sensitive, via each element's own `==`. Guarded by  
  // `isA(List)` so a non-List `other` is simply unequal (E2), never a dNU.  
  // Derived entirely over the floor (`size`/`at`/`isA`/`while`/`and`/`not`) —  
  // no new native primitive (ADR-0019 unchanged). `and`/`not` are the  
  // language's infix/prefix operator forms (`Bool#and(_:)`/`Bool#not`  
  // dispatched by the compiler, not dotted-call syntax — `and`/`not` are  
  // reserved words and cannot follow `.` as a bare identifier).  
  ==(_ other) {  
    if (other.is(List)) {  
      let same = (self.size == other.size)  
      let i = 0  
      // `and` is lazy (short-circuits); once `same` is false the loop  
      // condition is false without evaluating `i < self.size`, so the loop  
      // exits before `at(i)` can run out of bounds.  
      while (same and (i < self.size)) {  
        same = (self.at(i) == other.at(i))  
        i = i + 1  
      }  
      return same  
    } else {  
      return false  
    }  
  }  
  
  // U-CORE-5 (R-INV-5.3 E6): `!=` MUST route through `==`. The floor  
  // `Object#!=` (`object_neq`) negates identity `value_eq` directly, NOT  
  // `self.==` — without this override `list != other` would stay  
  // identity-based and contradict the structural `==` above (the `==`⊗`!=`  
  // decoupling hazard).  
  !=(_ other) {  
    return not (self == other)  
  }  
}  
  
// Kernel Map/Set (ADR-0032 §1, ADR-0039, U-COLLTYPES Phase 1): native  
// insertion-ordered hash collections — Object::Map/Object::Set, sharing the  
// MapObject backing struct (DEC-CT-B) but with distinct native-primitive  
// bindings and distinct classes. This skeleton reopens the bootstrapped rows  
// to define the public protocol over the native floor (ADR-0019's "hybrid: native  
// primitives, self-defined control"). Both are MUTABLE, so neither installs a  
// `hash` override — they inherit Object#hash (identity), so per Q5  
// (decisions.md, collection-protocol.md law 4) neither is a valid Map/Set key;  
// `put_`/`add_` enforce this (DEC-CT-C) by rejecting a mutable-collection  
// key (List/Map/Set) with a raised Error.
```

```ph
class Matcher {
	parse(_ expr: Expression, _ state: ParserState) {
		
	}
	
	parse(_ expr: Expression, _ state: ParserState) {
		
	}
	
	
	parse(_ expr: Expression, _ state: ParserState) {
		
	}
}

|x, y, z| x + y + z

_ + _
_ / _
_ * _
_ % _

((_: Int) ** (_: Int))

(Int, Int) -> Int

const call = Matcher::parse(Expression::Int, )

match call {
	parse()
}

type Parse(_ expr: Expression, _ state: ParserState)

case Parse(...)
case Parse(expr, ...)
```