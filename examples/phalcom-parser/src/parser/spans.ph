from parser.parser import Parser
from syntax.ast import Expr, TypeExpr, Pattern, RefinementExpr

impl Parser {
    exprSpan(_ id: ExprId) -> Span {
        exprSpan(arena.expr(id))
    }

    exprSpan(_ expr: Expr::Integer { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::Float { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::String { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::Character { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::Bool { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::Name { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::Tuple { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::List { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::Unary { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::Binary { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::Call { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::Member { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::Associated { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::If { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::Match { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::Closure { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::Block { span, .. }) -> Span { span }
    exprSpan(_ expr: Expr::Error { span }) -> Span { span }

    typeSpan(_ id: TypeExprId) -> Span {
        typeSpan(arena.type(id))
    }

    typeSpan(_ type: TypeExpr::Name { span, .. }) -> Span { span }
    typeSpan(_ type: TypeExpr::Tuple { span, .. }) -> Span { span }
    typeSpan(_ type: TypeExpr::Generic { span, .. }) -> Span { span }
    typeSpan(_ type: TypeExpr::Associated { span, .. }) -> Span { span }
    typeSpan(_ type: TypeExpr::Union { span, .. }) -> Span { span }
    typeSpan(_ type: TypeExpr::Intersection { span, .. }) -> Span { span }
    typeSpan(_ type: TypeExpr::Callable { span, .. }) -> Span { span }
    typeSpan(_ type: TypeExpr::Refined { span, .. }) -> Span { span }
    typeSpan(_ type: TypeExpr::Error { span }) -> Span { span }

    patternSpan(_ id: PatternId) -> Span {
        patternSpan(arena.pattern(id))
    }

    patternSpan(_ pattern: Pattern::Wildcard { span }) -> Span { span }
    patternSpan(_ pattern: Pattern::Name { span, .. }) -> Span { span }
    patternSpan(_ pattern: Pattern::Integer { span, .. }) -> Span { span }
    patternSpan(_ pattern: Pattern::String { span, .. }) -> Span { span }
    patternSpan(_ pattern: Pattern::Variant { span, .. }) -> Span { span }
    patternSpan(_ pattern: Pattern::Tuple { span, .. }) -> Span { span }
    patternSpan(_ pattern: Pattern::Range { span, .. }) -> Span { span }
    patternSpan(_ pattern: Pattern::Type { span, .. }) -> Span { span }
    patternSpan(_ pattern: Pattern::Or { span, .. }) -> Span { span }
    patternSpan(_ pattern: Pattern::Error { span }) -> Span { span }

    refinementSpan(_ id: RefinementExprId) -> Span {
        refinementSpan(arena.refinement(id))
    }

    refinementSpan(_ expr: RefinementExpr::Name { span, .. }) -> Span { span }
    refinementSpan(_ expr: RefinementExpr::Self { span }) -> Span { span }
    refinementSpan(_ expr: RefinementExpr::Integer { span, .. }) -> Span { span }
    refinementSpan(_ expr: RefinementExpr::Float { span, .. }) -> Span { span }
    refinementSpan(_ expr: RefinementExpr::Bool { span, .. }) -> Span { span }
    refinementSpan(_ expr: RefinementExpr::Member { span, .. }) -> Span { span }
    refinementSpan(_ expr: RefinementExpr::Unary { span, .. }) -> Span { span }
    refinementSpan(_ expr: RefinementExpr::Binary { span, .. }) -> Span { span }
    refinementSpan(_ expr: RefinementExpr::Error { span }) -> Span { span }
}
