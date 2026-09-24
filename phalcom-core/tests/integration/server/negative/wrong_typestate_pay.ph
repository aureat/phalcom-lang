// EXPECT: type mismatch at `pay`; expected Order<Submitted>, found Order<Draft>.
negativeWrongTypestatePay() {
    let draft = FixtureFactory.draftOrder
    let authorization = PaymentAuthorization {
        approved: true,
    }

    pay(draft, authorization)
}
