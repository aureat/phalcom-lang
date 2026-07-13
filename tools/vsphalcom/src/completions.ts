import {
    CompletionItemProvider,
    CompletionItemKind,
    TextDocument,
    Position,
    CompletionItem,
    CancellationToken,
    ProviderResult
} from "vscode"
import { keywords, operators } from "./language"
import { isPositionInComment, isPositionInString } from "./util"


export const PhalcomCompletionKind = {
    Keyword: CompletionItemKind.Keyword,
    Operator: CompletionItemKind.Operator,
    Type: CompletionItemKind.Class,
    Var: CompletionItemKind.Variable,
    Method: CompletionItemKind.Method,
    Field: CompletionItemKind.Field,
    PseudoVar: CompletionItemKind.Value,
    PseudoField: CompletionItemKind.Struct
}

export class PhalcomCompletionProvider implements CompletionItemProvider {

    private context: PhalcomContext

    constructor(private phalcomContext: PhalcomContext) {
        this.context = phalcomContext
    }

    provideCompletionItems(
        document: TextDocument,
        position: Position,
        token: CancellationToken): CompletionItem[] 
    {
        const filename = document.fileName
        const lineText = document.lineAt(position.line).text
        const lineTillCurrentPosition = lineText.substring(0, position.character)

        // Check if we're in a comment or string
        if (isPositionInComment(document, position) || isPositionInString(document, position)) {
            return []
        }

        // Get current word and check if it's a number
        const currentWord = getCurrentWord(document, position)
        if (currentWord.match(/^\d+$/)) {
            return []
        }

        const completions: CompletionItem[] = []

        // Push all completions
        completions.push(...getGlobalsCompletions(this.context, currentWord))
        completions.push(...getKeywordCompletions(currentWord))
        completions.push(...getOperatorCompletions(currentWord))

        if (!completions || completions.length === 0) {
            completions.push(...getLocalsCompletions(this.context, currentWord))
        }

        // If no completions, get methods
        if ((!completions || completions.length === 0) && lineTillCurrentPosition.endsWith('.')) {
            Object.keys(this.context.types).forEach(typeName => {
                let type = this.context.types[typeName]
                type.methods.forEach(method => {
                    completions.push({
                        label: method.signatureString,
                        kind: PhalcomCompletionKind.Method,
                        filterText: typeName + "." + method.signatureString,
                        detail: "method of " + typeName,
                        insertText: getInsertTextFromSignature(method.signature),
                        documentation: "Method of objects with type " + typeName + "."
                    })
                })
            })
        }

        return completions

    }
}

function getCurrentWord(document: TextDocument, position: Position): string {
    const wordAtPosition = document.getWordRangeAtPosition(position)
	let currentWord = ''
	if (wordAtPosition && wordAtPosition.start.character < position.character) {
		const word = document.getText(wordAtPosition)
		currentWord = word.substring(0, position.character - wordAtPosition.start.character)
	}

	return currentWord
}

function getInsertTextFromSignature(signature: PhalcomSig) {
    let methodString = signature.name
    if (signature.parameters) {
        methodString += "("
        if (signature.parameters.length) {
            methodString += signature.parameters.map(param => param.name + ": ").join(", ")
        }
        methodString += ")"
    }
    return methodString
}

function getMethodsCompletions(methods: PhalcomMethod[], currentWord: string): CompletionItem[] {
    return methods.filter(method => method.signatureString.startsWith(currentWord)).map(method => {
        let item = new CompletionItem(method.signatureString, PhalcomCompletionKind.Method)
        item.insertText = getInsertTextFromSignature(method.signature)
        return item
    })
}

function getCompletionKindFromName(context: PhalcomContext, kind: string): CompletionItemKind {

    // if it's in the types, it's a type
    if (context.types.hasOwnProperty(kind)) {
        return PhalcomCompletionKind.Type
    }

    // if it's in either locals or globals, it's a variable
    if (context.locals.hasOwnProperty(kind) || context.globals.hasOwnProperty(kind)) {
        return PhalcomCompletionKind.Var
    }

    // if it starts with a capital letter, it's likely a class
    if (kind.match(/^[A-Z]/)) {
        return PhalcomCompletionKind.Type
    }

    // null, void, true, false are all global objects
    if (['null', 'void', 'true', 'false'].includes(kind)) {
        return PhalcomCompletionKind.PseudoVar
    }

    // self and super are references in a method
    if (['self', 'super'].includes(kind)) {
        return PhalcomCompletionKind.PseudoField
    }

    // otherwise it's probably a field
    return PhalcomCompletionKind.Field

}

function getCompletionsFromList(list: any[], currentWord: string, context: PhalcomContext): CompletionItem[] {
    if (!currentWord.length) return []
    return list.filter(item => item.startsWith(currentWord)).map(item => {
        return new CompletionItem(item, getCompletionKindFromName(context, item))
    })
}

function getKeywordCompletions(currentWord: string): CompletionItem[] {
    if (!currentWord.length) {
        return []
    }

    return keywords.filter(keyword => keyword.startsWith(currentWord)).map(keyword => {
        return new CompletionItem(keyword, PhalcomCompletionKind.Keyword)
    })
}

function getOperatorCompletions(currentWord: string): CompletionItem[] {
    if (!currentWord.length) {
        return []
    }

    return operators.filter(operator => operator.startsWith(currentWord)).map(operator => {
        return new CompletionItem(operator, PhalcomCompletionKind.Operator)
    })
}

function getGlobalsCompletions(context: PhalcomContext, currentWord: string): CompletionItem[] {
    if (!currentWord.length) {
        return []
    }

    return Object.keys(context.globals).filter(global => global.startsWith(currentWord)).map(global => {
        return new CompletionItem(global, getCompletionKindFromName(context, global))
    })
}

function getLocalsCompletions(context: PhalcomContext, currentWord: string): CompletionItem[] {
    if (!currentWord.length) {
        return []
    }

    return Object.keys(context.locals).filter(local => local.startsWith(currentWord)).map(local => {
        return new CompletionItem(local, getCompletionKindFromName(context, local))
    })
}