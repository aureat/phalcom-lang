// Expected failure: a post-inference witness must not equate Ok with Created.
invalidFinalTypeWitness(
    effect: TestFx<
        Result<
            Response<Ok, CheckoutView>,
            CheckoutError
        >
    >
) {
    expectType<
        TestFx<
            Result<
                Response<Created, CheckoutView>,
                CheckoutError
            >
        >
    >(effect)
}
