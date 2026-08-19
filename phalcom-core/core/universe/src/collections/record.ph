class Record {
  size { self._$size }
  labelAt(_ index) { self._$labelAt(index) }

  ==(_ other) {
    if (other.isA(Record)) {
      let same = (self.size == other.size)
      let i = 0
      while (same and (i < self.size)) {
        let label = self._$labelAt(i)
        let j = 0
        let found = false
        while ((not found) and (j < other.size)) {
          if (other._$labelAt(j) == label) {
            found = (self._$valueAt(i) == other._$valueAt(j))
          }
          j = j + 1
        }
        same = found
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  hash {
    let acc = 101 + self.size
    let i = 0
    while (i < self.size) {
      acc = (acc + ((self._$labelAt(i).hash * 31) + self._$valueAt(i).hash)) % 999999937
      i = i + 1
    }
    return acc
  }
}
