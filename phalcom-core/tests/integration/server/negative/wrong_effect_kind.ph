// EXPECT: kind mismatch; String has kind Type, not Type -> Type.
requiresEffectConstructor<F>()
where F: Type -> Type
{
    ()
}

negativeWrongEffectKind() {
    requiresEffectConstructor<String>()
}
