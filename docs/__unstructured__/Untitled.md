```ph
intrinsic type TypePack // *Ps — positionals
intrinsic type LabeledTypePack // **Ls — labeled
intrinsic type CompleteTypePack // ***Es — positionals + labeled

intrinsic type Tuple<***Es: CompleteTypePack>(***Es)

intrinsic type Record<**Rs: RecordFields> { **Rs }

intrinsic type 
```

Custom tuple and record types

```ph
type TuplePositionals<*Ps: TypePack>(*Ps)
type TupleLabeled<**Ls: LabeledTypePack>(**Ls)
type CompleteTuple<***Es: CompleteTypePack>(***Es)
```

kind / shape algebra natively in phalcom

```ph

```

```ph
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data CompleteTypePack
intrinsic type data RecordFields


impl TypePack {
    intrinsic type Map<F: Type -> Type>: TypePack
    intrinsic type Concat<*Other: TypePack>: TypePack
}

impl LabeledTypePack {
    intrinsic type Map<F: Type -> Type>: LabeledTypePack
    intrinsic type Concat<**Other: LabeledTypePack>: LabeledTypePack
}

impl CompleteTypePack {
    intrinsic type Positionals: TypePack
    intrinsic type Labeled: LabeledTypePack

    intrinsic type Map<F: Type -> Type>: CompleteTypePack
    intrinsic type Concat<***Other: CompleteTypePack>: CompleteTypePack
}

impl RecordFields {
    intrinsic type Map<F: Type -> Type>: RecordFields
}


intrinsic data Tuple<***Es: CompleteTypePack>(
    ***Es
)

impl<***Es: CompleteTypePack> Tuple<***Es> {
    intrinsic type Entries: CompleteTypePack
    intrinsic type Positionals: TypePack
    intrinsic type Labeled: LabeledTypePack
    intrinsic type Element: Type

    intrinsic type At<I: Int>: Type
    intrinsic type Map<F: Type -> Type>: Type
}


intrinsic data Record<**Fs: RecordFields> {
    **Fs
}

impl<**Fs: RecordFields> Record<**Fs> {
    intrinsic type Fields: RecordFields
    intrinsic type Value: Type

    intrinsic type Map<F: Type -> Type>: Type
}


intrinsic type Unit = Tuple<>
intrinsic type EmptyRecord = Record<>
```