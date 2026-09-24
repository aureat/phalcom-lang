// EXPECT: type mismatch at `submit`; expected Order<Draft>, found Order<Paid>.
negativeWrongTypestateSubmit(
    paid: Order<Paid>
) {
    submit(paid)
}
