// Declaration-site variance is represented by ordinary immutable values.
// Unmarked parameters always use Variance.Invariant.
@data
@immutable
@sealed
class Variance {
  @variant Invariant
  @variant Covariant
  @variant Contravariant
}
