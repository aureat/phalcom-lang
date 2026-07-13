type PhalcomSymDescriptor = {
    name: PhalcomVar,
    type?: PhalcomSymDescriptor,
    isGlobal: boolean
}

type PhalcomTypeDescriptor = PhalcomSymDescriptor & {
    name: PhalcomTypeName,
    super?: PhalcomTypeDescriptor,
    type?: PhalcomTypeDescriptor,
    isGlobal: boolean,
    methods: PhalcomSigString[]
}

type PhalcomObjectDescriptor = PhalcomSymDescriptor & {
    name: PhalcomVar,
    type?: PhalcomTypeDescriptor,
    isGlobal: boolean,
    isLocal?: boolean
}

type PhalcomSigParam = {
    name: string
    type?: PhalcomTypeName
}

type PhalcomSig = {
    name: string
    type?: PhalcomTypeName
    parameters?: PhalcomSigParam[]
}

type PhalcomSigString = string

type PhalcomVar = string
type PhalcomTypeName = string
type PhalcomVars = {
    [key: PhalcomVar]: PhalcomTypeName
}

type PhalcomMethod = {
    signatureString: PhalcomSigString
    signature: PhalcomSig
}

type PhalcomType = {
    super?: PhalcomType
    type?: PhalcomType
    methods: PhalcomMethod[]
}

type PhalcomContext = {
    globals: PhalcomVars,
    locals: PhalcomVars,
    types: {[key: PhalcomTypeName]: PhalcomType}
}

type PhalcomGlobal = {
    context?: PhalcomContext
}