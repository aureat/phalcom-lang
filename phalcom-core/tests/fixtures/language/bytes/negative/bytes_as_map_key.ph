// area: bytes  status: NEGATIVE  spec: collection-protocol law 4; PDR-0011 ruling 4
// Mutable => identity hash => rejected as a Map key (DEC-CT-C's rejection
// set, extended to Bytes); toTuple is the sanctioned escape hatch.
Map.new().at(Bytes.new(2), put: "bad")
