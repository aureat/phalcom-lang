@data
class User
{
    const _name
    const _age

    @class
    anonymous() {
        User.new(name: Option<String>::None, age: Option<Int>::None)
    }
}

System.print(User.anonymous())
