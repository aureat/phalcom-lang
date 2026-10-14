@!documentation("Cryptographic primitives and secure host services. Exposes security-sensitive operations; does not expose deterministic PRNGs.")

expose .random
expose .sha1

from .random import SecureRandom, EntropyError
from .sha1 import SHA1

export SecureRandom, EntropyError, SHA1
