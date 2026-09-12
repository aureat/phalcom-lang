import .answer as B
import .ctor_named_module as R
import .isolated as Iso
import .kernel_user as K
import .shape as M
import .shared as A
import .shared as C

System.print(B.answer)
let p = R.Ref.new(3, 4)
System.print(p.row)
System.print(p.col)
let s = M.Shape.new(3)
System.print(s.sides)
System.print(A.Point == C.Point)
System.print(A.value)
let shared = 1
System.print(shared)
System.print(Iso.shared)
System.print(K.total)
