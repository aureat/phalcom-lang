import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';

import { runTests } from '@vscode/test-electron';

async function main() {
	try {
		// The folder containing the Extension Manifest package.json
		// Passed to `--extensionDevelopmentPath`
		const extensionDevelopmentPath = path.resolve(__dirname, '../../');

		// The path to test runner
		// Passed to --extensionTestsPath
		const extensionTestsPath = path.resolve(__dirname, './suite/index');

		// Exercise the real workspace module/semantic pipeline rather than
		// an unrelated loose-file path. The direct LSP harness also provides
		// a workspace root and a package manifest to the server.
		const projectDir = fs.mkdtempSync(path.join(os.tmpdir(), 'vsphalcom-e2e-'));
		fs.writeFileSync(path.join(projectDir, 'package.ph'), '');

		// macOS limits the IPC socket path to 103 characters. Worktrees can be
		// nested deeply enough that the runner's default user-data directory
		// exceeds that limit before any extension test starts.
		const userDataDir = path.join(os.tmpdir(), 'phalcom-vscode-test');
		try {
			await runTests({
				extensionDevelopmentPath,
				extensionTestsPath,
				launchArgs: [projectDir, `--user-data-dir=${userDataDir}`],
			});
		} finally {
			fs.rmSync(projectDir, { recursive: true, force: true });
		}
	} catch (err) {
		console.error('Failed to run tests', err);
		process.exit(1);
	}
}

main();
