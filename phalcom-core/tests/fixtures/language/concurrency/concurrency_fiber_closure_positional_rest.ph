// area: concurrency
// spec: callable-surface-object-model-and-parameter-foundations.md §4, §9.3
// status: PASS
// First-resume argument binding must use structured Closure shape metadata and
// pack positional residuals into the closure's rest List.

const fiber = Fiber.new(|head, *tail| { tail.size })
System.print(fiber.call(7))
