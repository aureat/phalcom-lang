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
  is(cls) {
    let c = self.class
    while (c != None) {
      (c == cls).ifTrue { return true }
      c = c.superclass
    }
    return false
  }

  // Exact test: true iff `cls` is the receiver's *live, direct* class —
  // no superclass walk. Backs the `x is! T` surface (is-tests.md).
  isExactly(cls) => self.class == cls

  // Back-compat alias (U-CORE-1) — `is(_)` is now the primary kind-of test;
  // `isA` is retained so existing internal (`List#==` etc.) and user callers
  // keep working unchanged.
  isA(cls) => self.is(cls)
}

class Class {
  new() => self.new_()
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
  new(msg) {
    _message = msg
    _kind = None
    _cause = None
    _displaced = None
  }
  kind => _kind
  kind=(val) { _kind = val }
  cause => _cause
  cause=(val) { _cause = val }
  displaced => _displaced
  displaced=(val) { _displaced = val }
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

// Raised while building an association Map literal when a logically equal key
// was already contributed. Ordinary post-construction Map insertion still
// overwrites by design.
class DuplicateKeyError is Error {}

class Number {}

class Int is Number {}

class Float is Number {
  // Absolute value. Returns Float; NaN returns NaN.
  // Native: float_abs
  abs => self.abs()

  // Sign: returns Int -1, 0, or 1. -0.0 and +0.0 both return 0.
  // Raises #nonFiniteNumber if receiver is NaN or infinite.
  // Native: float_sign
  sign => self.sign()

  // Greatest Int not greater than receiver (toward -∞).
  // Raises #nonFiniteNumber if receiver is NaN/infinite.
  // Native: float_floor
  floor => self.floor()

  // Least Int not less than receiver (toward +∞).
  // Raises #nonFiniteNumber if receiver is NaN/infinite.
  // Native: float_ceil
  ceil => self.ceil()

  // Int toward zero (truncation).
  // Raises #nonFiniteNumber if receiver is NaN/infinite.
  // Native: float_truncated
  truncated => self.truncated()

  // Nearest Int, ties-to-even.
  // Raises #nonFiniteNumber if receiver is NaN/infinite.
  // Native: float_rounded
  rounded => self.rounded()

  // Exact Int conversion — raises #numericConversion if fractional.
  // Raises #nonFiniteNumber if receiver is NaN/infinite.
  // Native: float_to_int_exact
  toIntExact => self.toIntExact()

  // true iff receiver is finite and has no fractional part.
  // Native: float_is_integer
  isInteger => self.isInteger

  // true iff receiver is IEEE 754 NaN.
  // Native: float_is_nan
  isNaN => self.isNaN

  // true iff receiver is neither NaN nor infinite.
  // Native: float_is_finite
  isFinite => self.isFinite

  // true iff receiver is +Infinity or -Infinity.
  // Native: float_is_infinite
  isInfinite => self.isInfinite
}

class String {
  // Display (U-CORE-4, R-INV-4.1): a string's display *is* itself — no
  // representation read, so this is `.ph`-derivable rather than a floor
  // primitive (ADR-0019's derivability test).
  toString => self

  // Byte count. UTF-8 buffer length in bytes (not codepoints).
  size => self.byteCount_
  isEmpty => self.byteCount_ == 0

  // Number of leading bytes in the UTF-8 sequence starting at byte offset `i`.
  // Read purely from the lead byte's numeric range: 1/2/3/4-byte sequences are
  // encoded by the lead byte's numeric value (no bitmask needed).
  leadByteLen_(i) {
    const b = self.byteAt_(i)
    return (b == None).ifTrue({ None }, ifFalse: {
      (b < 128).ifTrue({ 1 }, ifFalse: {
        (b < 224).ifTrue({ 2 }, ifFalse: {
          (b < 240).ifTrue({ 3 }, ifFalse: { 4 }) }) })
    })
  }

  // The Unicode scalar value at byte offset `i`, or `None` if out-of-range
  // or mid-sequence. UTF-8 decode via division/modulo (no bitwise ops).
  codePointAt(i) {
    const b0 = self.byteAt_(i)
    return (b0 == None).ifTrue({ None }, ifFalse: {
      (b0 < 128).ifTrue({
        // ASCII single byte (0xxxxxxx)
        b0
      }, ifFalse: {
        (b0 < 192).ifTrue({
          // Continuation byte (10xxxxxx), not a start byte
          None
        }, ifFalse: {
          (b0 < 224).ifTrue({
            // 2-byte sequence (110xxxxx 10xxxxxx)
            const b1 = self.byteAt_(i + 1)
            (b1 == None).ifTrue({ None }, ifFalse: {
              (b1 < 128).ifTrue({ None }, ifFalse: {
                (b1 >= 192).ifTrue({ None }, ifFalse: {
                  ((b0 - 192) * 64) + (b1 - 128)
                })
              })
            })
          }, ifFalse: {
            (b0 < 240).ifTrue({
              // 3-byte sequence (1110xxxx 10xxxxxx 10xxxxxx)
              const b1 = self.byteAt_(i + 1)
              const b2 = self.byteAt_(i + 2)
              (b1 == None).ifTrue({ None }, ifFalse: {
                (b2 == None).ifTrue({ None }, ifFalse: {
                  (b1 < 128).ifTrue({ None }, ifFalse: {
                    (b1 >= 192).ifTrue({ None }, ifFalse: {
                      (b2 < 128).ifTrue({ None }, ifFalse: {
                        (b2 >= 192).ifTrue({ None }, ifFalse: {
                          ((b0 - 224) * 4096) + ((b1 - 128) * 64) + (b2 - 128)
                        })
                      })
                    })
                  })
                })
              })
            }, ifFalse: {
              (b0 < 248).ifTrue({
                // 4-byte sequence (11110xxx 10xxxxxx 10xxxxxx 10xxxxxx)
                const b1 = self.byteAt_(i + 1)
                const b2 = self.byteAt_(i + 2)
                const b3 = self.byteAt_(i + 3)
                (b1 == None).ifTrue({ None }, ifFalse: {
                  (b2 == None).ifTrue({ None }, ifFalse: {
                    (b3 == None).ifTrue({ None }, ifFalse: {
                      (b1 < 128).ifTrue({ None }, ifFalse: {
                        (b1 >= 192).ifTrue({ None }, ifFalse: {
                          (b2 < 128).ifTrue({ None }, ifFalse: {
                            (b2 >= 192).ifTrue({ None }, ifFalse: {
                              (b3 < 128).ifTrue({ None }, ifFalse: {
                                (b3 >= 192).ifTrue({ None }, ifFalse: {
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
              }, ifFalse: {
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
  indexOf(needle) {
    (needle.isA(String)).ifTrue({}, ifFalse: {
      throw ArgumentError.new("indexOf: needle must be a String")
    })
    (needle.isEmpty).ifTrue({
      throw ArgumentError.new("indexOf: needle must be non-empty")
    })

    let i = 0
    while (i <= self.byteCount_ - needle.byteCount_) {
      let match = true
      let j = 0
      while (j < needle.byteCount_) {
        (self.byteAt_(i + j) == needle.byteAt_(j)).ifTrue({}, ifFalse: {
          match = false
        })
        (match).ifTrue({ j = j + 1 }, ifFalse: { j = needle.byteCount_ })
      }
      (match).ifTrue({ return i })
      i = i + 1
    }
    return -1
  }

  // Split by delimiter substring. Returns a List of String segments.
  split(delimiter) {
    (delimiter.isA(String)).ifTrue({}, ifFalse: {
      throw ArgumentError.new("split: delimiter must be a String")
    })
    (delimiter.isEmpty).ifTrue({
      throw ArgumentError.new("split: delimiter must be non-empty")
    })

    let result = List.new()
    let prev = 0
    let i = self.indexOf(delimiter)
    while (i != -1) {
      result.add(self.slice_(prev, i))
      prev = i + delimiter.byteCount_
      // Search for next occurrence after this delimiter
      let rest = self.slice_(prev, self.byteCount_)
      let nextIdx = rest.indexOf(delimiter)
      (nextIdx == -1).ifTrue({ i = -1 }, ifFalse: { i = prev + nextIdx })
    }
    result.add(self.slice_(prev, self.byteCount_))
    return result
  }

  // Replace all occurrences of `from` with `to`.
  replace(from, to) {
    (from.isA(String)).ifTrue({}, ifFalse: {
      throw ArgumentError.new("replace: from must be a String")
    })
    (to.isA(String)).ifTrue({}, ifFalse: {
      throw ArgumentError.new("replace: to must be a String")
    })
    (from.isEmpty).ifTrue({
      throw ArgumentError.new("replace: from must be non-empty")
    })

    let result = ""
    let prev = 0
    let i = self.indexOf(from)
    while (i != -1) {
      result = result + self.slice_(prev, i) + to
      prev = i + from.byteCount_
      let rest = self.slice_(prev, self.byteCount_)
      let nextIdx = rest.indexOf(from)
      (nextIdx == -1).ifTrue({ i = -1 }, ifFalse: { i = prev + nextIdx })
    }
    result = result + self.slice_(prev, self.byteCount_)
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

  trim(chars) => self.trimStart(chars).trimEnd(chars)

  // Trim from the start using the given charset.
  trimStart(chars) {
    (chars.isA(String)).ifTrue({}, ifFalse: {
      throw ArgumentError.new("trimStart: chars must be a String")
    })

    let i = 0
    let stop = false
    while ((i < self.byteCount_).and({ not stop })) {
      const cp = self.codePointAt(i)
      let found = false
      let j = 0
      while (j < chars.byteCount_) {
        (chars.codePointAt(j) == cp).ifTrue({ found = true })
        const len = chars.leadByteLen_(j)
        (len == None).ifTrue({ j = j + 1 }, ifFalse: { j = j + len })
      }
      (found).ifTrue({
        i = i + self.leadByteLen_(i)
      }, ifFalse: {
        stop = true  // exit loop, keeping i at the first non-trimmed byte
      })
    }
    return self.slice_(i, self.byteCount_)
  }

  // Trim from the end using the given charset.
  trimEnd(chars) {
    (chars.isA(String)).ifTrue({}, ifFalse: {
      throw ArgumentError.new("trimEnd: chars must be a String")
    })

    let i = self.byteCount_
    let stop = false
    while ((i > 0).and({ not stop })) {
      // Scan backward one byte at a time to find the previous lead byte
      i = i - 1
      let cp = self.codePointAt(i)
      (cp == None).ifTrue({
        // Not a lead byte, keep scanning back
      }, ifFalse: {
        // Found a lead byte; check if it's in the trim set
        let found = false
        let j = 0
        while (j < chars.byteCount_) {
          (chars.codePointAt(j) == cp).ifTrue({ found = true })
          const len = chars.leadByteLen_(j)
          (len == None).ifTrue({ j = j + 1 }, ifFalse: { j = j + len })
        }
        (found).ifTrue({}, ifFalse: {
          // Not in the set; keep this whole character and stop scanning
          i = i + self.leadByteLen_(i)
          stop = true
        })
      })
    }
    return self.slice_(0, i)
  }

  // Repeat the string `count` times.
  *(count) {
    (count.isA(Number)).ifTrue({}, ifFalse: {
      throw ArgumentError.new("*: count must be a Number")
    })
    (count >= 0).ifTrue({}, ifFalse: {
      throw ArgumentError.new("*: count must be >= 0")
    })
    (count % 1 == 0).ifTrue({}, ifFalse: {
      throw ArgumentError.new("*: count must be an integer")
    })

    (count == 0).ifTrue({ return "" })
    (count == 1).ifTrue({ return self })

    let result = ""
    let i = 0
    while (i < count) {
      result = result + self
      i = i + 1
    }
    return result
  }

  // Byte sequence accessor (U-STRING §2.4).
  bytes => StringByteSequence.new(self)

  // Codepoint sequence accessor (U-STRING §2.4).
  codePoints => StringCodePointSequence.new(self)
}

// Byte-level sequence view (U-STRING §2.4, ADR-0048 shaped).
class StringByteSequence {
  @constructor
  new(s) { _string = s }

  size => _string.byteCount_

  at(i) => _string.byteAt_(i)

  each(f) {
    let i = 0
    while (i < self.size) {
      f.call(self.at(i))
      i = i + 1
    }
  }

  // Iterate over byte offsets: cursor steps to next lead byte.
  nextCursor_(cursor) {
    const next = (cursor == None).ifTrue({ 0 }, ifFalse: {
      cursor + 1
    })
    return (next < _string.byteCount_).ifTrue({ next }, ifFalse: { None })
  }
}

// Codepoint-level sequence view (U-STRING §2.4, ADR-0048 shaped).
class StringCodePointSequence {
  @constructor
  new(s) { _string = s }

  // Codepoint count: full scan (no native "codepoint length").
  size {
    let n = 0
    let i = self.nextCursor_(None)
    while (i != None) {
      n = n + 1
      i = self.nextCursor_(i)
    }
    return n
  }

  at(byteOffset) => _string.codePointAt(byteOffset)

  each(f) {
    let i = self.nextCursor_(None)
    while (i != None) {
      f.call(self.at(i))
      i = self.nextCursor_(i)
    }
  }

  // Iterate over byte offsets: cursor steps by UTF-8 char boundary.
  nextCursor_(cursor) {
    const next = (cursor == None).ifTrue({ 0 }, ifFalse: {
      cursor + _string.leadByteLen_(cursor)
    })
    return (next < _string.byteCount_).ifTrue({ next }, ifFalse: { None })
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
    return self.ifTrue({ "true" }, ifFalse: { "false" })
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
  ifNone(f) {
    return self.match(some: { v => self }, none: { f.call(); self })
  }

  // `Some` passes through unchanged; `None` becomes `f`'s (0-arity) `Option`
  // result (values-and-absence.md §3.3's "Transform" group). This is the
  // `??` operator's target (§3.4: `a ?? b` === `a.orElse { b }`).
  orElse(f) {
    return self.match(some: { v => self }, none: { f.call() })
  }

  isSome => self.match(some: { v => true }, none: { false })

  isNone => self.match(some: { v => false }, none: { true })

  // U-STD (values-and-absence.md §3.3's "Transform" group; catalog-delta §2.2):
  // `Some(v)` becomes `Some(f(v))`; `None` passes through untouched. `f` is a
  // 1-arity block over the wrapped value; the result is re-wrapped so the
  // chain stays an `Option`.
  map(f) {
    return self.match(some: { v => Some.new(f.call(v)) }, none: { self })
  }

  // U-STD (values-and-absence.md §3.3's "Transform" group): like `map`, but `f`
  // already returns an `Option`, so its result is used directly rather than
  // re-wrapped — the monadic bind (`>>=`). `None` short-circuits to `self`.
  flatMap(f) {
    return self.match(some: { v => f.call(v) }, none: { self })
  }

  // U-STD (values-and-absence.md §3.3's "Filter" group): `Some(v)` stays `Some(v)`
  // when `pred(v)` is `true`, otherwise collapses to the shared `None` singleton;
  // `None` passes through. `pred` must return a real `Bool` (ADR-0021).
  filter(pred) {
    return self.match(some: { v => if (pred.call(v)) { self } else { None } }, none: { self })
  }

  // U-STD (values-and-absence.md §3.3's "Effect" group; mirror of `ifNone`): runs
  // the 1-arity block `f` for its side effect on the wrapped value when `Some`,
  // then returns `self` so calls chain; a `None` is passed through untouched.
  ifSome(f) {
    return self.match(some: { v => f.call(v); self }, none: { self })
  }

  // U-STD (values-and-absence.md §3.3's "Extract" group): unwraps a `Some` to its
  // value, or yields `default` for a `None`. The eager sibling of `orElse`
  // (which takes a block); here `default` is an already-evaluated fallback value.
  unwrapOr(default) {
    return self.match(some: { v => v }, none: { default })
  }

  // Display (values-and-absence §3, U-CORE-4, R-INV-4.3). Derived over
  // `match`, so a user-overridden `match` is respected (R-INV-2.4) and the
  // inner value is rendered via its OWN `toString` message (so a
  // value-typed payload agrees with the print path, R-INV-4.1).
  toString => self.match(some: { v => "Some(" + v.toString + ")" }, none: { "None" })

  // absence -> error bridge (error-handling.md §5, result.md §2, ADR-0007):
  // `Some(v)` already carries a real value, so no reason is needed; `None`
  // has no value, so `err` fills in the failure reason. Round-trips with
  // `Result#ok()` below (`Some(v).okOr(_)` -> `Ok(v)` -> `.ok()` -> `Some(v)`).
  okOr(err) {
    return self.match(some: { v => Ok.new(v) }, none: { Err.new(err) })
  }
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
  isOk => self.match(ok: { v => true }, err: { e => false })

  isErr => self.match(ok: { v => false }, err: { e => true })

  // Transforms the `Ok` value; an `Err` passes through unchanged (never
  // raises — a pure value transform, result.md §2).
  map(f) {
    return self.match(ok: { v => Ok.new(f.call(v)) }, err: { e => self })
  }

  // Transforms the `Err` reason; an `Ok` passes through unchanged — the
  // symmetric counterpart of `map` (result.md §2).
  mapErr(f) {
    return self.match(ok: { v => self }, err: { e => Err.new(f.call(e)) })
  }

  // Chains an `Ok` -> `Result` function (flat-map/monadic bind); short-
  // circuits on `Err` (result.md §2).
  andThen(f) {
    return self.match(ok: { v => f.call(v) }, err: { e => self })
  }

  // The `Ok` value, or **re-`throw`** the `Err` reason (`throw expr` ===
  // `expr.raise()`, ADR-0031 §1) — the value -> exception bridge
  // (error-handling.md §5, result.md §3). If the `Err` reason is not itself
  // an `Error` (a user built `Err.new(42)`), `raise` misses and this
  // surfaces the ordinary `doesNotUnderstand` miss — consistent with the
  // only-`Error`-throwable rule, not special-cased here.
  unwrap => self.match(ok: { v => v }, err: { e => e.raise() })

  unwrapOr(default) {
    return self.match(ok: { v => v }, err: { e => default })
  }

  // The `Err` reason, or re-`throw` if `Ok` — symmetric to `unwrap`.
  unwrapErr => self.match(ok: { v => v.raise() }, err: { e => e })

  // `Result` -> `Option`: drops the failure reason (result.md §2/§5). Round-
  // trips with `Option#okOr(_)` above.
  ok() {
    return self.match(ok: { v => Some.new(v) }, err: { e => None })
  }

  // Display: each arm renders its payload via its OWN `toString` (agrees
  // with `Option#toString`'s pattern, R-INV-4.1).
  toString => self.match(ok: { v => "Ok(" + v.toString + ")" }, err: { e => "Err(" + e.toString + ")" })
}

class Ok is Result {
  @constructor
  new(v) { _value = v }

  match(ok:, err:) => ok.call(_value)
}

class Err is Result {
  @constructor
  new(e) { _error = e }

  match(ok:, err:) => err.call(_error)
}

// The throw -> value bridge (error-handling.md §5): runs `self` (0-arity),
// capturing a `throw` into `Err(e)`; success is `Ok(v)`. Pure `.ph` over
// `on(_)(_)` (U-ERR, ADR-0038) — no floor cost. Installed on the abstract
// `Function` root so both `Block` and (reflectively) `Method` inherit it,
// mirroring how `call`/`on`/`ensure` are native on both.
class Function {
  // Explicit `()` (a method, not a getter — `attempt() { … }` vs `attempt =>
  // …`) so the call-site selector encodes as `attempt()`, matching the
  // spec's `{ risky() }.attempt()` call form (error-handling.md §5) exactly.
  attempt() {
    return { Ok.new(self.call()) }.on(Error) { e => Err.new(e) }
  }
}

// Kernel List (ADR-0020): a native array-backed heap object (ListObject),
// not an InstanceObject — bootstrapped in Rust (universe.rs) with five floor
// primitives (length_/at_/set_/push_, plus native `new()`). This
// skeleton reopens that bootstrapped row to define the public protocol over
// those primitives (ADR-0019's "hybrid: native primitives, self-defined
// control"). `toString` is ALSO a native primitive this unit, not defined
// here — see the U-LIST return contract for why (element-value stringification
// is blocked on U-CORE-4; DEFERRED.md #19). U-STD (catalog-delta §2.4;
// DEFERRED.md #18/#20/#25) discharges the deferral for the combinator layer:
// `map`/`reduce`/`filter`/`includes`/`isEmpty` and the `at(_:put:)` wrapper
// over `set_` now live below, all pure `.ph` over the floor. Only
// **list-literal syntax** `[a, b, c]` remains deferred (it needs a new ADR +
// parser work; DEFERRED.md #6) — do not add that here.

class Iterable {
  // Generic index-cursor walk over `self.size` (ADR-0048 §1/§3). A subclass whose
  // cursor is not a 0..size index (none in-kernel today) overrides this.
  iterate(cursor) {
    const next = (cursor == None).ifTrue({ 0 }, ifFalse: { cursor + 1 })
    return (next < self.size).ifTrue({ next }, ifFalse: { None })
  }

  each(f) {
    for (x in self) {
      f.call(x)
    }
  }

  // map/filter/reduce/includes walk `iterate`/`iteratorValue` DIRECTLY, not
  // `self.each` — `Map` overrides `each` with an incompatible 2-arity (k, v)
  // selector (DEC-CT-E); routing these through `self.each` would silently call a
  // 1-arity block with 2 arguments the moment `Map` inherits them. See the rubric.
  // U-SEQ DEC-SEQ-A branch (A): `map` is redefined lazy (was eager List-returning under U-ITERABLE —
  // BREAKING, see plan.md §8 migration-audit gate). Migration audit scope: core.ph, examples/*.ph,
  // test fixtures. Note: benchmarks/ outside audit scope; any broken .map() calls there require
  // manual .toList() repair. `filter` is NOT touched (stays the eager U-ITERABLE selector);
  // `where` is the new Wren-parity lazy filter. `.toList` is the materializer for all of them.
  map(f) {
    return MapView.new(self, f)
  }

  filter(pred) {
    let result = List.new()
    let c = self.iterate(None)
    while (c != None) {
      let x = self.iteratorValue(c)
      pred.call(x).ifTrue({ result.add(x) }, ifFalse: { None })
      c = self.iterate(c)
    }
    return result
  }

  reduce(init, f) {
    let acc = init
    let c = self.iterate(None)
    while (c != None) {
      acc = f.call(acc, self.iteratorValue(c))
      c = self.iterate(c)
    }
    return acc
  }

  includes(x) {
    let found = false
    let c = self.iterate(None)
    while (c != None) {
      (self.iteratorValue(c) == x).ifTrue({ found = true }, ifFalse: { None })
      c = self.iterate(c)
    }
    return found
  }

  isEmpty => self.size == 0

  // U-SEQ (iteration.md §5 extension, wren_core.wren Sequence L7-119 precedent): combinator breadth
  // Phalcom lacked. All written over `for (x in self)`, never `self.each { }` (Map's `each(f)` is 2-arg —
  // see plan.md §3.1) and never index math. Zero new floor primitives.

  all(f) {
    for (x in self) {
      f.call(x).ifFalse { return false }
    }
    return true
  }

  any(f) {
    for (x in self) {
      f.call(x).ifTrue { return true }
    }
    return false
  }

  count {
    let n = 0
    for (x in self) { n = n + 1 }
    return n
  }

  count(f) {
    let n = 0
    for (x in self) { f.call(x).ifTrue { n = n + 1 } }
    return n
  }

  find(f) {
    for (x in self) {
      f.call(x).ifTrue { return Some.new(x) }
    }
    return None
  }

  join => self.join("")

  join(sep) {
    // Note: O(N²) allocation cost due to naive string concatenation. Each `result = result + ...`
    // allocates a new string and copies all prior content. For N elements, total work is ~N²/2.
    // This is acceptable for Phalcom's interpreter domain (collections stay small) but users
    // joining large collections should be aware of this limitation.
    let first = true
    let result = ""
    for (x in self) {
      first.ifFalse { result = result + sep }
      first = false
      result = result + x.toString
    }
    return result
  }

  toList {
    let result = List.new()
    for (x in self) { result.add(x) }
    return result
  }

  where(pred) {
    return WhereView.new(self, pred)
  }

  skip(n) {
    return SkipView.new(self, n)
  }

  take(n) {
    return TakeView.new(self, n)
  }
}

class List {
  size => self.length_

  at(i) {
    return self.at_(i)
  }

  add(v) {
    self.push_(v)
    return self
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
  iteratorValue(cursor) => self.at(cursor)

  // U-STD (DEFERRED.md #18): the public `.ph` wrapper over the `set_(_,_)`
  // floor primitive — writes `put` at index `i` and returns `self` so writes
  // chain (mirrors `add`). Selector `at(_:put:)` matches `set_`'s 2 args;
  // the labeled parameter is named `put` (label == name, parser convention).
  at(i, put:) {
    self.set_(i, put)
    return self
  }

  // U-INDEX (ADR-0060): `[]` is its own dedicated, user-overridable
  // selector — not `at`'s call-site sugar — so `List` must opt in
  // explicitly with a thin delegation, same as any other collection
  // author would. `xs[i]` sends `[_]`; `xs[i] = v` sends `[_,put]`.
  [i] { return self.at(i) }

  [i, put:] { return self.at(i, put: put) }

  // U-CORE-5 (decisions.md Q5, R-INV-5.3 E1-E5): structural equality —
  // element-wise, order-sensitive, via each element's own `==`. Guarded by
  // `isA(List)` so a non-List `other` is simply unequal (E2), never a dNU.
  // Derived entirely over the floor (`size`/`at`/`isA`/`while`/`and`/`not`) —
  // no new native primitive (ADR-0019 unchanged). `and`/`not` are the
  // language's infix/prefix operator forms (`Bool#and(_:)`/`Bool#not()`
  // dispatched by the compiler, not dotted-call syntax — `and`/`not` are
  // reserved words and cannot follow `.` as a bare identifier).
  ==(other) {
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
  !=(other) {
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
  size => self.size_

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
    while (i < self.size_) {
      s = s + (i > 0).ifTrue({ ", " }, ifFalse: { "" })
      s = s + self.keyAt_(i).toString + ": " + self.valueAt_(i).toString
      i = i + 1
    }
    return s + "}"
  }

  // Safe association lookup: Some(value) on hit, None on absence.
  get(k) => self.get_(k)

  // Strict association lookup. Do one lookup so a stored None remains a
  // value rather than being confused with an absent key.
  [k] {
    return self.get(k).match(
      some: { value => value },
      none: { KeyError.new("Map key not found").raise() }
    )
  }

  // Explicit insert returns the previous value when replacing an association.
  insert(value, for: key) => self.put_(key, value)

  // Legacy mutation spelling retained only while B.3 still lowers association
  // literals into chained sends. `get` and `[]` are the lookup surface.
  at(k, put:) {
    self.put_(k, put)
    return self
  }

  // `m[k] = v` shares insert's key identity and encounter-order semantics.
  [k, put:] => self.put_(k, put)

  includes(k) => self.has_(k)

  // Removes an association and returns its former value when present.
  remove(k) => self.remove_(k)

  clear {
    while (self.size > 0) {
      self.remove_(self.keyAt_(0))
    }
    return ()
  }

  // Lightweight live encounter-order views. They retain the Map and read its
  // current slots; they never copy associations into a List.
  keys => MapKeysView.new(self)

  values => MapValuesView.new(self)

  entries => MapEntriesView.new(self)

  // Copies a positive Record's Symbol labels and values into a fresh mutable
  // Map. `#{}` canonicalizes to Unit, which represents the empty Record form.
  static from(record: record) {
    if ((not record.isA(Record)) and (not record.isA(Unit))) {
      throw ArgumentError.new("Map.from: argument must be a Record or Unit")
    }
    let result = Map.new()
    if (record.isA(Record)) {
      let i = 0
      while (i < record.size_) {
        result.insert(record.valueAt_(i), for: record.labelAt_(i))
        i = i + 1
      }
    }
    return result
  }

  // 2-arg block `{ k, v => ... }` per entry, in iteration order.
  each(f) {
    let i = 0
    while (i < self.size) {
      f.call(self.keyAt_(i), self.valueAt_(i))
      i = i + 1
    }
  }

  // DEC-CT-E: the cursor value `iteratorValue` yields is the KEY (both Map and Set yield keys);
  // `each(_)` above remains the 2-arg entry form for Map.

  iteratorValue(cursor) => self.keyAt_(cursor)

  // Structural equality: same key set, pairwise-== values (order-independent
  // over keys — `includes`/`get` do the membership + value work, not raw
  // index comparison). Guarded by `isA(Map)` so a non-Map is simply unequal.
  ==(other) {
    if (other.isA(Map)) {
      let same = (self.size == other.size)
      let i = 0
      while (same and (i < self.size)) {
        let k = self.keyAt_(i)
        same = other.get(k).match(
          some: { value => self.valueAt_(i) == value },
          none: { false }
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
  !=(other) {
    return not (self == other)
  }
}

class Set {
  size => self.size_

  // Display (U-CORE-4, R-INV-4.1; DEFERRED CB-1). Mirrors `Value::to_string`'s
  // native `Set` rendering exactly — `Set(a, b)`, `Set()` when empty. Derived
  // over `size_`/`at_`; each element renders via its OWN `toString`.
  toString {
    let s = "Set("
    let i = 0
    while (i < self.size_) {
      s = s + (i > 0).ifTrue({ ", " }, ifFalse: { "" })
      s = s + self.at_(i).toString
      i = i + 1
    }
    return s + ")"
  }

  add(v) {
    self.add_(v)
    return self
  }

  includes(v) => self.has_(v)

  remove(v) {
    self.remove_(v)
    return self
  }

  // Positional read in insertion order — not in the map-and-set.md selector
  // table, but a direct, zero-floor-cost derivation over at_ that the
  // U-CORE-5 conformance harness (collection-protocol.md §2) needs, and a
  // natural extension of the sequence protocol every collection instantiates.
  at(i) => self.at_(i)



  iteratorValue(cursor) => self.at_(cursor)

  // Structural equality: same members, order-independent. Same-size plus
  // "every element of self is in other" is sufficient since neither set
  // holds duplicates (add_ is idempotent).
  ==(other) {
    if (other.isA(Set)) {
      let same = (self.size == other.size)
      let i = 0
      while (same and (i < self.size)) {
        same = other.includes(self.at_(i))
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  !=(other) {
    return not (self == other)
  }
}

class Unit {
  toString => "()"
  hash => 0
}

// Kernel Tuple (ADR-0032 §1, ADR-0039, U-COLLTYPES Phase 2): a native fixed
// arity immutable slice — Object::Tuple, mirroring List's shape but with NO
// mutation selector (immutability is structural, TupleObject's Box<[Value]>).
// Product literals compile directly to native build bytecodes.

class Tuple {
  size => self.size_
  positionals => self.positionals_
  labeled => self.labeled_

  // Display (U-CORE-4, R-INV-4.1; DEFERRED CB-1). Mirrors `Value::to_string`'s
  // native `Tuple` rendering exactly — `(a, b)`, `()` when empty. Derived over
  // `size_`/`at_`; each element renders via its OWN `toString`.
  toString {
    let s = "("
    let i = 0
    while (i < self.size_) {
      s = s + (i > 0).ifTrue({ ", " }, ifFalse: { "" })
      s = s + self.at_(i).toString
      i = i + 1
    }
    return s + ")"
  }

  at(i) => self.at_(i)

  // U-INDEX (ADR-0060): read-only — `tup[i]` sends `[_]`. No `[_,put]`
  // wrapper: `Tuple` is immutable (no `at(_,put:)` either), so `tup[i] = v`
  // correctly `doesNotUnderstand` rather than raising a bespoke error.
  [i] { return self.at(i) }

  iteratorValue(cursor) => self.at_(cursor)

  // Structural equality: same arity, pairwise-==. Guarded by isA(Tuple) so a
  // non-Tuple (including a same-elements List — cross-kind, E2) is unequal.
  ==(other) {
    if (other.isA(Tuple)) {
      let same = (self.size == other.size) and (self.positionalSize_ == other.positionalSize_)
      let i = 0
      while (same and (i < self.size - self.positionalSize_)) {
        same = (self.labelAt_(i) == other.labelAt_(i))
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

  !=(other) {
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
    let acc = 17 + self.positionalSize_
    let i = 0
    while (i < self.size) {
      acc = (acc * 31 + self.at(i).hash) % 999999937
      i = i + 1
    }
    i = 0
    while (i < self.size - self.positionalSize_) {
      acc = (acc * 31 + self.labelAt_(i).hash) % 999999937
      i = i + 1
    }
    return acc
  }
}

class Record {
  size => self.size_

  ==(other) {
    if (other.isA(Record)) {
      let same = (self.size == other.size)
      let i = 0
      while (same and (i < self.size)) {
        let label = self.labelAt_(i)
        let j = 0
        let found = false
        while ((not found) and (j < other.size)) {
          if (other.labelAt_(j) == label) {
            found = (self.valueAt_(i) == other.valueAt_(j))
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
      acc = (acc + ((self.labelAt_(i).hash * 31) + self.valueAt_(i).hash)) % 999999937
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
  new(map) { _map = map }

  size => _map.size

  iteratorValue(cursor) => _map.keyAt_(cursor)
}

class MapValuesView is Iterable {
  @constructor
  new(map) { _map = map }

  size => _map.size

  iteratorValue(cursor) => _map.valueAt_(cursor)
}

// Immutable-by-surface association value for MapEntriesView. No setters or
// value-object equality/hash protocol are part of this phase.
class Entry {
  @constructor
  new(key, value) {
    _key = key
    _value = value
  }

  key => _key

  value => _value
}

class MapEntriesView is Iterable {
  @constructor
  new(map) { _map = map }

  size => _map.size

  iteratorValue(cursor) => Entry.new(_map.keyAt_(cursor), _map.valueAt_(cursor))
}

// Kernel Range (ADR-0032 §1, ADR-0039, U-COLLTYPES Phase 3): a native lazy
// numeric interval — Object::Range, three native bound-field getters and NOTHING
// else on the floor. `each`/`toList`/`size`/`includes`/`first`/`last` are all
// derived here over `start_`/`end_`/`inclusive_` + Number arithmetic —
// `each` GENERATES elements, never allocates (RG-2 laziness), so
// `Range.new(1, 1000000, true)` stays O(1) to construct and each step of
// `each` is O(1). Bound convention (RG-1): `a..b` inclusive (`inclusive_`
// true), `a...b` exclusive (false) — the reserved `..`/`...` literal's
// committed meaning, honored now via the explicit constructor.

class Range {
  first => self.start_

  // Display (U-CORE-4, R-INV-4.1; DEFERRED CB-1). Mirrors `Value::to_string`'s
  // native `Range` rendering exactly. Note the separator is NOT the intuitive
  // one: `..` is the INCLUSIVE range and `...` the exclusive (Ruby's spelling,
  // not Rust's) — `value/render.rs` reads
  // `if range.inclusive() { ".." } else { "..." }`, and this must not drift
  // from it.
  toString {
    let sep = self.inclusive_.ifTrue({ ".." }, ifFalse: { "..." })
    return self.start_.toString + sep + self.end_.toString
  }

  last {
    return self.inclusive_.ifTrue({ self.end_ }, ifFalse: { self.end_ - 1 })
  }

  size {
    let n = self.inclusive_.ifTrue({ self.end_ - self.start_ + 1 }, ifFalse: { self.end_ - self.start_ })
    return (n < 0).ifTrue({ 0 }, ifFalse: { n })
  }

  includes(n) {
    let upper = self.inclusive_.ifTrue({ n <= self.end_ }, ifFalse: { n < self.end_ })
    return (self.start_ <= n) and upper
  }

  // Total indexed read (mirrors every other collection's `at(_)`): raw value
  // on hit, the None singleton on miss — not part of map-and-set.md's/
  // tuple-and-range.md's enumerated Range selector table, but a direct,
  // zero-floor-cost derivation the U-CORE-5 conformance harness (every
  // collection instantiates it) needs (U-COLLTYPES plan.md §7).
  at(i) {
    return (i >= 0 and (i < self.size)).ifTrue({ self.start_ + i }, ifFalse: { None })
  }

  // Generates start, start+1, … up to the bound — no allocation (RG-2).


  // The explicit materialization escape hatch (RG-2).
  toList {
    let result = List.new()
    self.each { x => result.add(x) }
    return result
  }

  // Cursor iteration protocol (ADR-0035 §1) — identical shape to List's;
  // `iteratorValue` is `start + cursor`, never a materialized index into an
  // element buffer (there is none).


  iteratorValue(cursor) => self.start_ + cursor

  // Structural equality over the normalized bound fields (start/end/
  // inclusive) — not the generated sequence, which would defeat laziness for
  // a large range. Guarded by isA(Range).
  ==(other) {
    if (other.isA(Range)) {
      let sameStart = (self.start_ == other.start_)
      let sameEnd = (self.end_ == other.end_)
      let sameBound = (self.inclusive_ == other.inclusive_)
      return sameStart and sameEnd and sameBound
    } else {
      return false
    }
  }

  !=(other) {
    return not (self == other)
  }

  // Value hash (immutable ⇒ hashable, Q5): a .ph fold over the three bound
  // fields' own `hash` — zero new floor, consistent with == by construction.
  hash {
    let acc = 17
    acc = (acc * 31 + self.start_.hash) % 999999937
    acc = (acc * 31 + self.end_.hash) % 999999937
    acc = (acc * 31 + self.inclusive_.hash) % 999999937
    return acc
  }
}

// U-BYTES (PDR-0011, docs/spec/v0.2/core/bytes.md): the kernel octet buffer's
// `.ph` protocol over the eleven floor primitives. Bulk no-user-code work is
// native (bytes.md §3.1); everything here is validation-lifting, derivation,
// and the Iterable hookup. `each`/`map`/`filter`/`reduce` are deliberately
// ABSENT — inherited from `Iterable` so `Fiber.yield` works mid-iteration
// (law 8); adding native or local overrides is a spec violation.
class Bytes {
  size => self.size_

  // Bare-or-None passthrough (List#at's shape). Unlike List, the union is
  // unambiguous: an octet is never None (bytes.md §3).
  at(i) => self.at_(i)

  iteratorValue(cursor) => self.at_(cursor)

  // An octet is an integer Number in 0..255 (bytes.md §2). `and` is lazy,
  // so the arithmetic tests never run on a non-Number. (No trailing `_`:
  // that marker is reserved for native primitives, and this is pure .ph.)
  isOctet(v) {
    return v.isA(Number) and (v >= 0) and (v <= 255) and ((v % 1) == 0)
  }

  // Raise-lifting writes (bytes.md law 1: precondition violations raise,
  // reads stay total). The floor's set_ reports a bad write as a native
  // type error; the .ph surface names the contract instead.
  set(i, v) {
    if (not self.isOctet(v)) {
      throw ArgumentError.new("Bytes#set: value must be an integer in 0..255")
    }
    if ((i < 0) or (i >= self.size)) {
      throw ArgumentError.new("Bytes#set: index out of range")
    }
    self.set_(i, v)
    return self
  }

  at(i, put:) { return self.set(i, put) }

  [i] { return self.at(i) }

  [i, put:] { return self.at(i, put: put) }

  fill(v) {
    if (not self.isOctet(v)) {
      throw ArgumentError.new("Bytes#fill: value must be an integer in 0..255")
    }
    self.fill_(v)
    return self
  }

  // One native memset, complete because the length is fixed (bytes.md §7).
  // The guarantee is a documented obligation, not a mechanism — scope it
  // with `ensure`. A getter so the call reads `key.zeroize` (ADR-0012:
  // `zeroize` and `zeroize()` are different selectors).
  zeroize {
    self.fill_(0)
    return self
  }

  utf8 => self.utf8_

  // Display decode: total, lossy (invalid sequences become U+FFFD). Never
  // round-trip the result into data (PDR-0013 ruling 4).
  utf8Lossy => self.utf8Lossy_

  slice(start, end) {
    if ((start < 0) or (end < start) or (end > self.size)) {
      throw ArgumentError.new("Bytes#slice: range must satisfy 0 <= start <= end <= size")
    }
    return self.slice_(start, end)
  }

  copyInto(dst, offset) {
    if (not dst.isA(Bytes)) {
      throw ArgumentError.new("Bytes#copyInto: destination must be a Bytes")
    }
    if ((offset < 0) or ((offset + self.size) > dst.size)) {
      throw ArgumentError.new("Bytes#copyInto: offset + size must fit the destination")
    }
    self.copyInto_(dst, offset)
    return self
  }

  // Derivability with teeth (bytes.md §3.1): new + two native memmoves,
  // zero per-byte loops.
  concat(other) {
    if (not other.isA(Bytes)) {
      throw ArgumentError.new("Bytes#concat: argument must be a Bytes")
    }
    const out = Bytes.new(self.size + other.size)
    self.copyInto_(out, 0)
    other.copyInto_(out, self.size)
    return out
  }

  equalsConstantTime(other) => self.equalsConstantTime_(other)

  // Structural equality, List#=='s exact shape (collection-protocol §4).
  // Short-circuits — correct here, and exactly why it must never be the
  // secret-comparison spelling (bytes.md §8).
  ==(other) {
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
  !=(other) {
    return not (self == other)
  }

  toString => "Bytes(" + self.size.toString + ")"

  toList {
    const out = []
    for (b in self) {
      out.add(b)
    }
    return out
  }

  // The immutable, value-hashable snapshot — the Map-key escape hatch
  // (PDR-0011 ruling 4; Bytes itself is mutable => identity hash, never a
  // valid Map/Set key).
  toTuple => Tuple.__fromList(self.toList)

  static fromString(s) {
    if (not s.isA(String)) {
      throw ArgumentError.new("Bytes.fromString: argument must be a String")
    }
    return Bytes.fromString_(s)
  }

  // The builder story (bytes.md law 3 forecloses growth): build in a List,
  // freeze into Bytes — Tuple.fromList's shape. `set` (not `set_`) so a
  // non-octet element raises the named contract error.
  static fromList(list) {
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
  named_(n) { _name = n }
  static read => OpenMode.named_("read")
  static write => OpenMode.named_("write")
  static append => OpenMode.named_("append")
  static readWrite => OpenMode.named_("readWrite")
  name => _name
  ==(other) { return other.isA(OpenMode) and (_name == other.name) }
  !=(other) { return not (self == other) }
  toString => "OpenMode." + _name
}

class Path {
  @constructor
  of(s) {
    if (not s.isA(String)) {
      throw ArgumentError.new("Path.of: argument must be a String")
    }
    _bytes = Bytes.fromString(s)
    _hash = Path.contentHash(_bytes)
  }

  @constructor
  ofBytes(b) {
    if (not b.isA(Bytes)) {
      throw ArgumentError.new("Path.ofBytes: argument must be a Bytes")
    }
    _bytes = b.slice(0, b.size)
    _hash = Path.contentHash(_bytes)
  }

  static contentHash(bytes) {
    let acc = 1
    let i = 0
    while (i < bytes.size) {
      acc = (acc * 31 + bytes.at(i)) % 999999937
      i = i + 1
    }
    return acc
  }

  bytes => _bytes.slice(0, _bytes.size)
  hash => _hash

  ==(other) {
    if (not other.isA(Path)) { return false }
    if (_hash != other.hash) { return false }
    return _bytes == other.bytes
  }

  !=(other) { return not (self == other) }

  isAbsolute => (_bytes.size > 0) and (_bytes.at(0) == 47)

  join(other) {
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
        res.add(Path.ofBytes(_bytes.slice(start, i)))
      }
    }
    return res
  }

  toString => _bytes.utf8Lossy
}


// Lazy view classes (wren_core.wren MapSequence/WhereSequence/SkipSequence L121-152, 168-182), ported
// to Phalcom's bare-cursor protocol (post-U-ITERABLE: `iterate` returns the raw next cursor, or the
// `None` singleton at exhaustion — never Some-wrapped). `extends Iterable` so combinators, `for`,
// and every other view work on a view for free.

class MapView is Iterable {
  @constructor
  new(source, f) {
    // Note: validation of f as callable defers to iteratorValue() time to preserve
    // lazy evaluation semantics (no side effects at map construction, only on iteration).
    // Error timing: [1,2,3].map("not-a-function") succeeds at construction time but
    // fails during iteration with "does not understand 'call(_)'". This is intentional;
    // contrast with SkipView/TakeView which validate counts eagerly at construction.
    _source = source
    _f = f
  }
  iterate(cursor) => _source.iterate(cursor)
  iteratorValue(cursor) => _f.call(_source.iteratorValue(cursor))
}

class WhereView is Iterable {
  @constructor
  new(source, pred) {
    _source = source
    _pred = pred
  }
  iterate(cursor) {
    // Cost: O(selectivity) per element returned. The while-loop scans through non-matching
    // source elements to find each match. For highly selective predicates, scanning cost
    // compounds: [1..1M].where(x > 999900).take(10) requires ~999k inner iterations.
    // This is the correct tradeoff of lazy evaluation (no intermediate collection) for
    // CPU (scan through non-matches). For eager filtering, use filter() instead.
    let cur = cursor
    while (true) {
      cur = _source.iterate(cur)
      (cur == None).ifTrue { return None }
      _pred.call(_source.iteratorValue(cur)).ifTrue { return cur }
    }
  }
  iteratorValue(cursor) => _source.iteratorValue(cursor)
}

class SkipView is Iterable {
  @constructor
  new(source, count) {
    (count.isA(Number) and (count >= 0)).ifFalse {
      throw Error("skip: count must be a non-negative Number")
    }
    _source = source
    _count = count
  }
  iterate(cursor) {
    (cursor != None).ifTrue { return _source.iterate(cursor) }
    let cur = _source.iterate(None)
    let n = _count
    while ((n > 0) and (cur != None)) {
      cur = _source.iterate(cur)
      n = n - 1
    }
    return cur
  }
  iteratorValue(cursor) => _source.iteratorValue(cursor)
}

class TakeView is Iterable {
  @constructor
  new(source, count) {
    (count.isA(Number) and (count >= 0)).ifFalse {
      throw Error("take: count must be a non-negative Number")
    }
    _source = source
    _count = count
  }
  iterate(cursor) {
    let srcCursor = None
    let taken = 0
    (cursor != None).ifTrue {
      srcCursor = cursor.at(0)
      taken = cursor.at(1)
    }
    ((taken + 1) > _count).ifTrue { return None }
    let next = _source.iterate(srcCursor)
    (next == None).ifTrue { return None }
    return (next, taken + 1)
  }
  iteratorValue(cursor) => _source.iteratorValue(cursor.at(0))
}

class System {
  // U-STRING write funnel (ADR-0049 amendment): pure `.ph` control flow over
  // native `write_(_)` and the `toString` message. Additive-only: does not
  // touch the native `print(_)` pathway (pre-existing divergence between
  // `Value::to_string` and the `.toString` message is out of scope).
  static write(obj) {
    System.writeObject_(obj)
    return obj
  }

  static writeObject_(obj) {
    const s = obj.toString
    (s.isA(String)).ifTrue({
      System.write_(s)
    }, ifFalse: {
      System.write_("invalid toString")
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
  // (which runs its arm through `Block#call`'s native re-entrant
  // `run_until`, forbidding a fiber switch underneath, ADR-0030 §4): `f.try()`
  // must run at this method's own top level, not nested inside a block a
  // native primitive is driving, so the receiver is unwrapped via
  // `unwrapOr(_)` into a plain local first, and `try()` sent as its own
  // statement.
  static runScheduled() {
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
// of thing, `Fiber`s registered by `await` and `Block`s registered by
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
  value(v) {
    _state = "pending"
    _value = None
    _waiters = List.new()
    self.settleValue(v)
  }

  // Builds an already-`rejected` future wrapping `e` (concurrency.md §2
  // `@constructor error(_)`); see `value(_)` for why this routes
  // through `settleError` instead of assigning state directly.
  @constructor
  error(e) {
    _state = "pending"
    _value = None
    _waiters = List.new()
    self.settleError(e)
  }

  // `true` once `self` has settled (`fulfilled` or `rejected`); `false`
  // while `pending`.
  isReady => _state != "pending"

  // Settles `self` as `fulfilled` with `v`, unless already settled (settle-
  // once, C-FUT-3): a `self.isReady` receiver is a no-op that returns `self`
  // unchanged, so a second `settleValue`/`settleError` can never clobber the
  // first result. Returns `self` either way so callers can chain.
  settleValue(v) {
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
  settleError(e) {
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
  // A waiter is either a `Fiber` (registered by `await`) or a `Block`
  // (registered by `then`/`map`/`catch`); `System.schedule(_)` accepts both,
  // enqueueing a fiber as-is and wrapping anything else. A fiber waiter can,
  // however, be *finished* by the time we settle — it may have failed after
  // registering (E004(c): a caller that `await`s from under a native frame
  // raises out of `await` with its registration still in the list). Resuming a
  // finished fiber aborts the whole run, taking every other waiter on this
  // future down with it, so skip those rather than scheduling a corpse.
  drain() {
    _waiters.each { w =>
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
  value => (_state == "fulfilled").ifTrue({ Some.new(_value) }, ifFalse: { None })

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
  // that reaches `Block#call` — `.attempt()`, `.on(_)`, `ensure` — puts a native
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
        _waiters.add(Fiber.current)
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
  static async(action) {
    const f = Future.new()
    const driver = Fiber.new {
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
  static flatten(value) {
    if (value.isA(Future)) {
      return value
    }
    return Future.value(value)
  }

  // Registers a continuation on the settled/fulfilled path (concurrency.md
  // §2 `then(_)`). If pending, registers a continuation that will settle
  // the returned future when this receiver settles.
  then(f) {
    if (self.isReady) {
      if (_state == "fulfilled") {
        return Future.flatten(f.call(_value))
      } else {
        return self
      }
    } else {
      const f_next = Future.new()
      _waiters.add({
        if (_state == "fulfilled") {
          const fib = Fiber.new({ f.call(_value) })
          const res = fib.try()
          if (fib.error.isSome) {
            f_next.settleError(fib.error.unwrapOr(None))
          } else {
            const flattened = Future.flatten(res)
            flattened.then { value => f_next.settleValue(value) }
            flattened.catch { error => f_next.settleError(error) }
          }
        } else {
          f_next.settleError(_value)
        }
      })
      return f_next
    }
  }

  // `then(_)` restricted to the fulfilled path (concurrency.md §2 `map(_)`).
  map(f) {
    if (self.isReady) {
      if (_state == "fulfilled") {
        return Future.flatten(f.call(_value))
      } else {
        return self
      }
    } else {
      const f_next = Future.new()
      _waiters.add({
        if (_state == "fulfilled") {
          const fib = Fiber.new({ f.call(_value) })
          const res = fib.try()
          if (fib.error.isSome) {
            f_next.settleError(fib.error.unwrapOr(None))
          } else {
            const flattened = Future.flatten(res)
            flattened.then { value => f_next.settleValue(value) }
            flattened.catch { error => f_next.settleError(error) }
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
  catch(f) {
    if (self.isReady) {
      if (_state == "rejected") {
        return Future.flatten(f.call(_value))
      } else {
        return self
      }
    } else {
      const f_next = Future.new()
      _waiters.add({
        if (_state == "rejected") {
          const fib = Fiber.new({ f.call(_value) })
          const res = fib.try()
          if (fib.error.isSome) {
            f_next.settleError(fib.error.unwrapOr(None))
          } else {
            const flattened = Future.flatten(res)
            flattened.then { value => f_next.settleValue(value) }
            flattened.catch { error => f_next.settleError(error) }
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
  static stdout => Tracer.new()

  enter(name, args) { System.print("-> " + name.toString + " " + args.toString) }
  exit(name, result, elapsed) { System.print("<- " + name.toString + " = " + result.toString) }
  threw(name, err) { System.print("!! " + name.toString + " threw " + err.toString) }
}

// `OffBehavior` (decorators-dispatch-observability.md D-3, ratified
// 2026-07-13): `@featureFlag`'s off-path — what a gated call does when its
// flag reads false. `applyTo(inv)` (invoked by the not-yet-built
// `@featureFlag` Runtime interceptor's `aroundSend` hook against a
// not-yet-defined `inv` envelope) is deliberately NOT implemented here —
// this class ships now as pure value semantics only; wiring it to a real
// interception envelope is Install/Dispatch/Runtime mechanism work.
class OffBehavior {
  static raise => OffBehavior.new("raise", None)
  static fallback(sel) => OffBehavior.new("fallback", Some.new(sel))
  static skip(value) => OffBehavior.new("skip", Some.new(value))

  @constructor
  new(kind, payload) { _kind = kind; _payload = payload }

  kind => _kind
  payload => _payload
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
  static none => Backoff.new("none", 0, 0)
  static fixed(ms) => Backoff.new("fixed", ms, 0)
  static exponential(base:, max:) => Backoff.new("exponential", base, max)

  @constructor
  new(kind, a, b) { _kind = kind; _a = a; _b = b }

  waitBefore(attempt) {
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
  new(target) { _targets = target; _tier = None }
  @constructor
  new(target, tier) { _targets = target; _tier = tier }

  targets => _targets
  tier => _tier
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
  attributes => self.__attributes
  attributesOfType(cls) => self.__attributes.filter { a => a.isA(cls) }
}

// `Method#attributes`/`#attributesOfType(_)` — the same reflection surface
// as `Behavior` above, for the reified `Method` object a class's method
// dictionary holds.
class Method {
  attributes => self.__attributes
  attributesOfType(cls) => self.__attributes.filter { a => a.isA(cls) }
}

// ============================================================================
// U-RESOURCE & U-STREAMS
// ============================================================================

class Resource {
  close {
    self.close_()
    return Ok.new(None)
  }
  isClosed => self.isClosed_
}

class UseAfterCloseError is Error {}

class UnflushedError is Error {}

class BytesReader is Resource {
  @constructor
  new(source) {
    source.is(Bytes).ifFalse {
      throw ArgumentError.new("BytesReader source must be a Bytes")
    }
    _handle = Resource.register_("BytesReader")
    // snapshot: source is a Bytes, copied — the reader's contents never change under it
    _data = source.slice(0, source.size)
    _pos = 0
  }

  read(dst) {
    dst.is(Bytes).ifFalse {
      throw ArgumentError.new("dst must be a Bytes")
    }
    self.isClosed.ifTrue {
      throw UseAfterCloseError.new("cannot read from closed BytesReader")
    }
    let remaining = _data.size - _pos
    let n = dst.size
    (remaining < n).ifTrue { n = remaining }
    (n > 0).ifTrue {
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
    _handle = Resource.register_("BytesWriter")
    _chunks = List.new()
  }

  write(src) {
    src.is(Bytes).ifFalse {
      throw ArgumentError.new("src must be a Bytes")
    }
    self.isClosed.ifTrue {
      throw UseAfterCloseError.new("cannot write to closed BytesWriter")
    }
    _chunks.add(src.slice(0, src.size))
    return Future.value(src.size)
  }

  flush {
    return Future.value(None)
  }

  toBytes {
    let total = 0
    _chunks.each { c => total = total + c.size }
    let res = Bytes.new(total)
    let offset = 0
    _chunks.each { c =>
      c.copyInto(res, offset)
      offset = offset + c.size
    }
    return res
  }
}

class BufferedWriter is Resource {
  @constructor
  new(inner) {
    _handle = Resource.register_("BufferedWriter")
    _inner = inner
    _buf = Bytes.new(8192)
    _len = 0
  }

  pending => _len

  write(src) {
    src.is(Bytes).ifFalse {
      throw ArgumentError.new("src must be a Bytes")
    }
    self.isClosed.ifTrue {
      throw UseAfterCloseError.new("cannot write to closed BufferedWriter")
    }

    if (src.size >= _buf.size) {
      return self.flush.then { _ =>
        _inner.write(src)
      }
    }

    if ((_len + src.size) > _buf.size) {
      return self.flush.then { _ =>
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
    return _inner.write(chunk).then { bytesWritten =>
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
    return self.flush.then { _ =>
      self.close
    }
  }
}

class BufferedReader is Resource {
  @constructor
  new(inner) {
    _handle = Resource.register_("BufferedReader")
    _inner = inner
    _buf = Bytes.new(8192)
    _pos = 0
    _len = 0
  }

  read(dst) {
    dst.is(Bytes).ifFalse {
      throw ArgumentError.new("dst must be a Bytes")
    }
    self.isClosed.ifTrue {
      throw UseAfterCloseError.new("cannot read from closed BufferedReader")
    }

    if (_pos < _len) {
      let avail = _len - _pos
      let n = dst.size
      (avail < n).ifTrue { n = avail }
      _buf.slice(_pos, _pos + n).copyInto(dst, 0)
      _pos = _pos + n
      return Future.value(n)
    }

    return _inner.read(_buf).then { count =>
      if (count == 0) {
        return Future.value(0)
      }
      _pos = 0
      _len = count
      let avail = _len
      let n = dst.size
      (avail < n).ifTrue { n = avail }
      _buf.slice(_pos, _pos + n).copyInto(dst, 0)
      _pos = _pos + n
      Future.value(n)
    }
  }
}
