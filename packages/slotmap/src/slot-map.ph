from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable

/// A stable generational handle into a slot map.
///
/// `Tag` is a phantom type parameter used to create statically distinct key
/// domains. A `SlotKey<Users>` cannot be passed to a `SlotMap<Orders, T>`.
///
/// Keys are created only by the slotmap package implementation. A key is valid
/// exactly while the slot at `index` is occupied with the same `generation`.
data SlotKey<Tag> package(
  index: Int,
  generation: Int
)

impl<Tag> SlotKey<Tag> {
  toString -> String {
    "SlotKey(index: \(index), generation: \(generation))"
  }
}

data SlotState<T> {
  Occupied(_ value: T)

  Vacant(_ next: Option<Int>)
}

data Slot<T> module(
  generation: Int,
  state: SlotState<T>
)

/// Sparse generational storage with stable typed keys.
///
/// Values live directly in stable slots. Insert, lookup, and removal are O(1)
/// amortized. Iteration scans the slot array and therefore costs O(capacity).
///
/// The freelist is intrusive: every vacant slot stores the next vacant index.
/// Generations are ordinary Telos Int values and deliberately do not wrap.
class SlotMap<Tag, T> {
  mut _slots: List<Slot<T>>
  mut _free: Option<Int>
  mut _len: Int

  @constructor
  call() {
    _slots = List<Slot<T>>.new()
    _free = Option::None
    _len = 0
  }

  len -> Int { _len }

  size -> Int { _len }

  isEmpty -> Bool { _len == 0 }

  /// Returns true exactly when `key` currently denotes an occupied slot.
  containsKey(_ key: SlotKey<Tag>) -> Bool {
    match _slots.get(key.index) {
      Option::None => false
      Option::Some(slot) => {
        if slot.generation != key.generation {
          false
        } else {
          match slot.state {
            SlotState::Occupied(_) => true
            SlotState::Vacant(_) => false
          }
        }
      }
    }
  }

  /// Returns the value addressed by `key`, or None for an out-of-range,
  /// removed, or stale-generation key.
  get(_ key: SlotKey<Tag>) -> Option<T> {
    match _slots.get(key.index) {
      Option::None => Option::None
      Option::Some(slot) => {
        if slot.generation != key.generation {
          Option::None
        } else {
          match slot.state {
            SlotState::Occupied(value) => Option::Some(value)
            SlotState::Vacant(_) => Option::None
          }
        }
      }
    }
  }

  /// Inserts a value and returns its stable generational key.
  insert(_ value: T) -> SlotKey<Tag> {
    self.insertWithKey |_: SlotKey<Tag>| { value }
  }

  /// Inserts a value produced from the key allocated for it.
  ///
  /// This supports values that store their own key without a second lookup.
  insertWithKey(_ make: (SlotKey<Tag>) -> T) -> SlotKey<Tag> {
    match _free {
      Option::Some(index) => {
        let slot = _slots[index]

        match slot.state {
          SlotState::Occupied(_) => {
            throw Error.new("slotmap invariant violated: freelist points at an occupied slot")
          }

          SlotState::Vacant(next) => {
            let key = SlotKey<Tag>(
              index: index,
              generation: slot.generation
            )

            // Compute before mutating allocator state. If construction raises,
            // the map remains unchanged.
            let value = make.call(key)

            _slots[index] = Slot<T>(
              generation: slot.generation,
              state: SlotState::Occupied(value)
            )
            _free = next
            _len = _len + 1
            key
          }
        }
      }

      Option::None => {
        let index = _slots.size
        let generation = 0
        let key = SlotKey<Tag>(
          index: index,
          generation: generation
        )

        // As above, produce the value before committing the structural update.
        let value = make.call(key)

        _slots.append(
          Slot<T>(
            generation: generation,
            state: SlotState::Occupied(value)
          )
        )
        _len = _len + 1
        key
      }
    }
  }

  /// Removes `key` when it is live.
  ///
  /// Removal increments the slot generation before linking the slot into the
  /// freelist. Therefore every previously issued key for that occupancy remains
  /// stale even when this slot is reused later.
  remove(_ key: SlotKey<Tag>) -> Option<T> {
    match _slots.get(key.index) {
      Option::None => Option::None
      Option::Some(slot) => {
        if slot.generation != key.generation {
          Option::None
        } else {
          match slot.state {
            SlotState::Vacant(_) => Option::None

            SlotState::Occupied(value) => {
              _slots[key.index] = Slot<T>(
                generation: slot.generation + 1,
                state: SlotState::Vacant(_free)
              )
              _free = Option::Some(key.index)
              _len = _len - 1
              Option::Some(value)
            }
          }
        }
      }
    }
  }

  /// Removes every value while retaining allocated slots for reuse.
  ///
  /// Occupied generations are advanced. This is deliberately not implemented
  /// by replacing `_slots` with an empty list: doing that could make a
  /// pre-clear key become valid again after reinsertion.
  clear {
    let mut next: Option<Int> = Option::None
    let mut i = _slots.size - 1

    while i >= 0 {
      let slot = _slots[i]
      let generation = match slot.state {
        SlotState::Occupied(_) => slot.generation + 1
        SlotState::Vacant(_) => slot.generation
      }

      _slots[i] = Slot<T>(
        generation: generation,
        state: SlotState::Vacant(next)
      )
      next = Option::Some(i)
      i = i - 1
    }

    _free = next
    _len = 0
    ()
  }

  /// Retains exactly the entries accepted by `predicate`.
  ///
  /// The predicate receives both the stable key and value. Removed entries
  /// advance their generation and join the freelist.
  retain(_ predicate: (SlotKey<Tag>, T) -> Bool) {
    let mut i = 0

    while i < _slots.size {
      let slot = _slots[i]

      match slot.state {
        SlotState::Vacant(_) => ()

        SlotState::Occupied(value) => {
          let key = SlotKey<Tag>(
            index: i,
            generation: slot.generation
          )

          if not predicate.call(key, value) {
            self.remove(key)
          }
        }
      }

      i = i + 1
    }

    ()
  }

  /// A fresh iterator over live `(key, value)` pairs.
  iterator -> SlotMapIterator<Tag, T> {
    SlotMapIterator<Tag, T>(_slots)
  }

  module makeIterator -> SlotMapIterator<Tag, T> {
    SlotMapIterator<Tag, T>(_slots)
  }

  /// Checked indexing. Invalid or stale keys raise IndexError.
  [_ key] {
    match self.get(key) {
      Option::Some(value) => value
      Option::None => { throw IndexError.new("invalid or stale SlotMap key") }
    }
  }
}

/// Iterator for sparse SlotMap storage.
///
/// Structural mutation of the underlying map while iterating is intentionally
/// unsupported. Ordinary read-only iteration and value observation are safe.
class SlotMapIterator<Tag, T> {
  mut _slots: List<Slot<T>>
  mut _index: Int

  @constructor
  call(_ slots: List<Slot<T>>) {
    _slots = slots
    _index = 0
  }

  next -> Option<(SlotKey<Tag>, T)> {
    while _index < _slots.size {
      let index = _index
      let slot = _slots[index]
      _index = _index + 1

      match slot.state {
        SlotState::Vacant(_) => ()

        SlotState::Occupied(value) => {
          return Option::Some((
            SlotKey<Tag>(
              index: index,
              generation: slot.generation
            ),
            value
          ))
        }
      }
    }

    Option::None
  }

  iterator { self }
}

impl<Tag, T> CanonicalIterator for SlotMapIterator<Tag, T> {
  type Item = (SlotKey<Tag>, T)
}

impl<Tag, T> CanonicalIterable for SlotMap<Tag, T> {
  type Item = (SlotKey<Tag>, T)
  type Iterator = SlotMapIterator<Tag, T>

  iterator -> SlotMapIterator<Tag, T> {
    self.makeIterator
  }
}

export SlotKey, SlotMap, SlotMapIterator
