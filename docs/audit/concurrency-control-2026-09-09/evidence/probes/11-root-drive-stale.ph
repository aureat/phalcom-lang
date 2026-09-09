const f = Fiber.new || { System.print("once"); 1 }
System.schedule(f)
System.schedule(f)
System.schedule(|| { System.print("healthy root-drive task") })
System.print("main ended")
