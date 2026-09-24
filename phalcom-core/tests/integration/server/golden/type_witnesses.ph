// Task 30: these are post-inference witnesses. They must never be required in
// order for inference itself to succeed.
verifyFinalTypeWitnesses(
    response: Response<Ok, CheckoutView>,
    effect: TestFx<
        Result<
            Response<Ok, CheckoutView>,
            CheckoutError
        >
    >
) -> () {
    expectType<Response<Ok, CheckoutView>>(response)

    expectType<
        TestFx<
            Result<
              Response<Ok, CheckoutView>,
              CheckoutError
            >
        >
    >(effect)
}
