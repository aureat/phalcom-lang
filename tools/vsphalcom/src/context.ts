import { objects, types } from "./language";
import { parseSignatureString } from "./util";


export const phalcomGlobal: PhalcomGlobal = {
    context: null
}

export function createContext(): PhalcomContext {
    let context: PhalcomContext = {
        globals: {},
        locals: {},
        types: {},
    }

    // parse types and methods
    types.forEach(type => {
        context.types[type.name] = {
            super: null,
            type: null,
            methods: type.methods.map((signatureString: PhalcomSigString) => {
                let signature = parseSignatureString(signatureString)
                return { signatureString, signature }
            })
        }
    })

    // set super and type
    types.forEach(type => {
        context.types[type.name].super = context.types[type.super?.name]
        context.types[type.name].type = context.types[type.type?.name]
    })

    // add globals
    types.filter(type => type.isGlobal).forEach(type => context.globals[type.name] = type.type.name)
    objects.filter(type => type.isGlobal).forEach(object => context.globals[object.name] = object.type.name)

    // add locals
    objects.filter(type => type.isLocal).forEach(object => context.locals[object.name] = object.type.name)

    phalcomGlobal.context = context

    return context
}

export function destroyContext(): void {
    phalcomGlobal.context = null
}