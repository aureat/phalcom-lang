@!documentation("Durable package dependency requirement descriptor.")
native class PackageRequirement is Object {
  native alias -> Symbol
  native package -> String
  native versionRequirement -> String
  native optional -> Bool
}

export PackageRequirement
