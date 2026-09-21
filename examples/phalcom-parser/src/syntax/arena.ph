from core.domains import ExprId, StmtId, TypeExprId, PatternId, RefinementExprId, DeclId
from syntax.ast import Expr, Stmt, TypeExpr, Pattern, RefinementExpr, Decl

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

    expr(_ id: ExprId) -> Expr { expressions[id._value] }
    stmt(_ id: StmtId) -> Stmt { statements[id._value] }
    type(_ id: TypeExprId) -> TypeExpr { types[id._value] }
    pattern(_ id: PatternId) -> Pattern { patterns[id._value] }
    refinement(_ id: RefinementExprId) -> RefinementExpr { refinements[id._value] }
    declaration(_ id: DeclId) -> Decl { declarations[id._value] }
}
