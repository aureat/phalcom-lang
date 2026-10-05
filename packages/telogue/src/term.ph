from .variable import Variable

type Term<T> = T | Variable<T>

export Term
