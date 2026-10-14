// Declaration-site variance is represented by ordinary immutable values.
// Unmarked parameters always use Variance::Invariant.
data Variance {
  Invariant
  Covariant
  Contravariant
}
