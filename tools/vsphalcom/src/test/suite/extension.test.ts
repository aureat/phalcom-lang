import * as assert from 'assert'
import * as vscode from 'vscode'
import { createLspClientLifecycle, LspClientLifecycleHandle } from '../../extension'

function fakeClient(stop: () => Promise<void>): LspClientLifecycleHandle & { disposeCalls: number } {
    return {
        stop,
        disposeCalls: 0,
        dispose() {
            this.disposeCalls += 1
        }
    }
}

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

    test('starts replacement after stop failure while enabled', async () => {
        let enabled = true
        const oldClient = fakeClient(() => Promise.reject(new Error('shutdown failed')))
        let current: LspClientLifecycleHandle | undefined = oldClient
        const replacement = fakeClient(async () => undefined)
        const logs: string[] = []
        let starts = 0
        const lifecycle = createLspClientLifecycle({
            getClient: () => current,
            setClient: client => { current = client },
            isEnabled: () => enabled,
            start: () => {
                starts += 1
                return replacement
            },
            log: message => logs.push(message)
        })

        await lifecycle.restart()

        assert.strictEqual(starts, 1)
        assert.strictEqual(current, replacement)
        assert.strictEqual(oldClient.disposeCalls, 1)
        assert.strictEqual(replacement.disposeCalls, 0)
        assert.ok(logs.some(message => message.includes('shutdown failed')))
        enabled = false
    })

    test('serializes overlapping restarts and disposes each replaced client', async () => {
        let resolveStop: (() => void) | undefined
        let current: (LspClientLifecycleHandle & { disposeCalls: number }) | undefined = fakeClient(
            () => new Promise<void>(resolve => { resolveStop = resolve })
        )
        const replacement = fakeClient(async () => undefined)
        const finalClient = fakeClient(async () => undefined)
        const clients = [replacement, finalClient]
        let starts = 0
        const lifecycle = createLspClientLifecycle({
            getClient: () => current,
            setClient: client => { current = client as typeof current },
            isEnabled: () => true,
            start: () => clients[starts++],
            log: () => undefined
        })

        const firstRestart = lifecycle.restart()
        const secondRestart = lifecycle.restart()
        await Promise.resolve()
        assert.strictEqual(starts, 0)
        resolveStop?.()
        await Promise.all([firstRestart, secondRestart])

        assert.strictEqual(starts, 2)
        assert.strictEqual(current, finalClient)
        assert.strictEqual((current === finalClient ? replacement : undefined)?.disposeCalls, 1)
    })
})
