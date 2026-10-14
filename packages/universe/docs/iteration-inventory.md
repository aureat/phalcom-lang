# Iteration source inventory

This is the bounded S4 inventory of the former shadow `collections/iterator_laws.ph`, now kept intact at `collections/iteration/package.ph`. Line numbers refer to the 909-line moved file at the S4 delivery revision. This document records source shape; it does not claim semantic or runtime verification.

## Protocols and blanket implementation

### `Iterator` — lines 4–32

- Associated type: `Item`.
- Required member: `next -> Option<Self::Item>` (line 7).
- Default member `iterator` returns `self` (line 9).
- Default transformation selectors: `map(_ f)` (11), `filter(_ pred)` (12), `filterMap(_ f)` (13), `flatMap<S>(_ f)` (14–16), `flatten` (17–19), `chain<J>(_ other)` (20–22), `skip(_ n)` (23), `skipWhile(_ pred)` (24), `take(_ n)` (25), `takeWhile(_ pred)` (26), `mapWhile(_ f)` (27), `inspect(_ f)` (28), and `zip<J>(_ other)` (29–31).
- Explicit signatures and bounds:

  | Selector | Signature and constraint |
  |---|---|
  | `flatMap<S>(_ f)` | `(Self::Item) -> S` returns `FlatMapIterator<Self, S>`; `S |= Iterable` |
  | `flatten` | Returns `FlattenIterator<Self>`; `Self::Item |= Iterable` |
  | `chain<J>(_ other)` | Returns `ChainIterator<Self, J>`; `J |= Iterator` and `<J as Iterator>::Item == Self::Item` |
  | `zip<J>(_ other)` | Returns `StrictZipIterator<Self, J>`; `J |= Iterator` |

- The remaining adapter-returning defaults infer their return types from their constructor calls. They create the adapter instances directly, which is why moving adapters to child modules creates protocol-to-adapter dependencies.

### `Iterable` — lines 34–340

- Protocol bounds: `Self::Iterator |= Iterator` and `<Self::Iterator as Iterator>::Item == Self::Item` (lines 35–37).
- Associated types: `Item` and `Iterator` (39–40).
- Required member: `iterator -> Self::Iterator` (42).
- Default member selectors and spans:

  | Selector | Lines | Explicit return signature |
  |---|---:|---|
  | `iter` | 44 | `Self::Iterator` |
  | `indexed` | 46 | `IndexedIterator<Self::Iterator>` |
  | `each(_ f)` | 48–57 | inferred |
  | `each(indexed f)` | 59–70 | inferred |
  | `contains(_ x)` | 72–81 | inferred |
  | `includes(_ x)` | 83 | inferred |
  | property `count` | 85–94 | inferred |
  | `count(where f)` | 96–106 | inferred |
  | `find(where f)` | 108–117 | inferred |
  | `index(where f)` | 119–130 | inferred |
  | `any(where f)` | 132–141 | inferred |
  | `all(where f)` | 143–152 | inferred |
  | `none(where f)` | 154–163 | inferred |
  | `nth(_ n)` | 165–178 | inferred |
  | property `last` | 180–191 | inferred |
  | `fold(initial initial, using f)` | 193–203 | inferred |
  | `reduce(using f)` | 205–217 | inferred |
| property `join` | 219 | inferred |
  | `join(_ sep)` | 227–237 | inferred |
  | `group(by block)` | 239–258 | inferred |
  | `partition(where predicate)` | 260–274 | inferred |
  | property `toSet` | 276–288 | inferred |
  | `toMap` | 290–305 | `Result<Map<Dynamic, Dynamic>, DuplicateKeyError>` |
  | `toMap(merging block)` | 307–327 | inferred |
  | `toList` | 329–339 | inferred |

### Blanket implementation — lines 342–349

`impl<I> Iterable for I where I |= Iterator` sets `Item = <I as Iterator>::Item`, `Iterator = I`, and returns `self` from `iterator`. This blanket conformance remains in the moved source unchanged.

## Adapter declarations

Every adapter has `iterator { self }`. Constructor parameters without an explicit type annotation are shown as written. `next` entries marked inferred describe the source body’s resulting Option item shape; semantic typechecking has not run on the shadow package.

| Adapter and source span | Generic bounds | Constructor | `next` result | `Iterator` conformance / associated `Item` |
|---|---|---|---|---|
| `MappedIterator` 351–383 | `<I, U>`; constructor requires `I |= Iterator` | `new(_ upstream: I, _ mapper: (<I as Iterator>::Item) -> U)` (356–359) | inferred `Option<U>` | 379–383; `Item = U` |
| `FilterIterator` 385–421 | `<I>`; constructor requires `I |= Iterator` | `new(_ upstream: I, _ predicate: (<I as Iterator>::Item) -> Bool)` (390–393) | inferred `Option<<I as Iterator>::Item>` | 417–421; `Item = <I as Iterator>::Item` |
| `FilterMapIterator` 423–460 | `<I, U>`; constructor requires `I |= Iterator` | `new(_ upstream: I, _ mapper: (<I as Iterator>::Item) -> Option<U>)` (428–431) | inferred `Option<U>` | 456–460; `Item = U` |
| `FlatMapIterator` 462–506 | `<I, S>`; constructor requires `I |= Iterator` and `S |= Iterable` | `new(_ upstream: I, _ mapper: (<I as Iterator>::Item) -> S)` (468–471) | inferred `Option<<S as Iterable>::Item>` | 502–506; `Item = <S as Iterable>::Item` |
| `FlattenIterator` 508–549 | `<I>`; constructor requires `I |= Iterator` and `<I as Iterator>::Item |= Iterable` | `new(_ upstream: I)` (513–516) | inferred `Option<<<I as Iterator>::Item as Iterable>::Item>` | 545–549; `Item = <<I as Iterator>::Item as Iterable>::Item` |
| `ChainIterator` 551–588 | `<I, J>`; both conform to `Iterator`, with equal item types | `new(_ first: I, _ second: J)` (556–559) | `Option<<I as Iterator>::Item>` (565) | 584–588; `Item = <I as Iterator>::Item` |
| `SkipIterator` 590–632 | `<I>`; constructor requires `I |= Iterator` | `new(_ upstream: I, _ n)`; `n` is unannotated (595–598) | `Option<<I as Iterator>::Item>` (607) | 628–632; `Item = <I as Iterator>::Item` |
| `SkipWhileIterator` 634–679 | `<I>`; constructor requires `I |= Iterator` | `new(_ upstream: I, _ predicate: (<I as Iterator>::Item) -> Bool)` (640–643) | `Option<<I as Iterator>::Item>` (650) | 675–679; `Item = <I as Iterator>::Item` |
| `TakeIterator` 681–719 | `<I>`; constructor requires `I |= Iterator` | `new(_ upstream: I, _ n)`; `n` is unannotated (686–689) | `Option<<I as Iterator>::Item>` (698) | 715–719; `Item = <I as Iterator>::Item` |
| `TakeWhileIterator` 721–757 | `<I>`; constructor requires `I |= Iterator` | `new(_ upstream: I, _ predicate: (<I as Iterator>::Item) -> Bool)` (726–729) | `Option<<I as Iterator>::Item>` (735) | 753–757; `Item = <I as Iterator>::Item` |
| `MapWhileIterator` 759–796 | `<I, U>`; constructor requires `I |= Iterator` | `new(_ upstream: I, _ mapper: (<I as Iterator>::Item) -> Option<U>)` (764–767) | `Option<U>` (773) | 792–796; `Item = U` |
| `InspectIterator` 798–831 | `<I>`; constructor requires `I |= Iterator` | `new(_ upstream: I, _ callback: (<I as Iterator>::Item) -> Unit)` (803–806) | `Option<<I as Iterator>::Item>` (812) | 827–831; `Item = <I as Iterator>::Item` |
| `StrictZipIterator` 833–871 | `<I, J>`; both conform to `Iterator`; no item-equality bound | `new(_ left: I, _ right: J)` (838–841) | `Option<(<I as Iterator>::Item, <J as Iterator>::Item)>` (847) | 867–871; `Item = (<I as Iterator>::Item, <J as Iterator>::Item)` |
| `IndexedIterator` 873–907 | `<I>`; constructor requires `I |= Iterator` | `new(_ upstream: I)` (878–881) | `Option<(Int, <I as Iterator>::Item)>` (887) | 903–907; `Item = (Int, <I as Iterator>::Item)` |

The source `export` at line 909 names all sixteen declarations: `Iterator`, `Iterable`, and all fourteen adapters. `collections/package.ph` exposes `.iteration`; it does not currently re-export those names at the parent facade. The full split and its runtime-cycle constraint are tracked in [`iteration-split-incident.md`](iteration-split-incident.md).

## Import clients and adjacent iterator types

The 23 shadow source modules importing from `universe.collections.iteration` are:

- `collections/{array,bag,bijection,buffer,bytes,indexed_dict,list,map,mutable_bag,ordered_set,range,range_map,set,slice,tuple,vector}.ph`
- `functional/{chain,effects,fold,folds}.ph`
- `math/reduction.ph`, `scalar/string.ph`, and `text/substring.ph`

`effects.ph`, `fold.ph`, `folds.ph`, and `math/reduction.ph` import only `Iterable`; `text/substring.ph` imports both protocols, and `range.ph` also imports `TakeIterator`. Collection consumers generally alias the protocols to `CanonicalIterator` and `CanonicalIterable` to avoid colliding with their domain-specific iterator names.

`functional/chain.ph` declares its own `ChainIterator<T>` (line 282). It is a distinct functional-chain implementation, not the generic two-input adapter `ChainIterator<I, J>` owned by the iteration module. Its source owner and implementation remain unchanged.

## T7 adjacent ownership inventory

- `collections/bytes.ph` imports the canonical protocols at line 1. `Bytes.iterator` constructs `BytesIterator` at lines 89–90; that iterator is implemented at 226–250 and conforms with `Item = Int` at 252–254. `Bytes` conforms to `Iterable` with `Item = Int` and `Iterator = BytesIterator` at 256–260. The buffer `toList` uses iteration at 190–197, and the writer processes chunk lists at 523–541. This file does not use generic adapter constructors. Its distinct responsibilities remain co-located: byte values/iterator (3–260), `OpenMode`/`Path` (262–441), then `Resource`, stream, and buffered I/O types (444–819). `UniverseKey::Bytes` and `UniverseKey::Resource` still share the legacy source path; do not alter those native mappings here.
- `scalar/string.ph` imports the protocols plus String-related types at lines 7–15. `String` owns UTF-8 storage operations, indexing, search, trimming, replacement, split factories, and sequence accessors (22–879). `StringChars`/`StringCharIterator` occupy 881–944 and yield `Char`; `StringByteSequence`/`StringByteIterator` occupy 946–1017 and yield `Int`; `StringCodePointSequence`/`StringCodePointIterator` occupy 1019–1110 and yield `Int`. Each specialized iterator conforms to `Iterator` and each sequence conforms to `Iterable`. Keep these implementations with the String domain rather than generic adapter modules.
- Later work may separately assess focused Bytes/I/O/Path ownership and String submodule boundaries. The iterator behavior, views, and domain identities remain unchanged in S4.
