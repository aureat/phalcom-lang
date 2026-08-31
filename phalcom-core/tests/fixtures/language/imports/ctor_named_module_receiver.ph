import "./lib/ctor_named_module" as M
let p = M.Ref.at(3, 4)
System.print(p.row)
System.print(p.col)
