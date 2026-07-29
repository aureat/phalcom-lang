# Implicit self for calls — parenthesized self-send elision

- Status: **Proposed** (experimental; not ratified — exploratory)
- Date: 2026-07-13
- Depends on:
  [method-lookup.md](../method-lookup.md) (send semantics, `perform`, `doesNotUnderstand`) ·
  [object-model.md](../object-model.md) (`self`, the receiver as frame slot 0) ·
  [functions.md](../functions.md) (callables, blocks, `call`)
- Related:
  [dispatch.md — selector identity](../experimental/dispatch.md) (why elision is dispatch-safe here) ·
  [values-and-absence.md](../values-and-absence.md) (the `_field` sigil already implies self)

## Context

Phalcom already has **implicit self for state**: a field is written `_balance`, and
`_balance = _balance - amount` reads and writes the receiver's slot with no `self.`
prefix. The `_` sigil makes field access unambiguous and self-relative by
construction (core.ph).

Behavior has no such affordance. To call a sibling method from inside a method you
write the receiver explicitly:

```phalcom
withdraw(amount) {
  self.ensureOpen()                 // explicit receiver required today
  _balance = _balance - amount
}
```

This note removes that one asymmetry: a **parenthesized call** may elide `self.`, so
`ensureOpen()` means `self.ensureOpen()`. It does **not** touch the paren-less
bareword form — `foo` (a getter send) stays explicit as `self.foo`. That boundary is
the entire design; everything below defends it.

## Decision

Inside a method body, a **parenthesized call** `name(args)` resolves by a two-tier,
purely lexical rule:

1. **Local first.** If `name` is a lexically visible binding (parameter, `let`/`var`,
   or an enclosing block parameter) holding a callable, the call invokes that
   callable — `name.call(args)`.
2. **Self-send otherwise.** If `name` is not a local binding, the call lowers to
   `self.name(args)` — an ordinary virtual send of selector `name`.

A **paren-less bareword** `name` is *always* a variable/constant read, resolving
lexical-local → enclosing scope → global. It is **never** an implicit getter send. A
getter send is written `self.name`, with the receiver.

Field access (`_name`) is unchanged: always the receiver's slot, per the existing
sigil rule.

### The three surface forms

| Written | Means | Resolution |
|---|---|---|
| `_name` / `_name = v` | receiver field read/write | field slot (sigil, unchanged) |
| `name` (paren-less) | value read | local → enclosing → global; **never** a self-send |
| `name(args)` | call | local callable if bound, else `self.name(args)` |
| `self.name` / `self.name(args)` | explicit self-send | send, as today |

## Resolution algorithm

At compile time, per call site `name(args)` inside a method, with no types required:

```
resolve_call(name, args, scope):
    if scope.has_local(name):            # param, let/var, block param, in lexical reach
        emit  LoadLocal(name); CallValue(args)      # invoke the bound callable
    else:
        emit  LoadSelf; Send(name, args)             # self.name(args)
```

The rule is **lexical and monotone**: whether `name` is a local is decided by the
enclosing binding structure, which the compiler already tracks for closures. No
class-member table is consulted (Phalcom is dynamically typed; the full method set
of `self` is not statically known — methods are inherited, DNU-synthesized, or added
at runtime). The self-send branch is therefore the *default*, not a fallback that
requires proving `name` is a method.

### Scope of the rule

- **Instance method body.** `self` is the receiver; the self-send branch targets it.
- **Static / class-side method body.** `self` is the class; `foo()` self-sends to the
  class (calls a sibling class-side method). Same rule, different `self`.
- **Block inside a method.** A block inherits the enclosing method's `self`
  ([functions.md](../functions.md)), so `foo()` in a block obeys the same two-tier
  rule; a block *parameter* named `foo` is a local and wins (tier 1).
- **Module top level (no enclosing method).** There is no `self`. `name(args)`
  resolves local → global only; an unresolved call is a compile error, not a
  self-send.

### Why parens are load-bearing

The rule is safe **only** because parentheses distinguish a *call* from a *read*. In
a language whose getters are paren-less (`order.isCancellable`, `self.subtotal`),
making the bareword `subtotal` mean `self.subtotal` would make `let x = subtotal`
ambiguous: local read, or hidden self-send? That is Ruby's local-variable-versus-
method swamp, and its downstream warts (shadowing action-at-distance, the `self.x =`
setter exception). Phalcom avoids it categorically: the *only* form that elides self
is the one parens already mark as a call. The paren-less bareword carries no such
mark and so is reserved, permanently, for reads.

### Interaction with fields and setters (the Ruby wart Phalcom dodges)

Ruby cannot elide self for assignment: `x = v` always creates a local, so a setter
must be written `self.x = v`. Phalcom never has this problem because **state is
`_x`**: you write `_x = v` to write the field, and that is already implicit-self and
unambiguous. `x = v` in a Phalcom method is a plain local binding and was never meant
to reach a setter. The elision this note adds is calls-only; assignment semantics are
untouched.

## Bytecode & dispatch

- **No new opcode, no representation cost.** The self-send branch emits the same
  `LoadSelf` + `Send` a written `self.name(args)` already produces; `self` is frame
  slot 0. `name(args)` and `self.name(args)` compile to *identical* bytecode. The
  local branch is the existing load-local + call-value path.
- **Dispatch identity is unaffected.** Method lookup keys on `name + arity + kind`
  ([dispatch.md](../experimental/dispatch.md)). `self` is the *receiver*, not an
  argument, so eliding `self.` changes neither the selector nor the arity: `foo()`
  and `self.foo()` perform the *same* lookup. The crown-jewel **identity-dispatch ⊗
  implicit-self** hazard — where elision that varies effective arity misses the
  defined method — **does not apply**, because receiver elision drops no argument.
- **Late binding preserved.** The self-send branch is a virtual send: it walks the
  receiver's class chain and may reach an inherited method or `doesNotUnderstand`,
  exactly as `self.foo()` does. Elision is notation, not an early bind.

## Diagnostics

Because Phalcom cannot statically enumerate `self`'s methods, an unresolved bare call
**cannot be a hard error** — `wihtdraw()` might be inherited, DNU-handled, or
runtime-installed. To recover most of the compile-time catch without lying about what
the compiler knows, emit a **soft diagnostic** (warning, not error):

- **`self-send-unresolved`** — `name()` self-sends but no method `name/arity` is
  visible on this class or its *statically-known* superclasses. Advisory; suppressed
  when the class has a `doesNotUnderstand` or a Dispatch-tier `@delegate`
  ([decorators.md](decorators.md)) that could legitimately absorb it.
- **`local-shadows-selector`** — a local binding `name` shadows a same-class selector
  `name`, so `name()` now calls the local, not the method. Flags the exact
  action-at-distance Ruby suffers silently.

Both are lints, off the hard-error path, so dynamic idioms (DNU, mixins, runtime
`defineMethod`) are never rejected.

## Examples

```phalcom
class Account {
  var _balance

  @constructor
  new(balance:) { _balance = balance }

  withdraw(amount) {
    ensureOpen()                    // -> self.ensureOpen()  (no local `ensureOpen`)
    _balance = _balance - amount    // field write (sigil), unchanged
    return self
  }

  ensureOpen() {
    isOpen().ifFalse { AccountClosed.new().raise() }   // isOpen() -> self.isOpen()
  }

  isOpen => _balance >= 0           // getter (paren-less body)

  report(fmt) {
    let money = { v => "$\(v)" }    // local closure bound to `money`
    money(_balance)                 // tier 1: `money` IS a local -> calls the closure
    format(_balance)                // tier 2: no local `format` -> self.format(_balance)
    return self.isOpen              // getter send: STAYS explicit (paren-less)
  }

  format(v) { … }
}
```

Static-side self:

```phalcom
class Order {
  static place(draft) {
    validate(draft)                 // -> self.validate(draft), self == the class Order
    return Order.new(draft)
  }
  static validate(draft) { … }
}
```

Shadowing lint in action:

```phalcom
class Grid {
  size => _cells.count
  resizeTo(size) {                  // parameter `size` shadows the getter selector
    let n = size                    // reads the PARAMETER (paren-less = read)
    // size()  would be tier-1 local `size` — but `size` is not callable -> error
    return self.size                // explicit self-send to reach the getter
  }
}
```

## Hazards

- **Typo becomes a runtime DNU.** `wihtdraw()` self-sends and fails at runtime, not
  compile time — the price of implicit self in a language with no static member set.
  Mitigated, not eliminated, by the `self-send-unresolved` lint.
- **Silent shadowing.** Introducing a local named after a sibling selector silently
  redirects `name()` from the method to the local. The `local-shadows-selector` lint
  surfaces it; without heeding the lint it is genuine action-at-distance.
- **Global callables are unreachable by bare call.** By rule, a bare call in a method
  is local-or-self, *never* global — so a top-level function `helper` cannot be
  invoked as `helper()` inside a method (that self-sends). Call it via an explicit
  receiver, or bind it to a local first (`let h = helper; h()`). Deliberate: it keeps
  resolution two-tier and stops a global from shadowing a method.
- **Reader locality.** At a glance `foo()` could be a local closure (tier 1) or a
  self-send (tier 2); the reader must know the local bindings to tell. This is the
  ergonomic cost of elision and is why the paren-less read form is *not* also elided —
  one ambiguous form is the budget, not two.

## What this precludes

- **Implicit-self for paren-less getters.** Reserved out **permanently**. Once
  `foo()` resolves local-else-self, users will expect `foo` to do likewise; admitting
  that is Ruby's swamp (the ambiguity this design exists to avoid). The paren-less
  bareword is committed, for all time, to mean a read. Adopting this note *is*
  deciding that — it is not deferrable to later.
- **A global-function bare-call fallback.** Adding globals as a third resolution tier
  later would retroactively change the meaning of existing `name()` self-sends the day
  a matching global is introduced (global-shadows-method). The two-tier rule is fixed.
- **Hard "unknown method" errors.** The dynamic object model forecloses turning
  `self-send-unresolved` into a compile error without a whole-program closed-world
  assumption Phalcom does not make (inheritance, DNU, runtime `defineMethod`).

## Open questions

| # | Question |
|---|---|
| S-1 | Is `self-send-unresolved` on by default, or opt-in via a strict/`--lint` mode? Default-on catches typos; default-off avoids noise on DNU-heavy classes. |
| S-2 | Does `local-shadows-selector` fire for *inherited* selectors too, or only selectors defined on the immediate class? Inherited widens the check but needs the statically-known super chain. |
| S-3 | Should a bare call whose `name` resolves to a **non-callable** local (`let size = 3; size()`) be a compile error (tier-1 chosen, not callable) or fall through to a tier-2 self-send? Erroring is stricter; falling through is more forgiving but reintroduces a hidden precedence. |
| S-4 | May the paren-less bareword ever be elided in a **future** typed mode, where the compiler *does* know the member set (as Java/Scala safely do)? Or is the paren-less-is-read commitment absolute regardless of a later type layer? |
