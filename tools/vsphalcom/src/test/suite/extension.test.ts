import * as assert from 'assert'
import * as vscode from 'vscode'

suite('Phalcom extension', () => {
    suiteSetup(async () => {
        const extension = vscode.extensions.all.find(candidate => candidate.packageJSON.name === 'vsphalcom')
        assert.ok(extension, 'Phalcom extension must be installed for integration tests')
        await extension.activate()
    })

    test('registers language-server lifecycle commands', async () => {
        const commands = await vscode.commands.getCommands(true)
        assert.ok(commands.includes('phalcom.restartLanguageServer'))
        assert.ok(commands.includes('phalcom.showLanguageServerOutput'))
    })

    test('contributes standard LSP inlay-hint settings', () => {
        const configuration = vscode.workspace.getConfiguration('phalcom')
        assert.strictEqual(configuration.get<string>('inlayHints.types'), 'stable')
        assert.strictEqual(configuration.get<boolean>('inlayHints.suppressObvious'), true)
    })
})
