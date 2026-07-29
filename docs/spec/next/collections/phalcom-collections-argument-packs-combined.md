# Combined Phalcom Collections and Argument-Pack Specification

> This convenience file concatenates the normative suite. The split files remain authoritative for review and maintenance.


---

<!-- BEGIN 00-index.md -->

# Phalcom Collections and Argument-Pack Specification Suite

**Status:** Draft normative candidate for systematic language review  
**Scope:** Tuples, records, sets, argument packs, rest/spread operators, callable domains, reflection, normalization, diagnostics, and conformance  
**Audience:** Language architects, parser/compiler implementers, VM/runtime implementers, standard-library authors, tooling authors, and specification reviewers

## 1. Purpose

This suite converts the tuple/record/argument-pack design discussion into a reviewable specification. It is intentionally split into small documents so another agent can evaluate each semantic layer independently and then check cross-document consistency.

Every rule is tagged with one of four statuses:

- **RATIFIED** — explicitly accepted during the design discussion.
- **AMENDED** — explicitly replaces an earlier accepted rule.
- **PROVISIONAL** — recommended here to close a gap, but not yet explicitly ratified.
- **OPEN** — deliberately unresolved and listed for review.

Normative keywords **MUST**, **MUST NOT**, **SHOULD**, and **MAY** use their usual specification meanings.

## 2. Reading order

1. [`01-foundations-and-notation.md`](01-foundations-and-notation.md)
2. [`02-tuples.md`](02-tuples.md)
3. [`03-records.md`](03-records.md)
4. [`04-sets.md`](04-sets.md)
5. [`05-argument-packs.md`](05-argument-packs.md)
6. [`06-rest-spread-and-pack-operators.md`](06-rest-spread-and-pack-operators.md)
7. [`07-callable-domains.md`](07-callable-domains.md)
8. [`08-reflection-and-type-values.md`](08-reflection-and-type-values.md)
9. [`09-normalization-subtyping-and-satisfaction.md`](09-normalization-subtyping-and-satisfaction.md)
10. [`10-diagnostics.md`](10-diagnostics.md)
11. [`11-conformance-tests.md`](11-conformance-tests.md)
12. [`12-open-questions-and-gap-analysis.md`](12-open-questions-and-gap-analysis.md)

## 3. Core model

A Phalcom call is represented abstractly as a two-lane pack:

```text
CallPack = ⟨PositionalLane, LabeledLane⟩
```

Where:

```text
PositionalLane = finite ordered sequence of values
LabeledLane    = finite insertion-ordered mapping from call labels to values
```

A Tuple is the principal value representation capable of carrying both lanes. A Record carries a labeled structural shape and embeds into a call pack with an empty positional lane. A Set is not a pack and has no lane semantics.

The operator family is **AMENDED** to be lane-uniform:

```text
*     positional lane
**    labeled lane
***   complete pack, preserving both lanes
```

The same lane meaning applies in three distinct operations:

```text
capture   incoming CallPack → local binding
expansion value             → outgoing CallPack contribution
type unpack Type            → callable-domain contribution
```

The grammar and direction differ, but the selected lane does not.

## 4. Principal decisions

### 4.1 First-class types

**RATIFIED:** Complete type expressions evaluate to immutable, reflective, first-class `Type` values. Implementations MAY intern, constant-fold, or represent them through compact handles.

### 4.2 Contextual tuple typing

**RATIFIED:** A tuple expression is an ordinary Tuple in value context and may be contextually interpreted as a structural Tuple Type in a type-consuming context.

```phalcom
const values = (Int, String)
values.class == Tuple

type PairType = (Int, String)
```

### 4.3 Ellipsis and repeated tails

**RATIFIED:** `...` is a first-class immutable `Ellipsis` singleton. In a tuple Type, a final `...` repeats the immediately preceding positional type zero or more times.

```phalcom
type Ints = (Int, ...)
type Requests = (Context, Request, ...)
```

### 4.4 Symbolic and selector-valued structural labels

**RATIFIED:** Tuple and Record syntax may use symbolic labels and selector-shaped labels.

```phalcom
const operations = (
  +: family,
  +(): unary,
  +(_): binary
)

operations[#+]
operations[#+()]
operations[#+(_)]
```

The exact call-label domain remains separate and is tracked as an open question.

### 4.5 Argument-pack type

**PROVISIONAL, REQUIRED FOR CONSISTENCY:** Pack interpretation MUST produce a reflective `ArgumentPackType`, distinct from an ordinary exact `TupleType`.

```phalcom
type LiteralKeys = (*: Int, **: String)
// Exact TupleType with literal keys #* and #**.

const callback: (*: Int, **: String) -> Result
// CallableType whose domain is an ArgumentPackType with open lanes.
```

The source expression may be identical, but the consuming type constructor and resulting Type object differ.

### 4.6 Duplicate labels

**RATIFIED:** Duplicate labels during call assembly are errors. Source order never provides implicit overriding.

```phalcom
target(x: 1, x: 2)              // error
target(**first, **second)        // error if both contain #x
target(***pack, x: 2)            // error if pack contains #x
```

### 4.7 Spread scope

**RATIFIED:** Value spread syntax is restricted to call argument lists. Tuple, Record, and Set construction use explicit APIs for composition.

```phalcom
target(*positionals, **labels)

(prefix, *values)       // invalid
{ base: value, **more } // invalid
```

## 5. Cross-document invariants

All documents MUST preserve these invariants:

1. A Tuple value has one positional lane followed canonically by one labeled lane.
2. A Record has only labeled structural entries.
3. A Set has no lanes and cannot be projected or expanded as a pack.
4. `*`, `**`, and `***` always select positional, labeled, and complete lanes respectively.
5. Capture, expansion, and type unpacking are distinct operations.
6. A rest binder cannot describe or capture a lane it does not own.
7. Complete `***` capture/expansion is mutually exclusive with split `*`/`**` forms in the same declaration or call.
8. Call assembly evaluates operand expressions exactly once, left to right.
9. Duplicate call labels always fail; no implicit override semantics exist.
10. Callable types classify accepted call packs, not local parameter-binding strategy.
11. Structural annotations are reflective and inert unless explicitly checked.
12. `TupleType` and `ArgumentPackType` are not equal merely because they originated from the same tuple source spelling.

## 6. Review deliverables

A systematic review should produce:

- a contradiction list;
- a parser ambiguity list;
- a runtime representation audit;
- a static-checking soundness audit;
- a reflection/equality audit;
- an error-diagnostic audit;
- a list of unratified provisional rules;
- acceptance-test additions for every discovered gap.


<!-- END 00-index.md -->


---

<!-- BEGIN 01-foundations-and-notation.md -->

# Foundations and Formal Notation

## 1. Semantic universes

This suite uses the following abstract domains:

```text
Value       runtime Phalcom value
Type        first-class reflective type value
Symbol      interned symbolic name
Selector    complete message selector
LabelKey    structural Tuple/Record key
CallLabel   key legal in a call's labeled lane
```

**RATIFIED:** Structural labels may include at least:

```text
LabelKey ::= Symbol | Selector
```

**PROVISIONAL:** Call labels remain narrower:

```text
CallLabel ::= Symbol
```

Selector-valued call labels are reviewed separately in the open-questions document.

## 2. Lane notation

A pack is written:

```text
P = ⟨p, l⟩
```

Where:

```text
p = [v₀, v₁, …, vₙ]
l = { k₀ ↦ v₀, k₁ ↦ v₁, …, kₘ ↦ vₘ }
```

The labeled lane preserves insertion order for reflection and deterministic call assembly, while lookup and duplicate detection use key identity.

Projection functions:

```text
π*(⟨p, l⟩)   = ⟨p, ∅⟩
π**(⟨p, l⟩)  = ⟨[], l⟩
π***(⟨p, l⟩) = ⟨p, l⟩
```

For source-level operators:

```text
*P    denotes π*(P)
**P   denotes π**(P)
***P  denotes π***(P)
```

The notation describes lane selection only. The surrounding grammar determines whether the operation is capture, expansion, or type unpacking.

## 3. Partial pack concatenation

Call assembly uses a partial operation `⊕`:

```text
⟨p₁, l₁⟩ ⊕ ⟨p₂, l₂⟩ = ⟨p₁ ++ p₂, l₁ ∪ l₂⟩
```

This operation is defined only when:

```text
keys(l₁) ∩ keys(l₂) = ∅
```

If labels overlap, assembly fails with a duplicate-label error.

Properties:

```text
identity:      P ⊕ ⟨[], ∅⟩ = P
associativity: (A ⊕ B) ⊕ C = A ⊕ (B ⊕ C), when all are defined
non-override:  duplicate keys never choose a winner
```

The operation is not commutative because positional ordering and labeled insertion order are observable.

## 4. Structural type notation

An exact Tuple Type is represented abstractly as:

```text
TupleType {
  positional: [T₀, T₁, …, Tₙ]
  labels: OrderedMap<LabelKey, Type>
  repeatedTail: Option<Type>
}
```

An exact Record Type is represented as:

```text
RecordType {
  fields: OrderedMap<LabelKey, Type>
}
```

An argument-pack type is represented as:

```text
ArgumentPackType {
  fixedPositionals: [T₀, T₁, …, Tₙ]
  openPositional: Option<Type>
  fixedLabels: OrderedMap<CallLabel, Type>
  openLabeled: Option<Type>
}
```

A Set Type is represented as:

```text
SetType {
  element: Type
  mutability: SetMutability
}
```

## 5. Satisfaction notation

```text
v ⊨ T
```

means value `v` satisfies type `T` under explicit reflective satisfaction.

```text
P ⊨ D
```

means call pack `P` is accepted by argument domain `D`.

Annotations do not automatically enforce either relation at runtime.

## 6. Contextual interpretation

A tuple expression `E` may be interpreted differently by a consuming context:

```text
C ⊢ E ⇝ X
```

Meaning:

> In context `C`, expression `E` is interpreted as semantic object `X`.

Relevant contexts:

```text
ValueContext
TupleTypeContext
RecordTypeContext
ArgumentPackContext
CallableDomainContext
PositionalRestContext
LabeledRestContext
CompleteRestContext
```

Example:

```text
ValueContext ⊢ (*: Int) ⇝ Tuple([#* ↦ Int])
TupleTypeContext ⊢ (*: Int) ⇝ TupleType(labels = {#* ↦ Int})
PositionalRestContext ⊢ (*: Int) ⇝ ArgumentPackType(openPositional = Int)
```

The source tuple remains ordinary. The consuming context determines the produced semantic Type.

## 7. Source preservation and normalization

Reflection SHOULD preserve:

```text
sourceForm       original syntax or source-range-backed representation
normalizedType   canonical semantic Type value
```

Two source forms MAY normalize to equal Type values:

```phalcom
*args: Int
*args: (*: Int)
```

Both normalize to a positional open-lane schema of `Int`.

Source preservation is diagnostic metadata and MUST NOT alter type equality.


<!-- END 01-foundations-and-notation.md -->


---

<!-- BEGIN 02-tuples.md -->

# Tuple Specification

## 1. Definition

A Tuple is a finite structural value with:

1. zero or more positional entries;
2. followed canonically by zero or more labeled entries.

```text
Tuple = ⟨PositionalLane, StructuralLabeledLane⟩
```

The structural labeled lane may use `LabelKey`, which is broader than the call-label domain.

## 2. Literal grammar

Conceptual grammar:

```text
TupleLiteral ::= "(" TupleEntries? ")"
TupleEntries ::= PositionalEntries ("," LabeledEntries)?
               | LabeledEntries
PositionalEntries ::= Expression ("," Expression)* ","?
LabeledEntries ::= LabeledEntry ("," LabeledEntry)* ","?
LabeledEntry ::= LabelSyntax ":" Expression
               | "[" Expression "]" ":" Expression
```

Once the first labeled entry appears, no ordinary positional entry may follow.

Valid:

```phalcom
()
(1,)
(1, 2)
(1, name: "Phalcom")
(name: "Phalcom", version: 1)
```

Invalid:

```phalcom
(name: "Phalcom", 1)
```

## 3. Unit and singleton tuples

**RATIFIED:** `()` is the empty Tuple and the `Unit` value.

```phalcom
().class == Tuple
().size == 0
```

A trailing comma distinguishes a one-element Tuple where necessary:

```phalcom
(Int,) -> Result
```

means a callable receiving one positional `Int`.

```phalcom
((Int, String),) -> Result
```

means a callable receiving one positional Tuple.

## 4. Symbolic labels

**RATIFIED:** Tokens legal as symbolic selector heads may be used as labels when followed by `:`.

```phalcom
const metadata = (
  *: positionals,
  **: labels,
  ?: optional,
  !: required,
  +: arithmetic,
  ==: equality
)
```

The colon disambiguates the label from an operator or spread form.

Canonical access uses key lookup:

```phalcom
metadata[#*]
metadata[#**]
metadata[#?]
metadata[#+]
metadata[#==]
```

## 5. Selector-valued labels

**RATIFIED:** Selector-shaped symbolic labels produce first-class `Selector` keys.

```phalcom
const operations = (
  +(_): add,
  -(): negate,
  ==(_): equal
)
```

This is shorthand for:

```phalcom
const operations = (
  [#+(_)]: add,
  [#-()]: negate,
  [#==(_)]: equal
)
```

The following keys are distinct:

```text
#+      Symbol
#+()    Selector
#+(_)   Selector
```

Therefore:

```phalcom
operations[#+(_)]
```

is not equivalent to:

```phalcom
operations[#+]
```

## 6. Duplicate labels

Tuple labels MUST be unique under key equality.

```phalcom
(
  +(_): first,
  [#+(_)]: second
)
// error: duplicate label #+(_)
```

Likewise:

```phalcom
(
  *: first,
  [#*]: second
)
// error: duplicate label #*
```

## 7. Tuple Type interpretation

**RATIFIED:** In a type-consuming context, a tuple expression may denote a structural Tuple Type.

```phalcom
type Pair = (Int, String)
type Named = (Int, name: String)
```

In ordinary value context, the same source syntax remains a Tuple of Type values:

```phalcom
const pairDescription = (Int, String)
pairDescription.class == Tuple
```

A dynamic conversion MAY be exposed:

```phalcom
const PairType = pairDescription.asType
```

The exact conversion protocol remains provisional.

## 8. Repeated positional tail

**RATIFIED:** A final `...` repeats the immediately preceding positional type zero or more times.

```phalcom
type Ints = (Int, ...)
```

Accepts:

```phalcom
()
(1,)
(1, 2, 3)
```

A fixed prefix may precede the repeated type:

```phalcom
type Requests = (Context, Request, ...)
```

This means one required `Context`, followed by zero or more `Request` values.

It does not repeat the entire preceding sequence.

```phalcom
type Values = (Int, String, ...)
```

means one required `Int`, followed by zero or more `String` values.

## 9. Labeled slots after repetition

**RATIFIED:** Fixed labels may follow a repeated positional tail.

```phalcom
type Command = (
  String,
  Int,
  ...,
  timeout: Duration
)
```

Meaning:

- one required positional `String`;
- zero or more positional `Int` values;
- one required labeled `timeout: Duration`.

## 10. Exactness

An ordinary Tuple Type is exact unless it contains an explicit repeated tail or is contextually interpreted as an open argument-pack schema.

```phalcom
type Exact = (Int, name: String)
```

The value must contain exactly one positional entry and exactly one `#name` label.

Additional positionals or labels do not satisfy `Exact`.

## 11. Tuple access

Canonical access forms:

```phalcom
value[index]
value[#label]
value[#+(_)]
```

Identifier labels MAY support member access:

```phalcom
value.name
```

Key lookup remains canonical because it works for all Symbol and Selector keys.

## 12. Tuple spreading

**RATIFIED:** Value spreading is not legal inside Tuple construction.

```phalcom
(prefix, *values) // invalid
```

Composition must use explicit operations whose collision and normalization rules are independently specified:

```phalcom
left.concatPositionals(right)
left.withLabels(right)
```

The exact standard-library API is **OPEN**.

## 13. Tuple and argument-pack distinction

A Tuple value may carry both lanes and may be used as the runtime representation of a pack. However:

```text
TupleType ≠ ArgumentPackType
```

For example:

```phalcom
type LiteralKeys = (*: Int, **: String)
```

is an exact Tuple Type with literal keys `#*` and `#**`.

But:

```phalcom
const callback: (*: Int, **: String) -> Result
```

uses the tuple expression as an argument-pack schema and creates an `ArgumentPackType` with open lanes.


<!-- END 02-tuples.md -->


---

<!-- BEGIN 03-records.md -->

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


<!-- END 03-records.md -->


---

<!-- BEGIN 04-sets.md -->

# Set Specification

## 1. Status

The set design was not deeply ratified in the preceding discussion. This document supplies a deliberately conservative normative candidate so the collection family can be evaluated together.

Rules in this document are **PROVISIONAL** unless explicitly marked otherwise.

## 2. Definition

A Set is a finite collection of unique values with membership semantics independent of insertion position.

```text
Set<T> = finite mathematical set of values satisfying T
```

A Set has:

- no positional lane;
- no labeled lane;
- no argument-pack interpretation;
- no duplicate elements under Set equality.

## 3. Construction

Until literal syntax is ratified, the normative construction form is:

```phalcom
const values = Set.new(1, 2, 3)
```

Duplicates collapse:

```phalcom
Set.new(1, 1, 2) == Set.new(1, 2)
```

Potential literal syntaxes remain **OPEN** because `{}` is naturally associated with Records and `#` already participates in Symbols and Selectors.

Candidate syntaxes for review:

```phalcom
set(1, 2, 3)
Set{1, 2, 3}
#{1, 2, 3}
```

No candidate is ratified by this suite.

## 4. Type syntax

The canonical Set Type is generic application:

```phalcom
Set<Int>
Set<String>
```

A Set Type is not tuple-shaped and is never contextually interpreted as a pack schema.

## 5. Equality and hashing

Set equality is extensional:

```text
A = B iff every member of A is in B and every member of B is in A
```

Iteration order MUST NOT participate in equality or hashing.

Every Set element MUST be hashable under the language's ordinary hash/equality contract.

## 6. Iteration order

**PROVISIONAL:** Core Set iteration order is unspecified and MUST NOT be relied upon for semantic correctness.

An implementation MAY preserve insertion order as a quality-of-implementation property, but this is not observable language semantics unless later ratified.

A separate `OrderedSet<T>` may provide ordering guarantees.

## 7. Mutability

Recommended split:

```phalcom
Set<T>        mutable unique collection
FrozenSet<T>  immutable hashable unique collection
```

This naming and split remain **OPEN**.

## 8. Operations

Minimum protocol:

```phalcom
values.contains(value)
values.add(value)
values.remove(value)
values.union(other)
values.intersection(other)
values.difference(other)
values.isSubsetOf(other)
values.isSupersetOf(other)
values.size
```

For immutable Sets, mutating operations return new Sets or are unavailable.

## 9. Spread

**RATIFIED BY GENERAL SPREAD SCOPE:** Value spread syntax is call-only and therefore unavailable in a future Set literal itself.

A constructor call may still use ordinary call expansion:

```phalcom
Set.new(*positionals)
```

Here `*` expands the positional lane into the call to `Set.new`; it is not a Set-specific spread operation. Its legality depends on `Set.new`'s callable domain.

No special Set-literal spread operator is defined. Explicit collection composition uses operations such as `union`.

## 10. Pack conversion

A Set MUST NOT be directly used with `*`, `**`, or `***` expansion because it has no stable positional or labeled lane.

```phalcom
target(*values)   // error: Set has no positional lane
target(**values)  // error: Set has no labeled lane
target(***values) // error: Set is not a pack
```

Explicit conversion is required:

```phalcom
target(*values.toTuple)
```

Because Set iteration order is unspecified, such conversion SHOULD require the caller to accept or establish an order.

## 11. Satisfaction

```phalcom
Set<Int>.satisfiedBy(Set.new(1, 2, 3))
// true
```

Satisfaction checks every member and handles cycles according to the general reflective satisfaction algorithm.

## 12. Open set questions

1. Literal syntax.
2. Mutability model.
3. Iteration-order guarantee.
4. Hashability of mutable Sets.
5. Variance of `Set<T>`.
6. Whether `Set.new(*positionals)` should be a preferred constructor pattern.
7. Whether an immutable Set should be named `FrozenSet`, `ImmutableSet`, or represented through `const` construction.


<!-- END 04-sets.md -->


---

<!-- BEGIN 05-argument-packs.md -->

# Argument-Pack Specification

## 1. Definition

An argument pack is a call-shaped structure with two lanes:

```text
ArgumentPack = ⟨PositionalLane, LabeledLane⟩
```

At runtime, a Tuple is the canonical value capable of preserving both lanes.

A Record embeds as a labeled-only pack. A Set is not a pack.

## 2. ArgumentPackType

**PROVISIONAL, REQUIRED FOR CONSISTENCY:** The type system MUST expose a distinct reflective type:

```phalcom
class ArgumentPackType is Type {
  fixedPositionals -> const List<Type>
  openPositional -> Option<Type>
  fixedLabels -> const Record<Symbol, Type>
  openLabeled -> Option<Type>
}
```

This is not an ordinary `TupleType`, even if it was constructed from tuple syntax.

```phalcom
type Literal = (*: Int)
// TupleType with exact literal label #*.

const callback: (*: Int) -> Result
// CallableType with ArgumentPackType(openPositional: Int).
```

## 3. Pack-schema interpretation

In an argument-pack context, tuple keys receive the following meanings:

```text
#*  open positional lane
#** open labeled lane
```

Example:

```phalcom
(
  Request,
  *: Bytes,
  timeout: Duration,
  **: Metadata
)
```

normalizes to:

```text
ArgumentPackType {
  fixedPositionals: [Request]
  openPositional: Bytes
  fixedLabels: { #timeout: Duration }
  openLabeled: Metadata
}
```

## 4. Exact and open domains

Exact:

```phalcom
(Int, name: String)
```

Open positional:

```phalcom
(*: Int)
```

Open labeled:

```phalcom
(**: String)
```

Open complete:

```phalcom
(*: Int, **: String)
```

Fixed plus open:

```phalcom
(
  Context,
  *: Request,
  format: Format,
  **: Metadata
)
```

## 5. Satisfaction

A call pack:

```text
P = ⟨[p₀, …, pₙ], {k₀ ↦ v₀, …, kₘ ↦ vₘ}⟩
```

satisfies domain `D` exactly when:

1. every fixed positional slot exists and satisfies its corresponding type;
2. additional positional values exist only if `openPositional` is present;
3. every additional positional value satisfies `openPositional`;
4. every fixed label exists and satisfies its corresponding type;
5. additional labels exist only if `openLabeled` is present;
6. every additional labeled value satisfies `openLabeled`;
7. labels are unique;
8. every key is a legal `CallLabel`.

## 6. Rest-binder lane ownership

The binders own these lanes:

```text
*parameter    { positional }
**parameter   { labeled }
***parameter  { positional, labeled }
```

Let `lanes(D)` be the lanes described by annotation domain `D`.

A binder is legal only when:

```text
lanes(D) ⊆ ownedLanes(binder)
```

Valid:

```phalcom
method(*args: (*: Int))
method(**labels: (**: String))
method(***arguments: (*: Int, **: String))
```

Invalid:

```phalcom
method(*args: (**: String))
method(**labels: (*: Int))
method(*args: (*: Int, **: String))
method(**labels: (*: Int, **: String))
```

## 7. Homogeneous shorthand

**RATIFIED:** A non-tuple rest annotation applies homogeneously to the lane or lanes owned by the binder.

```phalcom
*args: T
```

is shorthand for:

```phalcom
*args: (*: T)
```

```phalcom
**labels: T
```

is shorthand for:

```phalcom
**labels: (**: T)
```

```phalcom
***arguments: T
```

is shorthand for:

```phalcom
***arguments: (*: T, **: T)
```

## 8. Exact capture

**RATIFIED:** Tuple-shaped annotations are exact unless openness is explicit.

```phalcom
method(
  ***arguments: (
    Int,
    name: String
  )
)
```

accepts exactly one positional `Int` and exactly one labeled `name: String`.

```phalcom
method(10, name: "Phalcom")
```

Additional arguments fail.

## 9. Split capture

A method may split the residual lanes:

```phalcom
method(
  fixed: Fixed,
  *args: Int,
  option: Bool,
  **labels: String
) {
  ...
}
```

Binding order:

1. fixed positional parameters consume their positions;
2. `*args` captures remaining positionals;
3. fixed labeled parameters consume matching labels;
4. `**labels` captures remaining labels.

The exact parameter-order grammar is specified in the rest/spread document.

## 10. Complete residual capture

**RATIFIED:** `***arguments` is a terminal residual binder that captures both remaining lanes after fixed parameters are matched.

```phalcom
method(
  request: Request,
  timeout: Duration,
  ***remaining: P
)
```

Given:

```phalcom
method(
  requestValue,
  extra1,
  extra2,
  timeout: second,
  debug: true
)
```

bindings are conceptually:

```phalcom
request == requestValue
timeout == second
remaining == (extra1, extra2, debug: true)
```

The annotation describes the residual pack, not the original call.

## 11. Mutual exclusion of capture modes

**RATIFIED:** A declaration chooses one rest-capture mode.

Split mode:

```phalcom
method(*args: A, **labels: B)
```

Complete mode:

```phalcom
method(***arguments: P)
```

They MUST NOT be combined:

```phalcom
method(*args: A, ***remaining: P)      // error
method(**labels: B, ***remaining: P)   // error
```

## 12. Structural kind of local bindings

**RATIFIED:** Capture preserves the annotation's structural kind where meaningful.

```phalcom
method(**labels: String)
```

binds a labeled-only Tuple view.

```phalcom
method(**config: ConnectionConfig)
```

binds a `ConnectionConfig` Record.

```phalcom
method(***arguments: P)
```

binds one Tuple preserving both lanes, unless `P` is a labeled-only Record Type and the residual positional lane is empty.

## 13. Call-label legality

**PROVISIONAL:** An ArgumentPack's labeled lane contains Symbol keys only.

A Tuple or Record with Selector keys remains valid structurally but cannot be expanded into a call pack unless call labels are generalized.

```phalcom
const operations = (+(_): handler)
target(**operations)
// provisional error: Selector-valued call label
```

## 14. Reflection

A parameter reflection object SHOULD expose:

```phalcom
parameter.restMode
// #none | #positional | #labeled | #complete

parameter.sourceAnnotation
parameter.interpretedPackType
parameter.localBindingType
```

For:

```phalcom
method(***arguments: Int)
```

reflection conceptually reports:

```text
sourceAnnotation     = Int
interpretedPackType  = (*: Int, **: Int)
localBindingType     = ArgumentPackType(openPositional: Int, openLabeled: Int)
```


<!-- END 05-argument-packs.md -->


---

<!-- BEGIN 06-rest-spread-and-pack-operators.md -->

# Rest, Spread, and Pack Operators

## 1. Operator family

**AMENDED AND RATIFIED:**

```text
*     positional lane
**    labeled lane
***   complete pack
```

Each token is reserved in pack grammar and is not a freely overloadable arithmetic operator.

The lexer MUST prefer longest-token matching:

```text
*** before ** before *
```

## 2. Three operations, one lane algebra

### 2.1 Capture

```phalcom
*args
**labels
***arguments
```

select lanes from an incoming call pack and bind local variables.

### 2.2 Expansion

```phalcom
target(*value)
target(**value)
target(***value)
```

select lanes from a value and contribute them to an outgoing call.

### 2.3 Type unpacking

```phalcom
(*P,) -> R
(**P,) -> R
(***P,) -> R
```

select lanes from a tuple/pack Type and insert them into a callable domain.

## 3. Value expansion scope

**RATIFIED:** Value expansion is legal only in call argument lists.

Valid:

```phalcom
target(*positionals)
target(**labels)
target(***arguments)
```

Invalid:

```phalcom
(prefix, *values)
{ base: value, **fields }
Set{*values}
```

## 4. Split call expansion

**RATIFIED:** A call may contain multiple `*` and multiple `**` expansions.

```phalcom
target(
  fixed,
  *first,
  *second,
  timeout: duration,
  **defaults,
  **metadata
)
```

Rules:

1. Every expression is evaluated exactly once, left to right.
2. All ordinary positionals and `*` expansions precede the first explicit label or `**` expansion.
3. Once the first `**` appears, only further `**` expansions may follow.
4. `*` contributes only positional entries.
5. `**` contributes only labeled entries.
6. Duplicate labels fail.
7. No source-order overriding exists.

Canonical grammar:

```text
CallArguments(split) ::= PositionalItem* FixedLabel* LabeledExpansion*
PositionalItem ::= Expression | "*" Expression
LabeledExpansion ::= "**" Expression
```

## 5. Complete call expansion

**RATIFIED:** A call using `***` MUST NOT also use `*` or `**` expansion.

Valid:

```phalcom
target(***arguments)
```

Invalid:

```phalcom
target(*prefix, ***arguments)
target(***arguments, **metadata)
```

### 5.1 Multiplicity

**PROVISIONAL:** At most one `***` expansion may occur in a call.

Rationale: two complete expansions may both contribute positionals after the first has closed the positional section.

```phalcom
target(***first, ***second)
// provisional error
```

### 5.2 Explicit arguments around `***`

**PROVISIONAL RECOMMENDATION:** The canonical complete-expansion form is:

```phalcom
target(
  fixedPositionals,
  ***pack,
  fixedLabels
)
```

Rules:

1. explicit positionals may precede `***pack`;
2. no positional argument may follow `***pack`;
3. explicit labels may follow `***pack`;
4. duplicates between pack labels and explicit labels fail;
5. `***pack` is evaluated once at its source position.

This placement was proposed but not explicitly ratified and is listed for review.

## 6. Expansion operand requirements

### 6.1 `*value`

The operand MUST expose a positional lane.

Valid:

```phalcom
target(*(1, 2, 3))
```

Invalid:

```phalcom
target(*record)
target(*set)
```

### 6.2 `**value`

The operand MUST expose a labeled lane or embed as labeled structure.

Valid:

```phalcom
target(**tuple)
target(**record)
```

For a Tuple, positionals are ignored.

### 6.3 `***value`

The operand MUST be a pack-capable Tuple or a labeled-only Record embedding.

```phalcom
target(***tuple)
target(***record)
```

A Record contributes an empty positional lane.

## 7. Rest parameter ordering

### 7.1 Split mode

**RATIFIED:** Parameter declarations follow Python-like lane ordering:

```text
fixed positionals
*positionalRest
fixed labeled parameters
**labeledRest
```

Example:

```phalcom
log(
  category: Symbol,
  *values: Any,
  format: Format,
  **fields: LogValue
)
```

`**fields` MUST be final.

### 7.2 Complete mode

**RATIFIED:** `***remaining` is terminal and may follow fixed parameters.

```phalcom
method(
  request: Request,
  timeout: Duration,
  ***remaining: P
)
```

No parameter follows `***remaining`.

### 7.3 Mutual exclusion

Split and complete modes MUST NOT coexist in one declaration.

## 8. Type-level unpack expressions

**RATIFIED:** Type-level unpacking follows the same lane meanings.

For:

```phalcom
type P = (
  Request,
  timeout: Duration
)
```

then:

```phalcom
(*P,) -> R
```

normalizes to:

```phalcom
(Request) -> R
```

```phalcom
(**P,) -> R
```

normalizes to:

```phalcom
(timeout: Duration) -> R
```

```phalcom
(***P,) -> R
```

normalizes to:

```phalcom
(Request, timeout: Duration) -> R
```

## 9. First-class pack projections

**RATIFIED IN PRINCIPLE:** An unpack/projection expression may be stored when observed.

```phalcom
const expansion = ***P
```

The runtime value SHOULD be a reflective immutable descriptor:

```phalcom
PackProjection.new(
  mode: #complete,
  operand: P
)
```

In a pack-consuming context, the compiler/VM MAY consume the projection without allocating the descriptor.

The exact public class name and API are **OPEN**.

## 10. Evaluation and failure timing

Call-argument expressions are evaluated left to right.

A dynamically invalid expansion operand fails when its contribution is assembled.

```phalcom
target(
  sideEffectA(),
  *invalidValue,
  sideEffectB()
)
```

`sideEffectA()` occurs before the expansion error. `sideEffectB()` does not occur if assembly fails first.

Static analysis MAY diagnose invalid operands earlier when their types are known.

## 11. Duplicate labels

All call contributions participate in one duplicate check:

```phalcom
target(x: 1, **(x: 2))
target(**first, **second)
target(***pack, x: 2)
```

Each fails if `#x` appears more than once.

Duplicate detection is based on `CallLabel` identity, not textual spelling.


<!-- END 06-rest-spread-and-pack-operators.md -->


---

<!-- BEGIN 07-callable-domains.md -->

# Callable-Domain Specification

## 1. Definition

A callable Type consists of:

```text
CallableType = ArgumentPackType → ResultType
```

The domain classifies accepted call packs. It does not prescribe local parameter names or whether an implementation uses split or complete rest capture.

## 2. Tuple-shaped domain syntax

**RATIFIED:** Callable domains always use tuple syntax.

```phalcom
() -> R
(Int,) -> R
(Int, String) -> R
(Int, timeout: Duration) -> R
(***P,) -> R
```

There is no `Int -> R` shorthand.

## 3. One argument versus multiple arguments

```phalcom
(Int,) -> R
```

accepts one positional `Int`.

```phalcom
((Int, String),) -> R
```

accepts one positional Tuple.

```phalcom
(Int, String) -> R
```

accepts two positional arguments.

## 4. Open lanes

Callable-domain context interprets reserved tuple keys as open lanes:

```phalcom
(*: Int) -> R
(**: String) -> R
(*: Int, **: String) -> R
```

With fixed entries:

```phalcom
(
  Request,
  *: Bytes,
  timeout: Duration,
  **: Metadata
) -> Response
```

## 5. Type-level pack unpacking

Given:

```phalcom
type P = (
  Request,
  timeout: Duration
)
```

then:

```phalcom
(*P,) -> R
```

projects only `P`'s positional lane.

```phalcom
(**P,) -> R
```

projects only `P`'s labeled lane.

```phalcom
(***P,) -> R
```

preserves the complete domain.

This is the canonical form for generic forwarding.

```phalcom
forward<P: Tuple, R>(
  callable: (***P,) -> R,
  ***arguments: P
) -> R {
  return callable(***arguments)
}
```

## 6. Arbitrary callable shorthand

**AMENDED:** Earlier discussion equated `(...) -> R` with a complete Tuple unpack using `*`. Under the three-operator model it MUST normalize to complete unpack:

```phalcom
(...) -> R
```

is sugar for:

```phalcom
(***Tuple,) -> R
```

or an equivalent canonical `ArgumentPackType.any` representation.

The result accepts any well-formed call pack.

## 7. Method declaration elaboration

These declarations may expose the same callable domain:

```phalcom
method(
  *args: Int,
  **labels: String
) -> R
```

```phalcom
method(
  ***arguments: (
    *: Int,
    **: String
  )
) -> R
```

Both elaborate to:

```phalcom
(*: Int, **: String) -> R
```

Their local bindings differ, but callable type identity does not.

## 8. Exact domains

```phalcom
(Int, name: String) -> R
```

accepts exactly one positional `Int` and one labeled `name: String` unless defaults or optional-domain expansion explicitly alter acceptance.

## 9. Defaults and optional parameters

**OPEN:** Default values create multiple accepted call shapes.

Example:

```phalcom
connect(host: String, port: Int = 443)
```

Possible models:

1. callable domain is a union of `(host: String)` and `(host: String, port: Int)`;
2. domain slots carry optionality metadata;
3. reflection stores one declaration domain and a separate accepted-domain expansion.

This suite recommends model 3, but it is not ratified.

## 10. Callable subtyping

Let `Calls(D)` be the set of call packs accepted by domain `D`.

```text
(D₁ → R₁) <: (D₂ → R₂)
```

iff:

```text
Calls(D₂) ⊆ Calls(D₁)
R₁ <: R₂
```

Thus parameters are contravariant by accepted-call-set inclusion and results are covariant.

Example:

```phalcom
(*: Any) -> String
```

is a subtype of:

```phalcom
(*: Int) -> Any
```

because it accepts at least every positional-`Int` call and returns a more specific result.

## 11. Labels in subtyping

Labels are part of the accepted call shape.

```phalcom
(timeout: Duration) -> R
```

and:

```phalcom
(deadline: Duration) -> R
```

are not substitutable merely because their value types match.

Open labeled domains compare through accepted pack inclusion:

```phalcom
(**: Any) -> R
```

accepts at least the calls accepted by:

```phalcom
(**: String) -> R
```

## 12. Reflection

A `CallableType` SHOULD expose:

```phalcom
callable.domain -> ArgumentPackType
callable.result -> Type
callable.accepts(pack) -> Bool
```

Source spelling MAY be retained separately:

```phalcom
callable.sourceDomain
callable.normalizedDomain
```


<!-- END 07-callable-domains.md -->


---

<!-- BEGIN 08-reflection-and-type-values.md -->

# Reflection and First-Class Type Values

## 1. Type values

**RATIFIED:** Complete type expressions evaluate to immutable, reflective, first-class `Type` values.

```phalcom
const pairType = (Int, String).asType
const packType = ArgumentPackType.new(
  openPositional: Int,
  openLabeled: String
)
```

Implementations MAY intern and canonicalize Type values.

## 2. Required reflective hierarchy

Recommended public model:

```phalcom
@abstract
class Type {
  satisfiedBy(value) -> Bool
  normalized -> Type
  sourceForm -> Option<TypeSource>
}
```

```phalcom
class TupleType is Type {
  positionals -> const List<Type>
  labels -> const Record<LabelKey, Type>
  repeatedTail -> Option<Type>
}
```

```phalcom
class RecordType is Type {
  fields -> const Record<LabelKey, Type>
}
```

```phalcom
class SetType is Type {
  element -> Type
}
```

```phalcom
class ArgumentPackType is Type {
  fixedPositionals -> const List<Type>
  openPositional -> Option<Type>
  fixedLabels -> const Record<Symbol, Type>
  openLabeled -> Option<Type>
}
```

```phalcom
class CallableType is Type {
  domain -> ArgumentPackType
  result -> Type
}
```

## 3. TupleType versus ArgumentPackType

These Types MUST remain distinct:

```phalcom
type Literal = (*: Int)
const Domain = CallableType.new(
  domain: (*: Int),
  result: Result
)
```

Conceptually:

```phalcom
Literal.class == TupleType
Domain.domain.class == ArgumentPackType
Literal != Domain.domain
```

The shared source syntax does not imply semantic equality.

## 4. Source and normalized forms

Reflection SHOULD preserve both:

```text
source annotation
contextual interpretation
canonical normalized type
```

For:

```phalcom
method(*args: Int)
```

reflection may report:

```text
sourceAnnotation     = Int
interpretedPackType  = (*: Int)
localBindingType     = (Int, ...)
```

For:

```phalcom
const callback: (...) -> R
```

reflection may report:

```text
sourceDomain         = (...)
normalizedDomain     = ArgumentPackType.any
```

## 5. Equality and interning

Type equality is structural after normalization.

```phalcom
(*: Int) -> R == (*: Int) -> R
```

Equivalent generic unpacking normalizes identically:

```phalcom
type P = (Int, name: String)

(***P,) -> R == (Int, name: String) -> R
```

Implementations SHOULD intern canonical Type values so identity comparison may also succeed, but semantic correctness MUST rely on equality, not identity.

## 6. Satisfaction API

**RATIFIED IN PRINCIPLE:** Explicit predicates such as:

```phalcom
List<Int>.satisfiedBy([1, 2, 3])
```

are allowed. Annotations do not invoke them automatically.

Representative examples:

```phalcom
(Int, String).satisfiedBy((1, "a"))
Set<Int>.satisfiedBy(Set.new(1, 2))
(*: Int, **: String).asArgumentPackType.satisfiedBy(
  (1, 2, name: "x")
)
```

The exact contextual conversion API remains provisional.

## 7. Parameter reflection

A reflected parameter SHOULD expose:

```phalcom
parameter.name -> Symbol
parameter.label -> Option<Symbol>
parameter.position -> Int
parameter.sourceType -> Option<TypeSource>
parameter.type -> Option<Type>
parameter.restMode -> Symbol
parameter.packType -> Option<ArgumentPackType>
parameter.bindingType -> Option<Type>
parameter.attributes -> const List<Attribute>
```

## 8. Method reflection

A reflected method SHOULD expose:

```phalcom
method.selector -> Selector
method.parameters -> const List<Parameter>
method.callableType -> CallableType
method.returnType -> Option<Type>
method.typeParameters -> const List<TypeParameter>
```

The callable type represents accepted calls after normalization. Parameter objects preserve the implementation's binding strategy.

## 9. Inert annotations

Type annotations MUST NOT automatically:

- change dispatch;
- wrap values;
- reject calls at runtime;
- alter allocation layout;
- insert collection element checks;
- mutate Tuple, Record, Set, or pack values.

Static checking and explicit reflective satisfaction may use them.


<!-- END 08-reflection-and-type-values.md -->


---

<!-- BEGIN 09-normalization-subtyping-and-satisfaction.md -->

# Normalization, Subtyping, and Satisfaction

## 1. Purpose

This document defines canonical forms and semantic relations so syntax does not become the source of type identity.

## 2. Tuple normalization

### 2.1 Exact tuples

```phalcom
(Int, String)
```

normalizes to:

```text
TupleType(
  positionals = [Int, String],
  labels = {},
  repeatedTail = None
)
```

### 2.2 Repeated tails

```phalcom
(Context, Request, ...)
```

normalizes to:

```text
TupleType(
  positionals = [Context],
  labels = {},
  repeatedTail = Request
)
```

### 2.3 Labeled tuples

```phalcom
(Int, name: String)
```

normalizes to:

```text
TupleType(
  positionals = [Int],
  labels = { #name: String },
  repeatedTail = None
)
```

## 3. Pack-schema normalization

### 3.1 Homogeneous rest shorthand

```phalcom
*args: T
```

normalizes to:

```text
ArgumentPackType(openPositional = T)
```

```phalcom
**labels: T
```

normalizes to:

```text
ArgumentPackType(openLabeled = T)
```

```phalcom
***arguments: T
```

normalizes to:

```text
ArgumentPackType(
  openPositional = T,
  openLabeled = T
)
```

### 3.2 Reserved-key schemas

In an argument-pack context:

```phalcom
(*: A, **: B)
```

normalizes to:

```text
ArgumentPackType(
  openPositional = A,
  openLabeled = B
)
```

### 3.3 Exact complete schemas

```phalcom
(Int, name: String)
```

in complete-pack context normalizes to:

```text
ArgumentPackType(
  fixedPositionals = [Int],
  fixedLabels = { #name: String }
)
```

## 4. Type-unpack normalization

Given normalized pack type `P`:

```text
P = ⟨Fp, Op, Fl, Ol⟩
```

where `Fp` and `Fl` are fixed lanes and `Op` and `Ol` are optional open-lane types:

```text
*P    = ⟨Fp, Op, {}, None⟩
**P   = ⟨[], None, Fl, Ol⟩
***P  = P
```

Example:

```phalcom
type P = (Request, timeout: Duration)
```

```phalcom
(*P,) -> R
```

normalizes to:

```phalcom
(Request) -> R
```

```phalcom
(**P,) -> R
```

normalizes to:

```phalcom
(timeout: Duration) -> R
```

```phalcom
(***P,) -> R
```

normalizes to:

```phalcom
(Request, timeout: Duration) -> R
```

## 5. Tuple satisfaction

An exact tuple value satisfies an exact Tuple Type when:

1. positional counts match;
2. corresponding positional values satisfy corresponding types;
3. label sets match exactly;
4. corresponding labeled values satisfy corresponding types.

For repeated tails:

1. fixed positionals must match;
2. every remaining positional value satisfies the repeated-tail type;
3. labeled slots still match exactly.

## 6. Record satisfaction

A Record satisfies an exact Record Type when:

1. key sets match exactly;
2. each field value satisfies its field type;
3. key identity uses `LabelKey` equality.

Open Record Types are not defined in this suite.

## 7. Set satisfaction

A Set satisfies `Set<T>` when every element satisfies `T`.

```text
S ⊨ Set<T> iff ∀v ∈ S, v ⊨ T
```

For mutable Sets, satisfaction is a property of the current contents and does not impose future mutation guards.

## 8. Argument-pack satisfaction

For domain:

```text
D = ⟨Fp, Op, Fl, Ol⟩
```

and pack:

```text
P = ⟨p, l⟩
```

`P ⊨ D` when:

```text
|p| ≥ |Fp|
```

and every fixed positional matches, and:

```text
|p| = |Fp| if Op is None
```

otherwise every remaining positional satisfies `Op`.

For labels:

```text
keys(Fl) ⊆ keys(l)
```

all fixed labeled values match, and:

```text
keys(l) = keys(Fl) if Ol is None
```

otherwise every additional label value satisfies `Ol`.

## 9. Tuple subtyping

### 9.1 Exact immutable tuples

**PROVISIONAL:** Exact immutable Tuple Types are covariant slot-wise.

```text
(A₀, …, Aₙ) <: (B₀, …, Bₙ)
```

when:

```text
Aᵢ <: Bᵢ for every i
```

and label sets are equal with covariant value types.

If Tuples are mutable, invariance may be required. This depends on the collection mutability model and remains reviewable.

### 9.2 Repeated tails

An exact Tuple Type is a subtype of a repeated-tail Tuple Type when its fixed prefix matches and all remaining elements satisfy the repeated type.

```phalcom
(Int, Int, Int) <: (Int, ...)
```

## 10. Record subtyping

**OPEN:** Width subtyping for Records has not been ratified.

Candidate A — exact only:

```text
{a: A, b: B} is not a subtype of {a: A}
```

Candidate B — immutable width subtyping:

```text
{a: A, b: B} <: {a: A}
```

The candidate interacts with exact `**record` capture and must be resolved explicitly.

## 11. Set variance

**PROVISIONAL:** Mutable `Set<T>` is invariant. Immutable `FrozenSet<T>` may be covariant.

```text
Set<Dog> </: Set<Animal>
FrozenSet<Dog> <: FrozenSet<Animal>
```

## 12. Callable subtyping

Let `Calls(D)` be the accepted-pack set of domain `D`.

```text
Callable<D₁, R₁> <: Callable<D₂, R₂>
```

when:

```text
Calls(D₂) ⊆ Calls(D₁)
R₁ <: R₂
```

This definition handles fixed, open, positional, and labeled domains uniformly.

## 13. Join and inference

**PROVISIONAL:** Finite Tuple values infer exact Tuple Types.

```phalcom
(1, "a")
```

infers:

```phalcom
(Int, String)
```

Repeated-tail types arise from annotations, widening, or joins rather than initial finite-literal inference.

Possible joins:

```text
join((Int), (Int, Int, Int)) = (Int, Int, ...)
join((), (Int), (Int, Int))  = (Int, ...)
```

Heterogeneous joins remain **OPEN**.

## 14. Cycles and satisfaction

Reflective satisfaction MUST terminate for cyclic values and recursive Types.

Recommended algorithm:

```text
visited = Set<(objectIdentity, typeIdentity)>
```

Before descending, insert the pair. Re-visiting an active pair succeeds coinductively unless a contradiction has already been found.

## 15. Static soundness boundary

The type calculus may be sound relative to declared annotations, but annotations are inert at runtime.

Therefore:

```phalcom
method(*args: Int)
```

is not a runtime guarantee unless the call is statically checked or explicitly validated.

The specification claim is:

> Well-typed checked programs preserve argument-pack compatibility. Dynamically unchecked programs remain permissive.


<!-- END 09-normalization-subtyping-and-satisfaction.md -->


---

<!-- BEGIN 10-diagnostics.md -->

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


<!-- END 10-diagnostics.md -->


---

<!-- BEGIN 11-conformance-tests.md -->

# Conformance and Acceptance Tests

## 1. Test conventions

Examples use conceptual helpers:

```phalcom
assertEqual(actual, expected)
assertTrue(condition)
assertType(value, type)
assertCompileError(source, code: Symbol)
assertRuntimeError(block, code: Symbol)
```

A conforming implementation may express these through its existing test framework.

## 2. Tuple values

```phalcom
const empty = ()
assertEqual(empty.size, 0)

const pair = (1, "a")
assertEqual(pair[0], 1)
assertEqual(pair[1], "a")
```

## 3. Symbolic tuple labels

```phalcom
const value = (
  *: 1,
  **: 2,
  ?: 3,
  +: 4
)

assertEqual(value[#*], 1)
assertEqual(value[#**], 2)
assertEqual(value[#?], 3)
assertEqual(value[#+], 4)
```

## 4. Selector-valued labels

```phalcom
const operations = (
  +(): "unary",
  +(_): "binary"
)

assertEqual(operations[#+()], "unary")
assertEqual(operations[#+(_)], "binary")
```

Negative:

```phalcom
assertCompileError(
  `( +(_): first, [#+(_)]: second )`,
  code: #duplicateTupleLabel
)
```

## 5. Tuple Type exactness

```phalcom
type Pair = (Int, String)

assertTrue(Pair.satisfiedBy((1, "a")))
assertTrue(not Pair.satisfiedBy((1,)))
assertTrue(not Pair.satisfiedBy((1, "a", true)))
```

## 6. Repeated tails

```phalcom
type Ints = (Int, ...)

assertTrue(Ints.satisfiedBy(()))
assertTrue(Ints.satisfiedBy((1,)))
assertTrue(Ints.satisfiedBy((1, 2, 3)))
assertTrue(not Ints.satisfiedBy((1, "a")))
```

```phalcom
type Requests = (Context, Request, ...)
```

Acceptance MUST require exactly one `Context` followed by zero or more `Request` values.

## 7. Record values and types

```phalcom
const user = {
  name: "Ada",
  age: 36
}

assertEqual(user.name, "Ada")
assertEqual(user[#age], 36)
```

```phalcom
type User = {
  name: String,
  age: Int
}

assertTrue(User.satisfiedBy(user))
assertTrue(not User.satisfiedBy({ name: "Ada" }))
```

## 8. Set semantics

```phalcom
const values = Set.new(1, 1, 2)
assertEqual(values.size, 2)
assertTrue(values.contains(1))
assertTrue(values.contains(2))
```

```phalcom
assertTrue(Set<Int>.satisfiedBy(Set.new(1, 2, 3)))
assertTrue(not Set<Int>.satisfiedBy(Set.new(1, "a")))
```

## 9. Positional rest capture

```phalcom
capture(*args: Int) {
  return args
}

assertEqual(capture(), ())
assertEqual(capture(1, 2, 3), (1, 2, 3))
```

Equivalent explicit schema:

```phalcom
captureExplicit(*args: (*: Int)) {
  return args
}

assertEqual(captureExplicit(1, 2), (1, 2))
```

## 10. Labeled rest capture

```phalcom
capture(**labels: String) {
  return labels
}

assertEqual(
  capture(name: "Ada", mode: "strict"),
  (name: "Ada", mode: "strict")
)
```

## 11. Complete rest capture

```phalcom
capture(***arguments: Any) {
  return arguments
}

assertEqual(capture(), ())
assertEqual(capture(1, 2), (1, 2))
assertEqual(capture(name: "Ada"), (name: "Ada"))
assertEqual(
  capture(1, 2, name: "Ada"),
  (1, 2, name: "Ada")
)
```

## 12. Exact complete capture

```phalcom
capture(
  ***arguments: (
    Int,
    name: String
  )
) {
  return arguments
}

assertEqual(capture(1, name: "Ada"), (1, name: "Ada"))
```

Negative calls:

```phalcom
capture()
capture(1)
capture(1, 2, name: "Ada")
capture(1, name: "Ada", debug: true)
```

Each MUST be rejected by static checking or explicit runtime validation at a checked boundary.

## 13. Lane mismatch diagnostics

```phalcom
assertCompileError(
  `method(**labels: (*: Int)) {}`,
  code: #restLaneMismatch
)
```

```phalcom
assertCompileError(
  `method(*args: (**: String)) {}`,
  code: #restLaneMismatch
)
```

```phalcom
assertCompileError(
  `method(*args: (*: Int, **: String)) {}`,
  code: #restLaneMismatch
)
```

## 14. Mutual exclusion

```phalcom
assertCompileError(
  `method(*args: Int, ***remaining: P) {}`,
  code: #conflictingRestModes
)
```

```phalcom
assertCompileError(
  `target(*args, ***pack)`,
  code: #conflictingExpansionModes
)
```

## 15. Split expansion

```phalcom
collect(***arguments: Any) {
  return arguments
}

const positionals = (1, 2)
const labels = (name: "Ada", mode: "strict")

assertEqual(
  collect(*positionals, **labels),
  (1, 2, name: "Ada", mode: "strict")
)
```

Multiple expansions:

```phalcom
assertEqual(
  collect(*(1, 2), *(3, 4), **(x: 5), **(y: 6)),
  (1, 2, 3, 4, x: 5, y: 6)
)
```

## 16. Complete expansion

```phalcom
const pack = (1, 2, name: "Ada")
assertEqual(collect(***pack), pack)
```

## 17. Duplicate labels

```phalcom
assertRuntimeError(
  { collect(x: 1, **(x: 2)) },
  code: #duplicateCallLabel
)
```

```phalcom
assertRuntimeError(
  { collect(**(x: 1), **(x: 2)) },
  code: #duplicateCallLabel
)
```

## 18. Strict ordering

```phalcom
assertCompileError(
  `target(timeout: second, *args)`,
  code: #positionalAfterLabeled
)
```

```phalcom
assertCompileError(
  `target(**labels, timeout: second)`,
  code: #fixedLabelAfterLabeledExpansion
)
```

## 19. Type-level projections

```phalcom
type P = (Request, timeout: Duration)

type Positional = (*P,) -> R
type Labeled = (**P,) -> R
type Complete = (***P,) -> R

assertEqual(Positional, (Request) -> R)
assertEqual(Labeled, (timeout: Duration) -> R)
assertEqual(Complete, (Request, timeout: Duration) -> R)
```

## 20. Generic forwarding

```phalcom
forward<P: Tuple, R>(
  callable: (***P,) -> R,
  ***arguments: P
) -> R {
  return callable(***arguments)
}
```

Acceptance tests MUST cover:

```phalcom
forward(positionalCallable, 1, 2)
forward(labeledCallable, name: "Ada")
forward(mixedCallable, 1, name: "Ada")
```

and verify that no lane is lost.

## 21. TupleType versus ArgumentPackType

```phalcom
type Literal = (*: Int)
const Domain = ((*: Int) -> R).domain

assertTrue(Literal.class == TupleType)
assertTrue(Domain.class == ArgumentPackType)
assertTrue(Literal != Domain)
```

## 22. Record expansion

```phalcom
const fields = {
  name: "Ada",
  mode: "strict"
}

assertEqual(
  collect(**fields),
  (name: "Ada", mode: "strict")
)
```

Negative:

```phalcom
assertRuntimeError(
  { collect(*fields) },
  code: #missingPositionalLane
)
```

## 23. Selector call-label boundary

Provisional negative test:

```phalcom
const operations = (+(_): handler)

assertRuntimeError(
  { collect(**operations) },
  code: #invalidCallLabel
)
```

This test must be removed or inverted if Selector-valued call labels are later ratified.

## 24. Evaluation order

```phalcom
var events = []

record(value) {
  events.add(value)
  return value
}

collect(
  record(#first),
  *record((#second,)),
  option: record(#third),
  **record((extra: #fourth))
)

assertEqual(events, [#first, #second, #third, #fourth])
```

Every operand MUST be evaluated exactly once.


<!-- END 11-conformance-tests.md -->


---

<!-- BEGIN 12-open-questions-and-gap-analysis.md -->

# Open Questions and Gap-Analysis Checklist

## 1. Purpose

This document lists decisions that remain unratified, consequences that need implementation review, and adversarial questions another agent should systematically investigate.

## 2. Highest-priority open decisions

### 2.1 Distinct `ArgumentPackType`

**Recommendation:** Ratify.

Without it, these become reflectively contradictory:

```phalcom
type Literal = (*: Int)
const callback: (*: Int) -> R
```

The first is an exact Tuple Type with literal key `#*`; the second needs an open positional domain.

Questions:

1. Is `ArgumentPackType` public?
2. Can users construct it directly?
3. What conversion message interprets a Tuple Type as a pack schema?
4. Are source and normalized forms both reflected?

### 2.2 Selector-valued call labels

Structural labels already allow:

```phalcom
+(_): handler
```

Should calls permit:

```phalcom
register(+(_): handler)
```

Options:

A. Call labels remain Symbols only.  
B. Call labels become `Symbol | Selector`.  
C. Direct syntax stays Symbol-only, but dynamic expansion may carry Selector labels.

Recommendation: A initially. B requires auditing selector interning, dispatch caches, method declaration grammar, protocol conformance, `doesNotUnderstand`, serialization, and diagnostics.

### 2.3 Complete expansion placement

Proposed grammar:

```phalcom
target(
  fixedPositionals,
  ***pack,
  fixedLabels
)
```

Questions:

1. May explicit labels follow `***pack`?
2. May explicit labels precede it? They probably cannot because it may emit positionals.
3. Is at most one `***` expansion mandatory?
4. Can `***record` appear after labels because its positional lane is statically empty, or is placement syntax-only?

Recommendation: syntax-only rule; one `***`; positionals before; labels after.

### 2.4 Record exactness and width subtyping

Questions:

1. Are Record Types exact by default?
2. Do immutable Records support width subtyping?
3. How does width subtyping interact with exact `**config: RecordType` capture?
4. Is there an explicit open-record type?

### 2.5 Set design

The entire Set surface remains provisional:

1. literal syntax;
2. mutability;
3. iteration order;
4. covariance;
5. hashability;
6. immutable Set naming.

## 3. Parser audit

The parser review must test:

```phalcom
(*P,)
(**P,)
(***P,)
(*: T)
(**: T)
***arguments: T
+(_): value
[#*]: value
```

Questions:

1. Does longest-token lexing make `***` unambiguous?
2. Can `(*: T)` be parsed without creating a special AST node?
3. Is `+(_): value` distinguishable from operator method syntax in all contexts?
4. Does a trailing comma remain necessary for a single unpacked domain item?
5. Can `...` remain both Ellipsis and repeated-tail marker without ambiguity?

## 4. Static-checker audit

The checker must answer:

1. Does every rest binder own exactly the declared lanes?
2. Are mixed schemas rejected on one-lane binders?
3. Are complete binders terminal?
4. Are split and complete modes mutually exclusive?
5. Are duplicate labels detectable statically when sources are known?
6. Are dynamic duplicate checks inserted otherwise?
7. Can generic `P: Tuple` retain both lanes through `***P`?
8. Does `*P` discard labels by type projection rather than by ad hoc syntax?
9. Are callable subtyping checks based on accepted packs?
10. Do defaults create unions, optional slots, or separate accepted-domain metadata?

## 5. Runtime and VM audit

Questions:

1. Is a call internally represented as one pack or separate arrays/maps?
2. Can split binders be zero-copy views?
3. Can complete capture reuse the original call pack?
4. When must a Tuple allocation occur?
5. How are duplicate labels detected efficiently across multiple expansions?
6. How are labels ordered reflectively?
7. Can pack projections be optimized away when unobserved?
8. How are Record captures materialized without violating Record construction rules?
9. What is the runtime failure when an unchecked value violates a rest annotation?
10. Do checked and unchecked invocation APIs differ?

## 6. Reflection audit

Verify that reflection distinguishes:

```text
source annotation
normalized ArgumentPackType
local binding Type
callable domain Type
rest mode
```

Adversarial examples:

```phalcom
method(*args: Int)
method(*args: (*: Int))
method(***args: Int)
method(***args: (*: Int, **: Int))
```

The first pair should normalize equally for the positional lane. The second pair should normalize equally for both lanes.

## 7. Equality and hashing audit

Questions:

1. Are `#+`, `#+()`, and `#+(_)` distinct keys?
2. Do `*:` and `[#*]:` collide?
3. Are normalized equivalent Types equal?
4. Is Type identity interned but non-normative?
5. Are ArgumentPackType hashes independent of source spelling?
6. Does labeled insertion order affect Type equality or only reflection?

Recommendation: structural Type equality should ignore source order where language semantics treat labels as keyed slots, while reflection may preserve declaration order. This needs ratification.

## 8. Call assembly audit

Stress cases:

```phalcom
target(*a, *b, fixed: value, **c, **d)
target(prefix, ***pack, fixed: value)
target(**tupleWithIgnoredPositionals)
target(***record)
```

Check:

1. left-to-right evaluation;
2. exact-once evaluation;
3. lane order;
4. duplicate errors;
5. partial side effects before failure;
6. static versus dynamic operand validation;
7. Selector-valued structural keys.

## 9. Generic forwarding audit

Canonical target:

```phalcom
forward<P: Tuple, R>(
  callable: (***P,) -> R,
  ***arguments: P
) -> R {
  return callable(***arguments)
}
```

Questions:

1. Is `P: Tuple` sufficient, or should the bound be `ArgumentPack`?
2. Does a broad `Tuple` bound include Selector-keyed tuples that are not call-compatible?
3. Is a dedicated `Pack` protocol/type needed?
4. Can a Record specialize `P`?
5. Can partially open pack types be generic arguments?
6. How are variance and substitution handled inside `***P`?

## 10. Tuple/Record composition audit

Because literal spread is call-only, explicit APIs are required.

Questions:

1. How are positional lanes concatenated?
2. How are labeled collisions handled?
3. Are override APIs separately named?
4. Does Tuple concatenation normalize lanes into positional-then-labeled order?
5. Can Records merge Selector keys?
6. Is composition lazy or eager?

## 11. Optionality and defaults audit

Cases:

```phalcom
method(timeout: Duration = second)
method(*args: Int, format: Format = defaultFormat)
```

Questions:

1. Is the callable domain a union of pack shapes?
2. How is optionality reflected?
3. Does omission differ from passing `None`?
4. Can open labeled capture receive a label also declared with a default? It should not; fixed binding should consume it first.

## 12. Error-recovery audit

The parser and checker should recover after malformed forms:

```phalcom
method(****args)
method(* *args)
method(**: T)
target(***, value)
```

Diagnostics should identify intended pack syntax without cascading into unrelated parse failures.

## 13. Security and robustness audit

1. Can malicious label hashing cause pathological duplicate checks?
2. Can recursive Type satisfaction overflow?
3. Can enormous unpacked packs exhaust memory before arity checks?
4. Are source locations preserved through expansion for tracebacks?
5. Are dynamic expansions exception-safe and cleanup-safe?

## 14. Recommended ratification sequence

1. `ArgumentPackType` as a distinct Type.
2. Call-label domain (`Symbol` versus `Symbol | Selector`).
3. Complete-expansion placement and one-per-call rule.
4. Default/optional callable-domain representation.
5. Record exactness and width subtyping.
6. Tuple/Record explicit composition APIs.
7. Set literal, mutability, and ordering.
8. Public reflection constructors and conversion APIs.

## 15. Final consistency criterion

The feature set is considered closed when every legal source form has:

1. one parse;
2. one contextual interpretation;
3. one normalized semantic object;
4. one reflection shape;
5. one satisfaction relation;
6. one failure rule for invalid use;
7. conformance tests covering static and dynamic paths.


<!-- END 12-open-questions-and-gap-analysis.md -->
