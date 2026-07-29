// Ordered search phases. The sealed family prevents impossible or misspelled
// phase state from entering Settings.

@data
@immutable
@sealed
class Phase {
  @variant Explicit
  @variant Reuse
  @variant Generate
  @variant Target
  @variant Shrink
  @variant Explain

  @class
  Explicit -> Phase => Explicit.new()

  @class
  Reuse -> Phase => Reuse.new()

  @class
  Generate -> Phase => Generate.new()

  @class
  Target -> Phase => Target.new()

  @class
  Shrink -> Phase => Shrink.new()

  @class
  Explain -> Phase => Explain.new()
}
