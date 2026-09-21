from parser.parser import Parser
from lex.token import TokenKind, Keyword
from syntax.ast import Decl, EnumVariantDecl, DataField, GenericParameter, MethodParameter

impl Parser {
    parseDeclaration -> DeclId {
        parseDeclaration(current.kind)
    }

    parseDeclaration(_ kind: Keyword(Class)) -> DeclId {
        parseClassDeclaration()
    }

    parseDeclaration(_ kind: Keyword(Data)) -> DeclId {
        parseDataDeclaration()
    }

    parseDeclaration(_ kind: Keyword(Enum)) -> DeclId {
        parseEnumDeclaration()
    }

    parseDeclaration(_ kind: Keyword(Trait)) -> DeclId {
        parseTraitDeclaration()
    }

    parseDeclaration(_ kind: Keyword(Impl)) -> DeclId {
        parseImplDeclaration()
    }

    parseDeclaration(_ kind: Keyword(Type)) -> DeclId {
        parseTypeAliasDeclaration()
    }

    parseDeclaration(_ kind: Identifier) -> DeclId {
        if peek(1).kind == LeftParen {
            parseMethodDeclaration()
        } else {
            parseDeclaration(kind as TokenKind)
        }
    }

    parseDeclaration(_ kind: TokenKind) -> DeclId {
        let span = current.span

        report(
            UnexpectedToken,
            span,
            "expected declaration"
        )

        recoverDeclaration()

        arena.addDecl(
            Decl::Error { span: span }
        )
    }

    parseClassDeclaration -> DeclId {
        let start = expect(Keyword(Class)).span.start
        let name = expectIdentifier()
        let generics = parseGenericParameters()

        expect(LeftBrace)

        let body = List<DeclId>.new()
        skipSeparators()

        while current.kind != RightBrace
            and current.kind != Eof
        {
            body.push(parseDeclaration())
            skipSeparators()
        }

        let close = expect(RightBrace)

        arena.addDecl(
            Decl::Class {
                name: name,
                generics: generics,
                body: body,
                span: Span(start, close.span.end)
            }
        )
    }

    parseDataDeclaration -> DeclId {
        let start = expect(Keyword(Data)).span.start
        let name = expectIdentifier()
        let generics = parseGenericParameters()

        expect(LeftParen)

        let fields = List<DataField>.new()

        if current.kind != RightParen {
            while true {
                let fieldName = expectIdentifier()
                expect(Colon)
                let fieldType = parseType()

                fields.push(
                    DataField(fieldName, fieldType)
                )

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        let close = expect(RightParen)

        let predicate =
            match advanceIfKeyword(Where) {
                Some(_) => Some(parseRefinementPredicate())
                None => None
            }

        arena.addDecl(
            Decl::Data {
                name: name,
                generics: generics,
                fields: fields,
                predicate: predicate,
                span: Span(start, close.span.end)
            }
        )
    }

    parseEnumDeclaration -> DeclId {
        let start = expect(Keyword(Enum)).span.start
        let name = expectIdentifier()
        let generics = parseGenericParameters()

        expect(LeftBrace)

        let variants = List<EnumVariantDecl>.new()
        skipSeparators()

        while current.kind != RightBrace
            and current.kind != Eof
        {
            variants.push(parseEnumVariant())
            advanceIf(Comma)
            skipSeparators()
        }

        let close = expect(RightBrace)

        arena.addDecl(
            Decl::Enum {
                name: name,
                generics: generics,
                variants: variants,
                span: Span(start, close.span.end)
            }
        )
    }

    parseEnumVariant -> EnumVariantDecl {
        let start = current.span.start
        let name = expectIdentifier()

        parseEnumVariant(name, start, current.kind)
    }

    parseEnumVariant(
        _ name: Token,
        _ start: SourceOffset,
        _ kind: LeftParen
    ) -> EnumVariantDecl {
        advance

        let payload = List<TypeExprId>.new()

        if current.kind != RightParen {
            while true {
                payload.push(parseType())

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        let close = expect(RightParen)

        EnumVariantDecl::Tuple {
            name: name,
            payload: payload,
            span: Span(start, close.span.end)
        }
    }

    parseEnumVariant(
        _ name: Token,
        _ start: SourceOffset,
        _ kind: LeftBrace
    ) -> EnumVariantDecl {
        advance

        let fields = List<DataField>.new()
        skipSeparators()

        while current.kind != RightBrace
            and current.kind != Eof
        {
            let fieldName = expectIdentifier()
            expect(Colon)
            let fieldType = parseType()

            fields.push(
                DataField(fieldName, fieldType)
            )

            advanceIf(Comma)
            skipSeparators()
        }

        let close = expect(RightBrace)

        EnumVariantDecl::Record {
            name: name,
            fields: fields,
            span: Span(start, close.span.end)
        }
    }

    parseEnumVariant(
        _ name: Token,
        _ start: SourceOffset,
        _ kind: TokenKind
    ) -> EnumVariantDecl {
        EnumVariantDecl::Unit {
            name: name,
            span: name.span
        }
    }

    parseTraitDeclaration -> DeclId {
        let start = expect(Keyword(Trait)).span.start
        let name = expectIdentifier()
        let generics = parseGenericParameters()

        expect(LeftBrace)

        let body = List<DeclId>.new()
        skipSeparators()

        while current.kind != RightBrace
            and current.kind != Eof
        {
            body.push(parseDeclaration())
            skipSeparators()
        }

        let close = expect(RightBrace)

        arena.addDecl(
            Decl::Trait {
                name: name,
                generics: generics,
                body: body,
                span: Span(start, close.span.end)
            }
        )
    }

    parseImplDeclaration -> DeclId {
        let start = expect(Keyword(Impl)).span.start
        let first = parseType()

        let (traitType, targetType) =
            if advanceIfKeyword(For).isSome {
                (Some(first), parseType())
            } else {
                (None, first)
            }

        let whereClause = parseWhereClause()

        expect(LeftBrace)

        let body = List<DeclId>.new()
        skipSeparators()

        while current.kind != RightBrace
            and current.kind != Eof
        {
            body.push(parseDeclaration())
            skipSeparators()
        }

        let close = expect(RightBrace)

        arena.addDecl(
            Decl::Impl {
                target: targetType,
                trait: traitType,
                whereClause: whereClause,
                body: body,
                span: Span(start, close.span.end)
            }
        )
    }

    parseTypeAliasDeclaration -> DeclId {
        let start = expect(Keyword(Type)).span.start
        let name = expectIdentifier()
        let generics = parseGenericParameters()

        expect(Equal)

        let value = parseType()

        arena.addDecl(
            Decl::TypeAlias {
                name: name,
                generics: generics,
                value: value,
                span: Span(start, typeSpan(value).end)
            }
        )
    }

    parseMethodDeclaration -> DeclId {
        let start = current.span.start
        let name = expectIdentifier()
        let parameters = parseMethodParameters()

        let result =
            match advanceIf(Arrow) {
                Some(_) => Some(parseType())
                None => None
            }

        let whereClause = parseWhereClause()

        let body =
            if current.kind == LeftBrace {
                Some(parseBlockExpression())
            } else {
                None
            }

        let end =
            match body {
                Some(expr) => exprSpan(expr).end
                None => previous.span.end
            }

        arena.addDecl(
            Decl::Method {
                name: name,
                parameters: parameters,
                result: result,
                whereClause: whereClause,
                body: body,
                span: Span(start, end)
            }
        )
    }

    parseGenericParameters -> List<GenericParameter> {
        let parameters = List<GenericParameter>.new()

        if current.kind != Less {
            return parameters
        }

        advance

        if current.kind != Greater {
            while true {
                let name = expectIdentifier()

                let type =
                    match advanceIf(Colon) {
                        Some(_) => Some(parseType())
                        None => None
                    }

                parameters.push(
                    GenericParameter(name, type)
                )

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        expect(Greater)

        parameters
    }

    parseMethodParameters -> List<MethodParameter> {
        let parameters = List<MethodParameter>.new()

        expect(LeftParen)

        if current.kind != RightParen {
            while true {
                let name = expectIdentifier()
                expect(Colon)
                let type = parseType()

                parameters.push(
                    MethodParameter(
                        None,
                        name,
                        type
                    )
                )

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        expect(RightParen)

        parameters
    }

    parseWhereClause -> List<ConstraintSyntax> {
        let constraints = List<ConstraintSyntax>.new()

        if not atKeyword(Where) {
            return constraints
        }

        advance

        while true {
            constraints.push(
                Refinement(parseRefinementPredicate())
            )

            match advanceIf(Comma) {
                Some(_) => ()
                None => break
            }
        }

        constraints
    }
}
