# SheetCalc — Design Patterns

Part of the [SheetCalc specification](README.md).
Grounded in [00-language-findings.md](00-language-findings.md).

Which patterns reduce boilerplate in Phalcom, which are unnecessary because the
language already gives you the thing, and which are unavailable.

The interesting result: Phalcom's Smalltalk lineage **dissolves** several
classic patterns outright, and its gaps **force** a few that a better-equipped
language would not need.

---

## 1. Patterns the language dissolves

### Visitor → polymorphic dispatch

The AST is the textbook Visitor case, and Visitor exists only because
statically-typed languages lack double dispatch on the receiver. Phalcom is a
message-send language: `node.eval(ctx)` already dispatches on the node's class.

```phalcom
// NOT this:
class EvalVisitor {
  visitBinOp(n) { ... }
  visitLit(n) { ... }
}

// This:
class BinOp is Ast {
  eval(ctx) { return _l.eval(ctx).perform(_op, [_r.eval(ctx)]) }
}
class Lit is Ast {
  eval(ctx) => _v
}
```

**DEC-PAT-1.** SheetCalc uses polymorphic `eval` on `Ast` nodes. No visitor.

Cost, stated: adding a *new operation* over the AST (a pretty-printer, a
dependency walker) means touching every node class, which is the trade Visitor
exists to invert. SheetCalc needs exactly three operations (`eval`,
`dependencies`, `toString`) and they are stable, so the trade is right here. If
the operation set were open-ended, the calculus would change.

### Strategy → blocks

A block is a first-class closure. Strategy is just passing one.

```phalcom
Sort.by(refs) { a, b => a.row < b.row }
```

### Command / Interpreter → the AST is already the interpreter

`Ast#eval` is the Interpreter pattern; naming it would add nothing.

### Iterator → the cursor protocol

`Iterable` + `iterate(cursor)`/`iteratorValue(cursor)` is already an iterator
protocol, and lazy views (`where`/`map`/`skip`/`take`) compose over it.
Verified working, including `break` inside `for` over a composed view.

**But see GAP-FIB-1.** Every one of these is unusable inside a fiber that
yields. The pattern is dissolved right up until you need concurrency, at which
point it comes back as a hand-rolled indexed `while`.

---

## 2. Patterns the language rewards

### Null Object → `CellEmpty`, and `Option` for real absence

`core.ph` seals `Option`/`Some`/`None` and there is no `nil` in user-reachable
surface (Invariant 4). This is a genuine strength: SheetCalc has **no null
checks anywhere**.

Two distinct absences, deliberately kept distinct:

| Concept | Type | Meaning |
|---|---|---|
| A cell with nothing in it | `CellEmpty` (a Null Object) | A *value*. Coerces to 0 in `+`, renders as "". |
| A key not present in a `Map` | `None` | Not a value. Must be unwrapped. |

**DEC-PAT-2.** `Grid#at(_)` returns a `CellValue` (`CellEmpty` for unset), not
an `Option`. Callers never unwrap. The `Option` stays inside `Grid`'s
implementation where the `Map` lookup happens.

> **Commentary.** The Null Object pattern usually needs justification. Here the
> language's own design argues for it: since `None` must be explicitly
> unwrapped at every site, letting `None` escape into the evaluator would put
> `.unwrapOr(_)` on every cell read. Absorbing it into `CellEmpty` at the
> boundary is the difference between an evaluator that reads cleanly and one
> that is 30% unwrapping. Good languages make the right pattern the lazy one;
> this is an instance of that.

### Template Method → the `CellValue` root defaults

The root defines every operator returning `#VALUE!`, so subclasses override only
the meaningful pairs. This collapses a 5×5 type matrix into a handful of
overrides. See [02-value-model.md §2](02-value-model.md). Biggest single
boilerplate saving in the program.

### Singleton → static getters

```phalcom
class CellEmpty is CellValue {
  static instance {
    if (CellEmpty.instance_ == None) { CellEmpty.instance_ = CellEmpty.new() }
    return CellEmpty.instance_
  }
}
```

Works, but see GAP-CLS-1 below: there are no class-side instance variables, so
the "static field" has to be faked.

---

## 3. Patterns forced on us by gaps

These are the ones that matter, because each is a pattern that exists **only to
route around a missing feature**.

### Boxed values (DEC-VM-1) — forced by no double dispatch

Every number is a `CellNum`. Forced by `1 + userObject` raising unfixably. See
[02-value-model.md §1](02-value-model.md). This is the Boxed Value pattern
adopted not for polymorphism but because the alternative is impossible.

### The `support/` shim — forced by an empty `Number`

`Num.floor`, `Num.round`, `Num.abs`, `Num.min`, `Num.max`, `Str.padLeft`,
`Sort.by`. None of this is spreadsheet code. It is a **stdlib shim**, a pattern
whose entire purpose is to be deleted when the core library grows.

**REQ-PAT-1.** `support/` must stay free of domain logic so it can be lifted
into `core.ph` verbatim.

### Result-chaining by hand — forced by no `?` operator

With no exceptions and no `?`/`try` sugar, every parser frame manually matches:

```phalcom
// Every. Single. Frame.
let lhs = self.parsePrimary()
if (lhs.isErr) { return lhs }
let l = lhs.unwrapOr(None)
```

`Result#andThen(_)` exists and chains, but a Pratt loop needs the intermediate
value in scope for the *next* iteration, so `andThen` nests rather than
sequences, and the nesting gets deep fast.

> **Commentary.** This is the highest-volume boilerplate in the program. Rust
> solves it with `?`, Swift with `try`, Haskell with `do`. Phalcom's
> `Result` is well-built (`map`/`mapErr`/`andThen`/`match(ok:err:)` all present
> and clean) but has no *syntactic* affordance, so a recursive-descent parser —
> the canonical `Result`-heavy program — pays a manual 3-line tax per frame.
> Roughly 40% of the parser's line count is error plumbing. See GAP-ERR-1.

### Proxy-instead-of-decorator — forced by no method installation

See [11-decorators.md](11-decorators.md). The proxy is a workaround for the
absent Install tier, and it misses self-sends.

---

## 4. Patterns unavailable

| Pattern | Blocked by |
|---|---|
| Method wrapping / around-advice | No method installation from `.ph`; `M-INSTALL` unlanded |
| Mixins / traits | No multiple inheritance; no method injection to simulate it |
| Class-side state (`static var`) | GAP-CLS-1 — no class-side instance variables |
| Dependency injection by constructor label overloading | Constructors are `@constructor
name(...)`; distinct labels are distinct selectors, which actually works *well* — see below |

### GAP-CLS-1 — no class-side instance variables

```phalcom
class CellEmpty {
  static var instance_       // does not exist
}
```

There is no `static var`. A class-side singleton cache must live in a module-level
`var`, which is reachable only through the module object. Minor, but it makes the
Singleton pattern uglier than it should be in a language whose metaclass tower
would otherwise support it naturally. The metaclass exists; class-side state does
not.

---

## 5. Where Phalcom is genuinely pleasant

Recorded deliberately, because a document this long about gaps skews the picture.

**Keyword constructors are excellent.** `@constructor
at(c, r)` /
`@constructor
of(n)` / `construct new(items:)` give named, self-documenting
construction with no builder pattern and no overload resolution:

```phalcom
Ref.at(1, 2)
CellNum.of(42)
ErrorVal.of(#DIV0)
Stack.new(items: List.new())
```

Because arity and labels are part of the selector, `Ref.at(_,_)` and
`Ref.a1(_)` coexist with zero ambiguity. This is better than constructor
overloading in every mainstream language, and it removed the Builder pattern
from the program entirely.

**Operator overloading on user classes is clean** and is what makes DEC-VM-1
survivable at all.

**The metaobject protocol** (`perform` / `doesNotUnderstand` / `Message` /
`methodFor` / `invokeOn`) is complete and works. See
[11-decorators.md §2](11-decorators.md).

**Sealed `Option`** with no `nil` escape hatch eliminates a whole bug class.

**Diagnostics are good.** `attr.dangling: attributes cannot be attached to a
constructor` is a better error than most production languages produce. The
runtime is honest about its guarded limits (`CannotYieldAcrossNativeFrame`,
`DeadFrameError`) rather than crashing or corrupting.

> **Commentary.** The pattern across all of this: **Phalcom's object model is
> its strong suit and its standard library is its weak one.** Every dissolved
> pattern in §1 and every pleasure in §5 comes from the object model. Every
> forced pattern in §3 comes from a missing library function or a missing bit of
> syntactic affordance. That is a good problem to have, because libraries are
> cheaper to fix than object models. See [13-language-gaps.md](13-language-gaps.md).
