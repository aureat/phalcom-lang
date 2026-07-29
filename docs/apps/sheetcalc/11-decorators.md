# SheetCalc — Runtime Decorators

Part of the [SheetCalc specification](README.md).
Grounded in [00-language-findings.md](00-language-findings.md) §9.

This document answers: *what decorators can a Phalcom user actually define
today, to improve their own code and developer experience?*

The answer is more interesting than expected, and it splits cleanly in two:
**one mechanism works and is genuinely good, one is half-built and silently
inert.**

---

## 1. The state of the attribute system, established by probe

Phalcom's attribute design (`docs/spec/current/decorators/on.md`) is a
tiered decorator system: an attribute class declares `@On(Target, tier: Tier)`
and implements a hook (`wrap`, `aroundSend`, `expand`, `resolveMissing`,
`finalizeLayout`) that the runtime dispatches at the right moment. It is a good
design. Here is what is actually wired:

| Layer | Status | Probe evidence |
|---|---|---|
| `@Attr` parses, attribute instantiated, retained | **WORKS** | `@Test("x")` on a method, reflectable |
| `Method#attributesOfType(_)`, `Behavior#attributesOfType(_)` | **WORKS** | returns `[<Test instance>]` |
| `@On(Method)` / `@On(Method, Install)` positional | **WORKS** | parses, accepted |
| Correctness floor: hook without a tier | **WORKS** | `attr.undeclared_hook` raised |
| Correctness floor: attribute on a constructor | **WORKS** | `attr.dangling` raised |
| `@Attr(label: value)` keyword args | **BROKEN** (DIV-ATTR-1) | `Expected ")"` |
| **Install-tier `wrap(_)` dispatch** | **NOT LANDED** | hook never called |

### DIV-ATTR-1 — keyword arguments at attribute call sites do not parse

The spec documents `@Author(name: "Ada")` (attribute-classes.md L169) and
`@On(Method, tier: Install)` (L86). Neither parses:

```phalcom
@Author(name: "Ada")
        ^ Expected ")"
```

The gap is **narrow and precisely located**. The language supports labeled
parameters everywhere else — both of these work:

```phalcom
@constructor
new(name:) { _name = name }    // declaration: parses
Author.new(name: "Ada")                  // normal call site: works, returns "Ada"
```

Only the **attribute call-site parser** lacks keyword-argument support. The
positional form is the workaround (`@Test("x")`, `@On(Method, Install)`), and it
is what SheetCalc uses. But it means the spec's own examples do not run, and
`tier:` — the thing that makes an attribute *behavioral* — is unwritable in its
documented form.

### BUG-ATTR-2 — the Install tier is accepted and silently does nothing

```phalcom
@On(Method, Install)
class Loud is Attribute {
  @constructor
  new() {}
  wrap(m) {
    System.print("  [WRAP HOOK CALLED]")   // never printed
    return m
  }
}
class Svc {
  @Loud
  ping => 1
}
Svc.new().ping        // => 1.  No wrap. No warning. No error.
```

`M-INSTALL` — "Install-tier `wrap(_)` composition dispatch" — is a **planned,
unlanded unit** (`docs/forge/PLAN-DECORATORS.md` §M-INSTALL). So the hook never
firing is *expected*, not a regression.

What is **not** defensible is the shape of the failure. The correctness floor is
**asymmetric**:

| You wrote | Result |
|---|---|
| `wrap(_)` with **no** declared tier | `attr.undeclared_hook` — loud, correct, helpful |
| `wrap(_)` **with** a declared tier | **silent no-op** |

The floor catches the *less* dangerous mistake and waves through the more
dangerous one. A user who declares the tier correctly — who reads the spec and
does exactly what it says — gets a decorator that silently does nothing. Their
`@Memoize` compiles, runs, returns correct answers, and memoizes nothing. The
only symptom is performance.

**Recommendation (independent of SheetCalc):** until `M-INSTALL` lands, a
declared behavioral tier whose hook cannot be dispatched should raise
`attr.tier_not_implemented` at class-definition time. Failing loudly on an
unimplemented feature costs one error message; failing silently costs a user
their afternoon. This is the single cheapest fix in this document.

> **Commentary.** This is the most instructive finding in the exercise, because
> nothing here is *broken* in the ordinary sense. The design is sound, the
> retention layer works, the diagnostics that exist are excellent (`attr.dangling`
> is a genuinely nice error). The system is simply half-landed, and the half
> that is missing is the half that the feature is *for*. A user reading
> `attribute-classes.md` would reasonably conclude that `@Memoize` works today.
> It does not, and nothing tells them.

---

## 2. What actually works: the `doesNotUnderstand` forwarding proxy

With method installation unavailable (`Class#+(_)` is string concatenation, not
method injection — findings §9), the classic decorator (read a method, wrap it,
reinstall it) is impossible. What survives is the **forwarding proxy**, and it
is verified working end to end for both getter and method sends:

```phalcom
class TracingProxy {
  @constructor
  on(t) { _t = t }

  doesNotUnderstand(msg) {
    System.print("  -> " + msg.name.toString + " args=" + msg.args.toString)
    let r = _t.perform(msg.selector, msg.args)
    System.print("  <- " + r.toString)
    return r
  }
}

let p = TracingProxy.on(Svc.new())
p.slow      //   -> slow args=[]      /   <- zzz      /  => zzz
p.ping(7)   //   -> ping args=[7]     /   <- pong:7   /  => pong:7
```

This is real, and it is the foundation of every decorator below. The mechanism:

1. The proxy defines almost nothing, so nearly every send misses.
2. The miss path reifies the send as a `Message` (`selector`, `name`, `labels`,
   `args`) and calls the overridable `doesNotUnderstand(_)`.
3. `Object#perform(sel, args)` forwards to the target.

`Message#labels` means even keyword sends forward correctly.

> **Commentary — this is Phalcom at its best.** The `doesNotUnderstand` +
> `perform` + `Message` triad is a complete, well-designed metaobject protocol,
> and it works exactly as a Smalltalker would expect. Amid a long list of gaps
> it is worth stating plainly: **this part of the language is good.** The
> reflection layer is the most finished thing I probed.

### The proxy's one structural limit

A proxy intercepts sends **from outside**. It cannot intercept `self`-sends
inside the target:

```phalcom
class Fib {
  fib(n) {
    if (n < 2) { return n }
    return self.fib(n - 1) + self.fib(n - 2)   // self-send: bypasses the proxy
  }
}
MemoProxy.on(Fib.new()).fib(30)   // memoizes ONLY the outermost call
```

The recursive calls go straight to `self`, never through the proxy, so a
memoizing proxy on a naive `fib` gives no speedup at all. This is inherent to
proxying (the same is true in every language) and is exactly what the Install
tier's `wrap` would fix, by replacing the method *in the class* so `self`-sends
hit the wrapper too.

**So the one decorator everyone reaches for first — `@Memoize` on a recursive
function — is precisely the one the proxy cannot deliver, and precisely the one
`M-INSTALL` is designed for and does not yet do.**

---

## 3. Decorators SheetCalc defines

### 3.1 `TracingProxy` — call tracing (works today)

Development aid: wrap any layer to see its traffic. Used to debug the evaluator
without touching evaluator code.

```phalcom
class TracingProxy {
  @constructor
  on(target, label) {
    _t = target
    _label = label
    _depth = 0
  }

  doesNotUnderstand(msg) {
    System.print(Str.repeat("  ", _depth) + "-> " + _label + "." + msg.name.toString)
    _depth = _depth + 1
    let r = _t.perform(msg.selector, msg.args)
    _depth = _depth - 1
    System.print(Str.repeat("  ", _depth) + "<- " + r.toString)
    return r
  }
}
```

**REQ-DEC-1.** `TracingProxy` must never appear in a golden-test path; its
output is non-deterministic in ordering only under fibers, and it is a debug
tool.

### 3.2 `MemoProxy` — caching (works, with the self-send caveat)

Useful for SheetCalc's `VLOOKUP` over a static range, which is called from
outside and never recurses on `self`:

```phalcom
class MemoProxy {
  @constructor
  on(t) {
    _t = t
    _cache = Map.new()
  }

  doesNotUnderstand(msg) {
    let key = MemoKey.of(msg.selector, msg.args)
    let hit = _cache.at(key)
    if (hit.isSome) { return hit.unwrapOr(None) }
    let r = _t.perform(msg.selector, msg.args)
    _cache.at(key, put: r)
    return r
  }
}
```

`MemoKey` needs `hash` + `==` over a selector and an argument list. Verified
viable: user classes work as `Map` keys (findings §7).

**REQ-DEC-2.** `MemoProxy` is only applied to referentially transparent
targets. SheetCalc applies it to `FunctionTable` lookups, never to `Grid`.

### 3.3 `@Test` — passive metadata (works today)

The one attribute SheetCalc actually ships, and it works because it is
**passive** — it declares no tier and implements no hook, so it needs nothing
from `M-INSTALL`:

```phalcom
class Test is Attribute {
  @constructor
  new(desc) { _desc = desc }
  desc => _desc
}

class ValueSuite {
  @Test("error absorbs addition")
  testErrorAbsorbs { ... }
}
```

Discovered by reflection at runtime — see [10-testing.md](10-testing.md). This
is the Java/C# annotation case, and it is fully supported.

> **Commentary.** Note what happened here: the decorator SheetCalc actually
> uses is the one that *changes no behavior*. Every behavioral decorator I
> wanted — `@Memoize`, `@Trace`, `@Validate` — is either unlanded (Install tier)
> or degraded to a proxy that misses self-sends. The gap between the attribute
> system's design and its delivered capability is the widest of any subsystem
> probed.

### 3.4 `@Requires` / `@Ensures` — contracts (LANDED, and usable)

Unlike the Install tier, the **contract** attributes are landed
(`U-ANNOT-CONTRACTS`, `dc01b07`) and weave at compile time via
`compiler/attributes.rs`. SheetCalc uses them on `support/num.ph`, where the
hand-rolled math most needs guarding:

```phalcom
class Num {
  @requires(#(n == n))            // reject NaN before it poisons the grid
  static floor(n) { ... }
}
```

**REQ-DEC-3.** `support/num.ph` guards every hand-rolled numeric helper with a
`@requires` precondition, because these functions reimplement primitives the
language should provide and are therefore the likeliest home for a subtle bug.

---

## 4. The decorators I wanted and could not build

This is the list that matters for the language, not for SheetCalc.

| Wanted | Blocked by | Notes |
|---|---|---|
| `@Memoize` on recursive `fib` | `M-INSTALL` unlanded | The canonical decorator. Proxy misses self-sends. |
| `@Trace` as an attribute | `M-INSTALL` unlanded | Have to hand-wrap with a proxy at every call site instead. |
| `@Deprecated("use X")` warning on call | `M-INSTALL` unlanded | Passive retention works; the *warning* needs `aroundSend`. |
| `@Timed` | no clock (findings §2) | Unbuildable regardless of tiers. |
| `@Retry(3)` | `M-INSTALL` + no clock for backoff | `core.ph` ships a `Backoff` class whose `waitBefore(_)` has no clock to wait on. |
| `@Validate` on cell setters | `M-INSTALL`; `@requires` covers the static case | Partially available via contracts. |

`core.ph` already contains `Tracer`, `Backoff`, `OffBehavior`, and the `Tier`
singletons — the *scaffolding* for all of this is written and shipped. It is
waiting on one unit (`M-INSTALL`) plus a clock.

> **Commentary — the honest summary.** Phalcom's decorator story today is:
> **excellent metaobject protocol, excellent compile-time contract weaving,
> excellent passive annotations, and no behavioral decorators.** The distance
> from here to a genuinely great story is short and well-understood — it is one
> planned unit and one primitive. That is a good position to be in, and it is
> worth not papering over how close it is.

---

## 5. Test hooks

| REQ | Test |
|---|---|
| REQ-DEC-1 | `suites/decorator_trace.ph` — proxy forwards getters and methods, preserves return values |
| REQ-DEC-2 | `suites/decorator_memo.ph` — cache hit/miss; documents the self-send miss as expected |
| REQ-DEC-3 | `suites/support_num_contracts.ph` — `@requires` rejects NaN |
| BUG-ATTR-2 | `suites/attr_install_inert.ph` — **a pinning test**: asserts the `wrap` hook is NOT called, so that when `M-INSTALL` lands this test fails and forces a deliberate update |
