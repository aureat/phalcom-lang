// U-CLASSCLOSE §8, decision 0066 ruling 8: an `import … as Name` and a
// `class Name` in the same module must collide, same diagnostic as two
// classes.
import "../lib/point_holder" as Point

class Point {}
