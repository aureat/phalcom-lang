/*
 * Provides language features such as keywords, operators and methods to vsphalcom.
 * Method signatures with type annotations are used to signify methods
 */

/*
 * Type
 */
const TypeType: PhalcomTypeDescriptor = {
    name: "Type",
    super: null,
    type: null,
    isGlobal: true,
    methods: [
        "getMethod(signature: String): Method",
        "getMethods(name: String): Array",
        "getMethods(): Array",
    ]
}

/*
 * Object
 */
const ObjectType: PhalcomTypeDescriptor = {
    name: "Object",
    super: null,
    type: TypeType,
    isGlobal: true,
    methods: [
        "name: String",
        "name(value: String): Void",
        "type: Type",
        "type(value: Type): Void",
        "getField(): Object",
        "new(): Object",
        "construct(): Void",
        "null?: Boolean",
        "void?: Boolean",
        "callable?: Boolean",
        "toString: String",
        "toBoolean: Boolean",
        "is(other: Object): Boolean",
        "==(other: Object): Boolean",
        "!=(other: Object): Boolean"
    ]
}
TypeType.super = ObjectType
TypeType.type = TypeType

const SelfObject: PhalcomObjectDescriptor = {
    name: "self",
    type: ObjectType,
    isGlobal: false,
    isLocal: true
}

const SuperObject: PhalcomObjectDescriptor = {
    name: "super",
    type: ObjectType,
    isGlobal: false,
    isLocal: true
}

/*
 * Null + Void
 */
const NullType: PhalcomTypeDescriptor = {
    name: "Null",
    super: ObjectType,
    type: TypeType,
    isGlobal: false,
    methods: []
}

const VoidType: PhalcomTypeDescriptor = {
    name: "Void",
    super: NullType,
    type: TypeType,
    isGlobal: false,
    methods: []
}

const NullObject: PhalcomObjectDescriptor = {
    name: "null",
    type: NullType,
    isGlobal: true
}

const VoidObject: PhalcomObjectDescriptor = {
    name: "void",
    type: VoidType,
    isGlobal: true
}

/*
 * Number
 */
const NumberMetaType: PhalcomTypeDescriptor = {
    name: "NumberMeta",
    super: ObjectType,
    type: TypeType,
    isGlobal: false,
    methods: [
        "fromString(value: String): Number",
    ]
}

const NumberType: PhalcomTypeDescriptor = {
    name: "Number",
    super: ObjectType,
    type: NumberMetaType,
    isGlobal: true,
    methods: [
        "+: Number",
        "-: Number",
        "not: Boolean",
        "+(other: Number): Number",
        "-(other: Number): Number",
        "*(other: Number): Number",
        "/(other: Number): Number",
        "%(other: Number): Number",
        "<(other: Number): Boolean",
        ">(other: Number): Boolean",
        "<=(other: Number): Boolean",
        ">=(other: Number): Boolean",
        "==(other: Number): Boolean",
        "!=(other: Number): Boolean",
        "floor: Number",
        "ceiling: Number",
        "rounded: Number",
        "truncated: Number",
        "fractional: Number",
        "sqrt: Number",
        "abs: Number",
        "max(other: Number): Number",
        "min(other: Number): Number",
        "exp: Number",
        "expt: Number",
        "expt(base: Number): Number",
        "power(exponent: Number): Number",
        "log: Number",
        "ln: Number",
        "log10: Number",
        "log(base: Number): Number",
        "sin: Number",
        "asin: Number",
        "sinh: Number",
        "cos: Number",
        "acos: Number",
        "cosh: Number",
        "tan: Number",
        "atan: Number",
        "tanh: Number",
    ]
}

/*
 * Boolean
 */
const BooleanMetaType: PhalcomTypeDescriptor = {
    name: "BooleanMeta",
    super: ObjectType,
    type: TypeType,
    isGlobal: false,
    methods: [
        "fromString(value: String)",
        "fromNumber(value: Number)",
        "fromList(value: List)",
    ]
}

const BooleanType: PhalcomTypeDescriptor = {
    name: "Boolean",
    super: ObjectType,
    type: BooleanMetaType,
    isGlobal: true,
    methods: []
}

const TrueObject: PhalcomObjectDescriptor = {
    name: "true",
    type: BooleanType,
    isGlobal: true
}

const FalseObject: PhalcomObjectDescriptor = {
    name: "false",
    type: BooleanType,
    isGlobal: true
}

/*
 * String
 */
const StringMetaType: PhalcomTypeDescriptor = {
    name: "StringMeta",
    super: ObjectType,
    type: TypeType,
    isGlobal: false,
    methods: [
        "fromNumber(value: Number): String",
    ]
}

const StringType: PhalcomTypeDescriptor = {
    name: "String",
    super: ObjectType,
    type: StringMetaType,
    isGlobal: true,
    methods: [
        "+(other: String): String",
        "get(at: Number): String",
        "set(at: Number, with: Object): Void",
    ]
}

/*
 * List
 */
const ListMetaType: PhalcomTypeDescriptor = {
    name: "ListMeta",
    super: ObjectType,
    type: TypeType,
    isGlobal: false,
    methods: []
}

const ListType: PhalcomTypeDescriptor = {
    name: "List",
    super: ObjectType,
    type: ListMetaType,
    isGlobal: true,
    methods: [
        "size: Number",
        "put(element: Object): Void",
        "pop(element: Object): Object",
        "pop(at: Number): Object",
        "pop(): Object",
        "in(element: Object): Boolean",
        "get(at: Number): Object",
        "set(at: Number, with: Object): Void",
    ]
}

/*
 * Map
 */
const MapMetaType: PhalcomTypeDescriptor = {
    name: "MapMeta",
    super: ObjectType,
    type: TypeType,
    isGlobal: false,
    methods: []
}

const MapType: PhalcomTypeDescriptor = {
    name: "Map",
    super: ObjectType,
    type: MapMetaType,
    isGlobal: true,
    methods: [
        "size: Number",
        "in(element: Object): Boolean",
        "get(at: Object): Object",
        "set(at: Object, with: Object): Void",
    ]
}

/*
 * Phalcom Types
 */
export const types = [
    TypeType,
    ObjectType,
    NullType,
    VoidType,
    NumberMetaType,
    NumberType,
    BooleanMetaType,
    BooleanType,
    ListMetaType,
    ListType,
    MapMetaType,
    MapType
]

/*
 * Phalcom Objects
 */
export const objects = [
    NullObject,
    VoidObject,
    TrueObject,
    FalseObject,
    SelfObject,
    SuperObject
]

/*
 * Phalcom Keywords
 */
export const keywords = [
    "let",
    "const",
    "import",
    "from",
    "if",
    "else",
    "while",
    "for",
    "return",
    "break",
    "continue",
]

/*
 * Phalcom Operators
 */
export const operators = [
    "+",
    "-",
    "*",
    "/",
    "%",
    "=",
    "+=",
    "-=",
    "*=",
    "/=",
    "%=",
    "++",
    "--",
    "==",
    "!=",
    ">",
    "<",
    ">=",
    "<=",
    "is",
    "in",
    "and",
    "or",
    "not",
]