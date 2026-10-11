// area: bytes  status: NEGATIVE  spec: bytes.md law 2
let invalidOctets: List<Int | String> = [1, "two"]
Bytes.fromList(invalidOctets)
