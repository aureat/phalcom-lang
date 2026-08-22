# Phalcom Value Ontology

## One value universe — multiple semantic levels

> Phalcom has one universe of values and objects, but multiple orthogonal semantic relations: runtime classification through `class`, value classification through types, and type-level classification through kinds. Types and kinds can be reified back into the value universe without collapsing those relations.

## Normative terminology and precedence

- `Type` is the atomic kind of proper types.
- `TypeForm` names the common semantic/behavioral role of values that denote
  type-level forms.
- `TypeForm` is not a superclass inserted into the runtime object hierarchy.
- A `TypeId` identifies a canonical type-level form within one `TypeStore`;
  the associated `KindId` determines whether that form is a proper type or a
  type constructor.
- The ordinary type of a class-object value is distinct from the type form
  denoted by that value.
- Runtime reflection reifies this semantic model; it does not define it.
- Older typing documents that define `Type` as a protocol are historical where
  they conflict with this ontology.

## Non-negotiable implementation invariants

- No `ClassType` runtime wrapper is introduced.
- No runtime type or kind heap objects are added in this milestone.
- `.class`, `:`, and `::` remain distinct relations.
- Raw semantic IDs (`TypeId`, `KindId`, `TypeParameterId`) must never be persisted in compiled artifacts.
- No new source syntax is introduced for generic classes, type lambdas, or kind annotations in this milestone.

```text
expression
    ↓ evaluates to
value
    ↓ is classified by
type
    ↓ is classified by
kind
```

Or in conventional judgment notation:

```text
e ⇓ v          expression e evaluates to value v

v : T          value v has type T

T :: K         type/type-constructor T has kind K
```

Everything else—objects, classes, type descriptors—is a particular sort of value or a particular role played by a value.

### Values

A value is the result of evaluating an expression.

```phalcom
1
"hello"
User
List<Int>
#+
{ x + 1 }
```

All of these can be values. A value is therefore the broadest runtime concept. Implementation representation does not matter:

```text
1              immediate tagged integer
#foo           interned symbol
someObject     heap reference
User           heap ClassObject
List<Int>      heap TypeDescriptor
```

Semantically, all are values. `Value` is a semantic/runtime category, not synonymous with heap allocation.

### Objects

An object is a value participating in Phalcom's object model. Conceptually, that means it has behavior:

```phalcom
x.class
x.foo(...)
```

and potentially identity/state depending on its nature. There are two reasonable meanings of “object,” and we should explicitly choose one in the specification.

> Every Phalcom value is conceptually an object, although not every value is represented by a heap object.

```text
42
true
#name
User
List<Int>
```

are all objects in the language model. But implementation documentation can distinguish:

```text
object             semantic object
heap object         Object enum / ObjRef allocation

Value ≈ semantic representation of an evaluated thing
Object ≈ that thing viewed through Phalcom's object model
```


### Classes

A class is an object/value whose primary runtime role is to define behavior and instance construction. For example:

```phalcom
User
List
Int
```

are class objects. A class contains or determines things like:

```text
instance behavior
superclass
method dictionaries
field layout
class-side behavior
class-side state
attributes
construction semantics
```

So: `class = runtime behavioral object`.

But in Phalcom a class additionally plays another role: `a class can denote a nominal type.`

This is the first major point where things appear intertwined. `Int` viewed as an ordinary runtime value is a `Class` object. Viewed semantically in type position `Int` denotes the nominal type `Int`. These are not two different `Int`s. It is one value with two legitimate interpretations/roles.

That leads to a crucial distinction: `the class object Int` vs `the nominal type denoted by Int`.

Mathematically we could write `the nominal type denoted by Int` as `⟦Int⟧type` for the type denoted by the class object. So `Int` itself has a runtime class: `Int.class`, which belongs to the metaclass tower.

But the type denoted by `Int` has a kind: `Int.kind` of kind `Type`.

Those are completely different relationships. This distinction is absolutely fundamental:

```text
Int.class
```

asks:

> What runtime behavior/classifies the value `Int`?

while:

```text
Int.kind
```

asks:

> What kind does the type denoted by `Int` have?

That immediately resolves a lot of the apparent entanglement.

Now, a type.

A type is not fundamentally a class.

A type is a semantic classification of values.

More rigorously:

> A type denotes a set/domain of admissible values together with the semantic structure required to reason about those values.

For a nominal type:

```text
User
```

its values are approximately:

```text
instances of User
+
instances of qualifying subclasses
```

depending on Phalcom's precise subtype semantics.

But other types are not classes at all:

```phalcom
Int | String
List<Int>
(Int, String)
#{name: String}
Int -> String
```

These are perfectly meaningful types without corresponding user-defined classes.

Therefore:

```text
Class ⊂ type-denoting values
```

but:

```text
Type ≠ Class
```

This is why `List<Int>` cannot simply be treated as another class.

Its semantic meaning is an applied type:

```text
Apply(List, Int)
```

and its reflective runtime representation can be:

```text
AppliedType object
```

So:

```phalcom
List<Int>.class
// AppliedType
```

but:

```phalcom
List<Int>.kind
// Type
```

Again, runtime classification and semantic classification remain cleanly distinct.

We can now define a type constructor.

A type constructor is something that produces a type when supplied appropriate type-level arguments.

For example:

```text
Int        :: Type

List       :: Type -> Type

Map        :: Type -> Type -> Type
```

`Int` is already a complete type.

`List` isn't complete as a unary generic type expression. It awaits one type.

Then:

```text
List<Int> :: Type
```

So a class can itself denote a type constructor rather than a completed type.

This means our earlier statement that “classes denote nominal types” needs a slight refinement.

A non-generic class such as:

```text
Int
```

denotes something of kind:

```text
Type
```

A generic class such as:

```text
List
```

denotes something of kind:

```text
Type -> Type
```

The runtime value is still a class object in both cases.

That is elegant.

Now, kind.

A kind classifies types and type constructors.

The smallest useful kind language is:

```text
K ::= Type
    | K -> K
```

```text
Int                  :: Type

String               :: Type

List                 :: Type -> Type

Map                  :: Type -> Type -> Type

List<Int>            :: Type

Map<String>          :: Type -> Type

Map<String, Int>     :: Type

Higher               :: (Type -> Type) -> Type
```

```text
value-level:

f : Int -> String
x : Int
──────────────
f(x) : String

and type-level:

List : Type -> Type
Int  :: Type
──────────────────
List<Int> :: Type
```

That is one of the pieces worth stealing from Haskell.

Now comes the key philosophical part.

Should `Type` itself be a type?

No.

At least not in the semantic calculus.

Do not write:

```text
Type :: Type
```

That creates exactly the kind of self-containing universe that leads toward logical paradoxes in sufficiently expressive systems.

Similarly, we do not need:

```text
Kind :: Kind
```

The language runtime may absolutely contain an object representing `Type`, but that does not mean the semantic calculus says `Type : Type`.

This is where reification saves us.

Phalcom can have:

```phalcom
Type
```

as an ordinary runtime value.

Its runtime class might eventually be something like:

```text
AtomicKind
```

or:

```text
Kind
```

But semantically it is the reification of the kind:

```text
Type
```

These are different statements:

```text
runtime:
    Type.class == AtomicKind

semantic:
    Type is the atomic kind classifying ordinary types
```

There is no contradiction.

And:

```phalcom
Type -> Type
```

can be a runtime `FunctionKind` object representing the semantic kind:

```text
Type -> Type
```

Again:

```text
(Type -> Type).class
    = FunctionKind
```

does not answer:

```text
what kind is Type -> Type?
```

Those are different levels.

This gives us a very important rule:

> Reflection reifies a higher semantic level as an ordinary value at a lower runtime level; it does not collapse the two levels.

That's probably the single best definition of Phalcom's “unification.”

We are not saying:

```text
values = types = kinds
```

because that would be incoherent.

We're saying:

```text
values can represent types
values can represent kinds
types and kinds can therefore be manipulated using ordinary Phalcom semantics
```

That is much stronger and cleaner.

Consider the entire system with concrete examples.

For the integer value:

```phalcom
42
```

we have:

```text
42
│
│ value/object
│
├── runtime class → Int
│
└── semantic type → Int
                      │
                      └── kind → Type
```

For the class value `Int`:

```text
Int
│
│ value/object
│
├── runtime class → Int.class
│
└── denotes nominal type Int
                      │
                      └── kind → Type
```

So notice the recursion:

```text
42 : Int

Int itself is also a value,
but Int denotes the type appearing on the right.
```

This is completely fine because “has runtime type” and “denotes a semantic type” are different relations.

For `List`:

```text
List
│
│ value/object
│
├── runtime class → List.class
│
└── denotes type constructor List
                         │
                         └── kind → Type -> Type
```

For:

```phalcom
List<Int>
```

we get:

```text
List<Int>
│
│ value/object
│
├── runtime class → AppliedType
│
└── denotes applied type List<Int>
                              │
                              └── kind → Type
```

For:

```phalcom
Type
```

we get:

```text
Type
│
│ runtime value/object
│
├── runtime class → AtomicKind / Kind
│
└── reifies semantic kind Type
```

For:

```phalcom
Type -> Type
```

we get:

```text
Type -> Type
│
│ runtime value/object
│
├── runtime class → FunctionKind
│
└── reifies semantic kind
        Type -> Type
```

This is the model.

We can therefore give each term a compact normative definition.

`Expression`

> Source/program construct that can be evaluated or semantically interpreted.

`Value`

> The result of evaluating an expression. Every observable runtime entity in Phalcom is a value.

`Object`

> A value viewed through Phalcom's object model: it has behavior and a runtime class. All values are conceptually objects, although their VM representation may be immediate rather than heap allocated.

`Class`

> A value/object representing a runtime behavior: it defines instance behavior, inheritance, construction, storage layout and related metadata. A class may additionally denote a nominal type or type constructor.

`Type`

> A semantic classification/descriptor of values. Types participate in equality, subtyping, application, union/product/callable formation, etc. A type may be reified as an ordinary Phalcom value.

`Type constructor`

> A type-level entity whose kind accepts one or more arguments before yielding a type or another constructor.

`Kind`

> A semantic classification of types and type constructors.

`Type object`

I would actually avoid using this phrase casually because it becomes ambiguous.

Use:

```text
type value
```

for any runtime value that denotes/reifies a type.

And:

```text
type descriptor
```

for synthetic runtime representations such as:

```text
AppliedType
UnionType
TupleType
CallableType
```

A class can be a type value without being a `TypeDescriptor`. The runtime/object structure:

```text
VALUE / OBJECT
│
├── ordinary instance
├── class
├── closure
├── symbol
├── type descriptor
├── kind descriptor
└── ...
```

And the semantic classification structure:

```text
VALUE
  │
  │ :
  ▼
TYPE / TYPE CONSTRUCTOR
  │
  │ ::
  ▼
KIND
```

with bridges between them:

```text
Class object ───────────────► nominal type/type constructor

AppliedType object ─────────► applied semantic type

UnionType object ───────────► union semantic type

AtomicKind object ──────────► Type

FunctionKind object ────────► Type -> Type
```

```text
Expression
    something that can be evaluated

Value
    result of evaluation

Object
    a value viewed through Phalcom's object semantics

Class
    an object defining runtime behavior, inheritance,
    construction and representation;
    may also denote a nominal type or type constructor

Type
    semantic classification of values

Type constructor
    type-level function awaiting type/kind arguments

Kind
    classification of types and type constructors

Reification
    representing a semantic type/kind as an ordinary Phalcom value
    without changing its semantic level
```


## Relationships


### Legend

```text
──class──▶     runtime object-model classification:   x.class
──type───▶     value typing judgment:                 v : T
──kind───▶     type/kind judgment:                    T :: K
──denotes▶     runtime value denotes semantic type
──reifies▶     runtime value represents semantic kind
──super──▶     inheritance
```


### Examples

```text
42
    class = Int
    type  = Int
    Int :: Type

Int
    runtime object = class
    denotes        = nominal type Int
    kind           = Type

List
    runtime object = class
    denotes        = type constructor List
    kind           = Type -> Type

List<Int>
    runtime object = AppliedType
    denotes        = applied type List<Int>
    kind           = Type

Int | String
    runtime object = UnionType
    denotes        = union type
    kind           = Type

Int -> String
    runtime object = CallableType
    denotes        = callable type
    kind           = Type

Type
    runtime object = reflected kind value
    denotes/reifies = kind Type

Type -> Type
    runtime object = reflected arrow-kind value
    denotes/reifies = kind Type -> Type
```


### Diagram

```text
╔══════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                  PHALCOM UNIFIED ONTOLOGY                                        ║
║                                                                                                  ║
║                  ONE VALUE/OBJECT LANGUAGE — MULTIPLE SEMANTIC LEVELS                            ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════╝


                                      EXPRESSIONS
                                          │
                                          │ evaluate
                                          ▼
╔══════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                      VALUES / OBJECTS                                            ║
║                                                                                                  ║
║        Every observable Phalcom value is conceptually an object.                                 ║
║        Some are immediate VM representations; some are heap allocated.                           ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════╝


     ORDINARY VALUE                     CLASS VALUE                       REFLECTED TYPE VALUE
     ──────────────                     ───────────                       ────────────────────

          42                               Int                              List<Int>
          │                                │                                    │
          │ .class                         │ .class                             │ .class
          ▼                                ▼                                    ▼
         Int                            Int.class                           AppliedType
          │                                │                                    │
          │                                │ .class                             │ .class
          │                                ▼                                    ▼
          │                            Metaclass                         AppliedType.class
          │                                ▲                                    │
          │                                │                                    │ .class
          │                                │                                    ▼
          │                          Int.class is a                         Metaclass
          │                          class object                               │
          │                                │                                    │ .class
          │                                │ superclass                         ▼
          │                                ▼                                Metaclass
          │                          Object.class                               ▲
          │                                │                                    │
          │                                └────────────── ... ─────────────────┘
          │
          │
          │ semantic type
          ▼
         Int
          │
          │ kind
          ▼
         Type


                     RUNTIME OBJECT/METACLASS RELATIONS
                     ══════════════════════════════════

                 instance
                    │
                    │ .class
                    ▼
                  User
                    │
                    │ .class
                    ▼
                User.class
                    │
                    │ .class
                    ▼
                Metaclass
                    │
                    │ .class
                    └──────────── .class ────▶ Metaclass.class ────┐
                                            ▲                      ▼
                                            │                 Metaclass
                                            │                      │
                                            └─────── .class ───────┘


                 User ───────super──────▶ Object
                   │                          │
                   │ .class                   │ .class
                   ▼                          ▼
               User.class ───super──────▶ Object.class
                   │                          │
                   └──────── .class ──────────┘
                              │
                              ▼
                          Metaclass


                         THIS TOWER CLASSIFIES OBJECTS
                                      │
                                      │
                                      │ but classes can ALSO denote
                                      │ semantic type-level entities
                                      ▼


╔══════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                  TYPE / KIND SEMANTICS                                           ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════╝


       CLASS OBJECT                   SEMANTIC ENTITY                         KIND
       ────────────                   ───────────────                         ────

          Int
           │
           │ denotes
           ▼
     nominal type Int ─────────────────────────────────────────────────────▶ Type
                                                                            ▲
                                                                            │
                                                                            │
          String                                                            │
           │                                                                │
           └──denotes──▶ nominal type String ───────────────────────────────┤
                                                                            │
                                                                            │
          User                                                              │
           │                                                                │
           └──denotes──▶ nominal type User ─────────────────────────────────┤
                                                                            │
                                                                            │
          List                                                              │
           │                                                                │
           │ denotes                                                        │
           ▼                                                                │
    type constructor List ────────────────────────────────────▶ Type ──▶ Type
                                                                \___________/
                                                                      │
                                                                      │
                                                                  arrow kind
                                                                      │
                                                                      ▼
                                                                Type -> Type


          Map
           │
           │ denotes
           ▼
    type constructor Map ──────────────────────────────▶ Type -> Type -> Type



                         TYPE APPLICATION
                         ════════════════

               List                         Int
                │                            │
                │ :: Type -> Type            │ :: Type
                │                            │
                └──────────────┬─────────────┘
                               │
                               │ apply
                               ▼
                           List<Int>
                               │
                               │ :: Type
                               ▼
                              Type


          RUNTIME REFLECTION OF THAT SAME SEMANTIC ENTITY:

                       List<Int>
                           │
                           │ runtime value
                           │
                           │ .class
                           ▼
                      AppliedType
                           │
                           │ .class
                           ▼
                    AppliedType.class
                           │
                           │ .class
                           ▼
                       Metaclass
                           │
                           └── .class ──▶ Metaclass
                                          │
                                          │ .class
                                          ▼
                                   Metaclass.class
                                          │
                                          │ .class
                                          ▼
                                   Metaclass.class.class
                                          │
                                          │ .class
                                          ▼
                                   Metaclass.class.class.class
                                          │
                                          └─────────────────▶ ... (infinite descent)

                       List<Int>
                           │
                           │ denotes
                           ▼
                  semantic List<Int>
                           │
                           │ :: kind
                           ▼
                          Type


              ┌───────────────────────────────────────────────────┐
              │  List<Int> therefore participates in TWO towers:  │
              │                                                   │
              │  runtime:                                         │
              │                                                   │
              │    List<Int> .class -> AppliedType                │
              │                                                   │
              │  semantic:                                        │
              │                                                   │
              │    List<Int> :: Type                              │
              └───────────────────────────────────────────────────┘


                         MORE TYPE VALUES
                         ════════════════


      RUNTIME VALUE               DENOTES SEMANTIC TYPE                 KIND
      ─────────────               ─────────────────────                 ────

        Int                         Int                                  Type

        User                        User                                 Type

        List                        List                                 Type -> Type

        Map                         Map                                  Type -> Type -> Type

        List<Int>                   List<Int>                            Type

        Map<String, Int>            Map<String, Int>                     Type

        Int | String                Int | String                         Type

        (Int, String)               (Int, String)                        Type

        #{name: String}             #{name: String}                      Type

        Int -> String               Int -> String                        Type

        Never                       Never                                Type

        Unit                        Unit                                 Type



                  SYNTHETIC TYPES ARE ORDINARY OBJECTS TOO
                  ═══════════════════════════════════════


            Int | String
            \__________/
                 │
                 │ .class
                 ▼
              UnionType
                 │
                 │ .class
                 ▼
           UnionType.class
                 │
                 ▼
             Metaclass


            (Int, String)
                 │
                 │ .class
                 ▼
             TupleType
                 │
                 ▼
          TupleType.class
                 │
                 ▼
             Metaclass


            Int -> String
                 │
                 │ .class
                 ▼
           CallableType
                 │
                 ▼
        CallableType.class
                 │
                 ▼
             Metaclass


     Yet semantically:

            Int | String ──────────────::──────────────▶ Type
            (Int, String) ─────────────::──────────────▶ Type
            Int -> String ─────────────::──────────────▶ Type



╔══════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                      KIND LEVEL                                                  ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════╝


                         Type
                          │
                          │ semantic kind atom
                          │
                          │ reflected into the value world
                          ▼
                    [value] Type
                          │
                          │ .class
                          ▼
                     AtomicKind
                          │
                          │ .class
                          ▼
                  AtomicKind.class
                          │
                          | .class
                          ▼
                      Metaclass
                          │
                          └── .class ─────▶ Metaclass.class ── .class ──▶ Metaclass []
                          Metaclass.class.class ... (infinite descent)



                   Type -> Type
                          │
                          │ semantic arrow kind
                          │
                          │ reflected into value world
                          ▼
                 [value] Type -> Type
                          │
                          │ .class
                          ▼
                    FunctionKind
                          │
                          │ .class
                          ▼
                FunctionKind.class
                          │
                          ▼
                      Metaclass


       So even KIND VALUES re-enter the exact same ordinary object model.

       They do not require a second runtime universe.



                              HIGHER-KINDED EXAMPLES
                              ══════════════════════


             Int                         :: Type

             List                        :: Type -> Type

             Map                         :: Type -> Type -> Type

             Map<String>                 :: Type -> Type

             List<Int>                   :: Type

             F                           :: Type -> Type

             Higher                      :: (Type -> Type) -> Type


             Functor                     :: (Type -> Type) -> Constraint
                                                   ▲
                                                   │
                                        if Constraint is later
                                        admitted as another
                                        atomic kind


╔══════════════════════════════════════════════════════════════════════════════════════════════════╗
║                            THE TWO ORTHOGONAL CLASSIFICATION RELATIONS                           ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════╝


                               RUNTIME AXIS

                                  object
                                    │
                                  .class
                                    ▼
                                  class
                                    │
                                  .class
                                    ▼
                                metaclass
                                    │
                                    ▼
                                Metaclass
                                    └──────────────┐
                                                   │
                                                   └── fixed point


                               SEMANTIC AXIS

                                   value
                                     │
                                     │ :
                                     ▼
                                    type
                                     │
                                     │ ::
                                     ▼
                                    kind



                             THE BRIDGE BETWEEN THEM

         ┌──────────────────── runtime value/object ─────────────────────┐
         │                                                               │
         │  Int                 List<Int>                Type            │
         │   │                      │                     │              │
         │   │ denotes              │ denotes             │ reifies      │
         │   ▼                      ▼                     ▼              │
         │  Int                 List<Int>                Type            │
         │   │                      │                                    │
         │   │ ::                   │ ::                                 │
         │   ▼                      ▼                                    │
         │  Type                   Type                                  │
         │                                                               │
         └───────────────────────────────────────────────────────────────┘


                     EVERY REFLECTIVE ENTITY IS STILL A VALUE
                                      │
                                      ▼

         ┌───────────────────────────────────────────────────────────────┐
         │                                                               │
         │  nominal type     → usually represented by its Class object   │
         │                                                               │
         │  applied type     → AppliedType object                        │
         │                                                               │
         │  union type       → UnionType object                          │
         │                                                               │
         │  tuple type       → TupleType object                          │
         │                                                               │
         │  record type      → RecordType object                         │
         │                                                               │
         │  callable type    → CallableType object                       │
         │                                                               │
         │  Type kind        → AtomicKind object                         │
         │                                                               │
         │  K1 -> K2         → FunctionKind object                       │
         │                                                               │
         └───────────────────────────────────────────────────────────────┘


╔══════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                   COMPLETE EXAMPLE                                               ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════╝


                                     xs
                                     │
                                     │ runtime value
                                     │
                            ┌────────┴─────────┐
                            │                  │
                            │ .class           │ static/semantic :
                            ▼                  ▼
                           List             List<Int>
                            │                  │
                            │ .class           │ :: kind
                            ▼                  ▼
                        List.class            Type
                            │
                            │ .class
                            ▼
                        Metaclass
                            │
                            └── .class ──▶ Metaclass


                  Meanwhile the TYPE VALUE `List<Int>` itself:

                                 List<Int>
                                     │
                     ┌───────────────┼────────────────┐
                     │               │                │
                     │ .class        │ denotes        │ origin / args
                     ▼               ▼                ▼
                AppliedType     semantic List<Int>   List, (Int)
                     │               │
                     │ .class        │ ::
                     ▼               ▼
             AppliedType.class      Type
                     │
                     ▼
                 Metaclass
                     │
                     └── .class ──▶ Metaclass
```


### Two orthogonal structures

```text
                         OBJECT MODEL                     SEMANTIC MODEL
                         ────────────                     ──────────────

                             x                                x
                             │                                │
                          .class                              :
                             │                                │
                             ▼                                ▼
                           Class                             Type
                             │                                │
                          .class                              ::
                             │                                │
                             ▼                                ▼
                         Metaclass                           Kind
```


### Reflection

```text
             RUNTIME / OBJECT WORLD                   SEMANTIC WORLD

                     Int ─────────── denotes ─────────────▶ Int
                      │                                      │
                   .class                                    :: Type
                      ▼                                      ▼
                  Int.class                                 Type


                 List<Int> ──────── denotes ───────────▶ List<Int>
                      │                                      │
                   .class                                    :: Type
                      ▼                                      ▼
                AppliedType                                 Type


                    Type ─────────── reifies ─────────────▶ Type
                      │                                  semantic kind
                   .class
                      ▼
                 AtomicKind
```
