import .a as a
import .b as b

let mut a_inst = a.Alpha.new()
let mut b_inst = b.Beta.new()
let mut msg = a_inst.name() + " " + b_inst.name()
export msg
