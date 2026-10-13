@!documentation("Language and runtime error types.")

expose .error
expose .argument
expose .iteration
expose .indexing
expose .contracts
expose .unsupported
expose .unimplemented

from .argument import ArgumentError, KeyError, IndexError, SliceError, DuplicateKeyError, BijectionConflictError, InvalidIntervalError, RangeOverlapError
export ArgumentError, KeyError, IndexError, SliceError, DuplicateKeyError, BijectionConflictError, InvalidIntervalError, RangeOverlapError
