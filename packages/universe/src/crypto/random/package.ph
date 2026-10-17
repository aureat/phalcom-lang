@!documentation("Cryptographically secure random byte generation backed by host operating-system entropy.")

from universe.random.secure import SecureRandom, EntropyError

export SecureRandom, EntropyError
