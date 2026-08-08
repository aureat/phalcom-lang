class A v => "A"
}

class B v => "B"
}

class C v => "C"
}

class D v => "D"
}

// One call site hit with different receiver classes
System.print(A.new().v)
System.print(B.new().v)
System.print(C.new().v)
System.print(D.new().v)
System.print(A.new().v)
