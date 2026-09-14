# Core Execution and Value Semantics

[← Overview](README.md) · [Callable object model →](02-callable-object-model.md)

---

## 1. Expression-oriented execution

Every executable construct has a semantic value, although the grammar may restrict where particular constructs may appear.

This rule does not require the implementation to materialize every value. A compiler may distinguish value-needed and value-discarded contexts and omit an otherwise unobservable Unit value.

---

## 2. Brace-delimited lexical scope

Every brace-delimited executable code block establishes lexical scope.

Bindings declared in a block are scoped to that block and its nested lexical regions according to the ordinary capture rules.

A brace-delimited block is not automatically a Closure. Closure creation is specified separately in [Callable object model](02-callable-object-model.md).

---

## 3. Method and Closure body results

Ordinary Methods and Closures use final-expression semantics.

```phalcom
answer {
    42
}
```

returns `42`.

```phalcom
const f = || {
    const x = 20
    x + 22
}
```

returns `42` when called.

An empty Method or Closure returns Unit:

```phalcom
noop {
}
```

returns:

```phalcom
()
```

The retired expression-body syntax using `=>` is not part of the language. Write brace bodies:

```phalcom
double(_ value) {
    value * 2
}
```

not an expression-bodied `=>` form.

---

## 4. Assignment

An assignment evaluates its right-hand side exactly once, performs the assignment, and evaluates to Unit.

```phalcom
const result = (x = 42)
```

produces:

```phalcom
result == ()
```

This rule applies to local, captured, field, static-field, subscript-set, and property/setter assignment syntax.

If the underlying setter Method returns another value, assignment syntax discards that value and produces Unit.

Assignment value semantics are therefore independent of the implementation convention of the underlying store operation.

---

## 5. Declarations

A variable declaration evaluates to Unit.

```phalcom
const result = {
    const x = compute()
}
```

The block's final construct is the declaration, so the block evaluates to:

```phalcom
()
```

The initializer is still evaluated exactly once.

Unless another specification explicitly defines a declaration form as value-producing, executable declarations follow this Unit rule.

---

## 6. Conditional values

### 6.1 Two-armed `if`

An `if` with an `else` evaluates to the selected branch's value.

```phalcom
const sign =
    if value < 0 {
        #negative
    } else {
        #nonnegative
    }
```

### 6.2 One-armed `if`

An `if` without `else` always evaluates to Unit.

```phalcom
const result =
    if ready {
        start()
    }
```

`result` is `()` whether or not the branch executes.

The value of the branch body is discarded by the one-armed conditional.

This avoids an implicit `T | Unit` result merely because an `else` is absent.

---

## 7. Loops

A source-level loop is a lexical control-flow construct compiled directly as a loop. It is not semantically defined by sending a looping message to Closure objects.

The normal result of a loop is Unit.

```phalcom
const result =
    while condition {
        work()
    }
```

If the loop ends normally:

```phalcom
result == ()
```

Per-iteration body values are discarded.

### 7.1 `break`

Bare `break` is equivalent to:

```phalcom
break ()
```

A `break value` causes the enclosing loop to evaluate to `value`.

```phalcom
const found =
    while search.hasNext {
        const item = search.next()

        if matches(item) {
            break item
        }
    }
```

If `break item` executes, `found` is that item. If the loop ends normally, `found` is `()`.

The expression supplied to `break` is evaluated exactly once.

### 7.2 Nested loops

`break` and `continue` apply to the innermost lexically enclosing loop in the same Method or Closure activation.

A `break` inside a nested real Closure does not target a loop outside that Closure.

```phalcom
while condition {
    const f = || {
        break 10       // invalid: no loop in this Closure
    }
}
```

### 7.3 `continue`

`continue` transfers control to the next iteration of the innermost lexical loop and does not independently determine the loop result.

---

## 8. `return`

`return` exits only the current Method or Closure activation.

There is no implicit non-local return.

```phalcom
return
```

is exactly equivalent to:

```phalcom
return ()
```

and:

```phalcom
return value
```

returns `value` from the current activation.

### 8.1 Closure-local return

```phalcom
make {
    const f = || {
        return 10
    }

    f()
    20
}
```

`return 10` exits `f`, not `make`. `make` returns `20`.

An escaping Closure containing `return` remains valid after the Method that created it has returned. There is no dead-home-frame condition.

### 8.2 Terminating expressions

At the point where they appear, `return`, `break`, `continue`, and `throw` transfer control rather than completing normally. A type system may model appropriate such positions as `Never`.

---

## 9. Constructors and initializers

`@constructor` is conceptually a compiler generator/annotation over ordinary Methods.

A source constructor declaration corresponds to:

1. a generated class-side factory Method;
2. a generated or hidden instance initializer Method.

The factory:

1. allocates or obtains the instance according to constructor/inheritance rules;
2. invokes the initializer on that instance;
3. discards the initializer's return value;
4. returns the allocated instance.

Conceptually:

```text
@constructor declaration
        ↓ compiler generation
class-side factory
        ↓
allocate instance
        ↓
ordinary instance initializer Method
        ↓
discard initializer result
        ↓
return instance
```

### 9.1 Initializer Method semantics

The generated initializer is an ordinary Method at runtime. Purely as a Method, its final expression has ordinary Method result semantics.

For example, an initializer body whose final expression evaluates to `42` would, when considered as an ordinary exact Method activation, produce `42`.

The generated constructor factory does not expose that value; it discards it and returns the allocated instance.

### 9.2 Explicit `return` in source constructors

Because a source Method marked `@constructor` is used as initialization code for a generated factory, the compiler imposes one additional source restriction:

```phalcom
return value
```

inside a source `@constructor` body is a compile-time error.

Bare:

```phalcom
return
```

is permitted and returns `()` early from the generated initializer. The factory still returns the allocated instance.

A class-side factory that intentionally wants to return some value other than a new instance should be an ordinary class-side Method, not `@constructor`.

---

## 10. Lexical name resolution and implicit `self`

Within an executable lexical context, a bare callable-looking name is resolved lexically before falling back to an implicit receiver send.

Conceptually the priority is:

1. local parameter or local declaration;
2. captured lexical binding;
3. applicable module/global binding;
4. otherwise implicit `self` message resolution where `self` exists.

Therefore:

```phalcom
helper()
```

means application of the lexical `helper` value if such a binding exists.

Only when no lexical/global binding claims the name does it become equivalent to:

```phalcom
self.helper()
```

in a context that supports implicit `self`.

---

## 11. Trailing Closure syntax

Canonical Closure syntax uses `||` or `|parameters|`, but a zero-argument Closure may omit `||` when used as a trailing brace body after an eligible method send.

```phalcom
items.each {
    consume()
}
```

is contextual sugar for:

```phalcom
items.each || {
    consume()
}
```

A parameterized trailing Closure remains explicit:

```phalcom
users.any where: |user| {
    not user.expired
}
```

### 11.1 Braces are not general Closure literals

The trailing form does not make:

```phalcom
{ ... }
```

a general Closure expression.

Outside the trailing-send context, braces retain their ordinary grammatical meanings, including collection-literal forms.

When a collection literal is intended as an argument immediately after a send, parentheses disambiguate it:

```phalcom
receiver.accept({ key: value })
```

---

## 12. Unit versus None

Use Unit for:

- empty executable bodies;
- assignment results;
- declaration results;
- one-armed `if`;
- normal loop termination;
- bare `break`;
- bare `return`;
- successful side-effect operations whose protocol defines no payload.

Use `None` only where the program means absence, including optional missing values or another protocol explicitly defined in terms of absence.

`None` must not be used as a generic no-result value.
