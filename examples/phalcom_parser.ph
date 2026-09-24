// phalcom_parser.ph
//
// Self-hosted Phalcom parser design draft.
//
// Goals:
// - parse Phalcom using Phalcom's own object/type model;
// - exercise enums with record-shaped variants, generics, associated types,
//   pattern matching, closures, iterator-friendly collections, and controlled
//   refinement types;
// - keep refinement predicates inside a restricted proposition language;
// - preserve enough syntax structure for diagnostics, tooling, and later CST
//   construction.
//
// Refinement policy in this file:
// - allowed: literals, immutable bindings, field projection, arithmetic,
//   comparisons, equality, `and`, `or`, `not`;
// - excluded: arbitrary method execution inside refinement predicates;
// - refinement strengthening is either statically proven or explicit;
// - refinements are representation-erased.
//
// Variant conventions:
// - variant names are unqualified in patterns and contextual construction:
//     Some(value), None, Identifier, Keyword(Class)
// - record-shaped AST variants are declared inline and may be explicitly
//   constructed when qualification improves readability:
//     Expr::Binary { left: ..., right: ..., ... }


// -----------------------------------------------------------------------------
// Fundamental domains
// -----------------------------------------------------------------------------

type Natural =
    Int where self >= 0

type Positive =
    Int where self > 0

type SourceOffset =
    Int where self >= 0

type BindingPower =
    Int where self >= 0


data Span(
    start: SourceOffset,
    end: SourceOffset
)
where start <= end


impl Span {
    @class
    empty(at offset: SourceOffset) -> Span {
        Span(offset, offset)
    }

    length -> Natural {
        end - start
    }

    contains(_ offset: SourceOffset) -> Bool {
        start <= offset and offset < end
    }

    join(_ other: Span) -> Span {
        let joinedStart =
            if start <= other.start { start } else { other.start }

        let joinedEnd =
            if end >= other.end { end } else { other.end }

        Span(joinedStart, joinedEnd)
    }
}


// Nominal source/token/arena identifiers intentionally remain distinct even
// though their storage domains overlap. Refinements express numeric validity;
// nominal data types express semantic identity.

data ExprId private(_ value: Natural)
data StmtId private(_ value: Natural)
data TypeExprId private(_ value: Natural)
data PatternId private(_ value: Natural)
data RefinementExprId private(_ value: Natural)
data DeclId private(_ value: Natural)


// -----------------------------------------------------------------------------
// Source text
// -----------------------------------------------------------------------------

data SourceText private(
    _ text: String
)

impl SourceText {
    @class
    new(_ text: String) -> SourceText {
        SourceText(text)
    }

    length -> Natural {
        text.length
    }

    slice(_ span: Span) -> String
    where span.end <= self.length
    {
        text.slice(from: span.start, to: span.end)
    }

    charAt(_ offset: SourceOffset) -> Option<Char> {
        if offset < length {
            Some(text[offset])
        } else {
            None
        }
    }
}


// -----------------------------------------------------------------------------
// Tokens
// -----------------------------------------------------------------------------

data Keyword {
    As
    Break
    Class
    Continue
    Data
    Else
    Enum
    False
    For
    If
    Impl
    In
    Is
    Let
    Match
    None
    Not
    Or
    And
    Private
    Return
    Self
    Trait
    True
    Type
    Var
    Where
    While
}


data TokenKind {
    Identifier
    Integer
    Float
    String
    Character

    Newline
    Eof

    Keyword(Keyword)

    LeftParen
    RightParen
    LeftBrace
    RightBrace
    LeftBracket
    RightBracket

    Comma
    Colon
    Semicolon
    Dot
    DoubleColon
    Question
    At

    Arrow
    FatArrow

    Equal
    EqualEqual
    BangEqual

    Less
    LessEqual
    Greater
    GreaterEqual

    Plus
    Minus
    Star
    Slash
    Percent

    Ampersand
    Pipe
    Bang

    PlusEqual
    MinusEqual
    StarEqual
    SlashEqual

    DotDot
    DotDotEqual
}


data Token(
    kind: TokenKind,
    span: Span
)


class TokenBuffer {
    _ source: SourceText
    _ tokens: List<Token>
}

impl TokenBuffer {
    @class
    new(
        _ source: SourceText,
        _ tokens: List<Token>
    ) -> TokenBuffer {
        // Construction authority guarantees an EOF sentinel.
        //
        // This is intentionally a nominal invariant rather than a refinement:
        // `tokens.last.kind == Eof` would require method execution in the
        // refinement proposition.
        if tokens.length == 0 {
            tokens.push(Token(Eof, Span.empty(at: source.length)))
        } else {
            match tokens.last.kind {
                Eof => ()
                _ => tokens.push(
                    Token(Eof, Span.empty(at: source.length))
                )
            }
        }

        TokenBuffer(source, tokens)
    }

    length -> Natural {
        _tokens.length
    }

    source -> SourceText {
        _source
    }

    at(_ index: Natural) -> Token
    where index < self.length
    {
        _tokens[index]
    }

    atOrEof(_ index: Natural) -> Token {
        if index < length {
            _tokens[index]
        } else {
            _tokens[length - 1]
        }
    }

    text(_ token: Token) -> String {
        _source.slice(token.span)
    }
}


// -----------------------------------------------------------------------------
// Diagnostics
// -----------------------------------------------------------------------------

data DiagnosticKind {
    UnexpectedToken
    ExpectedToken(TokenKind)
    ExpectedExpression
    ExpectedType
    ExpectedPattern
    InvalidRefinementExpression
    UnterminatedString
    UnterminatedCharacter
    InvalidNumber
}


data Diagnostic(
    kind: DiagnosticKind,
    span: Span,
    message: String
)


// -----------------------------------------------------------------------------
// Lexer
// -----------------------------------------------------------------------------

class Lexer {
    source: SourceText
    var cursor: SourceOffset
    var tokens: List<Token>
    var diagnostics: List<Diagnostic>
}

impl Lexer {
    @class
    lex(_ source: SourceText) -> (TokenBuffer, List<Diagnostic>) {
        let lexer = Lexer(
            source: source,
            cursor: 0,
            tokens: List<Token>.new(),
            diagnostics: List<Diagnostic>.new()
        )

        lexer.run()

        (
            TokenBuffer.new(source, lexer.tokens),
            lexer.diagnostics
        )
    }

    run {
        while cursor < source.length {
            scanToken()
        }

        tokens.push(
            Token(
                Eof,
                Span.empty(at: source.length)
            )
        )
    }

    scanToken {
        let start = cursor

        match advanceChar() {
            None => ()

            Some(' ' | '\t' | '\r') =>
                ()

            Some('\n') =>
                emit(Newline, from: start)

            Some('(') => emit(LeftParen, from: start)
            Some(')') => emit(RightParen, from: start)
            Some('{') => emit(LeftBrace, from: start)
            Some('}') => emit(RightBrace, from: start)
            Some('[') => emit(LeftBracket, from: start)
            Some(']') => emit(RightBracket, from: start)

            Some(',') => emit(Comma, from: start)
            Some(';') => emit(Semicolon, from: start)
            Some('@') => emit(At, from: start)
            Some('?') => emit(Question, from: start)

            Some(':') => {
                if advanceIfChar(':') {
                    emit(DoubleColon, from: start)
                } else {
                    emit(Colon, from: start)
                }
            }

            Some('.') => {
                if advanceIfChar('.') {
                    if advanceIfChar('=') {
                        emit(DotDotEqual, from: start)
                    } else {
                        emit(DotDot, from: start)
                    }
                } else {
                    emit(Dot, from: start)
                }
            }

            Some('+') => {
                if advanceIfChar('=') {
                    emit(PlusEqual, from: start)
                } else {
                    emit(Plus, from: start)
                }
            }

            Some('-') => {
                if advanceIfChar('>') {
                    emit(Arrow, from: start)
                } else if advanceIfChar('=') {
                    emit(MinusEqual, from: start)
                } else {
                    emit(Minus, from: start)
                }
            }

            Some('*') => {
                if advanceIfChar('=') {
                    emit(StarEqual, from: start)
                } else {
                    emit(Star, from: start)
                }
            }

            Some('/') => {
                if advanceIfChar('/') {
                    skipLineComment()
                } else if advanceIfChar('=') {
                    emit(SlashEqual, from: start)
                } else {
                    emit(Slash, from: start)
                }
            }

            Some('%') => emit(Percent, from: start)

            Some('=') => {
                if advanceIfChar('=') {
                    emit(EqualEqual, from: start)
                } else if advanceIfChar('>') {
                    emit(FatArrow, from: start)
                } else {
                    emit(Equal, from: start)
                }
            }

            Some('!') => {
                if advanceIfChar('=') {
                    emit(BangEqual, from: start)
                } else {
                    emit(Bang, from: start)
                }
            }

            Some('<') => {
                if advanceIfChar('=') {
                    emit(LessEqual, from: start)
                } else {
                    emit(Less, from: start)
                }
            }

            Some('>') => {
                if advanceIfChar('=') {
                    emit(GreaterEqual, from: start)
                } else {
                    emit(Greater, from: start)
                }
            }

            Some('&') => emit(Ampersand, from: start)
            Some('|') => emit(Pipe, from: start)

            Some('"') =>
                scanString(from: start)

            Some('\'') =>
                scanCharacter(from: start)

            Some(c) if c.isAsciiDigit =>
                scanNumber(from: start)

            Some(c) if c.isIdentifierStart =>
                scanIdentifier(from: start)

            Some(_) =>
                diagnostics.push(
                    Diagnostic(
                        UnexpectedToken,
                        Span(start, cursor),
                        "unexpected character"
                    )
                )
        }
    }

    currentChar -> Option<Char> {
        source.charAt(cursor)
    }

    advanceChar -> Option<Char> {
        match currentChar {
            None => None

            Some(char) => {
                cursor += 1
                Some(char)
            }
        }
    }

    advanceIfChar(_ expected: Char) -> Bool {
        match currentChar {
            Some(char) if char == expected => {
                cursor += 1
                true
            }

            _ => false
        }
    }

    skipLineComment {
        while cursor < source.length {
            match currentChar {
                Some('\n') | None => return
                _ => cursor += 1
            }
        }
    }

    scanString(from start: SourceOffset) {
        while cursor < source.length {
            match advanceChar() {
                Some('"') => {
                    emit(String, from: start)
                    return
                }

                Some('\\') => {
                    if cursor < source.length {
                        cursor += 1
                    }
                }

                None => break
                _ => ()
            }
        }

        diagnostics.push(
            Diagnostic(
                UnterminatedString,
                Span(start, cursor),
                "unterminated string literal"
            )
        )

        emit(String, from: start)
    }

    scanCharacter(from start: SourceOffset) {
        if cursor < source.length {
            advanceChar()

            if advanceIfChar('\'') {
                emit(Character, from: start)
                return
            }
        }

        diagnostics.push(
            Diagnostic(
                UnterminatedCharacter,
                Span(start, cursor),
                "unterminated character literal"
            )
        )

        emit(Character, from: start)
    }

    scanNumber(from start: SourceOffset) {
        while match currentChar {
            Some(c) if c.isAsciiDigit => true
            _ => false
        } {
            cursor += 1
        }

        if match currentChar {
            Some('.') => true
            _ => false
        } {
            match source.charAt(cursor + 1) {
                Some(c) if c.isAsciiDigit => {
                    cursor += 1

                    while match currentChar {
                        Some(d) if d.isAsciiDigit => true
                        _ => false
                    } {
                        cursor += 1
                    }

                    emit(Float, from: start)
                    return
                }

                _ => ()
            }
        }

        emit(Integer, from: start)
    }

    scanIdentifier(from start: SourceOffset) {
        while match currentChar {
            Some(c) if c.isIdentifierContinue => true
            _ => false
        } {
            cursor += 1
        }

        let span = Span(start, cursor)
        let text = source.slice(span)

        match keyword(text) {
            Some(keyword) =>
                tokens.push(Token(Keyword(keyword), span))

            None =>
                tokens.push(Token(Identifier, span))
        }
    }

    keyword(_ text: String) -> Option<Keyword> {
        match text {
            "as" => Some(As)
            "break" => Some(Break)
            "class" => Some(Class)
            "continue" => Some(Continue)
            "data" => Some(Data)
            "else" => Some(Else)
            "enum" => Some(Enum)
            "false" => Some(False)
            "for" => Some(For)
            "if" => Some(If)
            "impl" => Some(Impl)
            "in" => Some(In)
            "is" => Some(Is)
            "let" => Some(Let)
            "match" => Some(Match)
            "none" => Some(None)
            "not" => Some(Not)
            "or" => Some(Or)
            "and" => Some(And)
            "private" => Some(Private)
            "return" => Some(Return)
            "self" => Some(Self)
            "trait" => Some(Trait)
            "true" => Some(True)
            "type" => Some(Type)
            "var" => Some(Var)
            "where" => Some(Where)
            "while" => Some(While)
            _ => None
        }
    }

    emit(_ kind: TokenKind, from start: SourceOffset) {
        tokens.push(
            Token(
                kind,
                Span(start, cursor)
            )
        )
    }
}


// -----------------------------------------------------------------------------
// Syntax / AST
// -----------------------------------------------------------------------------

data UnaryOperator {
    Positive
    Negative
    LogicalNot
    Reference
}


data BinaryOperator {
    Add
    Subtract
    Multiply
    Divide
    Remainder

    Equal
    NotEqual

    Less
    LessEqual
    Greater
    GreaterEqual

    LogicalAnd
    LogicalOr

    Is
    As
}


data Expr {
    Integer {
        token: Token,
        span: Span
    }

    Float {
        token: Token,
        span: Span
    }

    String {
        token: Token,
        span: Span
    }

    Character {
        token: Token,
        span: Span
    }

    Bool {
        value: Bool,
        span: Span
    }

    Name {
        name: Token,
        span: Span
    }

    Tuple {
        elements: List<ExprId>,
        span: Span
    }

    List {
        elements: List<ExprId>,
        span: Span
    }

    Unary {
        operator: UnaryOperator,
        operand: ExprId,
        span: Span
    }

    Binary {
        left: ExprId,
        operator: BinaryOperator,
        right: ExprId,
        span: Span
    }

    Call {
        callee: ExprId,
        arguments: List<CallArgument>,
        span: Span
    }

    Member {
        receiver: ExprId,
        member: Token,
        span: Span
    }

    Associated {
        receiver: ExprId,
        member: Token,
        span: Span
    }

    If {
        condition: ExprId,
        thenBranch: ExprId,
        elseBranch: Option<ExprId>,
        span: Span
    }

    Match {
        subject: ExprId,
        arms: List<MatchArm>,
        span: Span
    }

    Closure {
        parameters: List<ClosureParameter>,
        body: ExprId,
        span: Span
    }

    Block {
        statements: List<StmtId>,
        trailing: Option<ExprId>,
        span: Span
    }

    Error {
        span: Span
    }
}


data CallArgument(
    label: Option<Token>,
    value: ExprId
)


data ClosureParameter(
    name: Token,
    type: Option<TypeExprId>
)


data MatchArm(
    pattern: PatternId,
    guard: Option<ExprId>,
    body: ExprId,
    span: Span
)


data Stmt {
    Let {
        mutable: Bool,
        name: Token,
        type: Option<TypeExprId>,
        value: Option<ExprId>,
        span: Span
    }

    Expr {
        expression: ExprId,
        span: Span
    }

    Return {
        value: Option<ExprId>,
        span: Span
    }

    Break {
        value: Option<ExprId>,
        span: Span
    }

    Continue {
        span: Span
    }

    Error {
        span: Span
    }
}


data TypeExpr {
    Name {
        name: Token,
        span: Span
    }

    Tuple {
        elements: List<TupleTypeElement>,
        span: Span
    }

    Generic {
        base: TypeExprId,
        arguments: List<TypeArgument>,
        span: Span
    }

    Associated {
        base: TypeExprId,
        member: Token,
        span: Span
    }

    Union {
        left: TypeExprId,
        right: TypeExprId,
        span: Span
    }

    Intersection {
        left: TypeExprId,
        right: TypeExprId,
        span: Span
    }

    Callable {
        parameters: List<TypeExprId>,
        result: TypeExprId,
        span: Span
    }

    Refined {
        base: TypeExprId,
        predicate: RefinementExprId,
        span: Span
    }

    Error {
        span: Span
    }
}


data TypeArgument {
    Type(TypeExprId)
    Value(ExprId)
}


data TupleTypeElement {
    Positional(TypeExprId)

    Named {
        name: Token,
        type: TypeExprId
    }
}


data Pattern {
    Wildcard {
        span: Span
    }

    Name {
        name: Token,
        span: Span
    }

    Integer {
        token: Token,
        span: Span
    }

    String {
        token: Token,
        span: Span
    }

    Variant {
        name: Token,
        payload: List<PatternId>,
        span: Span
    }

    Tuple {
        elements: List<PatternId>,
        span: Span
    }

    Range {
        start: PatternId,
        end: PatternId,
        inclusive: Bool,
        span: Span
    }

    Type {
        pattern: PatternId,
        type: TypeExprId,
        span: Span
    }

    Or {
        alternatives: List<PatternId>,
        span: Span
    }

    Error {
        span: Span
    }
}


// Dedicated logical syntax for controlled refinements.
//
// There is intentionally no Call variant here.

data RefinementExpr {
    Name {
        name: Token,
        span: Span
    }

    Self {
        span: Span
    }

    Integer {
        token: Token,
        span: Span
    }

    Float {
        token: Token,
        span: Span
    }

    Bool {
        value: Bool,
        span: Span
    }

    Member {
        receiver: RefinementExprId,
        member: Token,
        span: Span
    }

    Unary {
        operator: RefinementUnaryOperator,
        operand: RefinementExprId,
        span: Span
    }

    Binary {
        left: RefinementExprId,
        operator: RefinementBinaryOperator,
        right: RefinementExprId,
        span: Span
    }

    Error {
        span: Span
    }
}


data RefinementUnaryOperator {
    Positive
    Negative
    Not
}


data RefinementBinaryOperator {
    Add
    Subtract
    Multiply
    Divide
    Remainder

    Equal
    NotEqual

    Less
    LessEqual
    Greater
    GreaterEqual

    And
    Or
}


// -----------------------------------------------------------------------------
// Declarations
// -----------------------------------------------------------------------------

data Decl {
    Class {
        name: Token,
        generics: List<GenericParameter>,
        body: List<DeclId>,
        span: Span
    }

    Data {
        name: Token,
        generics: List<GenericParameter>,
        fields: List<DataField>,
        predicate: Option<RefinementExprId>,
        span: Span
    }

    Enum {
        name: Token,
        generics: List<GenericParameter>,
        variants: List<EnumVariantDecl>,
        span: Span
    }

    Trait {
        name: Token,
        generics: List<GenericParameter>,
        body: List<DeclId>,
        span: Span
    }

    Impl {
        target: TypeExprId,
        trait: Option<TypeExprId>,
        whereClause: List<ConstraintSyntax>,
        body: List<DeclId>,
        span: Span
    }

    TypeAlias {
        name: Token,
        generics: List<GenericParameter>,
        value: TypeExprId,
        span: Span
    }

    Method {
        name: Token,
        parameters: List<MethodParameter>,
        result: Option<TypeExprId>,
        whereClause: List<ConstraintSyntax>,
        body: Option<ExprId>,
        span: Span
    }

    Error {
        span: Span
    }
}


data GenericParameter(
    name: Token,
    type: Option<TypeExprId>
)


data DataField(
    name: Token,
    type: TypeExprId
)


data EnumVariantDecl {
    Unit {
        name: Token,
        span: Span
    }

    Tuple {
        name: Token,
        payload: List<TypeExprId>,
        span: Span
    }

    Record {
        name: Token,
        fields: List<DataField>,
        span: Span
    }
}


data MethodParameter(
    label: Option<Token>,
    name: Token,
    type: TypeExprId
)


data ConstraintSyntax {
    Refinement(RefinementExprId)

    Conformance {
        left: TypeExprId,
        right: TypeExprId
    }

    Subtype {
        left: TypeExprId,
        right: TypeExprId
    }

    Equality {
        left: TypeExprId,
        right: TypeExprId
    }
}


// -----------------------------------------------------------------------------
// Arena
// -----------------------------------------------------------------------------

class AstArena {
    var expressions: List<Expr>
    var statements: List<Stmt>
    var types: List<TypeExpr>
    var patterns: List<Pattern>
    var refinements: List<RefinementExpr>
    var declarations: List<Decl>
}

impl AstArena {
    @class
    new() -> AstArena {
        AstArena(
            expressions: List<Expr>.new(),
            statements: List<Stmt>.new(),
            types: List<TypeExpr>.new(),
            patterns: List<Pattern>.new(),
            refinements: List<RefinementExpr>.new(),
            declarations: List<Decl>.new()
        )
    }

    addExpr(_ expression: Expr) -> ExprId {
        let id = ExprId(expressions.length)
        expressions.push(expression)
        id
    }

    addStmt(_ statement: Stmt) -> StmtId {
        let id = StmtId(statements.length)
        statements.push(statement)
        id
    }

    addType(_ type: TypeExpr) -> TypeExprId {
        let id = TypeExprId(types.length)
        types.push(type)
        id
    }

    addPattern(_ pattern: Pattern) -> PatternId {
        let id = PatternId(patterns.length)
        patterns.push(pattern)
        id
    }

    addRefinement(_ expression: RefinementExpr) -> RefinementExprId {
        let id = RefinementExprId(refinements.length)
        refinements.push(expression)
        id
    }

    addDecl(_ declaration: Decl) -> DeclId {
        let id = DeclId(declarations.length)
        declarations.push(declaration)
        id
    }

    expr(_ id: ExprId) -> Expr {
        expressions[id._value]
    }

    stmt(_ id: StmtId) -> Stmt {
        statements[id._value]
    }

    type(_ id: TypeExprId) -> TypeExpr {
        types[id._value]
    }

    pattern(_ id: PatternId) -> Pattern {
        patterns[id._value]
    }

    refinement(_ id: RefinementExprId) -> RefinementExpr {
        refinements[id._value]
    }

    declaration(_ id: DeclId) -> Decl {
        declarations[id._value]
    }
}


// -----------------------------------------------------------------------------
// Pratt rules
// -----------------------------------------------------------------------------

data InfixRule(
    operator: BinaryOperator,
    left: BindingPower,
    right: BindingPower
)


// -----------------------------------------------------------------------------
// Parser
// -----------------------------------------------------------------------------

class Parser {
    tokens: TokenBuffer
    arena: AstArena

    var cursor: Natural
    var diagnostics: List<Diagnostic>
}

impl Parser {
    @class
    parse(_ tokens: TokenBuffer) -> ParseResult {
        let parser = Parser(
            tokens: tokens,
            arena: AstArena.new(),
            cursor: 0,
            diagnostics: List<Diagnostic>.new()
        )

        let declarations = parser.parseModule()

        ParseResult(
            declarations: declarations,
            arena: parser.arena,
            diagnostics: parser.diagnostics
        )
    }

    parseModule -> List<DeclId> {
        let declarations = List<DeclId>.new()

        skipSeparators()

        while current.kind != Eof {
            declarations.push(parseDeclaration())
            skipSeparators()
        }

        declarations
    }

    // -------------------------------------------------------------------------
    // Token primitives
    // -------------------------------------------------------------------------

    current -> Token {
        tokens.atOrEof(cursor)
    }

    previous -> Token {
        if cursor == 0 {
            current
        } else {
            tokens.atOrEof(cursor - 1)
        }
    }

    peek(_ distance: Natural) -> Token {
        tokens.atOrEof(cursor + distance)
    }

    at(_ kind: TokenKind) -> Bool {
        current.kind == kind
    }

    atKeyword(_ keyword: Keyword) -> Bool {
        current.kind == Keyword(keyword)
    }

    advance -> Token {
        let token = current

        if token.kind != Eof {
            cursor += 1
        }

        token
    }

    advanceIf(_ kind: TokenKind) -> Option<Token> {
        if current.kind == kind {
            Some(advance)
        } else {
            None
        }
    }

    advanceIfKeyword(_ keyword: Keyword) -> Option<Token> {
        if current.kind == Keyword(keyword) {
            Some(advance)
        } else {
            None
        }
    }

    expect(_ kind: TokenKind) -> Token {
        if current.kind == kind {
            return advance
        }

        report(
            ExpectedToken(kind),
            current.span,
            "expected token"
        )

        // Recovery uses a synthetic zero-width token at the current position.
        Token(
            kind,
            Span.empty(at: current.span.start)
        )
    }

    expectIdentifier -> Token {
        match current.kind {
            Identifier => advance

            _ => {
                report(
                    UnexpectedToken,
                    current.span,
                    "expected identifier"
                )

                Token(
                    Identifier,
                    Span.empty(at: current.span.start)
                )
            }
        }
    }

    skipSeparators {
        while match current.kind {
            Newline | Semicolon => true
            _ => false
        } {
            advance
        }
    }

    report(
        _ kind: DiagnosticKind,
        _ span: Span,
        _ message: String
    ) {
        diagnostics.push(
            Diagnostic(kind, span, message)
        )
    }

    // -------------------------------------------------------------------------
    // Declarations
    // -------------------------------------------------------------------------

    parseDeclaration -> DeclId {
        match current.kind {
            Keyword(Class) => parseClassDeclaration()
            Keyword(Data) => parseDataDeclaration()
            Keyword(Enum) => parseEnumDeclaration()
            Keyword(Trait) => parseTraitDeclaration()
            Keyword(Impl) => parseImplDeclaration()
            Keyword(Type) => parseTypeAliasDeclaration()

            Identifier if peek(1).kind == LeftParen =>
                parseMethodDeclaration()

            _ => {
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
        }
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
            let variantStart = current.span.start
            let variantName = expectIdentifier()

            let variant =
                match current.kind {
                    LeftParen => {
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
                            name: variantName,
                            payload: payload,
                            span: Span(variantStart, close.span.end)
                        }
                    }

                    LeftBrace => {
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
                            name: variantName,
                            fields: fields,
                            span: Span(variantStart, close.span.end)
                        }
                    }

                    _ =>
                        EnumVariantDecl::Unit {
                            name: variantName,
                            span: variantName.span
                        }
                }

            variants.push(variant)

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
                (
                    Some(first),
                    parseType()
                )
            } else {
                (
                    None,
                    first
                )
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
        let end = typeSpan(value).end

        arena.addDecl(
            Decl::TypeAlias {
                name: name,
                generics: generics,
                value: value,
                span: Span(start, end)
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
                // Initial draft keeps selector labels and binding names equal.
                // The AST shape allows them to diverge later.
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

        // Controlled V1: parse a comma-separated list of refinement
        // propositions. Trait/subtype/equality constraint forms can be added
        // to this dispatch without changing the predicate parser.
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

    // -------------------------------------------------------------------------
    // Statements / blocks
    // -------------------------------------------------------------------------

    parseStatement -> StmtId {
        match current.kind {
            Keyword(Let) =>
                parseLetStatement(mutable: false)

            Keyword(Var) =>
                parseLetStatement(mutable: true)

            Keyword(Return) =>
                parseReturnStatement()

            Keyword(Break) =>
                parseBreakStatement()

            Keyword(Continue) => {
                let token = advance

                arena.addStmt(
                    Stmt::Continue {
                        span: token.span
                    }
                )
            }

            _ => {
                let expression = parseExpression()
                let span = exprSpan(expression)

                arena.addStmt(
                    Stmt::Expr {
                        expression: expression,
                        span: span
                    }
                )
            }
        }
    }

    parseLetStatement(mutable: Bool) -> StmtId {
        let start = advance.span.start
        let name = expectIdentifier()

        let type =
            match advanceIf(Colon) {
                Some(_) => Some(parseType())
                None => None
            }

        let value =
            match advanceIf(Equal) {
                Some(_) => Some(parseExpression())
                None => None
            }

        let end =
            match value {
                Some(expr) => exprSpan(expr).end
                None => previous.span.end
            }

        arena.addStmt(
            Stmt::Let {
                mutable: mutable,
                name: name,
                type: type,
                value: value,
                span: Span(start, end)
            }
        )
    }

    parseReturnStatement -> StmtId {
        let start = advance.span.start

        let value =
            if beginsExpression(current.kind) {
                Some(parseExpression())
            } else {
                None
            }

        let end =
            match value {
                Some(expr) => exprSpan(expr).end
                None => previous.span.end
            }

        arena.addStmt(
            Stmt::Return {
                value: value,
                span: Span(start, end)
            }
        )
    }

    parseBreakStatement -> StmtId {
        let start = advance.span.start

        let value =
            if beginsExpression(current.kind) {
                Some(parseExpression())
            } else {
                None
            }

        let end =
            match value {
                Some(expr) => exprSpan(expr).end
                None => previous.span.end
            }

        arena.addStmt(
            Stmt::Break {
                value: value,
                span: Span(start, end)
            }
        )
    }

    parseBlockExpression -> ExprId {
        let open = expect(LeftBrace)
        let statements = List<StmtId>.new()
        var trailing: Option<ExprId> = None

        skipSeparators()

        while current.kind != RightBrace
            and current.kind != Eof
        {
            let statement = parseStatement()

            match arena.stmt(statement) {
                Stmt::Expr { expression, .. } if current.kind == RightBrace => {
                    trailing = Some(expression)
                }

                _ =>
                    statements.push(statement)
            }

            skipSeparators()
        }

        let close = expect(RightBrace)

        arena.addExpr(
            Expr::Block {
                statements: statements,
                trailing: trailing,
                span: Span(open.span.start, close.span.end)
            }
        )
    }

    // -------------------------------------------------------------------------
    // Expressions
    // -------------------------------------------------------------------------

    parseExpression -> ExprId {
        parseExpressionWithPower(0)
    }

    parseExpressionWithPower(
        _ minimum: BindingPower
    ) -> ExprId {
        var left = parsePrefixExpression()

        while true {
            // Postfix/member/call binding is intentionally above all infix
            // operators and handled before the Pratt table.
            match current.kind {
                LeftParen => {
                    left = parseCallExpression(left)
                    continue
                }

                Dot => {
                    left = parseMemberExpression(left)
                    continue
                }

                DoubleColon => {
                    left = parseAssociatedExpression(left)
                    continue
                }

                _ => ()
            }

            match infixRule(current.kind) {
                None =>
                    break

                Some(rule) if rule.left < minimum =>
                    break

                Some(rule) => {
                    let operatorToken = advance
                    let right =
                        parseExpressionWithPower(rule.right)

                    let span =
                        Span(
                            exprSpan(left).start,
                            exprSpan(right).end
                        )

                    left =
                        arena.addExpr(
                            Expr::Binary {
                                left: left,
                                operator: rule.operator,
                                right: right,
                                span: span
                            }
                        )
                }
            }
        }

        left
    }

    parsePrefixExpression -> ExprId {
        match current.kind {
            Integer => {
                let token = advance

                arena.addExpr(
                    Expr::Integer {
                        token: token,
                        span: token.span
                    }
                )
            }

            Float => {
                let token = advance

                arena.addExpr(
                    Expr::Float {
                        token: token,
                        span: token.span
                    }
                )
            }

            String => {
                let token = advance

                arena.addExpr(
                    Expr::String {
                        token: token,
                        span: token.span
                    }
                )
            }

            Character => {
                let token = advance

                arena.addExpr(
                    Expr::Character {
                        token: token,
                        span: token.span
                    }
                )
            }

            Keyword(True) => {
                let token = advance

                arena.addExpr(
                    Expr::Bool {
                        value: true,
                        span: token.span
                    }
                )
            }

            Keyword(False) => {
                let token = advance

                arena.addExpr(
                    Expr::Bool {
                        value: false,
                        span: token.span
                    }
                )
            }

            Identifier => {
                let token = advance

                arena.addExpr(
                    Expr::Name {
                        name: token,
                        span: token.span
                    }
                )
            }

            LeftParen =>
                parseParenOrTupleExpression()

            LeftBracket =>
                parseListExpression()

            LeftBrace =>
                parseBlockExpression()

            Keyword(If) =>
                parseIfExpression()

            Keyword(Match) =>
                parseMatchExpression()

            Pipe =>
                parseClosureExpression()

            Plus | Minus | Bang | Keyword(Not) | Ampersand =>
                parseUnaryExpression()

            _ => {
                let span = current.span

                report(
                    ExpectedExpression,
                    span,
                    "expected expression"
                )

                advance

                arena.addExpr(
                    Expr::Error {
                        span: span
                    }
                )
            }
        }
    }

    parseParenOrTupleExpression -> ExprId {
        let open = expect(LeftParen)

        if current.kind == RightParen {
            let close = advance

            return arena.addExpr(
                Expr::Tuple {
                    elements: List<ExprId>.new(),
                    span: Span(open.span.start, close.span.end)
                }
            )
        }

        let first = parseExpression()

        if current.kind != Comma {
            expect(RightParen)
            return first
        }

        let elements = List<ExprId>.new()
        elements.push(first)

        while advanceIf(Comma).isSome {
            if current.kind == RightParen {
                break
            }

            elements.push(parseExpression())
        }

        let close = expect(RightParen)

        arena.addExpr(
            Expr::Tuple {
                elements: elements,
                span: Span(open.span.start, close.span.end)
            }
        )
    }

    parseListExpression -> ExprId {
        let open = expect(LeftBracket)
        let elements = List<ExprId>.new()

        if current.kind != RightBracket {
            while true {
                elements.push(parseExpression())

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        let close = expect(RightBracket)

        arena.addExpr(
            Expr::List {
                elements: elements,
                span: Span(open.span.start, close.span.end)
            }
        )
    }

    parseUnaryExpression -> ExprId {
        let operatorToken = advance

        let operator =
            match operatorToken.kind {
                Plus => Positive
                Minus => Negative
                Bang | Keyword(Not) => LogicalNot
                Ampersand => Reference
                _ => unreachable
            }

        let operand = parseExpressionWithPower(90)

        arena.addExpr(
            Expr::Unary {
                operator: operator,
                operand: operand,
                span: Span(
                    operatorToken.span.start,
                    exprSpan(operand).end
                )
            }
        )
    }

    parseCallExpression(_ callee: ExprId) -> ExprId {
        let arguments = List<CallArgument>.new()

        expect(LeftParen)

        if current.kind != RightParen {
            while true {
                let label =
                    if current.kind == Identifier
                        and peek(1).kind == Colon
                    {
                        let label = advance
                        advance
                        Some(label)
                    } else {
                        None
                    }

                arguments.push(
                    CallArgument(
                        label,
                        parseExpression()
                    )
                )

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        let close = expect(RightParen)

        arena.addExpr(
            Expr::Call {
                callee: callee,
                arguments: arguments,
                span: Span(
                    exprSpan(callee).start,
                    close.span.end
                )
            }
        )
    }

    parseMemberExpression(_ receiver: ExprId) -> ExprId {
        expect(Dot)
        let member = expectIdentifier()

        arena.addExpr(
            Expr::Member {
                receiver: receiver,
                member: member,
                span: Span(
                    exprSpan(receiver).start,
                    member.span.end
                )
            }
        )
    }

    parseAssociatedExpression(_ receiver: ExprId) -> ExprId {
        expect(DoubleColon)
        let member = expectIdentifier()

        arena.addExpr(
            Expr::Associated {
                receiver: receiver,
                member: member,
                span: Span(
                    exprSpan(receiver).start,
                    member.span.end
                )
            }
        )
    }

    parseIfExpression -> ExprId {
        let start = expect(Keyword(If)).span.start
        let condition = parseExpression()
        let thenBranch = parseBlockExpression()

        let elseBranch =
            match advanceIfKeyword(Else) {
                None => None

                Some(_) =>
                    if atKeyword(If) {
                        Some(parseIfExpression())
                    } else {
                        Some(parseBlockExpression())
                    }
            }

        let end =
            match elseBranch {
                Some(branch) => exprSpan(branch).end
                None => exprSpan(thenBranch).end
            }

        arena.addExpr(
            Expr::If {
                condition: condition,
                thenBranch: thenBranch,
                elseBranch: elseBranch,
                span: Span(start, end)
            }
        )
    }

    parseMatchExpression -> ExprId {
        let start = expect(Keyword(Match)).span.start
        let subject = parseExpression()

        expect(LeftBrace)

        let arms = List<MatchArm>.new()

        skipSeparators()

        while current.kind != RightBrace
            and current.kind != Eof
        {
            let armStart = current.span.start
            let pattern = parsePattern()

            let guard =
                match advanceIfKeyword(If) {
                    Some(_) => Some(parseExpression())
                    None => None
                }

            expect(FatArrow)

            let body = parseExpression()

            arms.push(
                MatchArm(
                    pattern: pattern,
                    guard: guard,
                    body: body,
                    span: Span(
                        armStart,
                        exprSpan(body).end
                    )
                )
            )

            advanceIf(Comma)
            skipSeparators()
        }

        let close = expect(RightBrace)

        arena.addExpr(
            Expr::Match {
                subject: subject,
                arms: arms,
                span: Span(start, close.span.end)
            }
        )
    }

    parseClosureExpression -> ExprId {
        let start = expect(Pipe).span.start
        let parameters = List<ClosureParameter>.new()

        if current.kind != Pipe {
            while true {
                let name = expectIdentifier()

                let type =
                    match advanceIf(Colon) {
                        Some(_) => Some(parseType())
                        None => None
                    }

                parameters.push(
                    ClosureParameter(name, type)
                )

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        expect(Pipe)

        let body =
            if current.kind == LeftBrace {
                parseBlockExpression()
            } else {
                parseExpression()
            }

        arena.addExpr(
            Expr::Closure {
                parameters: parameters,
                body: body,
                span: Span(start, exprSpan(body).end)
            }
        )
    }

    infixRule(_ kind: TokenKind) -> Option<InfixRule> {
        match kind {
            Keyword(Or) =>
                Some(InfixRule(LogicalOr, 10, 11))

            Keyword(And) =>
                Some(InfixRule(LogicalAnd, 20, 21))

            EqualEqual =>
                Some(InfixRule(Equal, 30, 31))

            BangEqual =>
                Some(InfixRule(NotEqual, 30, 31))

            Less =>
                Some(InfixRule(Less, 40, 41))

            LessEqual =>
                Some(InfixRule(LessEqual, 40, 41))

            Greater =>
                Some(InfixRule(Greater, 40, 41))

            GreaterEqual =>
                Some(InfixRule(GreaterEqual, 40, 41))

            Keyword(Is) =>
                Some(InfixRule(Is, 40, 41))

            Keyword(As) =>
                Some(InfixRule(As, 40, 41))

            Plus =>
                Some(InfixRule(Add, 50, 51))

            Minus =>
                Some(InfixRule(Subtract, 50, 51))

            Star =>
                Some(InfixRule(Multiply, 60, 61))

            Slash =>
                Some(InfixRule(Divide, 60, 61))

            Percent =>
                Some(InfixRule(Remainder, 60, 61))

            _ =>
                None
        }
    }

    beginsExpression(_ kind: TokenKind) -> Bool {
        match kind {
            Identifier
            | Integer
            | Float
            | String
            | Character
            | LeftParen
            | LeftBracket
            | LeftBrace
            | Pipe
            | Plus
            | Minus
            | Bang
            | Ampersand
            | Keyword(True)
            | Keyword(False)
            | Keyword(If)
            | Keyword(Match)
            | Keyword(Not) =>
                true

            _ =>
                false
        }
    }

    // -------------------------------------------------------------------------
    // Type syntax
    // -------------------------------------------------------------------------

    parseType -> TypeExprId {
        parseRefinedType()
    }

    parseRefinedType -> TypeExprId {
        let base = parseUnionType()

        if not atKeyword(Where) {
            return base
        }

        advance

        let predicate = parseRefinementPredicate()

        arena.addType(
            TypeExpr::Refined {
                base: base,
                predicate: predicate,
                span: Span(
                    typeSpan(base).start,
                    refinementSpan(predicate).end
                )
            }
        )
    }

    parseUnionType -> TypeExprId {
        var left = parseIntersectionType()

        while current.kind == Pipe {
            advance
            let right = parseIntersectionType()

            left =
                arena.addType(
                    TypeExpr::Union {
                        left: left,
                        right: right,
                        span: Span(
                            typeSpan(left).start,
                            typeSpan(right).end
                        )
                    }
                )
        }

        left
    }

    parseIntersectionType -> TypeExprId {
        var left = parsePostfixType()

        while current.kind == Ampersand {
            advance
            let right = parsePostfixType()

            left =
                arena.addType(
                    TypeExpr::Intersection {
                        left: left,
                        right: right,
                        span: Span(
                            typeSpan(left).start,
                            typeSpan(right).end
                        )
                    }
                )
        }

        left
    }

    parsePostfixType -> TypeExprId {
        var base = parsePrimaryType()

        while true {
            match current.kind {
                Less => {
                    let start = typeSpan(base).start
                    let arguments = parseTypeArguments()
                    let end = previous.span.end

                    base =
                        arena.addType(
                            TypeExpr::Generic {
                                base: base,
                                arguments: arguments,
                                span: Span(start, end)
                            }
                        )
                }

                DoubleColon => {
                    advance
                    let member = expectIdentifier()

                    base =
                        arena.addType(
                            TypeExpr::Associated {
                                base: base,
                                member: member,
                                span: Span(
                                    typeSpan(base).start,
                                    member.span.end
                                )
                            }
                        )
                }

                _ =>
                    break
            }
        }

        base
    }

    parsePrimaryType -> TypeExprId {
        match current.kind {
            Identifier => {
                let name = advance

                arena.addType(
                    TypeExpr::Name {
                        name: name,
                        span: name.span
                    }
                )
            }

            LeftParen =>
                parseTupleOrGroupedType()

            Pipe =>
                parseCallableType()

            _ => {
                let span = current.span

                report(
                    ExpectedType,
                    span,
                    "expected type"
                )

                advance

                arena.addType(
                    TypeExpr::Error {
                        span: span
                    }
                )
            }
        }
    }

    parseTupleOrGroupedType -> TypeExprId {
        let open = expect(LeftParen)

        if current.kind == RightParen {
            let close = advance

            return arena.addType(
                TypeExpr::Tuple {
                    elements: List<TupleTypeElement>.new(),
                    span: Span(open.span.start, close.span.end)
                }
            )
        }

        // Named tuple element: `name: Type`
        let first =
            if current.kind == Identifier
                and peek(1).kind == Colon
            {
                let name = advance
                advance

                Named {
                    name: name,
                    type: parseType()
                }
            } else {
                Positional(parseType())
            }

        if current.kind != Comma {
            match first {
                Positional(type) => {
                    expect(RightParen)
                    return type
                }

                Named { .. } => ()
            }
        }

        let elements = List<TupleTypeElement>.new()
        elements.push(first)

        while advanceIf(Comma).isSome {
            if current.kind == RightParen {
                break
            }

            if current.kind == Identifier
                and peek(1).kind == Colon
            {
                let name = advance
                advance

                elements.push(
                    Named {
                        name: name,
                        type: parseType()
                    }
                )
            } else {
                elements.push(
                    Positional(parseType())
                )
            }
        }

        let close = expect(RightParen)

        arena.addType(
            TypeExpr::Tuple {
                elements: elements,
                span: Span(open.span.start, close.span.end)
            }
        )
    }

    parseCallableType -> TypeExprId {
        let start = expect(Pipe).span.start
        let parameters = List<TypeExprId>.new()

        if current.kind != Pipe {
            while true {
                parameters.push(parseType())

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        expect(Pipe)
        expect(Arrow)

        let result = parseType()

        arena.addType(
            TypeExpr::Callable {
                parameters: parameters,
                result: result,
                span: Span(start, typeSpan(result).end)
            }
        )
    }

    parseTypeArguments -> List<TypeArgument> {
        let arguments = List<TypeArgument>.new()

        expect(Less)

        if current.kind != Greater {
            while true {
                // Initial stress-test policy:
                // literals are parsed as value arguments; everything else
                // defaults to a type argument.
                match current.kind {
                    Integer | Float | String | Character =>
                        arguments.push(
                            Value(parseExpression())
                        )

                    _ =>
                        arguments.push(
                            Type(parseType())
                        )
                }

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        expect(Greater)

        arguments
    }

    // -------------------------------------------------------------------------
    // Patterns
    // -------------------------------------------------------------------------

    parsePattern -> PatternId {
        parseOrPattern()
    }

    parseOrPattern -> PatternId {
        let first = parseRangePattern()

        if current.kind != Pipe {
            return first
        }

        let alternatives = List<PatternId>.new()
        alternatives.push(first)

        while current.kind == Pipe {
            advance
            alternatives.push(parseRangePattern())
        }

        let span =
            Span(
                patternSpan(alternatives.first).start,
                patternSpan(alternatives.last).end
            )

        arena.addPattern(
            Pattern::Or {
                alternatives: alternatives,
                span: span
            }
        )
    }

    parseRangePattern -> PatternId {
        let startPattern = parsePrimaryPattern()

        match current.kind {
            DotDot | DotDotEqual => {
                let inclusive = current.kind == DotDotEqual
                advance

                let endPattern = parsePrimaryPattern()

                arena.addPattern(
                    Pattern::Range {
                        start: startPattern,
                        end: endPattern,
                        inclusive: inclusive,
                        span: Span(
                            patternSpan(startPattern).start,
                            patternSpan(endPattern).end
                        )
                    }
                )
            }

            _ =>
                startPattern
        }
    }

    parsePrimaryPattern -> PatternId {
        match current.kind {
            Identifier => {
                let name = advance

                if tokens.text(name) == "_" {
                    return arena.addPattern(
                        Pattern::Wildcard {
                            span: name.span
                        }
                    )
                }

                if current.kind == LeftParen {
                    let payload = List<PatternId>.new()
                    let start = name.span.start

                    advance

                    if current.kind != RightParen {
                        while true {
                            payload.push(parsePattern())

                            match advanceIf(Comma) {
                                Some(_) => ()
                                None => break
                            }
                        }
                    }

                    let close = expect(RightParen)

                    return arena.addPattern(
                        Pattern::Variant {
                            name: name,
                            payload: payload,
                            span: Span(start, close.span.end)
                        }
                    )
                }

                arena.addPattern(
                    Pattern::Name {
                        name: name,
                        span: name.span
                    }
                )
            }

            Integer => {
                let token = advance

                arena.addPattern(
                    Pattern::Integer {
                        token: token,
                        span: token.span
                    }
                )
            }

            String => {
                let token = advance

                arena.addPattern(
                    Pattern::String {
                        token: token,
                        span: token.span
                    }
                )
            }

            LeftParen => {
                let open = advance
                let elements = List<PatternId>.new()

                if current.kind != RightParen {
                    while true {
                        elements.push(parsePattern())

                        match advanceIf(Comma) {
                            Some(_) => ()
                            None => break
                        }
                    }
                }

                let close = expect(RightParen)

                arena.addPattern(
                    Pattern::Tuple {
                        elements: elements,
                        span: Span(open.span.start, close.span.end)
                    }
                )
            }

            _ => {
                let span = current.span

                report(
                    ExpectedPattern,
                    span,
                    "expected pattern"
                )

                advance

                arena.addPattern(
                    Pattern::Error {
                        span: span
                    }
                )
            }
        }
    }

    // -------------------------------------------------------------------------
    // Controlled refinement predicate grammar
    // -------------------------------------------------------------------------

    parseRefinementPredicate -> RefinementExprId {
        parseRefinementOr()
    }

    parseRefinementOr -> RefinementExprId {
        var left = parseRefinementAnd()

        while atKeyword(Or) {
            advance
            let right = parseRefinementAnd()

            left =
                makeRefinementBinary(
                    left,
                    Or,
                    right
                )
        }

        left
    }

    parseRefinementAnd -> RefinementExprId {
        var left = parseRefinementComparison()

        while atKeyword(And) {
            advance
            let right = parseRefinementComparison()

            left =
                makeRefinementBinary(
                    left,
                    And,
                    right
                )
        }

        left
    }

    parseRefinementComparison -> RefinementExprId {
        var left = parseRefinementAdditive()

        while true {
            let operator =
                match current.kind {
                    EqualEqual => Some(Equal)
                    BangEqual => Some(NotEqual)
                    Less => Some(Less)
                    LessEqual => Some(LessEqual)
                    Greater => Some(Greater)
                    GreaterEqual => Some(GreaterEqual)
                    _ => None
                }

            match operator {
                None =>
                    break

                Some(operator) => {
                    advance
                    let right = parseRefinementAdditive()

                    left =
                        makeRefinementBinary(
                            left,
                            operator,
                            right
                        )
                }
            }
        }

        left
    }

    parseRefinementAdditive -> RefinementExprId {
        var left = parseRefinementMultiplicative()

        while true {
            let operator =
                match current.kind {
                    Plus => Some(Add)
                    Minus => Some(Subtract)
                    _ => None
                }

            match operator {
                None =>
                    break

                Some(operator) => {
                    advance
                    let right =
                        parseRefinementMultiplicative()

                    left =
                        makeRefinementBinary(
                            left,
                            operator,
                            right
                        )
                }
            }
        }

        left
    }

    parseRefinementMultiplicative -> RefinementExprId {
        var left = parseRefinementUnary()

        while true {
            let operator =
                match current.kind {
                    Star => Some(Multiply)
                    Slash => Some(Divide)
                    Percent => Some(Remainder)
                    _ => None
                }

            match operator {
                None =>
                    break

                Some(operator) => {
                    advance
                    let right =
                        parseRefinementUnary()

                    left =
                        makeRefinementBinary(
                            left,
                            operator,
                            right
                        )
                }
            }
        }

        left
    }

    parseRefinementUnary -> RefinementExprId {
        match current.kind {
            Plus | Minus | Keyword(Not) | Bang => {
                let operatorToken = advance

                let operator =
                    match operatorToken.kind {
                        Plus => Positive
                        Minus => Negative
                        Keyword(Not) | Bang => Not
                        _ => unreachable
                    }

                let operand = parseRefinementUnary()

                arena.addRefinement(
                    RefinementExpr::Unary {
                        operator: operator,
                        operand: operand,
                        span: Span(
                            operatorToken.span.start,
                            refinementSpan(operand).end
                        )
                    }
                )
            }

            _ =>
                parseRefinementPrimary()
        }
    }

    parseRefinementPrimary -> RefinementExprId {
        var expression =
            match current.kind {
                Integer => {
                    let token = advance

                    arena.addRefinement(
                        RefinementExpr::Integer {
                            token: token,
                            span: token.span
                        }
                    )
                }

                Float => {
                    let token = advance

                    arena.addRefinement(
                        RefinementExpr::Float {
                            token: token,
                            span: token.span
                        }
                    )
                }

                Keyword(True) => {
                    let token = advance

                    arena.addRefinement(
                        RefinementExpr::Bool {
                            value: true,
                            span: token.span
                        }
                    )
                }

                Keyword(False) => {
                    let token = advance

                    arena.addRefinement(
                        RefinementExpr::Bool {
                            value: false,
                            span: token.span
                        }
                    )
                }

                Keyword(Self) => {
                    let token = advance

                    arena.addRefinement(
                        RefinementExpr::Self {
                            span: token.span
                        }
                    )
                }

                Identifier => {
                    let name = advance

                    arena.addRefinement(
                        RefinementExpr::Name {
                            name: name,
                            span: name.span
                        }
                    )
                }

                LeftParen => {
                    advance
                    let nested = parseRefinementPredicate()
                    expect(RightParen)
                    nested
                }

                _ => {
                    let span = current.span

                    report(
                        InvalidRefinementExpression,
                        span,
                        "expected refinement proposition operand"
                    )

                    advance

                    arena.addRefinement(
                        RefinementExpr::Error {
                            span: span
                        }
                    )
                }
            }

        // Only structural member projection is represented here.
        // Calls are not part of RefinementExpr and therefore cannot be parsed
        // accidentally as legal controlled refinements.
        while current.kind == Dot {
            advance

            let member = expectIdentifier()

            expression =
                arena.addRefinement(
                    RefinementExpr::Member {
                        receiver: expression,
                        member: member,
                        span: Span(
                            refinementSpan(expression).start,
                            member.span.end
                        )
                    }
                )
        }

        expression
    }

    makeRefinementBinary(
        _ left: RefinementExprId,
        _ operator: RefinementBinaryOperator,
        _ right: RefinementExprId
    ) -> RefinementExprId {
        arena.addRefinement(
            RefinementExpr::Binary {
                left: left,
                operator: operator,
                right: right,
                span: Span(
                    refinementSpan(left).start,
                    refinementSpan(right).end
                )
            }
        )
    }

    // -------------------------------------------------------------------------
    // Recovery
    // -------------------------------------------------------------------------

    recoverDeclaration {
        while current.kind != Eof {
            match current.kind {
                Keyword(Class)
                | Keyword(Data)
                | Keyword(Enum)
                | Keyword(Trait)
                | Keyword(Impl)
                | Keyword(Type)
                | RightBrace =>
                    return

                _ =>
                    advance
            }
        }
    }

    // -------------------------------------------------------------------------
    // Span helpers
    // -------------------------------------------------------------------------

    exprSpan(_ id: ExprId) -> Span {
        match arena.expr(id) {
            Integer { span, .. }
            | Float { span, .. }
            | String { span, .. }
            | Character { span, .. }
            | Bool { span, .. }
            | Name { span, .. }
            | Tuple { span, .. }
            | List { span, .. }
            | Unary { span, .. }
            | Binary { span, .. }
            | Call { span, .. }
            | Member { span, .. }
            | Associated { span, .. }
            | If { span, .. }
            | Match { span, .. }
            | Closure { span, .. }
            | Block { span, .. }
            | Error { span } =>
                span
        }
    }

    typeSpan(_ id: TypeExprId) -> Span {
        match arena.type(id) {
            Name { span, .. }
            | Tuple { span, .. }
            | Generic { span, .. }
            | Associated { span, .. }
            | Union { span, .. }
            | Intersection { span, .. }
            | Callable { span, .. }
            | Refined { span, .. }
            | Error { span } =>
                span
        }
    }

    patternSpan(_ id: PatternId) -> Span {
        match arena.pattern(id) {
            Wildcard { span }
            | Name { span, .. }
            | Integer { span, .. }
            | String { span, .. }
            | Variant { span, .. }
            | Tuple { span, .. }
            | Range { span, .. }
            | Type { span, .. }
            | Or { span, .. }
            | Error { span } =>
                span
        }
    }

    refinementSpan(_ id: RefinementExprId) -> Span {
        match arena.refinement(id) {
            Name { span, .. }
            | Self { span }
            | Integer { span, .. }
            | Float { span, .. }
            | Bool { span, .. }
            | Member { span, .. }
            | Unary { span, .. }
            | Binary { span, .. }
            | Error { span } =>
                span
        }
    }
}


data ParseResult(
    declarations: List<DeclId>,
    arena: AstArena,
    diagnostics: List<Diagnostic>
)
