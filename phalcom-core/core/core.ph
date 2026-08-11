class Object {
  // Is-kind-of test: true iff `cls` is the receiver's class or an ancestor of
  // it (object-model.md §8, is-tests.md — U-IS). Derived purely over the floor
  // — class/==/superclass — so it needs no native primitive (ADR-0019/0023).
  // The superclass chain is a run of class objects terminating in the `None`
  // singleton (class_superclass returns `None` at the root), so the walk
  // stops on `c == None`. The `ifTrue` result is in pop (statement) position,
  // so U-CORE-2's Some-lift is elided; the body neither reads nor depends on
  // `ifTrue`'s return shape.
  //
  // No RHS-is-a-class guard: a non-class `cls` never equals any `c` in the
  // chain, so the walk naturally falls through to `false` (is-tests.md I-4,
  // ratified `false`). Do not add a `cls.isA(...)`-style guard here — it
  // would recurse through the `isA` alias below forever, and it would target
  // `Behavior`, which is not bootstrapped in this codebase (ADR-0003 designs
  // it; core.ph has only `Object`/`Class`/`Metaclass`).
  is(_ cls) {
    let c = self.class
    while (c != None) {
      (c == cls).ifTrue || { return true }
      c = c.superclass
    }
    return false
  }

  // Exact test: true iff `cls` is the receiver's *live, direct* class —
  // no superclass walk. Backs the `x is! T` surface (is-tests.md).
  isExactly(_ cls) { self.class == cls }

  // Back-compat alias (U-CORE-1) — `is(_)` is now the primary kind-of test;
  // `isA` is retained so existing internal (`List#==` etc.) and user callers
  // keep working unchanged.
  isA(_ cls) { self.is(cls) }
}

class Class {
  new() { self._$new() }
}

class Metaclass {}

// `Error` (U-CORE-6, ADR-0008) is bootstrapped natively with `_message` at
// fixed slot 0 (`message`/`raise` are native primitives reading/using it
// directly — `primitive/error.rs`), but ships with no user-visible way to
// *set* that slot: bare `Error.new()` (the generic 0-arg allocator) leaves it
// unset, and — per ADR-0011/U-INH §3.5 "fields stack, never alias" — a `.ph`
// *subclass* that independently assigns a same-named `_message` field gets
// its own **fresh** slot, not this one (`subclass_field_offset_stability`).
// So the only way to reach the native slot 0 from `.ph` is an `@constructor`
// declared on `Error` **itself**: this reopen (recognised at compile time as
// reopening the already-registered native class, not a fresh `extends`) adds
// exactly that, giving `Error.new(msg)` a working 1-arg constructor and every
// subclass a `super.new(msg)` to route through (error-handling.md §1,
// U-ERR: `throw ArgumentError("age must be >= 0")`-style user `Error`
// subclasses need this to carry a real `message`).
class Error {
  // Preserved bare 0-arg form (many pre-U-ERR call sites — `Error.new()` in
  // Fiber/Future fixtures — rely on it; declaring *any* `new`-named
  // `@constructor` drops the generic inherited bare allocator, U7, so this must
  // be declared explicitly alongside the 1-arg form below, not left implicit).
  @constructor
  new() {
    _message = None
    _kind = None
    _cause = None
    _displaced = None
  }
  @constructor
  new(_ msg) {
    _message = msg
    _kind = None
    _cause = None
    _displaced = None
  }
  kind { _kind }
  kind=(put val) { _kind = val }
  cause { _cause }
  cause=(put val) { _cause = val }
  displaced { _displaced }
  displaced=(put val) { _displaced = val }
}

// Design-by-Contract failure classes (U-ANNOT-CONTRACTS,
// `docs/spec/v0.2/experimental/annotations-contracts.md`). Each is raised by
// the woven `@requires`/`@ensures`/`@invariant` check emitted by
// `phalcom-core/src/compiler/attributes.rs`'s `build_check_stmt` — zero Rust,
// zero new primitive, ordinary `Error` subclasses that inherit `message`/
// `raise` from the `_message`-slot machinery above.
class PreconditionError is Error {}

class PostconditionError is Error {}

class InvariantError is Error {}

// Boundary-guard exception (error-handling.md §1, U-STRING). Raised by library
// code to indicate invalid argument values or arities. Zero fields — the
// inherited `Error` `@constructor new(msg)` gives `ArgumentError.new(msg)` a working
// constructor for `throw ArgumentError.new("msg")` sites (U-INH inherited-ctor
// resolution).
class ArgumentError is Error {}

// Raised by strict Map subscript lookup when no equal key is present.
class KeyError is Error {}

// Raised when a sequence index is out of bounds.
class IndexError is Error {}

// Raised when a Range cannot describe a sequence slice or replacement.
class SliceError is Error {}

// Raised while building an association Map literal when a logically equal key
// was already contributed. Ordinary post-construction Map insertion still
// overwrites by design.
class DuplicateKeyError is Error {
  @constructor
  new(_ key) {
    super.new("Duplicate key: " + key.toString)
    _key = key
  }
  key { _key }
}

class Number {}

class Int is Number {}

class Float is Number {}

class String {
  // Display (U-CORE-4, R-INV-4.1): a string's display *is* itself — no
  // representation read, so this is `.ph`-derivable rather than a floor
  // primitive (ADR-0019's derivability test).
  toString { self }

  // Byte count. UTF-8 buffer length in bytes (not codepoints).
  size { self._$byteCount }
  isEmpty { self._$byteCount == 0 }

  // Byte-range slice. Native storage operation stays internal; public wrapper
  // preserves existing bounds/UTF-8-boundary diagnostics.
  slice(_ start, _ end) { self._$slice(start, end) }

  // Number of leading bytes in the UTF-8 sequence starting at byte offset `i`.
  // Read purely from the lead byte's numeric range: 1/2/3/4-byte sequences are
  // encoded by the lead byte's numeric value (no bitmask needed).
  leadByteLen(_ i) {
    const b = self._$byteAt(i)
    return (b == None).ifTrue(|| { None }, ifFalse: || {
      (b < 128).ifTrue(|| { 1 }, ifFalse: || {
        (b < 224).ifTrue(|| { 2 }, ifFalse: || {
          (b < 240).ifTrue(|| { 3 }, ifFalse: || { 4 }) }) })
    })
  }

  // The Unicode scalar value at byte offset `i`, or `None` if out-of-range
  // or mid-sequence. UTF-8 decode via division/modulo (no bitwise ops).
  codePointAt(_ i) {
    const b0 = self._$byteAt(i)
    return (b0 == None).ifTrue(|| { None }, ifFalse: || {
      (b0 < 128).ifTrue(|| {
        // ASCII single byte (0xxxxxxx)
        b0
      }, ifFalse: || {
        (b0 < 192).ifTrue(|| {
          // Continuation byte (10xxxxxx), not a start byte
          None
        }, ifFalse: || {
          (b0 < 224).ifTrue(|| {
            // 2-byte sequence (110xxxxx 10xxxxxx)
            const b1 = self._$byteAt(i + 1)
            (b1 == None).ifTrue(|| { None }, ifFalse: || {
              (b1 < 128).ifTrue(|| { None }, ifFalse: || {
                (b1 >= 192).ifTrue(|| { None }, ifFalse: || {
                  ((b0 - 192) * 64) + (b1 - 128)
                })
              })
            })
          }, ifFalse: || {
            (b0 < 240).ifTrue(|| {
              // 3-byte sequence (1110xxxx 10xxxxxx 10xxxxxx)
              const b1 = self._$byteAt(i + 1)
              const b2 = self._$byteAt(i + 2)
              (b1 == None).ifTrue(|| { None }, ifFalse: || {
                (b2 == None).ifTrue(|| { None }, ifFalse: || {
                  (b1 < 128).ifTrue(|| { None }, ifFalse: || {
                    (b1 >= 192).ifTrue(|| { None }, ifFalse: || {
                      (b2 < 128).ifTrue(|| { None }, ifFalse: || {
                        (b2 >= 192).ifTrue(|| { None }, ifFalse: || {
                          ((b0 - 224) * 4096) + ((b1 - 128) * 64) + (b2 - 128)
                        })
                      })
                    })
                  })
                })
              })
            }, ifFalse: || {
              (b0 < 248).ifTrue(|| {
                // 4-byte sequence (11110xxx 10xxxxxx 10xxxxxx 10xxxxxx)
                const b1 = self._$byteAt(i + 1)
                const b2 = self._$byteAt(i + 2)
                const b3 = self._$byteAt(i + 3)
                (b1 == None).ifTrue(|| { None }, ifFalse: || {
                  (b2 == None).ifTrue(|| { None }, ifFalse: || {
                    (b3 == None).ifTrue(|| { None }, ifFalse: || {
                      (b1 < 128).ifTrue(|| { None }, ifFalse: || {
                        (b1 >= 192).ifTrue(|| { None }, ifFalse: || {
                          (b2 < 128).ifTrue(|| { None }, ifFalse: || {
                            (b2 >= 192).ifTrue(|| { None }, ifFalse: || {
                              (b3 < 128).ifTrue(|| { None }, ifFalse: || {
                                (b3 >= 192).ifTrue(|| { None }, ifFalse: || {
                                  ((b0 - 240) * 262144) + ((b1 - 128) * 4096) + ((b2 - 128) * 64) + (b3 - 128)
                                })
                              })
                            })
                          })
                        })
                      })
                    })
                  })
                })
              }, ifFalse: || {
                // Invalid UTF-8 start byte
                None
              })
            })
          })
        })
      })
    })
  }

  // Find first occurrence of a substring, scanning left-to-right by byte.
  // O(n·m) naive search. Returns the byte offset, or -1 if not found.
  indexOf(_ needle) {
    (needle.isA(String)).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("indexOf: needle must be a String")
    })
    (needle.isEmpty).ifTrue(|| {
      throw ArgumentError.new("indexOf: needle must be non-empty")
    })

    let i = 0
    while (i <= self._$byteCount - needle._$byteCount) {
      let match = true
      let j = 0
      while (j < needle._$byteCount) {
        (self._$byteAt(i + j) == needle._$byteAt(j)).ifTrue(|| {}, ifFalse: || {
          match = false
        })
        (match).ifTrue(|| { j = j + 1 }, ifFalse: || { j = needle._$byteCount })
      }
      (match).ifTrue(|| { return i })
      i = i + 1
    }
    return -1
  }

  // Split by delimiter substring. Returns a List of String segments.
  split(_ delimiter) {
    (delimiter.isA(String)).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("split: delimiter must be a String")
    })
    (delimiter.isEmpty).ifTrue(|| {
      throw ArgumentError.new("split: delimiter must be non-empty")
    })

    let result = List.new()
    let prev = 0
    let i = self.indexOf(delimiter)
    while (i != -1) {
      result._$push(self._$slice(prev, i))
      prev = i + delimiter._$byteCount
      // Search for next occurrence after this delimiter
      let rest = self._$slice(prev, self._$byteCount)
      let nextIdx = rest.indexOf(delimiter)
      (nextIdx == -1).ifTrue(|| { i = -1 }, ifFalse: || { i = prev + nextIdx })
    }
    result._$push(self._$slice(prev, self._$byteCount))
    return result
  }

  // Replace all occurrences of `from` with `to`.
  replace(_ from, _ to) {
    (from.isA(String)).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("replace: from must be a String")
    })
    (to.isA(String)).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("replace: to must be a String")
    })
    (from.isEmpty).ifTrue(|| {
      throw ArgumentError.new("replace: from must be non-empty")
    })

    let result = ""
    let prev = 0
    let i = self.indexOf(from)
    while (i != -1) {
      result = result + self._$slice(prev, i) + to
      prev = i + from._$byteCount
      let rest = self._$slice(prev, self._$byteCount)
      let nextIdx = rest.indexOf(from)
      (nextIdx == -1).ifTrue(|| { i = -1 }, ifFalse: || { i = prev + nextIdx })
    }
    result = result + self._$slice(prev, self._$byteCount)
    return result
  }

  // Trim whitespace from start and end, default or custom charset.
  trim() {
    return self.trim(" \t\n\r")
  }
  trimStart() {
    return self.trimStart(" \t\n\r")
  }
  trimEnd() {
    return self.trimEnd(" \t\n\r")
  }

  trim(_ chars) { self.trimStart(chars).trimEnd(chars) }

  // Trim from the start using the given charset.
  trimStart(_ chars) {
    (chars.isA(String)).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("trimStart: chars must be a String")
    })

    let i = 0
    let stop = false
    while ((i < self._$byteCount).and(|| { not stop })) {
      const cp = self.codePointAt(i)
      let found = false
      let j = 0
      while (j < chars._$byteCount) {
        (chars.codePointAt(j) == cp).ifTrue(|| { found = true })
        const len = chars.leadByteLen(j)
        (len == None).ifTrue(|| { j = j + 1 }, ifFalse: || { j = j + len })
      }
      (found).ifTrue(|| {
        i = i + self.leadByteLen(i)
      }, ifFalse: || {
        stop = true  // exit loop, keeping i at the first non-trimmed byte
      })
    }
    return self._$slice(i, self._$byteCount)
  }

  // Trim from the end using the given charset.
  trimEnd(_ chars) {
    (chars.isA(String)).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("trimEnd: chars must be a String")
    })

    let i = self._$byteCount
    let stop = false
    while ((i > 0).and(|| { not stop })) {
      // Scan backward one byte at a time to find the previous lead byte
      i = i - 1
      let cp = self.codePointAt(i)
      (cp == None).ifTrue(|| {
        // Not a lead byte, keep scanning back
      }, ifFalse: || {
        // Found a lead byte; check if it's in the trim set
        let found = false
        let j = 0
        while (j < chars._$byteCount) {
          (chars.codePointAt(j) == cp).ifTrue(|| { found = true })
          const len = chars.leadByteLen(j)
          (len == None).ifTrue(|| { j = j + 1 }, ifFalse: || { j = j + len })
        }
        (found).ifTrue(|| {}, ifFalse: || {
          // Not in the set; keep this whole character and stop scanning
          i = i + self.leadByteLen(i)
          stop = true
        })
      })
    }
    return self._$slice(0, i)
  }

  // Repeat the string `count` times.
  *(_ count) {
    (count.isA(Number)).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("*: count must be a Number")
    })
    (count >= 0).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("*: count must be >= 0")
    })
    (count % 1 == 0).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("*: count must be an integer")
    })

    (count == 0).ifTrue(|| { return "" })
    (count == 1).ifTrue(|| { return self })

    let result = ""
    let i = 0
    while (i < count) {
      result = result + self
      i = i + 1
    }
    return result
  }

  // Byte sequence accessor (U-STRING §2.4).
  bytes { StringByteSequence.new(self) }

  // Codepoint sequence accessor (U-STRING §2.4).
  codePoints { StringCodePointSequence.new(self) }
}

// Byte-level sequence view (U-STRING §2.4, ADR-0048 shaped).
class StringByteSequence {
  @constructor
  new(_ s) { _string = s }

  size { _string._$byteCount }

  at(_ i) { _string._$byteAt(i) }

  each(_ f) {
    let i = 0
    while (i < self.size) {
      f.call(self.at(i))
      i = i + 1
    }
  }

  // Iterate over byte offsets: cursor steps to next lead byte.
  @private
  nextCursor(_ cursor) {
    const next = (cursor == None).ifTrue(|| { 0 }, ifFalse: || {
      cursor + 1
    })
    return (next < _string._$byteCount).ifTrue(|| { next }, ifFalse: || { None })
  }
}

// Codepoint-level sequence view (U-STRING §2.4, ADR-0048 shaped).
class StringCodePointSequence {
  @constructor
  new(_ s) { _string = s }

  // Codepoint count: full scan (no native "codepoint length").
  size {
    let n = 0
    let i = self.nextCursor(None)
    while (i != None) {
      n = n + 1
      i = self.nextCursor(i)
    }
    return n
  }

  at(_ byteOffset) { _string.codePointAt(byteOffset) }

  each(_ f) {
    let i = self.nextCursor(None)
    while (i != None) {
      f.call(self.at(i))
      i = self.nextCursor(i)
    }
  }

  // Iterate over byte offsets: cursor steps by UTF-8 char boundary.
  @private
  nextCursor(_ cursor) {
    const next = (cursor == None).ifTrue(|| { 0 }, ifFalse: || {
      cursor + _string.leadByteLen(cursor)
    })
    return (next < _string._$byteCount).ifTrue(|| { next }, ifFalse: || { None })
  }
}

class Bool {
  // Display (U-CORE-4, R-INV-4.1): derived over the sacred `ifTrue(_,
  // ifFalse)` selector (proven syntax:
  // `control-flow/control_flow_send_equivalence.ph` L9). This `toString` is
  // NOT itself sacred (floor-census §5's `bool_sacred_pristine` tracks only
  // the six original selectors), so adding it does not trip the inliner
  // deopt.
  toString {
    return self.ifTrue(|| { "true" }, ifFalse: || { "false" })
  }
}

// The boolean tower (ADR-0004): `Bool` is abstract; `True` and `False` are its
// two concrete singleton subclasses — the surface classes of `true`/`false`
// (so `true.class == True`). Their control-flow behaviour (`not`/`and`/`or`/
// `ifTrue`/`ifFalse`/`ifTrue:ifFalse:`) lives on `Bool` as sacred native
// primitives and is reached by inheritance (KEEP; see floor-census.md §2.6/§5),
// so these bodies are intentionally empty. The globals are already bound in Rust
// (VM::install_core, add_class!) — unlike `None`, they name the class objects,
// so these reopens re-emit the identical DefineGlobal binding (a harmless no-op).
class True {}

class False {}

class Symbol {}

// Absence is an Option (ADR-0007), not a surface `nil`. `Option` is abstract;
// `Some` wraps one value and `None` is a single shared singleton. These are
// bootstrapped in Rust (universe.rs): the classes, the `Some(_)` construction
// primitive, and the `match(some:, none:)` eliminator. The skeletons below only
// *reopen* those bootstrapped rows so the class names are surface-visible.
//
// U-CORE-2 (catalog-delta.md §2.2) adds the four combinators that make
// `ifTrue`/`ifFalse`'s newly-well-formed `Option` result actually chainable —
// `ifNone(_)`, `orElse(_)`, `isSome`, `isNone`, every one defined over `match`
// (values-and-absence.md §3.3), so `Some>>_` / `None>>_` branching stays
// dispatch, not a variant check. The richer suite (`map`, `flatMap`,
// `filter`, `ifSome`, `unwrapOr`, …) is still deliberately NOT defined here —
// that remains U-STD's job. Do not add those bodies to this skeleton.
//
// `None` (the name) resolves to the shared singleton *value*, not the `None`
// class; that global is bound in Rust (VM::install_core).
//
// There is deliberately NO `class None {}` reopen here (unlike `Option`/
// `Some`): `Statement::Class` unconditionally emits `DefineGlobal` at the end
// of every class body, reopen or not (compiler/lib.rs). For every other core
// class that's a harmless no-op — the global already points at that same
// class object — but `None`'s global is bound to the *singleton instance*,
// not the class, so reopening it here would silently clobber that binding
// back to the class object the moment core.ph runs. See DEFERRED.md: a
// future unit that needs to add real members to `None` must fix that
// compiler special case first, not just re-add this skeleton.

class Option {
  // Runs `f` (0-arity) for its side effect when `self` is `None`; passes
  // `Some` through untouched. Never extracts — returns `self` so calls chain
  // (values-and-absence.md §3.3's "Effect" group).
  ifNone(_ f) {
    return self.match(some: |v| { self }, none: || { f.call(); self })
  }

  // `Some` passes through unchanged; `None` becomes `f`'s (0-arity) `Option`
  // result (values-and-absence.md §3.3's "Transform" group). This is the
  // `??` operator's target (§3.4: `a ?? b` === `a.orElse || { b }`).
  orElse(_ f) {
    return self.match(some: |v| { self }, none: || { f.call() })
  }

  isSome { self.match(some: |v| { true }, none: || { false }) }

  isNone { self.match(some: |v| { false }, none: || { true }) }

  // U-STD (values-and-absence.md §3.3's "Transform" group; catalog-delta §2.2):
  // `Some(v)` becomes `Some(f(v))`; `None` passes through untouched. `f` is a
  // 1-arity block over the wrapped value; the result is re-wrapped so the
  // chain stays an `Option`.
  map(_ f) {
    return self.match(some: |v| { Some(f.call(v)) }, none: || { self })
  }

  // U-STD (values-and-absence.md §3.3's "Transform" group): like `map`, but `f`
  // already returns an `Option`, so its result is used directly rather than
  // re-wrapped — the monadic bind (`>>=`). `None` short-circuits to `self`.
  flatMap(_ f) {
    return self.match(some: |v| { f.call(v) }, none: || { self })
  }

  // U-STD (values-and-absence.md §3.3's "Filter" group): `Some(v)` stays `Some(v)`
  // when `pred(v)` is `true`, otherwise collapses to the shared `None` singleton;
  // `None` passes through. `pred` must return a real `Bool` (ADR-0021).
  filter(_ pred) {
    return self.match(some: |v| { if (pred.call(v)) { self } else { None } }, none: || { self })
  }

  // U-STD (values-and-absence.md §3.3's "Effect" group; mirror of `ifNone`): runs
  // the 1-arity block `f` for its side effect on the wrapped value when `Some`,
  // then returns `self` so calls chain; a `None` is passed through untouched.
  ifSome(_ f) {
    return self.match(some: |v| { f.call(v); self }, none: || { self })
  }

  // U-STD (values-and-absence.md §3.3's "Extract" group): unwraps a `Some` to its
  // value, or yields `default` for a `None`. The eager sibling of `orElse`
  // (which takes a block); here `default` is an already-evaluated fallback value.
  unwrapOr(_ default) {
    return self.match(some: |v| { v }, none: || { default })
  }

  // Display (values-and-absence §3, U-CORE-4, R-INV-4.3). Derived over
  // `match`, so a user-overridden `match` is respected (R-INV-2.4) and the
  // inner value is rendered via its OWN `toString` message (so a
  // value-typed payload agrees with the print path, R-INV-4.1).
  toString { self.match(some: |v| { "Some(" + v.toString + ")" }, none: || { "None" }) }

  // absence -> error bridge (error-handling.md §5, result.md §2, ADR-0007):
  // `Some(v)` already carries a real value, so no reason is needed; `None`
  // has no value, so `err` fills in the failure reason. Round-trips with
  // `Result#ok()` below (`Some(v).okOr(_)` -> `Ok(v)` -> `.ok()` -> `Some(v)`).
  okOr(_ err) {
    return self.match(some: |v| { Ok.new(v) }, none: || { Err.new(err) })
  }

  ==(_ other) {
    other.isA(Option).ifFalse || { return false }
    return self.match(
      some: |v| { other.match(some: |ov| { v == ov }, none: || { false }) },
      none: || { other.isNone }
    )
  }

  hash { self.match(some: |v| { v.hash }, none: || { 0 }) }
}

class Some {}

// `Result`/`Ok`/`Err` (U-ERR, result.md §1-§3; ADR-0008 the error model,
// ADR-0007 the abstract-root-plus-two-subclasses machinery `Option`/`Some`/
// `None` already established). Unlike `Some`/`None` — bootstrapped natively
// because U6 predated U7's user-facing `@constructor` — `Result`/`Ok`/`Err` are
// **pure `.ph`**: U7's `@constructor` + `_`-prefixed instance fields need no
// floor primitive at all (net floor delta for this whole file: **0**).
//
// `Result` gets its **own** `match(ok:,err:)`, deliberately not reusing
// `Option`'s native one (forward-compat.md §2: the two must not couple, so a
// future migration of `Option` to `.ph` stays symmetric and doesn't touch
// `Result`).
class Result {
  isOk { self.match(ok: |v| { true }, err: |e| { false }) }

  isErr { self.match(ok: |v| { false }, err: |e| { true }) }

  // Transforms the `Ok` value; an `Err` passes through unchanged (never
  // raises — a pure value transform, result.md §2).
  map(_ f) {
    return self.match(ok: |v| { Ok.new(f.call(v)) }, err: |e| { self })
  }

  // Transforms the `Err` reason; an `Ok` passes through unchanged — the
  // symmetric counterpart of `map` (result.md §2).
  mapErr(_ f) {
    return self.match(ok: |v| { self }, err: |e| { Err.new(f.call(e)) })
  }

  // Chains an `Ok` -> `Result` function (flat-map/monadic bind); short-
  // circuits on `Err` (result.md §2).
  andThen(_ f) {
    return self.match(ok: |v| { f.call(v) }, err: |e| { self })
  }

  // The `Ok` value, or **re-`throw`** the `Err` reason (`throw expr` ===
  // `expr.raise()`, ADR-0031 §1) — the value -> exception bridge
  // (error-handling.md §5, result.md §3). If the `Err` reason is not itself
  // an `Error` (a user built `Err.new(42)`), `raise` misses and this
  // surfaces the ordinary `doesNotUnderstand` miss — consistent with the
  // only-`Error`-throwable rule, not special-cased here.
  unwrap { self.match(ok: |v| { v }, err: |e| { e.raise() }) }

  unwrapOr(_ default) {
    return self.match(ok: |v| { v }, err: |e| { default })
  }

  // The `Err` reason, or re-`throw` if `Ok` — symmetric to `unwrap`.
  unwrapErr { self.match(ok: |v| { v.raise() }, err: |e| { e }) }

  // `Result` -> `Option`: drops the failure reason (result.md §2/§5). Round-
  // trips with `Option#okOr(_)` above.
  ok() {
    return self.match(ok: |v| { Some(v) }, err: |e| { None })
  }

  // Display: each arm renders its payload via its OWN `toString` (agrees
  // with `Option#toString`'s pattern, R-INV-4.1).
  toString { self.match(ok: |v| { "Ok(" + v.toString + ")" }, err: |e| { "Err(" + e.toString + ")" }) }
}

class Ok is Result {
  @constructor
  new(_ v) { _value = v }

  match(ok, err) { ok.call(_value) }
}

class Err is Result {
  @constructor
  new(_ e) { _error = e }

  match(ok, err) { err.call(_error) }
}

// The throw -> value bridge (error-handling.md §5): runs `self` (0-arity),
// capturing a `throw` into `Err(e)`; success is `Ok(v)`. Pure `.ph` over
// `on(_)(_)` (U-ERR, ADR-0038) — no floor cost. Installed on the abstract
// `Function` root so both `Closure` and (reflectively) `Method` inherit it,
// mirroring how `call`/`on`/`ensure` are native on both.
class Function {
  // Explicit `()` (a method, not a getter — `attempt() { … }` vs `attempt {
  // …`) so the call-site selector encodes as `attempt()`, matching the
  // spec's `{ risky() }.attempt()` call form (error-handling.md §5) exactly.
  attempt() {
    return || { Ok.new(self.call()) }.on(Error) |e| { Err.new(e) }
  }
}

// Kernel List (ADR-0020): a native array-backed heap object (ListObject),
// not an InstanceObject — bootstrapped in Rust (universe.rs) with five floor
// implementation primitives (`_$length`/`_$at`/`_$set`/`_$push`, plus native `new()`). This
// skeleton reopens that bootstrapped row to define the public protocol over
// those primitives (ADR-0019's "hybrid: native primitives, self-defined
// control"). `toString` is ALSO a native primitive this unit, not defined
// here — see the U-LIST return contract for why (element-value stringification
// is blocked on U-CORE-4; DEFERRED.md #19). U-STD (catalog-delta §2.4;
// DEFERRED.md #18/#20/#25) discharges the deferral for the combinator layer:
// `map`/`reduce`/`filter`/`includes`/`isEmpty` and the `at(_:put:)` wrapper
// over `_$set` now live below, all pure `.ph` over the floor. Only
// **list-literal syntax** `[a, b, c]` remains deferred (it needs a new ADR +
// parser work; DEFERRED.md #6) — do not add that here.

class Iterable {
  // Generic index-cursor walk over `self.size` (ADR-0048 §1/§3). A subclass whose
  // cursor is not a 0..size index (none in-kernel today) overrides this.
  iterate(_ cursor) {
    const next = (cursor == None).ifTrue(|| { 0 }, ifFalse: || { cursor + 1 })
    return (next < self.size).ifTrue(|| { next }, ifFalse: || { None })
  }

  each(_ f) {
    for (x in self) {
      f.call(x)
    }
    return ()
  }

  // map/filter/reduce/includes walk `iterate`/`iteratorValue` DIRECTLY, not
  // `self.each`, so generic operations remain protocol-driven and independent
  // of any receiver-specific traversal convenience.
  // Concrete Iterable transforms are eager. Lazy transforms live behind `.iter`,
  // so the receiver makes evaluation timing visible at every call site.
  map(_ f) {
    let result = List.new()
    let c = self.iterate(None)
    while (c != None) {
      result.append(f.call(self.iteratorValue(c)))
      c = self.iterate(c)
    }
    return result
  }

  map(indexed f) {
    let result = List.new()
    let index = 0
    let c = self.iterate(None)
    while (c != None) {
      result.append(f.call(index, self.iteratorValue(c)))
      index = index + 1
      c = self.iterate(c)
    }
    return result
  }

  each(indexed f) {
    let index = 0
    let c = self.iterate(None)
    while (c != None) {
      f.call(index, self.iteratorValue(c))
      index = index + 1
      c = self.iterate(c)
    }
    return ()
  }

  flatMap(_ f) {
    let result = List.new()
    let outer = self.iterate(None)
    while (outer != None) {
      let inner = f.call(self.iteratorValue(outer))
      let ic = inner.iterate(None)
      while (ic != None) {
        result.append(inner.iteratorValue(ic))
        ic = inner.iterate(ic)
      }
      outer = self.iterate(outer)
    }
    return result
  }

  filter(_ pred) {
    let result = List.new()
    let c = self.iterate(None)
    while (c != None) {
      let x = self.iteratorValue(c)
      pred.call(x).ifTrue(|| { result.append(x) }, ifFalse: || { None })
      c = self.iterate(c)
    }
    return result
  }

  includes(_ x) {
    let found = false
    let c = self.iterate(None)
    while (c != None) {
      (self.iteratorValue(c) == x).ifTrue(|| { found = true }, ifFalse: || { None })
      c = self.iterate(c)
    }
    return found
  }

  isEmpty { self.size == 0 }

  all(where f) {
    for (x in self) {
      f.call(x).ifFalse || { return false }
    }
    return true
  }

  any(where f) {
    for (x in self) {
      f.call(x).ifTrue || { return true }
    }
    return false
  }

  none(where f) {
    for (x in self) {
      f.call(x).ifTrue || { return false }
    }
    return true
  }

  count {
    let n = 0
    for (x in self) { n = n + 1 }
    return n
  }

  count(where f) {
    let n = 0
    for (x in self) { f.call(x).ifTrue || { n = n + 1 } }
    return n
  }

  find(where f) {
    for (x in self) {
      f.call(x).ifTrue || { return Some(x) }
    }
    return None
  }

  index(where f) {
    let index = 0
    for (x in self) {
      f.call(x).ifTrue || { return Some(index) }
      index = index + 1
    }
    return None
  }

  join { self.join("") }

  join(_ sep) {
    // Note: O(N²) allocation cost due to naive string concatenation. Each `result = result + ...`
    // allocates a new string and copies all prior content. For N elements, total work is ~N²/2.
    // This is acceptable for Phalcom's interpreter domain (collections stay small) but users
    // joining large collections should be aware of this limitation.
    let first = true
    let result = ""
    for (x in self) {
      first.ifFalse || { result = result + sep }
      first = false
      result = result + x.toString
    }
    return result
  }

  // D.1 splits explicit-initial accumulation from no-initial reduction.
  // Labels are selector identity, so neither historical positional form is
  // retained as an alias.
  fold(initial initial, using f) {
    let acc = initial
    for (x in self) {
      acc = f.call(acc, x)
    }
    return acc
  }

  reduce(using f) {
    let c = self.iterate(None)
    if (c == None) { return None }

    let acc = self.iteratorValue(c)
    c = self.iterate(c)
    while (c != None) {
      acc = f.call(acc, self.iteratorValue(c))
      c = self.iterate(c)
    }
    return Some(acc)
  }

  group(by block) {
    let result = Map.new()
    for (x in self) {
      let key = block.call(x)
      let list = result.get(key).match(
        some: |list| { list },
        none: || {
          let new_list = List.new()
          result.insert(new_list, for: key)
          new_list
        }
      )
      list.append(x)
    }
    return result
  }

  partition(where predicate) {
    let accepted = List.new()
    let rejected = List.new()
    for (x in self) {
      predicate.call(x).ifTrue(|| { accepted.append(x) }, ifFalse: || { rejected.append(x) })
    }
    return (accepted, rejected)
  }

  toSet {
    let result = Set.new()
    for (x in self) {
      result.add(x)
    }
    return result
  }

  toMap {
    let result = Map.new()
    for (entry in self) {
      let key = entry.key
      if (result.includes(key)) {
        return Err.new(DuplicateKeyError.new(key))
      }
      result.insert(entry.value, for: key)
    }
    return Ok.new(result)
  }

  toMap(merging block) {
    let result = Map.new()
    for (entry in self) {
      let key = entry.key
      let val = entry.value
      result.get(key).match(
        some: |existingVal| {
          let merged = block.call(existingVal, val)
          result.insert(merged, for: key)
        },
        none: || {
          result.insert(val, for: key)
        }
      )
    }
    return result
  }

  toList {
    let result = List.new()
    for (x in self) { result.append(x) }
    return result
  }

  iter { SourceIterator.new(self) }
}

// Stateless lazy pipeline root. Traversal state is carried only in cursors,
// allowing one pipeline instance to be traversed independently and repeatedly.
class Iterator is Iterable {
  iter { self }
  map(_ f) { MapIterator.new(self, f) }
  filter(_ pred) { FilterIterator.new(self, pred) }
  flatMap(_ f) { FlatMapIterator.new(self, f) }
  skip(_ n) { SkipIterator.new(self, n) }
  take(_ n) { TakeIterator.new(self, n) }
  takeWhile(_ pred) { TakeWhileIterator.new(self, pred) }
}

class List {
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
    if (index.isA(Range)) { return self.sliceByRange(index) }
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
    for (x in self) {
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
  // rather than a raw `size`/`at(_)` index walk. `for (x in self)` compiles
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
    if (not range.isA(Range)) {
      return Err.new(SliceError.new("List#replace: first argument must be a Range"))
    }
    if (not replacements.isA(List)) {
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
    if (i.isA(Range)) { return self.replace(i, with: val).unwrap }
    return self.at(i, put: val)
  }

  // U-CORE-5 (decisions.md Q5, R-INV-5.3 E1-E5): structural equality —
  // element-wise, order-sensitive, via each element's own `==`. Guarded by
  // `isA(List)` so a non-List `other` is simply unequal (E2), never a dNU.
  // Derived entirely over the floor (`size`/`at`/`isA`/`while`/`and`/`not`) —
  // no new native primitive (ADR-0019 unchanged). `and`/`not` are the
  // language's infix/prefix operator forms (`Bool#and(_:)`/`Bool#not()`
  // dispatched by the compiler, not dotted-call syntax — `and`/`not` are
  // reserved words and cannot follow `.` as a bare identifier).
  ==(_ other) {
    if (other.isA(List)) {
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

class Map {
  size { self._$size }

  // Display (U-CORE-4, R-INV-4.1; DEFERRED CB-1). Mirrors `Value::to_string`'s
  // native `Map` rendering exactly — `{k: v, k2: v2}`, `{}` when empty — so the
  // `.toString` message and the native render agree. Derived over the floor
  // (`size_`/`keyAt_`/`valueAt_`), not a primitive: ADR-0019's default answer to
  // "add a primitive" is no, and this is expressible.
  //
  // Each key and value renders via its OWN `toString` — which the native path
  // cannot do. That is the point of CB-1: once `\(…)` sends `toString`, this is
  // the path it takes.
  toString {
    let s = "{"
    let i = 0
    while (i < self._$size) {
      s = s + (i > 0).ifTrue(|| { ", " }, ifFalse: || { "" })
      s = s + self._$keyAt(i).toString + ": " + self._$valueAt(i).toString
      i = i + 1
    }
    return s + "}"
  }

  // Safe association lookup: Some(value) on hit, None on absence.
  get(_ k) { self._$get(k) }

  // Strict association lookup. Do one lookup so a stored None remains a
  // value rather than being confused with an absent key.
  [_ k] {
    return self.get(k).match(
      some: |value| { value },
      none: || { KeyError.new("Map key not found").raise() }
    )
  }

  [_ key, default fallback] {
    return self.get(key).match(
      some: |value| { value },
      none: || { fallback }
    )
  }

  get(_ key, orElse block) {
    return self.get(key).match(
      some: |value| { value },
      none: || { block.call(key) }
    )
  }

  get(_ key, orPut block) {
    return self.get(key).match(
      some: |value| { value },
      none: || {
        let value = block.call(key)
        self.insert(value, for: key)
        value
      }
    )
  }

  // Explicit insert returns the previous value when replacing an association.
  insert(_ value, for key) { self._$put(key, value) }

  // Legacy mutation spelling retained only while B.3 still lowers association
  // literals into chained sends. `get` and `[]` are the lookup surface.
  at(_ k, put) {
    self._$put(k, put)
    return self
  }

  // `m[k] = v` shares insert's key identity and encounter-order semantics.
  [_ k]=(put val) { self._$put(k, val) }

  includes(_ k) { self._$has(k) }

  // Removes an association. The raw primitive returns its former value, but
  // the public mutable-collection protocol is chainable.
  remove(_ k) {
    self._$remove(k)
    return self
  }

  clear {
    while (self.size > 0) {
      self._$remove(self._$keyAt(0))
    }
    return ()
  }

  // Lightweight live encounter-order views. They retain the Map and read its
  // current slots; they never copy associations into a List.
  keys { MapKeysView.new(self) }

  values { MapValuesView.new(self) }

  entries { MapEntriesView.new(self) }

  // Copies a positive Record's Symbol labels and values into a fresh mutable
  // Map. `#{}` canonicalizes to Unit, which represents the empty Record form.
  @class
  from(_ record) {
    if ((not record.isA(Record)) and (not record.isA(Unit))) {
      throw ArgumentError.new("Map.from: argument must be a Record or Unit")
    }
    let result = Map.new()
    if (record.isA(Record)) {
      let i = 0
      while (i < record._$size) {
        result.insert(record._$valueAt(i), for: record._$labelAt(i))
        i = i + 1
      }
    }
    return result
  }

  // DEC-CT-E: the cursor value `iteratorValue` yields is the KEY (both Map and Set yield keys).
  // Pair traversal uses `entries.each`, not a receiver-specific callback arity.
  iteratorValue(_ cursor) { self._$keyAt(cursor) }

  // Structural equality: same key set, pairwise-== values (order-independent
  // over keys — `includes`/`get` do the membership + value work, not raw
  // index comparison). Guarded by `isA(Map)` so a non-Map is simply unequal.
  ==(_ other) {
    if (other.isA(Map)) {
      let same = (self.size == other.size)
      let i = 0
      while (same and (i < self.size)) {
        let k = self._$keyAt(i)
        same = other.get(k).match(
          some: |value| { self._$valueAt(i) == value },
          none: || { false }
        )
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  // MUST route through == (the ==/!= decoupling hazard) — Object#!= negates
  // identity, not this structural ==.
  !=(_ other) {
    return not (self == other)
  }
}

class Set {
  size { self._$size }

  // Display (U-CORE-4, R-INV-4.1; DEFERRED CB-1). Mirrors `Value::to_string`'s
  // native `Set` rendering exactly — `Set(a, b)`, `Set()` when empty. Derived
  // over `size_`/`at_`; each element renders via its OWN `toString`.
  toString {
    let s = "Set("
    let i = 0
    while (i < self._$size) {
      s = s + (i > 0).ifTrue(|| { ", " }, ifFalse: || { "" })
      s = s + self._$at(i).toString
      i = i + 1
    }
    return s + ")"
  }

  add(_ v) {
    self._$add(v)
    return self
  }

  includes(_ v) { self._$has(v) }

  remove(_ v) {
    self._$remove(v)
    return self
  }

  // Positional read in insertion order — not in the map-and-set.md selector
  // table, but a direct, zero-floor-cost derivation over at_ that the
  // U-CORE-5 conformance harness (collection-protocol.md §2) needs, and a
  // natural extension of the sequence protocol every collection instantiates.
  at(_ i) { self._$at(i) }



  iteratorValue(_ cursor) { self._$at(cursor) }

  // Structural equality: same members, order-independent. Same-size plus
  // "every element of self is in other" is sufficient since neither set
  // holds duplicates (add_ is idempotent).
  ==(_ other) {
    if (other.isA(Set)) {
      let same = (self.size == other.size)
      let i = 0
      while (same and (i < self.size)) {
        same = other.includes(self._$at(i))
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  !=(_ other) {
    return not (self == other)
  }
}

class Unit {
  toString { "()" }
  hash { 0 }
}

// Kernel Tuple (ADR-0032 §1, ADR-0039, U-COLLTYPES Phase 2): a native fixed
// arity immutable slice — Object::Tuple, mirroring List's shape but with NO
// mutation selector (immutability is structural, TupleObject's Box<[Value]>).
// Product literals compile directly to native build bytecodes.

class Tuple {
  size { self._$size }
  positionals { self._$positionals }
  labeled { self._$labeled }
  labelAt(_ index) { self._$labelAt(index) }

  first {
    if (self.size == 0) { return None }
    return Some(self.at(0))
  }

  last {
    if (self.size == 0) { return None }
    return Some(self.at(self.size - 1))
  }

  // Display (U-CORE-4, R-INV-4.1; DEFERRED CB-1). Mirrors `Value::to_string`'s
  // native `Tuple` rendering exactly — `(a, b)`, `()` when empty. Derived over
  // `size_`/`at_`; each element renders via its OWN `toString`.
  toString {
    let s = "("
    let i = 0
    while (i < self._$size) {
      s = s + (i > 0).ifTrue(|| { ", " }, ifFalse: || { "" })
      s = s + self._$at(i).toString
      i = i + 1
    }
    return s + ")"
  }

  at(_ i) { self._$at(i) }

  @private
  findLabel(_ sym) {
    let num_labeled = self.size - self._$positionalSize
    let i = 0
    while (i < num_labeled) {
      if (self._$labelAt(i) == sym) {
        return Some(self._$positionalSize + i)
      }
      i = i + 1
    }
    return None
  }

  @private
  access(_ key) {
    if (key.isA(Symbol)) {
      return self.findLabel(key).match(
        some: |idx| { Some(self._$at(idx)) },
        none: || { None }
      )
    }
    let raw = self._$at(key)
    let len = self.size
    let i = key
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return Some(raw)
    }
    return None
  }

  get(_ key) { self.access(key) }

  [_ key] {
    if (key.isA(Range)) {
      return key._$sliceBounds(self.size).match(
        ok: |bounds| {
          let start = bounds[0]
          let end = bounds[1]
          if (start > end) { end = start }
          self._$slice(start, end)
        },
        err: |error| { error.raise() }
      )
    }
    if (key.isA(Symbol)) {
      return self.findLabel(key).match(
        some: |idx| { self._$at(idx) },
        none: || { throw KeyError.new("Tuple label not found") }
      )
    }
    let raw = self._$at(key)
    let len = self.size
    let i = key
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return raw
    }
    throw IndexError.new("Tuple index out of range")
  }

  [_ key, default] {
    if (key.isA(Symbol)) {
      return self.findLabel(key).match(
        some: |idx| { self._$at(idx) },
        none: || { default }
      )
    }
    let raw = self._$at(key)
    let len = self.size
    let i = key
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return raw
    }
    return default
  }

  get(_ key, orElse) {
    if (key.isA(Symbol)) {
      return self.findLabel(key).match(
        some: |idx| { self._$at(idx) },
        none: || { orElse.call(key) }
      )
    }
    let raw = self._$at(key)
    let len = self.size
    let i = key
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return raw
    }
    return orElse.call(key)
  }

  iteratorValue(_ cursor) { self._$at(cursor) }

  // Structural equality: same arity, pairwise-==. Guarded by isA(Tuple) so a
  // non-Tuple (including a same-elements List — cross-kind, E2) is unequal.
  ==(_ other) {
    if (other.isA(Tuple)) {
      let same = (self.size == other.size) and (self._$positionalSize == other._$positionalSize)
      let i = 0
      while (same and (i < self.size - self._$positionalSize)) {
        same = (self._$labelAt(i) == other._$labelAt(i))
        i = i + 1
      }
      i = 0
      while (same and (i < self.size)) {
        same = (self.at(i) == other.at(i))
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  !=(_ other) {
    return not (self == other)
  }

  // Value hash (DEC-CT-D): a .ph fold over each element's own `hash` —
  // order-sensitive, zero new floor. Consistent with == by construction (a
  // deterministic function of the same elements == compares), and survives
  // the future Int/Float split for free (forward-compat §4): it folds
  // *mathematical-value* hashes (whatever Number#hash decides), never bits.
  // Bounded by a large prime modulus so the accumulator stays a stable,
  // comparable Number regardless of tuple length.
  hash {
    let acc = 17 + self._$positionalSize
    let i = 0
    while (i < self.size) {
      acc = (acc * 31 + self.at(i).hash) % 999999937
      i = i + 1
    }
    i = 0
    while (i < self.size - self._$positionalSize) {
      acc = (acc * 31 + self._$labelAt(i).hash) % 999999937
      i = i + 1
    }
    return acc
  }
}

class Record {
  size { self._$size }
  labelAt(_ index) { self._$labelAt(index) }

  ==(_ other) {
    if (other.isA(Record)) {
      let same = (self.size == other.size)
      let i = 0
      while (same and (i < self.size)) {
        let label = self._$labelAt(i)
        let j = 0
        let found = false
        while ((not found) and (j < other.size)) {
          if (other._$labelAt(j) == label) {
            found = (self._$valueAt(i) == other._$valueAt(j))
          }
          j = j + 1
        }
        same = found
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  hash {
    let acc = 101 + self.size
    let i = 0
    while (i < self.size) {
      acc = (acc + ((self._$labelAt(i).hash * 31) + self._$valueAt(i).hash)) % 999999937
      i = i + 1
    }
    return acc
  }
}

// Map projections are ordinary retained-source Iterable views. They inherit
// the generic cursor walk and deliberately leave active-iteration mutation
// behavior unspecified.
class MapKeysView is Iterable {
  @constructor
  new(_ map) { _map = map }

  size { _map.size }

  iteratorValue(_ cursor) { _map._$keyAt(cursor) }
}

class MapValuesView is Iterable {
  @constructor
  new(_ map) { _map = map }

  size { _map.size }

  iteratorValue(_ cursor) { _map._$valueAt(cursor) }
}

// Immutable-by-surface association value for MapEntriesView. No setters or
// value-object equality/hash protocol are part of this phase.
class Entry {
  @constructor
  new(_ key, _ value) {
    _key = key
    _value = value
  }

  key { _key }

  value { _value }
}

class MapEntriesView is Iterable {
  @constructor
  new(_ map) { _map = map }

  size { _map.size }

  iteratorValue(_ cursor) { Entry.new(_map._$keyAt(cursor), _map._$valueAt(cursor)) }
}

// Range is a native bounds descriptor. Its lower_/upper_/upperInclusive_
// observations preserve omitted endpoints. Progression, equality, hashing,
// and traversal are deliberately deferred.
class Range is Iterable {
  @class
  new(_ lower, _ upper, _ upperInclusive) {
    if (lower == None) {
      if (upper == None) { return .. }
      return upperInclusive.ifTrue(|| { ..=upper }, ifFalse: || { ..upper })
    }
    if (upper == None) { return lower.. }
    return upperInclusive.ifTrue(|| { lower..=upper }, ifFalse: || { lower..upper })
  }

  @private
  isSliceCoordinate(_ value) {
    // TODO(NUMERIC-TOWER): require Int once the tower is fully landed.
    return value.isA(Number) and ((value % 1) == 0)
  }

  @private
  sliceBoundary(_ coordinate, _ size) {
    if (coordinate < 0) {
      if (coordinate < -size) { return 0 }
      return size + coordinate
    }
    if (coordinate > size) { return size }
    return coordinate
  }

  @private
  sliceInclusiveEnd(_ coordinate, _ size) {
    if (coordinate < 0) {
      if (coordinate < -size) { return 0 }
      // Here -size <= coordinate < 0, so adding one cannot overflow and
      // denotes the exclusive boundary after the included element.
      return size + coordinate + 1
    }
    if (coordinate >= size) { return size }
    return coordinate + 1
  }

  // Normalizes this bound descriptor for a finite sequence of `size` elements.
  // Omitted endpoints are distinct from a supplied None, which is malformed.
  _$sliceBounds(_ size) {
    let start = 0
    let end = size
    let lower = self._$lower
    if (lower.isSome) {
      let coordinate = lower.unwrapOr(None)
      if (not self.isSliceCoordinate(coordinate)) {
        return Err.new(SliceError.new("Range lower bound must be an integer coordinate"))
      }
      start = self.sliceBoundary(coordinate, size)
    }
    let upper = self._$upper
    if (upper.isSome) {
      let coordinate = upper.unwrapOr(None)
      if (not self.isSliceCoordinate(coordinate)) {
        return Err.new(SliceError.new("Range upper bound must be an integer coordinate"))
      }
      if (self._$upperInclusive) {
        end = self.sliceInclusiveEnd(coordinate, size)
      } else {
        end = self.sliceBoundary(coordinate, size)
      }
    }
    return Ok.new((start, end))
  }

  // iterate and iteratorValue for forward integer iteration (E.2).
  // The cursor is the current yielded integer value, not an offset from lower.
  iterate(_ previous) {
    let lowerOpt = self._$lower
    let upperOpt = self._$upper

    // lower is required for iteration
    lowerOpt.isNone.ifTrue || {
      throw ArgumentError.new("Range iteration unsupported when lower bound is absent")
    }
    let lower = lowerOpt.unwrapOr(None)
    (self.isSliceCoordinate(lower)).ifFalse || {
      throw ArgumentError.new("Range iteration unsupported: lower bound must be an integer")
    }

    let hasUpper = upperOpt.isSome
    let upper = hasUpper.ifTrue(|| { upperOpt.unwrapOr(None) }, ifFalse: || { None })
    hasUpper.ifTrue || {
      (self.isSliceCoordinate(upper)).ifFalse || {
        throw ArgumentError.new("Range iteration unsupported: upper bound must be an integer")
      }
      (lower > upper).ifTrue || {
        throw ArgumentError.new("Range iteration unsupported: lower bound exceeds upper (descending traversal not supported)")
      }
    }

    let candidate = (previous == None).ifTrue(|| { lower }, ifFalse: || { previous + 1 })

    hasUpper.ifFalse || {
      return candidate
    }

    let inclusive = self._$upperInclusive
    let live = inclusive.ifTrue(|| { candidate <= upper }, ifFalse: || { candidate < upper })
    return live.ifTrue(|| { candidate }, ifFalse: || { None })
  }

  iteratorValue(_ cursor) { cursor }

  // first, last, size, includes (Spec E.2 / Range specs)
  first {
    let lowerOpt = self._$lower
    lowerOpt.isNone.ifTrue || {
      throw Error.new("Range has no first element because lower bound is absent")
    }
    return lowerOpt.unwrapOr(None)
  }

  last {
    let upperOpt = self._$upper
    upperOpt.isNone.ifTrue || {
      throw Error.new("Range has no last element because upper bound is absent")
    }
    let upper = upperOpt.unwrapOr(None)
    return self._$upperInclusive.ifTrue(|| { upper }, ifFalse: || { upper - 1 })
  }

  size {
    let lowerOpt = self._$lower
    let upperOpt = self._$upper
    (lowerOpt.isNone or upperOpt.isNone).ifTrue || {
      throw Error.new("Unbounded Range has no size")
    }
    let lower = lowerOpt.unwrapOr(None)
    let upper = upperOpt.unwrapOr(None)
    (lower > upper).ifTrue || {
      return 0
    }
    let diff = upper - lower
    return self._$upperInclusive.ifTrue(|| { diff + 1 }, ifFalse: || { diff })
  }

  includes(_ x) {
    let lowerOpt = self._$lower
    let upperOpt = self._$upper

    let lowerOk = lowerOpt.isNone.ifTrue(|| { true }, ifFalse: || { x >= lowerOpt.unwrapOr(None) })
    let upperOk = upperOpt.isNone.ifTrue(|| { true }, ifFalse: || {
      self._$upperInclusive.ifTrue(|| { x <= upperOpt.unwrapOr(None) }, ifFalse: || { x < upperOpt.unwrapOr(None) })
    })
    return lowerOk and upperOk
  }

  at(_ i) {
    let lowerOpt = self._$lower
    lowerOpt.isNone.ifTrue || {
      throw Error.new("Range index out of range (lower bound is absent)")
    }
    let lower = lowerOpt.unwrapOr(None)
    let index = i
    // Support negative indexing
    (index < 0).ifTrue || {
      let sz = self.size
      index = sz + index
    }
    // Check range bounds
    let sz = self.size
    (index < 0 or index >= sz).ifTrue || {
      throw Error.new("Range index out of range")
    }
    return lower + index
  }

  ==(_ other) {
    other.isA(Range).ifFalse || { return false }
    return (self._$lower == other._$lower) and
           (self._$upper == other._$upper) and
           (self._$upperInclusive == other._$upperInclusive)
  }

  toString {
    let lowerOpt = self._$lower
    let upperOpt = self._$upper
    let lowerStr = lowerOpt.isNone.ifTrue(|| { "" }, ifFalse: || { lowerOpt.unwrapOr(None).toString })
    let upperStr = upperOpt.isNone.ifTrue(|| { "" }, ifFalse: || { upperOpt.unwrapOr(None).toString })
    let op = self._$upperInclusive.ifTrue(|| { "..=" }, ifFalse: || { ".." })
    return lowerStr + op + upperStr
  }

  hash {
    let h1 = self._$lower.isSome.ifTrue(|| { self._$lower.unwrapOr(None).hash }, ifFalse: || { 17 })
    let h2 = self._$upper.isSome.ifTrue(|| { self._$upper.unwrapOr(None).hash }, ifFalse: || { 31 })
    let h3 = self._$upperInclusive.ifTrue(|| { 1 }, ifFalse: || { 0 })
    return h1 + h2 * 37 + h3 * 97
  }
}

// U-BYTES (PDR-0011, docs/spec/v0.2/core/bytes.md): the kernel octet buffer's
// `.ph` protocol over the eleven floor primitives. Bulk no-user-code work is
// native (bytes.md §3.1); everything here is validation-lifting, derivation,
// and the Iterable hookup. `each`/`map`/`filter`/`reduce` are deliberately
// ABSENT — inherited from `Iterable` so `Fiber.yield` works mid-iteration
// (law 8); adding native or local overrides is a spec violation.
class Bytes {
  size { self._$size }

  first {
    if (self.size == 0) { return None }
    return Some(self.at(0))
  }

  last {
    if (self.size == 0) { return None }
    return Some(self.at(self.size - 1))
  }

  at(_ i) { self._$at(i) }

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

  [_ index] {
    if (index.isA(Range)) {
      return index._$sliceBounds(self.size).match(
        ok: |bounds| {
          let start = bounds[0]
          let end = bounds[1]
          if (start > end) { end = start }
          self._$slice(start, end)
        },
        err: |error| { error.raise() }
      )
    }
    let raw = self._$at(index)
    let len = self.size
    let i = index
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return raw
    }
    throw IndexError.new("Bytes index out of range")
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

  iteratorValue(_ cursor) { self._$at(cursor) }

  // An octet is an integer Number in 0..255 (bytes.md §2). `and` is lazy,
  // so the arithmetic tests never run on a non-Number. (No trailing `_`:
  // that marker is reserved for native primitives, and this is pure .ph.)
  isOctet(_ v) {
    return v.isA(Number) and (v >= 0) and (v <= 255) and ((v % 1) == 0)
  }

  // Raise-lifting writes (bytes.md law 1: precondition violations raise,
  // reads stay total). The floor's set_ reports a bad write as a native
  // type error; the .ph surface names the contract instead.
  set(_ i, _ v) {
    if (not self.isOctet(v)) {
      throw ArgumentError.new("Bytes#set: value must be an integer in 0..255")
    }
    let len = self.size
    let norm = i
    if (norm < 0) { norm = len + norm }
    if ((norm < 0) or (norm >= len)) {
      throw IndexError.new("Bytes index out of range")
    }
    self._$set(i, v)
    return self
  }

  at(_ i, put) { return self.set(i, put) }

  [_ i]=(put val) { return self.set(i, val) }

  fill(_ v) {
    if (not self.isOctet(v)) {
      throw ArgumentError.new("Bytes#fill: value must be an integer in 0..255")
    }
    self._$fill(v)
    return self
  }

  // One native memset, complete because the length is fixed (bytes.md §7).
  // The guarantee is a documented obligation, not a mechanism — scope it
  // with `ensure`. A getter so the call reads `key.zeroize` (ADR-0012:
  // `zeroize` and `zeroize()` are different selectors).
  zeroize {
    self._$fill(0)
    return self
  }

  utf8 { self._$utf8 }

  // Display decode: total, lossy (invalid sequences become U+FFFD). Never
  // round-trip the result into data (PDR-0013 ruling 4).
  utf8Lossy { self._$utf8Lossy }

  slice(_ start, _ end) {
    if ((start < 0) or (end < start) or (end > self.size)) {
      throw ArgumentError.new("Bytes#slice: range must satisfy 0 <= start <= end <= size")
    }
    return self._$slice(start, end)
  }

  copyInto(_ dst, _ offset) {
    if (not dst.isA(Bytes)) {
      throw ArgumentError.new("Bytes#copyInto: destination must be a Bytes")
    }
    if ((offset < 0) or ((offset + self.size) > dst.size)) {
      throw ArgumentError.new("Bytes#copyInto: offset + size must fit the destination")
    }
    self._$copyInto(dst, offset)
    return self
  }

  // Derivability with teeth (bytes.md §3.1): new + two native memmoves,
  // zero per-byte loops.
  concat(_ other) {
    if (not other.isA(Bytes)) {
      throw ArgumentError.new("Bytes#concat: argument must be a Bytes")
    }
    const out = Bytes.new(self.size + other.size)
    self._$copyInto(out, 0)
    other._$copyInto(out, self.size)
    return out
  }

  equalsConstantTime(_ other) { self._$equalsConstantTime(other) }

  // Structural equality, List#=='s exact shape (collection-protocol §4).
  // Short-circuits — correct here, and exactly why it must never be the
  // secret-comparison spelling (bytes.md §8).
  ==(_ other) {
    if (other.isA(Bytes)) {
      let same = (self.size == other.size)
      let i = 0
      // `and` is lazy: once `same` is false the loop exits without another
      // `at(i)` (List#=='s exact shape).
      while (same and (i < self.size)) {
        same = (self.at(i) == other.at(i))
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  // MUST route through == (the ==/!= decoupling hazard) — Object#!= negates
  // identity, not this structural ==.
  !=(_ other) {
    return not (self == other)
  }

  toString { "Bytes(" + self.size.toString + ")" }

  toList {
    const out = []
    for (b in self) {
      out.append(b)
    }
    return out
  }

  // The immutable, value-hashable snapshot — the Map-key escape hatch
  // (PDR-0011 ruling 4; Bytes itself is mutable => identity hash, never a
  // valid Map/Set key).
  toTuple { Tuple._$fromList(self.toList) }

  @class
  fromString(_ s) {
    if (not s.isA(String)) {
      throw ArgumentError.new("Bytes.fromString: argument must be a String")
    }
    return Bytes._$fromString(s)
  }

  // The builder story (bytes.md law 3 forecloses growth): build in a List,
  // freeze into Bytes — Tuple.fromList's shape. `set` (not `set_`) so a
  // non-octet element raises the named contract error.
  @class
  fromList(_ list) {
    if (not list.isA(List)) {
      throw ArgumentError.new("Bytes.fromList: argument must be a List")
    }
    const out = Bytes.new(list.size)
    let i = 0
    while (i < list.size) {
      out.set(i, list.at(i))
      i = i + 1
    }
    return out
  }
}

class OpenMode {
  @constructor
  @private
  named(_ n) { _name = n }
  @class
  read { OpenMode.named("read") }
  @class
  write { OpenMode.named("write") }
  @class
  append { OpenMode.named("append") }
  @class
  readWrite { OpenMode.named("readWrite") }
  name { _name }
  ==(_ other) { return other.isA(OpenMode) and (_name == other.name) }
  !=(_ other) { return not (self == other) }
  toString { "OpenMode." + _name }
}

class Path {
  @constructor
  of(_ s) {
    if (not s.isA(String)) {
      throw ArgumentError.new("Path.of: argument must be a String")
    }
    _bytes = Bytes.fromString(s)
    _hash = Path.contentHash(_bytes)
  }

  @constructor
  ofBytes(_ b) {
    if (not b.isA(Bytes)) {
      throw ArgumentError.new("Path.ofBytes: argument must be a Bytes")
    }
    _bytes = b.slice(0, b.size)
    _hash = Path.contentHash(_bytes)
  }

  @class
  contentHash(_ bytes) {
    let acc = 1
    let i = 0
    while (i < bytes.size) {
      acc = (acc * 31 + bytes.at(i)) % 999999937
      i = i + 1
    }
    return acc
  }

  bytes { _bytes.slice(0, _bytes.size) }
  hash { _hash }

  ==(_ other) {
    if (not other.isA(Path)) { return false }
    if (_hash != other.hash) { return false }
    return _bytes == other.bytes
  }

  !=(_ other) { return not (self == other) }

  isAbsolute { (_bytes.size > 0) and (_bytes.at(0) == 47) }

  join(_ other) {
    if (not other.isA(Path)) {
      throw ArgumentError.new("Path#join: argument must be a Path")
    }
    if (other.isAbsolute) {
      return other
    }
    let recv = _bytes
    let recvLen = recv.size
    while ((recvLen > 0) and (recv.at(recvLen - 1) == 47)) {
      recvLen = recvLen - 1
    }
    const trimmedRecv = recv.slice(0, recvLen)
    const sep = Bytes.fromString("/")
    const combined = trimmedRecv.concat(sep).concat(other.bytes)
    return Path.ofBytes(combined)
  }

  parent {
    let len = _bytes.size
    while ((len > 0) and (_bytes.at(len - 1) == 47)) {
      len = len - 1
    }
    let idx = len - 1
    while ((idx >= 0) and (_bytes.at(idx) != 47)) {
      idx = idx - 1
    }
    if (idx < 0) {
      return None
    }
    if (idx == 0) {
      return Path.of("/")
    }
    let pLen = idx
    while ((pLen > 0) and (_bytes.at(pLen - 1) == 47)) {
      pLen = pLen - 1
    }
    if (pLen == 0) {
      return Path.of("/")
    }
    return Path.ofBytes(_bytes.slice(0, pLen))
  }

  fileName {
    let len = _bytes.size
    if ((len > 0) and (_bytes.at(len - 1) == 47)) {
      return None
    }
    let idx = len - 1
    while ((idx >= 0) and (_bytes.at(idx) != 47)) {
      idx = idx - 1
    }
    if ((idx < 0) and (len == 0)) {
      return None
    }
    const nameBytes = _bytes.slice(idx + 1, len)
    if (nameBytes.size == 0) {
      return None
    }
    return Path.ofBytes(nameBytes)
  }

  extension {
    const namePath = self.fileName
    if (namePath == None) {
      return None
    }
    const nb = namePath.bytes
    let idx = nb.size - 1
    while ((idx >= 0) and (nb.at(idx) != 46)) {
      idx = idx - 1
    }
    if ((idx <= 0) or (idx == nb.size - 1)) {
      return None
    }
    const extBytes = nb.slice(idx + 1, nb.size)
    return extBytes.utf8
  }

  components {
    let res = List.new()
    let i = 0
    let len = _bytes.size
    while (i < len) {
      while ((i < len) and (_bytes.at(i) == 47)) {
        i = i + 1
      }
      if (i < len) {
        let start = i
        while ((i < len) and (_bytes.at(i) != 47)) {
          i = i + 1
        }
        res.append(Path.ofBytes(_bytes.slice(start, i)))
      }
    }
    return res
  }

  toString { _bytes.utf8Lossy }
}


// Explicit lazy iterator stages. Each is an ordinary `.ph` wrapper over the
// cursor protocol; no stage stores traversal state on its instance.
class SourceIterator is Iterator {
  @constructor
  new(_ source) {
    _source = source
  }
  iterate(_ cursor) { _source.iterate(cursor) }
  iteratorValue(_ cursor) { _source.iteratorValue(cursor) }
}

class MapIterator is Iterator {
  @constructor
  new(_ source, _ f) {
    _source = source
    _f = f
  }
  iterate(_ cursor) { _source.iterate(cursor) }
  iteratorValue(_ cursor) { _f.call(_source.iteratorValue(cursor)) }
}

class FilterIterator is Iterator {
  @constructor
  new(_ source, _ pred) {
    _source = source
    _pred = pred
  }
  iterate(_ cursor) {
    let cur = _source.iterate(cursor)
    while (cur != None) {
      if (_pred.call(_source.iteratorValue(cur))) { return cur }
      cur = _source.iterate(cur)
    }
    return None
  }
  iteratorValue(_ cursor) { _source.iteratorValue(cursor) }
}

class SkipIterator is Iterator {
  @constructor
  new(_ source, _ n) {
    (n.isA(Number) and n >= 0 and n % 1 == 0).ifFalse || {
      throw ArgumentError.new("skip: n must be a non-negative integer")
    }
    _source = source
    _n = n
  }
  iterate(_ cursor) {
    if (cursor != None) { return _source.iterate(cursor) }
    let cur = _source.iterate(None)
    let rem = _n
    while (cur != None and rem > 0) {
      cur = _source.iterate(cur)
      rem = rem - 1
    }
    return cur
  }
  iteratorValue(_ cursor) { _source.iteratorValue(cursor) }
}

class TakeIterator is Iterator {
  @constructor
  new(_ source, _ n) {
    (n.isA(Number) and n >= 0 and n % 1 == 0).ifFalse || {
      throw ArgumentError.new("take: n must be a non-negative integer")
    }
    _source = source
    _n = n
  }
  iterate(_ cursor) {
    if (_n == 0) { return None }
    if (cursor == None) {
      let up = _source.iterate(None)
      if (up == None) { return None }
      return (up, 1)
    }
    let up = cursor.at(0)
    let yielded = cursor.at(1)
    if (yielded >= _n) { return None }
    let next = _source.iterate(up)
    if (next == None) { return None }
    return (next, yielded + 1)
  }
  iteratorValue(_ cursor) { _source.iteratorValue(cursor.at(0)) }
}

class TakeWhileIterator is Iterator {
  @constructor
  new(_ source, _ pred) { _source = source; _pred = pred }
  iterate(_ cursor) {
    let cand = _source.iterate(cursor)
    if (cand == None) { return None }
    if (_pred.call(_source.iteratorValue(cand))) { return cand }
    return None
  }
  iteratorValue(_ cursor) { _source.iteratorValue(cursor) }
}

class FlatMapIterator is Iterator {
  @constructor
  new(_ source, _ f) { _source = source; _f = f }

  @private
  seekFromOuter(_ outerCursor) {
    let oc = outerCursor
    while (oc != None) {
      let inner = _f.call(_source.iteratorValue(oc))
      let ic = inner.iterate(None)
      if (ic != None) { return (oc, inner, ic) }
      oc = _source.iterate(oc)
    }
    return None
  }

  iterate(_ cursor) {
    if (cursor == None) { return self.seekFromOuter(_source.iterate(None)) }
    let outer = cursor.at(0)
    let inner = cursor.at(1)
    let ic = cursor.at(2)
    let nextIc = inner.iterate(ic)
    if (nextIc != None) { return (outer, inner, nextIc) }
    return self.seekFromOuter(_source.iterate(outer))
  }
  iteratorValue(_ cursor) { cursor.at(1).iteratorValue(cursor.at(2)) }
}

class System {
  // U-STRING write funnel (ADR-0049 amendment): pure `.ph` control flow over
  // native `write_(_)` and the `toString` message. Additive-only: does not
  // touch the native `print(_)` pathway (pre-existing divergence between
  // `Value::to_string` and the `.toString` message is out of scope).
  @class
  write(_ obj) {
    System.writeObject(obj)
    return obj
  }

  @private
  @class
  writeObject(_ obj) {
    const s = obj.toString
    (s.isA(String)).ifTrue(|| {
      System._$write(s)
    }, ifFalse: || {
      System._$write("invalid toString")
    })
    return obj
  }

  // U-SCHED: the `.ph`-callable counterpart to `VM::run`'s native
  // root-drive belt-and-suspenders pump (`vm/dispatch.rs`) — pumps
  // `System.nextScheduled` to exhaustion, `try()`-resuming each queued
  // fiber (capture-not-propagate, so one scheduled task's uncaught raise
  // cannot abort another) — including any fiber a running scheduled fiber
  // itself schedules mid-drain, since `nextScheduled` is re-read every
  // iteration. Deliberately does **not** unwrap via `.match(some:none:)`
  // (which runs its arm through `Closure#call`'s native re-entrant
  // `run_until`, forbidding a fiber switch underneath, ADR-0030 §4): `f.try()`
  // must run at this method's own top level, not nested inside a block a
  // native primitive is driving, so the receiver is unwrapped via
  // `unwrapOr(_)` into a plain local first, and `try()` sent as its own
  // statement.
  @class
  runScheduled() {
    let next = System.nextScheduled
    while (next.isSome) {
      let f = next.unwrapOr(None)
      f.try()
      next = System.nextScheduled
    }
  }
}

// `Future` (concurrency.md §2; ADR-0030 §1): a settle-once state machine over
// a fulfilled/rejected result. A **plain `InstanceObject`** (concurrency.md §2
// "Implementation" ¶1) — zero new floor, written entirely in Phalcom over the
// same two public seams user code has, `System.schedule(_)` and `Fiber.yield`.
//
// **Both slices are landed.** Slice A is the scheduler-free half:
// `value(_)`/`error(_)` construct an already-settled future, `isReady`/`value`
// read it, and `then`/`map`/`catch` fire synchronously on an already-settled
// receiver. Slice B added `async(_)`, `await`, and the pending→settle `drain`
// over the native ready queue (`06432bd`, 2026-07-14). Note that this comment
// previously still described Slice B as "deliberately NOT built" long after it
// shipped, eleven lines above its own implementation — see
// `docs/learn/concurrency/future-await.md`.
//
// State lives in three private fields (plan §6.1): `_state` (one of the
// strings `"pending"`, `"fulfilled"`, `"rejected"`), `_value` (the settled
// value or the captured `Error`), and `_waiters` — a `List` holding two kinds
// of thing, `Fiber`s registered by `await` and `Closure`s registered by
// `then`/`map`/`catch`, unified by `System.schedule(_)` accepting both.
class Future {
  // Builds a pending future (U-FUTURE Slice B).
  @constructor
  new() {
    _state = "pending"
    _value = None
    _waiters = List.new()
  }

  // Builds an already-`fulfilled` future wrapping `v` (concurrency.md §2
  // `@constructor value(_)`). Goes through the pending→`settleValue` path
  // rather than setting `_state`/`_value` directly so construction and
  // post-construction settlement share one settle-once code path.
  @constructor
  value(_ v) {
    _state = "pending"
    _value = None
    _waiters = List.new()
    self.settleValue(v)
  }

  // Builds an already-`rejected` future wrapping `e` (concurrency.md §2
  // `@constructor error(_)`); see `value(_)` for why this routes
  // through `settleError` instead of assigning state directly.
  @constructor
  error(_ e) {
    _state = "pending"
    _value = None
    _waiters = List.new()
    self.settleError(e)
  }

  // `true` once `self` has settled (`fulfilled` or `rejected`); `false`
  // while `pending`.
  isReady { _state != "pending" }

  // Settles `self` as `fulfilled` with `v`, unless already settled (settle-
  // once, C-FUT-3): a `self.isReady` receiver is a no-op that returns `self`
  // unchanged, so a second `settleValue`/`settleError` can never clobber the
  // first result. Returns `self` either way so callers can chain.
  settleValue(_ v) {
    if (self.isReady) {
      return self
    } else {
      _state = "fulfilled"
      _value = v
      self.drain()
      return self
    }
  }

  // Settles `self` as `rejected` with `e` (an `Error`), unless already
  // settled — the rejection sibling of `settleValue(_)`; see it for
  // the settle-once contract (C-FUT-3).
  settleError(_ e) {
    if (self.isReady) {
      return self
    } else {
      _state = "rejected"
      _value = e
      self.drain()
      return self
    }
  }

  // Reschedules all waiters once settled.
  //
  // A waiter is either a `Fiber` (registered by `await`) or a `Closure`
  // (registered by `then`/`map`/`catch`); `System.schedule(_)` accepts both,
  // enqueueing a fiber as-is and wrapping anything else. A fiber waiter can,
  // however, be *finished* by the time we settle — it may have failed after
  // registering (E004(c): a caller that `await`s from under a native frame
  // raises out of `await` with its registration still in the list). Resuming a
  // finished fiber aborts the whole run, taking every other waiter on this
  // future down with it, so skip those rather than scheduling a corpse.
  drain() {
    _waiters.each |w| {
      const dead = w.isA(Fiber) and w.isDone
      if (not dead) {
        System.schedule(w)
      }
    }
    _waiters = List.new()
  }

  // The settled value as an `Option` (concurrency.md §2): `Some(v)` once
  // `fulfilled`, `None` while `pending` or once `rejected` (the rejection
  // reason is reached via `catch(_)`/`then(_)`, not `value`).
  value { (_state == "fulfilled").ifTrue(|| { Some(_value) }, ifFalse: || { None }) }

  // Suspends the current fiber until settled (U-FUTURE Slice B). On the root
  // fiber — which has no resumer and so cannot yield — degrades to driving the
  // scheduler here instead.
  //
  // The branch is chosen by **asking** (`Fiber#isRoot`), not by attempting a
  // yield and inspecting the failure. It used to do the latter, via
  // `{ Fiber.yield(None) }.attempt()`, and that could never work: `.attempt()`
  // is two nested native re-entrant frames (`block_on` + `block_call`), so the
  // probe tripped the restricted-yield guard (ADR-0030 §4) it was probing for,
  // unconditionally, for every fiber. `await` therefore never suspended anyone
  // — E004. Attempt-and-inspect cannot work when the attempt changes the answer.
  //
  // Consequently the `Fiber.yield` below is **bare**. Wrapping it in anything
  // that reaches `Closure#call` — `.attempt()`, `.on(_)`, `ensure` — puts a native
  // frame between the fiber floor and the switch and reinstates the bug. A
  // `CannotYieldAcrossNativeFrame` raised from here is now a real one: it means
  // the *caller* invoked `await` from inside a block a native primitive is
  // driving, which is genuinely unsupported and correctly propagates.
  await {
    if (not self.isReady) {
      if (Fiber.current.isRoot) {
        // Pump until someone settles us. If the ready queue drains while we are
        // still pending, nothing can settle us and looping again would spin
        // forever in silence (E004(b)) — report it instead. `try()`-resume, so
        // one scheduled task's uncaught raise cannot abort the others, and as
        // its own statement rather than inside a block, for the same reason
        // `System.runScheduled` is written that way.
        while (not self.isReady) {
          const next = System.nextScheduled
          if (next.isNone) {
            return Error.new("await: the future is still pending and the scheduler is empty; nothing can settle it").raise()
          }
          const f = next.unwrapOr(None)
          f.try()
        }
      } else {
        _waiters._$push(Fiber.current)
        Fiber.yield(None)
      }
    }
    if (_state == "rejected") {
      return _value.raise()
    }
    return _value
  }

  // Runs `action` on a fresh fiber and settles the returned future with its
  // result (or captured error if it fails).
  @class
  async(_ action) {
    const f = Future.new()
    const driver = Fiber.new || {
      const fib = Fiber.new(action)
      const res = fib.try()
      if (fib.error.isSome) {
        f.settleError(fib.error.unwrapOr(None))
      } else {
        f.settleValue(res)
      }
    }
    System.schedule(driver)
    return f
  }

  // Normalizes a continuation result into a single Future layer. A callback
  // returning a Future is adopted; a plain value becomes an already-fulfilled
  // Future. This is the Future assimilation rule used by then/map/catch.
  @class
  flatten(_ value) {
    if (value.isA(Future)) {
      return value
    }
    return Future.value(value)
  }

  // Registers a continuation on the settled/fulfilled path (concurrency.md
  // §2 `then(_)`). If pending, registers a continuation that will settle
  // the returned future when this receiver settles.
  then(_ f) {
    if (self.isReady) {
      if (_state == "fulfilled") {
        return Future.flatten(f.call(_value))
      } else {
        return self
      }
    } else {
      const f_next = Future.new()
      _waiters._$push(|| {
        if (_state == "fulfilled") {
          const fib = Fiber.new(|| { f.call(_value) })
          const res = fib.try()
          if (fib.error.isSome) {
            f_next.settleError(fib.error.unwrapOr(None))
          } else {
            const flattened = Future.flatten(res)
            flattened.then |value| { f_next.settleValue(value) }
            flattened.catch |error| { f_next.settleError(error) }
          }
        } else {
          f_next.settleError(_value)
        }
      })
      return f_next
    }
  }

  // `then(_)` restricted to the fulfilled path (concurrency.md §2 `map(_)`).
  map(_ f) {
    if (self.isReady) {
      if (_state == "fulfilled") {
        return Future.flatten(f.call(_value))
      } else {
        return self
      }
    } else {
      const f_next = Future.new()
      _waiters._$push(|| {
        if (_state == "fulfilled") {
          const fib = Fiber.new(|| { f.call(_value) })
          const res = fib.try()
          if (fib.error.isSome) {
            f_next.settleError(fib.error.unwrapOr(None))
          } else {
            const flattened = Future.flatten(res)
            flattened.then |value| { f_next.settleValue(value) }
            flattened.catch |error| { f_next.settleError(error) }
          }
        } else {
          f_next.settleError(_value)
        }
      })
      return f_next
    }
  }

  // Registers an error handler on the rejected path (concurrency.md §2
  // `catch(_)`).
  catch(_ f) {
    if (self.isReady) {
      if (_state == "rejected") {
        return Future.flatten(f.call(_value))
      } else {
        return self
      }
    } else {
      const f_next = Future.new()
      _waiters._$push(|| {
        if (_state == "rejected") {
          const fib = Fiber.new(|| { f.call(_value) })
          const res = fib.try()
          if (fib.error.isSome) {
            f_next.settleError(fib.error.unwrapOr(None))
          } else {
            const flattened = Future.flatten(res)
            flattened.then |value| { f_next.settleValue(value) }
            flattened.catch |error| { f_next.settleError(error) }
          }
        } else {
          f_next.settleValue(_value)
        }
      })
      return f_next
    }
  }
}

// `Tracer` (decorators-dispatch-observability.md D-2, ratified 2026-07-13):
// the pluggable observability sink `@traced`'s `sink:` argument targets —
// duck-typed (`enter`/`exit`/`threw`), so any object answering this protocol
// drops in. `Tracer.stdout` is the shipped default, routing through
// `System.print` (Phalcom has no dedicated logging primitive, system.md).
// Ships standalone: `@traced` itself is Install/Dispatch/Runtime-tier
// decorator-mechanism work, not yet built (see PLAN-DECORATORS.md), so this
// class has no caller yet — the sink protocol is ready when it lands.
class Tracer {
  @class
  stdout { Tracer.new() }

  enter(_ name, _ args) { System.print("-> " + name.toString + " " + args.toString) }
  exit(_ name, _ result, _ elapsed) { System.print("<- " + name.toString + " = " + result.toString) }
  threw(_ name, _ err) { System.print("!! " + name.toString + " threw " + err.toString) }
}

// `OffBehavior` (decorators-dispatch-observability.md D-3, ratified
// 2026-07-13): `@featureFlag`'s off-path — what a gated call does when its
// flag reads false. `applyTo(inv)` (invoked by the not-yet-built
// `@featureFlag` Runtime interceptor's `aroundSend` hook against a
// not-yet-defined `inv` envelope) is deliberately NOT implemented here —
// this class ships now as pure value semantics only; wiring it to a real
// interception envelope is Install/Dispatch/Runtime mechanism work.
class OffBehavior {
  @class
  raise { OffBehavior.new("raise", None) }
  @class
  fallback(_ sel) { OffBehavior.new("fallback", Some(sel)) }
  @class
  skip(_ value) { OffBehavior.new("skip", Some(value)) }

  @constructor
  new(_ kind, _ payload) { _kind = kind; _payload = payload }

  kind { _kind }
  payload { _payload }
}

// `Backoff` (decorators-behavioral.md B-2, ratified 2026-07-13): `@retry`'s
// backoff strategy. `.none` is fully usable today — no suspension needed,
// matching `@retry`'s own default. `.fixed(ms)`/`.exponential(base:,max:)`
// need a real suspending wait between attempts, which needs `System.sleep(_)`
// — explicitly **not landed** (system.md: "still open", gated on a
// timer-completion-source follow-on unit, itself gated on U-SCHED's ready-
// queue/timer split per open-questions.md §15). Rather than silently busy-
// waiting or lying about elapsed time, `.fixed`/`.exponential`'s
// `waitBefore` raises until that primitive exists — a real gap, not a stub
// pretending to work.
class Backoff {
  @class
  none { Backoff.new("none", 0, 0) }
  @class
  fixed(_ ms) { Backoff.new("fixed", ms, 0) }
  @class
  exponential(base, max) { Backoff.new("exponential", base, max) }

  @constructor
  new(_ kind, _ a, _ b) { _kind = kind; _a = a; _b = b }

  waitBefore(_ attempt) {
    if (_kind == "none") {
      return None
    } else {
      return Error.new("Backoff." + _kind + " needs System.sleep(_), not yet landed (system.md)").raise()
    }
  }
}

// `Attribute`/`On`/`Tier` (M-ATTR-ROOT, attribute-classes.md §"Decision"/
// §"`@On`"/§"Bootstrap"): the reified-descriptor root every attribute class
// extends, the `@On` builtin attribute carrying legality + declared tier, and
// the tier marker classes. `@Name(args)` desugars, at the enclosing class's
// definition time, to `Name.new(args)` + `artifact.__attach(_a)`
// (`compiler::attributes`/`compiler::lib::class_decl`) — the constructed
// instance is retained on the decorated artifact's native `_attributes` store
// (`ClassObject`/`MethodObject`/`ModuleObject`, `primitive/attribute.rs`),
// reflectable via `Behavior#attributes`/`Method#attributes` below.
//
// **Forced deviation (positional-only args, filed to `docs/forge/DEFERRED.md`):**
// attribute-arg lists are positional-only — `parser.rs`'s
// `parse_attribute_arg_list` has no label grammar — so `On`'s own
// constructors are positional (`On.new(target)` / `On.new(target, tier)`),
// not the spec's labeled `tier:`/`inherited:` form. `inherited:` is dropped
// entirely for the same reason (v0.3 follow-on once labeled attribute args
// exist). A single `target` (not a list) is stored, since the parser also has
// no list-literal syntax yet (`core.ph` L306) to build a multi-target list at
// a use site — multi-target `@On` is deferred alongside labeled args.

// Root. Every attribute extends this — usage (retention, `resolves_to_
// attribute_class`'s `extends` chain walk) is fixed in
// `compiler::attributes` at this root.
class Attribute {}

// Builtin attribute carrying legality + declared tier (A-1) — recursion
// bottoms out here: `On` is itself an `Attribute` subclass, so `@On(...)` on
// an `Attribute` subclass's own header is retained/reflectable like any other
// attribute. `tier` is `None` for passive metadata (no hook selector may be
// implemented — `attr.undeclared_hook`) or one of the `Tier` marker classes
// below (`Install`/`Dispatch`/`Runtime` — `Compile`/`Layout` are reserved for
// compiler-native hooks only, `attr.compile_tier_reserved`).
class On is Attribute {
  _targets
  _tier

  @constructor
  new(_ target) { _targets = target; _tier = None }
  @constructor
  new(_ target, _ tier) { _targets = target; _tier = tier }

  targets { _targets }
  tier { _tier }
}

// The tier marker classes (attribute-classes.md: "same pattern Phalcom
// already uses for `Bool`'s `True`/`False`" — real singleton objects, not
// symbols; a bare class, used purely by identity, is the same pattern
// `True`/`False` already establish). `Compile`/`Layout` are reserved for
// compiler-native hooks only; `Install`/`Dispatch`/`Runtime` are the
// user-facing tiers (M-INSTALL/M-DISPATCH/M-RUNTIME, PLAN-DECORATORS.md).
class Tier {}
class Compile is Tier {}
class Layout is Tier {}
class Install is Tier {}
class Dispatch is Tier {}
class Runtime is Tier {}

// `Behavior#attributes`/`#attributesOfType(_)` (object-model.md's metaclass
// tower superclass of `Class`+`Metaclass`) — the reflection surface over the
// native `_attributes` store every class object carries (M-ATTR-ROOT).
// Method-only reopen (no new fields) — safe on a bootstrap class (a
// reopen-with-fields would trip read-before-write).
class Behavior {
  attributes { self._$attributes }
  attributesOfType(_ cls) { self._$attributes.filter |a| { a.isA(cls) } }
}

// `Method#attributes`/`#attributesOfType(_)` — the same reflection surface
// as `Behavior` above, for the reified `Method` object a class's method
// dictionary holds.
class Method {
  attributes { self._$attributes }
  attributesOfType(_ cls) { self._$attributes.filter |a| { a.isA(cls) } }
}

// ============================================================================
// U-RESOURCE & U-STREAMS
// ============================================================================

class Resource {
  close {
    self._$close()
    return Ok.new(None)
  }
  isClosed { self._$isClosed }
}

class UseAfterCloseError is Error {}

class UnflushedError is Error {}

class BytesReader is Resource {
  @constructor
  new(_ source) {
    source.is(Bytes).ifFalse || {
      throw ArgumentError.new("BytesReader source must be a Bytes")
    }
    _handle = Resource._$register("BytesReader")
    // snapshot: source is a Bytes, copied — the reader's contents never change under it
    _data = source.slice(0, source.size)
    _pos = 0
  }

  read(_ dst) {
    dst.is(Bytes).ifFalse || {
      throw ArgumentError.new("dst must be a Bytes")
    }
    self.isClosed.ifTrue || {
      throw UseAfterCloseError.new("cannot read from closed BytesReader")
    }
    let remaining = _data.size - _pos
    let n = dst.size
    (remaining < n).ifTrue || { n = remaining }
    (n > 0).ifTrue || {
      _data.slice(_pos, _pos + n).copyInto(dst, 0)
      _pos = _pos + n
    }
    // In-memory operation cannot block, honest return type per spec section 2
    return Future.value(n)
  }
}

class BytesWriter is Resource {
  @constructor
  new() {
    _handle = Resource._$register("BytesWriter")
    _chunks = List.new()
  }

  write(_ src) {
    src.is(Bytes).ifFalse || {
      throw ArgumentError.new("src must be a Bytes")
    }
    self.isClosed.ifTrue || {
      throw UseAfterCloseError.new("cannot write to closed BytesWriter")
    }
    _chunks._$push(src.slice(0, src.size))
    return Future.value(src.size)
  }

  flush {
    return Future.value(None)
  }

  toBytes {
    let total = 0
    _chunks.each |c| { total = total + c.size }
    let res = Bytes.new(total)
    let offset = 0
    _chunks.each |c| {
      c.copyInto(res, offset)
      offset = offset + c.size
    }
    return res
  }
}

class BufferedWriter is Resource {
  @constructor
  new(_ inner) {
    _handle = Resource._$register("BufferedWriter")
    _inner = inner
    _buf = Bytes.new(8192)
    _len = 0
  }

  pending { _len }

  write(_ src) {
    src.is(Bytes).ifFalse || {
      throw ArgumentError.new("src must be a Bytes")
    }
    self.isClosed.ifTrue || {
      throw UseAfterCloseError.new("cannot write to closed BufferedWriter")
    }

    if (src.size >= _buf.size) {
      return self.flush.then |_| {
        _inner.write(src)
      }
    }

    if ((_len + src.size) > _buf.size) {
      return self.flush.then |_| {
        src.copyInto(_buf, _len)
        _len = _len + src.size
        Future.value(src.size)
      }
    } else {
      src.copyInto(_buf, _len)
      _len = _len + src.size
      return Future.value(src.size)
    }
  }

  flush {
    if (_len == 0) {
      return Future.value(None)
    }
    let chunk = _buf.slice(0, _len)
    return _inner.write(chunk).then |bytesWritten| {
      _len = 0
      Future.value(None)
    }
  }

  close {
    if (_len > 0) {
      throw UnflushedError.new("BufferedWriter closed with " + _len.toString + " pending bytes")
    }
    super.close
    return _inner.close
  }

  finish {
    return self.flush.then |_| {
      self.close
    }
  }
}

class BufferedReader is Resource {
  @constructor
  new(_ inner) {
    _handle = Resource._$register("BufferedReader")
    _inner = inner
    _buf = Bytes.new(8192)
    _pos = 0
    _len = 0
  }

  read(_ dst) {
    dst.is(Bytes).ifFalse || {
      throw ArgumentError.new("dst must be a Bytes")
    }
    self.isClosed.ifTrue || {
      throw UseAfterCloseError.new("cannot read from closed BufferedReader")
    }

    if (_pos < _len) {
      let avail = _len - _pos
      let n = dst.size
      (avail < n).ifTrue || { n = avail }
      _buf.slice(_pos, _pos + n).copyInto(dst, 0)
      _pos = _pos + n
      return Future.value(n)
    }

    return _inner.read(_buf).then |count| {
      if (count == 0) {
        return Future.value(0)
      }
      _pos = 0
      _len = count
      let avail = _len
      let n = dst.size
      (avail < n).ifTrue || { n = avail }
      _buf.slice(_pos, _pos + n).copyInto(dst, 0)
      _pos = _pos + n
      Future.value(n)
    }
  }
}
