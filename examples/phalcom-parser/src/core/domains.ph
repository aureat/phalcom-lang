type Natural =
    Int where self >= 0

type Positive =
    Int where self > 0

type SourceOffset =
    Int where self >= 0

type BindingPower =
    Int where self >= 0


data ExprId private(_ value: Natural)
data StmtId private(_ value: Natural)
data TypeExprId private(_ value: Natural)
data PatternId private(_ value: Natural)
data RefinementExprId private(_ value: Natural)
data DeclId private(_ value: Natural)
