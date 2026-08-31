// Modules v1 retirement guard: imports after body statements are rejected
// before the old U-CLASSCLOSE binding collision can be reached.
class Point {}

import "../lib/point_holder" as Point
