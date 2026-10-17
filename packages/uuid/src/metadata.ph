/// The version field is meaningful only for UUIDs of the RFC variant.
data UUIDVersion {
  TimeBased
  DceSecurity
  NameBasedMD5
  Random
  NameBasedSHA1
  ReorderedTime
  UnixTime
  Custom
  Reserved(_ field: Int)
}

data UUIDVariant {
  NCS
  RFC9562
  Microsoft
  Future
}

export UUIDVersion, UUIDVariant
