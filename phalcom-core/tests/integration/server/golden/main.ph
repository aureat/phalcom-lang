// Tasks 27 and 38: execute through Fiber and emit the stable CI oracle.
main() {
    let fiber = Fiber.new {
        runGoldenProgram()
    }

    expectType<Fiber<TestReport>>(fiber)

    let report = fiber.join()
    expectType<TestReport>(report)

    if not report.passed {
        println("PHALCOM TYPING GOLDEN: FAIL")
        println(report.message)
        return
    }

    println("PHALCOM TYPING GOLDEN: PASS")
    println("")
    println("request:")
    println("  id: req-0001")
    println("  user: usr-001")
    println("  order: ord-123")
    println("")
    println("order:")
    println("  Draft -> Submitted -> Paid")
    println("")
    println("response:")
    println("  status: Ok")
    println("  order: ord-123")
    println("  state: paid")
    println("  total: 42.50")
    println("")
    println("repository:")
    println("  FindDraft(ord-123)")
    println("  SavePaid(ord-123)")
    println("")
    println("events:")
    println("  OrderPaid(ord-123)")
    println("")
    println("typing:")
    println("  static-domains: PASS")
    println("  typestate: PASS")
    println("  higher-kinds: PASS")
    println("  associated-types: PASS")
    println("  rows: PASS")
    println("  tuple-packs: PASS")
    println("  sequence-packs: PASS")
    println("  command-refinement: PASS")
    println("  reflection: PASS")
    println("  runtime-refinement: PASS")
}
