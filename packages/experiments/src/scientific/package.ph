expose .vector
expose .linear
expose .quantities
expose .model_vectors
import .model_vectors
from .vector import Vector, Point
from .linear import LinearMap, AffineMap
export Vector, Point, LinearMap, AffineMap
