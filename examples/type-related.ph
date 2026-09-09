class User
{
    _name
    _age

    @constructor
    anonymous() {
        _name = Option<String>::None
        _age = Option<Int>::None
    }
}

System.print((User.class >> #anonymous()))
