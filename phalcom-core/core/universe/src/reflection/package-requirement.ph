@!documentation("Durable package dependency requirement descriptor.")
@native
class PackageRequirement is Object {
  @native alias -> Symbol
  @native package -> PackageInfo
  @native versionRequirement -> String
  @native optional -> Bool
}
