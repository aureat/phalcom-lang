@!documentation("Optional special functions, statistics, and probability built on the foundational Universe math API.")

expose .special
expose .statistics
expose .probability

// The numerics modules materialize List-backed intermediate values and expose
// Iterable-based APIs. Load List's canonical Iterable conformance into the
// package dependency graph so those generic calls see the owner contribution.
import universe.collections.list

from .errors import UndefinedStatisticError

export UndefinedStatisticError
