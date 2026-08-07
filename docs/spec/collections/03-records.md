# Record Specification

## 1. Definition

A Record is a structural value consisting only of labeled fields.

```text
Record = OrderedMap<LabelKey, Value>
```

Unlike a Tuple, a Record has no positional lane.

## 2. Literal grammar

Conceptual grammar:

```text
RecordLiteral ::= "{" RecordEntries? "}"
RecordEntries ::= RecordEntry ("," RecordEntry)* ","?
RecordEntry ::= LabelSyntax ":" Expression
              | "[" Expression "]" ":" Expression
```

Examples:

```phalcom
const user = {
  name: "Ada",
  age: 36
}
```

```phalcom
const operations = {
  +: family,
  +(): unary,
  +(_): binary
}
```

## 3. Keys

**RATIFIED:** Records may use Symbol and Selector keys.

```phalcom
operations[#+]
operations[#+()]
operations[#+(_)]
```

Identifier-symbol keys MAY support member access:

```phalcom
user.name
user.age
```

Computed lookup remains canonical.

## 4. Duplicate fields

Record field keys MUST be unique.

```phalcom
{
  name: "first",
  [#name]: "second"
}
// error: duplicate record field #name
```

No source-order override exists in a Record literal.

## 5. Record Type interpretation

In a type-consuming context:

```phalcom
type ConnectionConfig = {
  host: String,
  port: Int
}
```

produces an exact structural Record Type.

**PROVISIONAL:** Record Types are exact by default. Additional fields require an explicit open-record mechanism, which is not yet ratified.

## 6. Structural kind preservation during capture

**RATIFIED:** A rest capture preserves the structural kind prescribed by its annotation.

```phalcom
method(**config: ConnectionConfig) {
  config.host
  config.port
}
```

The local value is a `ConnectionConfig` Record rather than a generic labeled Tuple.

Likewise, a Tuple annotation produces a Tuple.

## 7. Record embedding into call packs

A Record embeds as a labeled-only pack:

```text
embedRecord(R) = ⟨[], fields(R)⟩
```

Therefore:

```phalcom
target(**record)
```

is valid when every Record key is a legal `CallLabel`.

Under the three-operator model:

```phalcom
target(***record)
```

has the same outgoing contribution because the Record's positional projection is empty.

```phalcom
target(*record)
```

MUST fail rather than silently contribute zero positionals.

## 8. Call-label compatibility

**PROVISIONAL:** Selector-valued Record keys are valid structurally but not yet legal call labels.

```phalcom
const operations = {
  +(_): handler
}

target(**operations)
// provisional error: #+(_) is not a legal call label
```

This restriction is a dispatch-system boundary, not a Record restriction.

## 9. Record spreading and merging

**RATIFIED:** Value spread syntax is not legal in Record construction.

```phalcom
{
  base: value,
  **additional
}
// invalid
```

Record composition MUST use explicit operations.

Possible APIs:

```phalcom
left.mergedWith(right)
left.mergedWith(right, onConflict: #error)
left.overridingWith(right)
```

The default conflict behavior and naming remain **OPEN**. Implicit source-order overriding MUST NOT be inferred from call-spread semantics.

## 10. Record mutability

**OPEN:** Whether the core `Record` is deeply immutable, shallowly immutable, or a fixed-shape mutable value has not been ratified.

This suite recommends:

- Record shape is immutable.
- Field values are write-once for ordinary Record values.
- Dynamic mutable labeled storage belongs to `Map` or a dedicated mutable record object.

This recommendation is provisional and must be reviewed against the existing object model.
