from .slot_map import SlotKey
from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable

data DenseSlotState {
  Occupied(_ denseIndex: Int)

  Vacant(_ next: Option<Int>)
}

data DenseSlot module(
  generation: Int,
  state: DenseSlotState
)

/// Dense generational storage with stable typed keys.
///
/// Stable slots map keys to positions in compact `_keys` and `_values` arrays.
/// Removal fills holes by moving the last dense entry into the removed entry's
/// position and repairing that moved key's stable-slot mapping.
///
/// Insert, lookup, removal, and iteration are O(1) amortized per element.
/// Iteration is O(len), independent of stable-slot fragmentation.
class DenseSlotMap<Tag, T> {
  mut _slots: List<DenseSlot>
  mut _keys: List<SlotKey<Tag>>
  mut _values: List<T>
  mut _free: Option<Int>

  @constructor
  call() {
    _slots = List<DenseSlot>.new()
    _keys = List<SlotKey<Tag>>.new()
    _values = List<T>.new()
    _free = Option::None
  }

  len -> Int { _values.size }

  size -> Int { _values.size }

  isEmpty -> Bool { _values.size == 0 }

  containsKey(_ key: SlotKey<Tag>) -> Bool {
    match _slots.get(key.index) {
      Option::None => false
      Option::Some(slot) => {
        if slot.generation != key.generation {
          false
        } else {
          match slot.state {
            DenseSlotState::Occupied(_) => true
            DenseSlotState::Vacant(_) => false
          }
        }
      }
    }
  }

  get(_ key: SlotKey<Tag>) -> Option<T> {
    match _slots.get(key.index) {
      Option::None => Option::None
      Option::Some(slot) => {
        if slot.generation != key.generation {
          Option::None
        } else {
          match slot.state {
            DenseSlotState::Vacant(_) => Option::None
            DenseSlotState::Occupied(denseIndex) => {
              Option::Some(_values[denseIndex])
            }
          }
        }
      }
    }
  }

  insert(_ value: T) -> SlotKey<Tag> {
    self.insertWithKey |_: SlotKey<Tag>| { value }
  }

  insertWithKey(_ make: (SlotKey<Tag>) -> T) -> SlotKey<Tag> {
    match _free {
      Option::Some(slotIndex) => {
        let slot = _slots[slotIndex]

        match slot.state {
          DenseSlotState::Occupied(_) => {
            throw Error.new("dense slotmap invariant violated: freelist points at an occupied slot")
          }

          DenseSlotState::Vacant(next) => {
            let key = SlotKey<Tag>(
              index: slotIndex,
              generation: slot.generation
            )
            let value = make.call(key)
            let denseIndex = _values.size

            _keys.append(key)
            _values.append(value)
            _slots[slotIndex] = DenseSlot(
              generation: slot.generation,
              state: DenseSlotState::Occupied(denseIndex)
            )
            _free = next

            key
          }
        }
      }

      Option::None => {
        let slotIndex = _slots.size
        let generation = 0
        let key = SlotKey<Tag>(
          index: slotIndex,
          generation: generation
        )
        let value = make.call(key)
        let denseIndex = _values.size

        _keys.append(key)
        _values.append(value)
        _slots.append(
          DenseSlot(
            generation: generation,
            state: DenseSlotState::Occupied(denseIndex)
          )
        )

        key
      }
    }
  }

  remove(_ key: SlotKey<Tag>) -> Option<T> {
    match _slots.get(key.index) {
      Option::None => Option::None
      Option::Some(slot) => {
        if slot.generation != key.generation {
          Option::None
        } else {
          match slot.state {
            DenseSlotState::Vacant(_) => Option::None

            DenseSlotState::Occupied(denseIndex) => {
              let lastIndex = _values.size - 1
              let removed = _values[denseIndex]

              if denseIndex != lastIndex {
                let movedKey = _keys[lastIndex]
                let movedValue = _values[lastIndex]

                _keys[denseIndex] = movedKey
                _values[denseIndex] = movedValue

                let movedSlot = _slots[movedKey.index]
                _slots[movedKey.index] = DenseSlot(
                  generation: movedSlot.generation,
                  state: DenseSlotState::Occupied(denseIndex)
                )
              }

              _keys.popLast
              _values.popLast

              _slots[key.index] = DenseSlot(
                generation: slot.generation + 1,
                state: DenseSlotState::Vacant(_free)
              )
              _free = Option::Some(key.index)

              Option::Some(removed)
            }
          }
        }
      }
    }
  }

  /// Clears dense values while retaining and rebuilding all stable slots.
  ///
  /// Generations for occupied slots are advanced, so pre-clear handles remain
  /// permanently stale.
  clear {
    _keys.clear
    _values.clear

    let mut next: Option<Int> = Option::None
    let mut i = _slots.size - 1

    while i >= 0 {
      let slot = _slots[i]
      let generation = match slot.state {
        DenseSlotState::Occupied(_) => slot.generation + 1
        DenseSlotState::Vacant(_) => slot.generation
      }

      _slots[i] = DenseSlot(
        generation: generation,
        state: DenseSlotState::Vacant(next)
      )
      next = Option::Some(i)
      i = i - 1
    }

    _free = next
    ()
  }

  /// Retains values accepted by the predicate.
  ///
  /// Dense swap-removal means the current index is rechecked after a removal.
  retain(_ predicate: (SlotKey<Tag>, T) -> Bool) {
    let mut i = 0

    while i < _values.size {
      let key = _keys[i]
      let value = _values[i]

      if predicate.call(key, value) {
        i = i + 1
      } else {
        self.remove(key)
      }
    }

    ()
  }


  iterator -> DenseSlotMapIterator<Tag, T> {
    DenseSlotMapIterator<Tag, T>(_keys, _values)
  }

  [_ key] {
    match self.get(key) {
      Option::Some(value) => value
      Option::None => { throw IndexError.new("invalid or stale DenseSlotMap key") }
    }
  }
}

class DenseSlotMapIterator<Tag, T> {
  mut _keys: List<SlotKey<Tag>>
  mut _values: List<T>
  mut _index: Int

  @constructor
  call(
    _ keys: List<SlotKey<Tag>>,
    _ values: List<T>
  ) {
    _keys = keys
    _values = values
    _index = 0
  }

  next -> Option<(SlotKey<Tag>, T)> {
    if _index >= _values.size {
      return Option::None
    }

    let index = _index
    _index = _index + 1
    Option::Some((_keys[index], _values[index]))
  }

  iterator { self }
}

impl<Tag, T> CanonicalIterator for DenseSlotMapIterator<Tag, T> {
  type Item = (SlotKey<Tag>, T)
}

impl<Tag, T> CanonicalIterable for DenseSlotMap<Tag, T> {
  type Item = (SlotKey<Tag>, T)
  type Iterator = DenseSlotMapIterator<Tag, T>

  iterator -> DenseSlotMapIterator<Tag, T> {
    DenseSlotMapIterator<Tag, T>(_keys, _values)
  }
}

export DenseSlotMap, DenseSlotMapIterator
