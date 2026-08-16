import .a as a
import .b as b

let a_inst = a.Alpha.new()
let b_inst = b.Beta.new()
let msg = a_inst.name() + " " + b_inst.name()
export msg
