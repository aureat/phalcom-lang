data Money module(_ cents: Int)

impl Money {
    @invariant(cents >= 0)

    class.fromCents(_ cents: Int) -> Money {
      Money(cents)
    }

    class.fromDollars(_ amount: Int) -> Money {
      Money(amount * 100)
    }

    class.zero -> Money {
      Money(0)
    }

    dollars(self) -> Int { cents / 100 }

    +(_ other: Money) -> Money {
        Money(cents + other.cents)
    }

}
