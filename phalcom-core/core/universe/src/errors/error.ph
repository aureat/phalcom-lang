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
