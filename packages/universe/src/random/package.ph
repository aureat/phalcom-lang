@!documentation("Explicit deterministic generators, typed distributions, and OS-backed secure random bytes. Xoshiro256PlusPlus is reproducible but not cryptographically secure; use SecureRandom for secrets.")

from .generator import Generator
from .xoshiro256pp import Xoshiro256PlusPlus
from .distribution import Distribution
from .uniform_int import UniformInt
from .bernoulli import Bernoulli
from .secure import SecureRandom, EntropyError

export Generator, Xoshiro256PlusPlus, Distribution, UniformInt, Bernoulli, SecureRandom, EntropyError
