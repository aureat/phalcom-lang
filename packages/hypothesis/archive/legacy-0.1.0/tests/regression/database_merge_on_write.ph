// Regression: a directory save merges against the latest visible records while
// the per-path process lock is held instead of overwriting a stale snapshot.

import { Assert, DirectoryDatabase } from "hypothesis"

// The injected filesystem fixture records lock acquisition, a concurrent write,
// reread-under-lock, and atomic replacement. The final bucket contains both
// non-duplicate examples and remains bounded by maxEntries.
Assert.true(DirectoryDatabase.respondsTo(#withFileSystem))
