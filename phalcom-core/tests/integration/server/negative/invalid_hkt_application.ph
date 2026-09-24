// Expected failure: Count(3) is a Retry static value, not a Type argument.
invalidHigherKindApplication<F>(
    value: F<Count(3)>
) -> F<Count(3)>
where F: Type -> Type
{
    value
}
