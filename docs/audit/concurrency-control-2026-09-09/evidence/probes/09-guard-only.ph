const gate = Future.new()
const result = Future.new()
const driver = Fiber.new || {
  const action = Fiber.new || { gate.await; System.print("action done"); 42 }
  const value = action.try()
  if action.isDone { if action.error.isSome { result.settleError(action.error.unwrapOr(None)) } else { result.settleValue(value) } }
}
System.schedule(driver)
System.runScheduled()
System.print(result.isReady)
System.print(driver.isDone)
gate.settleValue(1)
System.runScheduled()
System.print(result.isReady)
