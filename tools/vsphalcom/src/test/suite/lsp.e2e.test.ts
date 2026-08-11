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

async function completionLabels(
    document: vscode.TextDocument,
    receiverNeedle: string
): Promise<string[]> {
    const offset = document.getText().indexOf(receiverNeedle);
    assert.ok(offset >= 0, `needle not found: ${receiverNeedle}`);

    // receiverNeedle includes its trailing dot.
    const position = document.positionAt(offset + receiverNeedle.length);

    const list =
        await vscode.commands.executeCommand<vscode.CompletionList>(
            'vscode.executeCompletionItemProvider',
            document.uri,
            position,
            '.'
        );

    assert.ok(list, 'VS Code returned no CompletionList');
    return list.items.map(item => item.label.toString());
}

suite('Phalcom LSP extension E2E', () => {
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
                'const dog = Dog.new()',
                'dog.bark()',
                ''
            ].join('\n');

            const file = path.join(tempDir, 'completion.ph');
            fs.writeFileSync(file, source);

            const document =
                await vscode.workspace.openTextDocument(file);
            await vscode.window.showTextDocument(document);

            const labels =
                await completionLabels(document, 'dog.');

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
                    'const value = Cat.new()',
                    'value.meow()',
                    ''
                ].join('\n')
            );

            const document =
                await vscode.workspace.openTextDocument(file);
            const editor =
                await vscode.window.showTextDocument(document);

            let labels =
                await completionLabels(document, 'value.');
            assert.ok(
                labels.includes('meow()'),
                `initial Cat completion missing: ${labels}`
            );

            const replacement = [
                'class Dog {',
                '  bark() {}',
                '}',
                'const value = Dog.new()',
                'value.bark()',
                ''
            ].join('\n');

            const fullRange = new vscode.Range(
                document.positionAt(0),
                document.positionAt(
                    document.getText().length
                )
            );

            const applied = await editor.edit(edit => {
                edit.replace(fullRange, replacement);
            });
            assert.ok(applied, 'VS Code rejected live-edit replacement');

            labels =
                await completionLabels(document, 'value.');

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
