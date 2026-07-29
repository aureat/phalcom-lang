# Diagnostics Specification

## 1. Principles

Diagnostics SHOULD:

1. name the lane involved;
2. show the source form;
3. show the normalized interpretation when useful;
4. identify ownership conflicts;
5. suggest the nearest legal spelling;
6. distinguish structural-label errors from call-label errors;
7. never imply override semantics where none exist.

## 2. Positional binder with labeled schema

Source:

```phalcom
method(*args: (**: String))
```

Diagnostic:

```text
Invalid positional-rest annotation.

`*args` captures only the positional lane, but its annotation
specifies the open labeled lane `#**`.

Use:
    *args: (*: String)

or capture labels separately:
    **labels: (**: String)
```

## 3. Labeled binder with positional schema

Source:

```phalcom
method(**labels: (*: SomeType))
```

Diagnostic:

```text
Invalid labeled-rest annotation.

`**labels` captures only the labeled lane, but its annotation
specifies the open positional lane `#*`.

Use:
    **labels: (**: SomeType)

or its shorthand:
    **labels: SomeType
```

## 4. Mixed schema on one-lane binder

Source:

```phalcom
method(*args: (*: Int, **: String))
```

Diagnostic:

```text
Invalid positional-rest annotation.

`*args` owns only the positional lane, but the annotation also
specifies the labeled lane.

Capture both lanes with:
    ***arguments: (*: Int, **: String)

or split the captures:
    *args: Int,
    **labels: String
```

## 5. Mixing split and complete capture

Source:

```phalcom
method(*args: Int, ***remaining: P)
```

Diagnostic:

```text
Conflicting rest-capture modes.

A declaration may use split rest capture (`*` and `**`) or one
complete rest capture (`***`), but not both.
```

## 6. Mixing split and complete expansion

Source:

```phalcom
target(*prefix, ***arguments)
```

Diagnostic:

```text
Conflicting pack-expansion modes.

A call may use lane-specific expansion (`*` and `**`) or complete
pack expansion (`***`), but not both.
```

## 7. Duplicate call label

Source:

```phalcom
target(timeout: first, **options)
```

Where `options` contains `#timeout`.

Diagnostic:

```text
Duplicate call label `#timeout`.

The label is supplied explicitly and by `**options`.
Call expansion never overrides an existing label.
```

## 8. Invalid expansion operand

Source:

```phalcom
target(*record)
```

Diagnostic:

```text
Cannot apply positional expansion `*` to Record.

Record has no positional lane. Use `**record` or `***record`
to contribute its labeled fields.
```

Source:

```phalcom
target(***set)
```

Diagnostic:

```text
Cannot apply complete-pack expansion `***` to Set.

Set has no positional or labeled argument-pack lanes.
Convert it explicitly to an ordered Tuple before expansion.
```

## 9. Selector-valued call label

Source:

```phalcom
const operations = (+(_): handler)
target(**operations)
```

Provisional diagnostic:

```text
Selector `#+(_)` cannot currently be used as a call argument label.

Tuple and Record keys may be Symbols or Selectors, but call labels
are currently limited to Symbols.
```

## 10. Duplicate structural key

Source:

```phalcom
(
  *: Int,
  [#*]: String
)
```

Diagnostic:

```text
Duplicate tuple label `#*`.

The symbolic label `*:` and computed label `[#*]:` denote the same key.
```

## 11. Invalid ordering

Source:

```phalcom
target(timeout: duration, *args)
```

Diagnostic:

```text
Positional expansion cannot follow the labeled argument section.

Move `*args` before the first explicit labeled argument.
```

Source:

```phalcom
target(**labels, timeout: duration)
```

Diagnostic:

```text
Explicit labeled arguments cannot follow a `**` expansion.

Place all fixed labeled arguments before the first `**` expansion.
```

## 12. Multiple complete expansions

Source:

```phalcom
target(***first, ***second)
```

Provisional diagnostic:

```text
A call may contain at most one complete-pack expansion.

Use explicit pack-composition APIs before the call, or choose
lane-specific expansion where appropriate.
```

## 13. Residual-schema overlap

Source:

```phalcom
method(
  timeout: Duration,
  ***remaining: (timeout: Duration)
)
```

Diagnostic:

```text
Residual pack schema overlaps fixed parameter `#timeout`.

`***remaining` describes only arguments left after fixed parameters
are bound, so `#timeout` cannot appear as a required residual label.
```

## 14. Ordinary TupleType versus pack schema

Source:

```phalcom
type T = (*: Int)
```

Tooling SHOULD explain on request:

```text
`T` is an exact Tuple Type with literal label `#*`.
It becomes an open positional lane only when interpreted by a
callable-domain or rest-parameter pack context.
```
