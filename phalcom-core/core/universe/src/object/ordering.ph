@native
enum Ordering {
    @variant Less
    @variant Equal
    @variant Greater
    @variant Unordered

    @class
    less { Ordering::Less }

    @class
    equal { Ordering::Equal }

    @class
    greater { Ordering::Greater }

    @class
    unordered { Ordering::Unordered }

    reverse {
        match self {
            Less => Ordering::Greater
            Equal => Ordering::Equal
            Greater => Ordering::Less
            Unordered => Ordering::Unordered
        }
    }

    toString { toRepr }

    toRepr {
        match self {
            Less => "Ordering.less"
            Equal => "Ordering.equal"
            Greater => "Ordering.greater"
            Unordered => "Ordering.unordered"
        }
    }
}

export Ordering
