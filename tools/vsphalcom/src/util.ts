import { Position, TextDocument } from "vscode";

export function parseSignatureString(signature: PhalcomSigString): PhalcomSig {
    let result: PhalcomSig = {
        name: '',
        parameters: null,
        type: null
    };

    // separate return type if it exists
    let lastColonIndex = signature.lastIndexOf(':');
    let returnType = null;
    if (lastColonIndex !== -1) {
        returnType = signature.slice(lastColonIndex + 1).trim();
        signature = signature.slice(0, lastColonIndex);
    }

    // separate the function name and parameters
    let [name, parameters] = signature.split('(').map(s => s.trim());
    result.name = name;

    // if there are parameters, parse them
    if(parameters && parameters.endsWith(')')) {
        parameters = parameters.slice(0, -1).trim();
        result.parameters = parameters.split(',').map(parameter => {
            let [name, type] = parameter.split(':').map(s => s.trim());
            return { name, type };
        });
    }

    result.type = returnType;

    return result;
}

/**
 * Returns a boolean whether the current position lies within a string or not
 * @param document
 * @param position
 */
export function isPositionInString(document: TextDocument, position: Position): boolean {
	const lineText = document.lineAt(position.line).text;
	const lineTillCurrentPosition = lineText.substring(0, position.character);

	// Count the number of double quotes in the line till current position. Ignore escaped double quotes
	let doubleQuotesCnt = (lineTillCurrentPosition.match(/"/g) || []).length;
	const escapedDoubleQuotesCnt = (lineTillCurrentPosition.match(/\\"/g) || []).length;

	doubleQuotesCnt -= escapedDoubleQuotesCnt;
	return doubleQuotesCnt % 2 === 1;
}

/**
 * Returns a boolean whether the current position lies within a comment or not
 * @param document
 * @param position
 */
export function isPositionInComment(document: TextDocument, position: Position): boolean {
	const lineText = document.lineAt(position.line).text;
	const commentIndex = lineText.indexOf('//');

	if (commentIndex >= 0 && position.character > commentIndex) {
		const commentPosition = new Position(position.line, commentIndex);
		const isCommentInString = isPositionInString(document, commentPosition);

		return !isCommentInString;
	}
	return false;
}