// Expected failure: row occurs-check. R cannot contain a record whose tail is R.
invalidRecursiveRow<R>(
    context: { next: Int, **R }
) -> ()
where
    R == { next: Int, **R }
{
    ()
}
