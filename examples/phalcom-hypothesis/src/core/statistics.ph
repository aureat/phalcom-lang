// Immutable statistics snapshots and the private mutable collector used while
// a search is running.

@data
@immutable
class Statistics {
  const _validExamples: Int
  const _discardedExamples: Int
  const _successfulShrinks: Int
  const _replayedExamples: Int
  const _events: Map<Symbol, Int>

  @class
  empty -> Statistics {
    return Statistics.new(
      validExamples: 0,
      discardedExamples: 0,
      successfulShrinks: 0,
      replayedExamples: 0,
      events: Map.new()
    )
  }

  // Compatibility getters for the temporary adapter and reporter.
  valid -> Int => _validExamples
  discarded -> Int => _discardedExamples
  shrinks -> Int => _successfulShrinks
  replayed -> Int => _replayedExamples

  eventCounts -> Map<Symbol, Int> {
    return _StatisticsCopies.map(_events)
  }
}

class _StatisticsCollector {
  @constructor
  new() {
    _validExamples = 0
    _discardedExamples = 0
    _successfulShrinks = 0
    _replayedExamples = 0
    _events = Map.new()
  }

  validExamples -> Int => _validExamples
  discardedExamples -> Int => _discardedExamples
  successfulShrinks -> Int => _successfulShrinks
  replayedExamples -> Int => _replayedExamples
  events -> Map<Symbol, Int> => _StatisticsCopies.map(_events)

  valid -> Int => _validExamples
  discarded -> Int => _discardedExamples
  shrinks -> Int => _successfulShrinks
  replayed -> Int => _replayedExamples

  recordPass(context: Any) -> None {
    _validExamples++
    self.recordContext(context)
  }

  recordReject(context: Any) -> None {
    _discardedExamples++
    self.recordContext(context)
  }

  recordFailure(context: Any) -> None {
    self.recordContext(context)
  }

  recordShrink() -> None {
    _successfulShrinks++
  }

  recordReplay() -> None {
    _replayedExamples++
  }

  recordContext(context: Any) -> None {
    context.events.entries.each |entry| {
      let label = entry.key
      let count = entry.value
      let total = _events.at(label)
      if total == None {
        total = 0
      }
      _events.at(label, put: total + count)
    }
  }

  snapshot -> Statistics {
    return Statistics.new(
      validExamples: _validExamples,
      discardedExamples: _discardedExamples,
      successfulShrinks: _successfulShrinks,
      replayedExamples: _replayedExamples,
      events: _StatisticsCopies.map(_events)
    )
  }
}

class _StatisticsCopies {
  @class
  map(values: Map<Symbol, Int>) -> Map<Symbol, Int> {
    const copied = Map.new()
    values.entries.each |entry| {
      copied.at(entry.key, put: entry.value)
    }
    return copied
  }
}
