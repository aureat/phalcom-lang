from core.domains import ExprId, StmtId, TypeExprId, PatternId, RefinementExprId, DeclId
from core.source import Span
from lex.token import Token

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
