@native
class List<T> is Iterable {
  @class @native new() -> List
  @internal @native _$length -> Int
  @internal @native _$at(_ index: Int) -> Dynamic
  @internal @native _$set(_ index: Int, _ value: Dynamic) -> Dynamic
  @internal @native _$push(_ value: Dynamic) -> Dynamic
  @internal @native _$replaceSlice(_ start: Int, _ end: Int, _ replacement: List) -> Dynamic
  @native toString -> String
  size { self._$length }

  first {
    if (self.size == 0) { return None }
    return Some(self.at(0))
  }

  last {
    if (self.size == 0) { return None }
    return Some(self.at(self.size - 1))
  }

  at(_ i) {
    return self._$at(i)
  }

  get(_ index) {
    let raw = self._$at(index)
    let len = self.size
    let i = index
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return Some(raw)
    }
    return None
  }

  @private
  sliceByRange(_ range) {
    return range._$sliceBounds(self.size).match(
      ok: |bounds| {
        let start = bounds[0]
        let end = bounds[1]
        // C.3's consumer-local rule: a reversed normalized interval selects
        // no ascending elements. Range itself gains no descending semantics.
        if (start > end) { end = start }
        let result = List.new()
        let i = start
        while (i < end) {
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

  insert(_ value, at index) {
    let n = self.size
    let p = index
    if (p < 0) { p = n + p }
    if (p < 0 or p > n) {
      return Err.new(IndexError.new("List#insert: index out of bounds"))
    }
    let oneElementList = [value]
    self._$replaceSlice(p, p, oneElementList)
    return Ok.new(())
  }

  remove(at index) {
    let n = self.size
    let p = index
    if (p < 0) { p = n + p }
    if (p < 0 or p >= n) {
      return Err.new(IndexError.new("List#remove: index out of bounds"))
    }
    let captured = self._$at(p)
    let emptyList = []
    self._$replaceSlice(p, p + 1, emptyList)
    return Ok.new(captured)
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
      return Err.new(IndexError.new("List#swap: first index out of bounds"))
    }
    let idxB = b
    if (idxB < 0) { idxB = n + idxB }
    if (idxB < 0 or idxB >= n) {
      return Err.new(IndexError.new("List#swap: second index out of bounds"))
    }
    if (idxA == idxB) {
      return Ok.new(())
    }
    let valA = self._$at(idxA)
    let valB = self._$at(idxB)
    self._$set(idxA, valB)
    self._$set(idxB, valA)
    return Ok.new(())
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
      return Err.new(SliceError.new("List#replace: first argument must be a Range"))
    }
    if (not replacements.is(List)) {
      return Err.new(SliceError.new("List#replace: replacement must be a List"))
    }
    return range._$sliceBounds(self.size).match(
      ok: |bounds| {
        let start = bounds[0]
        let end = bounds[1]
        if (start > end) { end = start }
        self._$replaceSlice(start, end, replacements)
        Ok.new(())
      },
      err: |error| { Err.new(error) }
    )
  }

  // U-INDEX (ADR-0060): `[]` is its own dedicated, user-overridable
  // selector — not `at`'s call-site sugar — so `List` must opt in
  // explicitly with a thin delegation, same as any other collection
  // author would. `xs[i]` sends `[_]`; `xs[i] = v` sends `[_]=(put)`.
  [_ i]=(put val) {
    if (i.is(Range)) { return self.replace(i, with: val).unwrap }
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
