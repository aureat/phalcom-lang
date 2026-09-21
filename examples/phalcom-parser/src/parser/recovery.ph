from parser.parser import Parser

impl Parser {
    recoverDeclaration {
        recoverDeclaration(current.kind)
    }

    recoverDeclaration(
        _ kind:
            Keyword(Class)
            | Keyword(Data)
            | Keyword(Enum)
            | Keyword(Trait)
            | Keyword(Impl)
            | Keyword(Type)
            | RightBrace
            | Eof
    ) {
        ()
    }

    recoverDeclaration(_ kind: TokenKind) {
        advance
        recoverDeclaration()
    }
}
