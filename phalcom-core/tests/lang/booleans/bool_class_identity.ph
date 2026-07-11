// area: boolean tower
// spec: ADR-0004; floor-census.md §2.6
// status: PASS
// True/False are concrete singleton subclasses of the abstract Bool.
System.print(true.class == True)
System.print(false.class == False)
System.print(true.class == False)
System.print(true.class == Bool)
System.print(True.superclass == Bool)
System.print(False.superclass == Bool)
