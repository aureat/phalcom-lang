// Task 26: deterministic executable report.
data TestReport {
    passed: Bool
    message: String
    requestId: Option<RequestId>
    orderId: Option<OrderId>
    response: Option<Response<Ok, CheckoutView>>
    trace: List<TraceEvent>
}

passedReport(
    requestId: RequestId,
    response: Response<Ok, CheckoutView>,
    trace: List<TraceEvent>
) -> TestReport {
    TestReport {
        passed: true,
        message: "all typing golden checks passed",
        requestId: Some(requestId),
        orderId: Some(response.body.orderId),
        response: Some(response),
        trace,
    }
}

failedReport(
    message: String,
    trace: List<TraceEvent>
) -> TestReport {
    TestReport {
        passed: false,
        message,
        requestId: None,
        orderId: None,
        response: None,
        trace,
    }
}
