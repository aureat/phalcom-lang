import * as assert from 'assert';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import * as vscode from 'vscode';

function repositoryRoot(): string {
    // Compiled test location:
    // tools/vsphalcom/out/test/suite/lsp.e2e.test.js
    return path.resolve(__dirname, '../../../../..');
}

function lspBinary(): string {
    const executable =
        process.platform === 'win32' ? 'phalcom-lsp.exe' : 'phalcom-lsp';
    return path.join(
        repositoryRoot(),
        'target',
        'debug',
        executable
    );
}

interface AnalysisPublicationStatus {
    phase?: string;
    complete?: boolean;
    generation?: number;
    analyzedFiles?: number;
    message?: string;
}

async function analysisStatus(): Promise<AnalysisPublicationStatus | undefined> {
    return vscode.commands.executeCommand<AnalysisPublicationStatus>('phalcom.inspectAnalysisStatus');
}

/**
 * The first source update starts a cold compiler-owned Universe session in the
 * debug language server. Wait for publication explicitly: a live LSP client
 * and syntax-word completions do not imply a canonical semantic snapshot.
 * This is a correctness gate, not an assertion on compiler startup latency.
 */
async function awaitSemanticPublication(previousGeneration: number, document: vscode.TextDocument): Promise<number> {
    const deadline = Date.now() + 180_000;
    let last: AnalysisPublicationStatus | undefined;
    while (Date.now() < deadline) {
        last = await analysisStatus();
        if (last?.phase === 'error') {
            assert.fail(`Phalcom analysis failed: ${JSON.stringify(last)}`);
        }
        if (last?.phase === 'ready' && last.complete && last.generation !== undefined
            && last.generation > previousGeneration && (last.analyzedFiles ?? 0) > 0) {
            return last.generation;
        }
        await new Promise(resolve => setTimeout(resolve, 250));
    }
    assert.fail(
        `Canonical semantic publication never completed for ${document.uri}: ${JSON.stringify({ status: last, diagnostics: vscode.languages.getDiagnostics(document.uri).map(diagnostic => diagnostic.message) })}`
    );
}

async function completionLabels(
    document: vscode.TextDocument,
    receiverNeedle: string,
    predicate?: (labels: string[]) => boolean
): Promise<string[]> {
    const offset = document.getText().indexOf(receiverNeedle);
    assert.ok(offset >= 0, `needle not found: ${receiverNeedle}`);

    // receiverNeedle includes its trailing dot.
    const position = document.positionAt(offset + receiverNeedle.length);

    let lastLabels: string[] = [];
    for (let attempt = 0; attempt < 250; attempt++) {
        const list = await vscode.commands.executeCommand<vscode.CompletionList>(
            'vscode.executeCompletionItemProvider',
            document.uri,
            position,
            '.'
        );
        if (list && list.items.length > 0) {
            lastLabels = list.items.map(item => item.label.toString());
            if (!predicate || predicate(lastLabels)) {
                return lastLabels;
            }
        }
        await new Promise(resolve => setTimeout(resolve, 100));
    }

    if (predicate && !predicate(lastLabels)) {
        const status = await vscode.commands.executeCommand('phalcom.inspectAnalysisStatus');
        console.error(
            `Phalcom completion remained unresolved: ${JSON.stringify({ needle: receiverNeedle, labels: lastLabels, status, diagnostics: vscode.languages.getDiagnostics(document.uri).map(diagnostic => diagnostic.message) })}`
        );
    }
    return lastLabels;
}

suite('Phalcom LSP extension E2E', function () {
    // Completion probes deliberately poll for semantic publication; Mocha's
    // default two-second limit expires before this contract can be observed.
    this.timeout(240_000);
    let tempDir: string;

    suiteSetup(async () => {
        const server = lspBinary();

        assert.ok(
            fs.existsSync(server),
            `phalcom-lsp was not built at ${server}; run cargo build -p phalcom-lsp first`
        );

        // Configure before opening any .ph document so extension activation
        // observes the test server path.
        await vscode.workspace
            .getConfiguration('phalcom')
            .update(
                'lsp.serverPath',
                server,
                vscode.ConfigurationTarget.Global
            );

        await vscode.workspace
            .getConfiguration('phalcom')
            .update(
                'lsp.enabled',
                true,
                vscode.ConfigurationTarget.Global
            );

        // An open .ph editor alone is not evidence that the language client
        // has completed its asynchronous extension activation.
        const extension = vscode.extensions.getExtension('Phalcom.vsphalcom');
        assert.ok(extension, 'Phalcom VS Code extension is not installed');
        await extension.activate();
        await vscode.commands.executeCommand('phalcom.restartLanguageServer');

        tempDir = fs.mkdtempSync(
            path.join(os.tmpdir(), 'vsphalcom-e2e-')
        );
    });

    suiteTeardown(() => {
        if (tempDir) {
            fs.rmSync(tempDir, {
                recursive: true,
                force: true
            });
        }
    });

    test(
        'declared and inherited members reach VS Code completion',
        async () => {
            const source = [
                'class Animal {',
                '  move() {}',
                '}',
                '',
                'class Dog is Animal {',
                '  bark() {}',
                '}',
                '',
                'let dog = Dog.new()',
                'dog.bark()',
                ''
            ].join('\n');

            const file = path.join(tempDir, 'completion.ph');
            fs.writeFileSync(file, source);

            const previousGeneration = (await analysisStatus())?.generation ?? -1;
            const document =
                await vscode.workspace.openTextDocument(file);
            await vscode.window.showTextDocument(document);
            await awaitSemanticPublication(previousGeneration, document);

            const labels =
                await completionLabels(
                    document,
                    'dog.',
                    labels => labels.includes('bark()') && labels.includes('move()')
                );

            assert.ok(
                labels.includes('bark()'),
                `missing declared member: ${labels}`
            );
            assert.ok(
                labels.includes('move()'),
                `missing inherited member: ${labels}`
            );
        }
    );

    test(
        'live edit changes completion without restarting extension',
        async () => {
            const file = path.join(tempDir, 'live-edit.ph');
            fs.writeFileSync(
                file,
                [
                    'class Cat {',
                    '  meow() {}',
                    '}',
                    'let value = Cat.new()',
                    'value.meow()',
                    ''
                ].join('\n')
            );

            const previousGeneration = (await analysisStatus())?.generation ?? -1;
            const document =
                await vscode.workspace.openTextDocument(file);
            const editor =
                await vscode.window.showTextDocument(document);
            await awaitSemanticPublication(previousGeneration, document);

            let labels =
                await completionLabels(
                    document,
                    'value.',
                    labels => labels.includes('meow()')
                );
            assert.ok(
                labels.includes('meow()'),
                `initial Cat completion missing: ${labels}`
            );

            const replacement = [
                'class Dog {',
                '  bark() {}',
                '}',
                'let value = Dog.new()',
                'value.bark()',
                ''
            ].join('\n');

            const fullRange = new vscode.Range(
                document.positionAt(0),
                document.positionAt(
                    document.getText().length
                )
            );

            const beforeEditGeneration = (await analysisStatus())?.generation ?? -1;
            const applied = await editor.edit(edit => {
                edit.replace(fullRange, replacement);
            });
            assert.ok(applied, 'VS Code rejected live-edit replacement');
            await awaitSemanticPublication(beforeEditGeneration, document);

            labels =
                await completionLabels(
                    document,
                    'value.',
                    labels => labels.includes('bark()') && !labels.includes('meow()')
                );

            assert.ok(
                labels.includes('bark()'),
                `updated Dog completion missing: ${labels}`
            );
            assert.ok(
                !labels.includes('meow()'),
                `stale Cat completion survived didChange: ${labels}`
            );
        }
    );
});
