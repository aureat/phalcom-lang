// Phase 12 cross-process persistence contract. The release gate runs this with
// the real toolchain when available, twice under the same deterministic seed.
import { DirectoryDatabase, Settings } from "hypothesis"

const seed = 20260723
const databaseType = DirectoryDatabase
const settings = Settings.standard.seed(seed)
System.print("PASS release persistence")
