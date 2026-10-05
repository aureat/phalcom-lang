from universe.collections.tuple import CompleteTypePack

intrinsic data Pack<***Es>(***Es)

type PositionalPack<T: Type> = Pack<*: T>
type LabeledPack<T: Type> = Pack<**: T>
type CompletePack<T: Type> = Pack<***: T>

data PositionalPackView<T> {
  @variant
  Empty
  @variant
  Entry(_ value: T, _ rest: PositionalPack<T>)
}

data LabeledPackView<T> {
  @variant
  Empty
  @variant
  Entry(_ label: Symbol, _ value: T, _ rest: LabeledPack<T>)
}

type PositionalTypePackView = PositionalPackView<Dynamic>
type LabeledTypePackView = LabeledPackView<Dynamic>

export PositionalPackView
export LabeledPackView
export PositionalTypePackView
export LabeledTypePackView

impl<***Es: CompleteTypePack> Pack<***Es> {
  internal native rawPositionalSize -> Int
  internal native rawLabeledSize -> Int
  internal native rawPositionalAt(_ index: Int) -> Dynamic
  internal native rawLabelAt(_ index: Int) -> Symbol
  internal native rawLabeledValueAt(_ index: Int) -> Dynamic
  internal native rawPositionalRestSlice -> Dynamic
  internal native rawLabeledRestSlice -> Dynamic

  positionalSize { self.rawPositionalSize }
  labeledSize { self.rawLabeledSize }
  size { self.rawPositionalSize + self.rawLabeledSize }
  isEmpty { self.size == 0 }

  positionalView -> PositionalPackView<Dynamic> {
    if self.rawPositionalSize == 0 {
      return PositionalPackView<Dynamic>::Empty
    }
    return PositionalPackView<Dynamic>::Entry(self.rawPositionalAt(0), self.rawPositionalRestSlice)
  }

  labeledView -> LabeledPackView<Dynamic> {
    if self.rawLabeledSize == 0 {
      return LabeledPackView<Dynamic>::Empty
    }
    return LabeledPackView<Dynamic>::Entry(self.rawLabelAt(0), self.rawLabeledValueAt(0), self.rawLabeledRestSlice)
  }

  ==(_ other) {
    if other is not Pack {
      return false
    }
    let mut same = self.rawPositionalSize == other.rawPositionalSize and self.rawLabeledSize == other.rawLabeledSize
    let mut i = 0
    while same and i < self.rawPositionalSize {
      same = self.rawPositionalAt(i) == other.rawPositionalAt(i)
      i = i + 1
    }
    let mut j = 0
    while same and j < self.rawLabeledSize {
      same = self.rawLabelAt(j) == other.rawLabelAt(j) and self.rawLabeledValueAt(j) == other.rawLabeledValueAt(j)
      j = j + 1
    }
    return same
  }

  !=(_ other) {
    return not (self == other)
  }

  hash {
    let mut acc = 17 + self.rawPositionalSize
    let mut i = 0
    while i < self.rawPositionalSize {
      acc = (acc * 31 + self.rawPositionalAt(i).hash) % 999999937
      i = i + 1
    }
    let mut j = 0
    while j < self.rawLabeledSize {
      acc = (acc * 31 + self.rawLabelAt(j).hash) % 999999937
      acc = (acc * 31 + self.rawLabeledValueAt(j).hash) % 999999937
      j = j + 1
    }
    return acc
  }
}
