import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import * as https from 'https';
import { exec } from 'child_process';

const LATEST_RELEASE_API = 'https://api.github.com/repos/isaim0011/git-notes/releases/latest';

export interface BinaryResolution {
    command: string;
    isBundled: boolean;
}

/**
 * Checks if a command can be executed successfully
 */
function isBinaryExecutable(cmd: string): Promise<boolean> {
    return new Promise((resolve) => {
        exec(`"${cmd}" --version`, (err) => {
            resolve(!err);
        });
    });
}

/**
 * Downloads a file following HTTP 3xx redirects
 */
function downloadFile(url: string, destPath: string): Promise<void> {
    return new Promise((resolve, reject) => {
        const fileStream = fs.createWriteStream(destPath);
        const options = {
            headers: {
                'User-Agent': 'vscode-git-notes-extension'
            }
        };

        const getWithRedirect = (currentUrl: string) => {
            https.get(currentUrl, options, (res) => {
                if (res.statusCode && res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
                    getWithRedirect(res.headers.location);
                    return;
                }
                if (res.statusCode !== 200) {
                    reject(new Error(`Download failed with status ${res.statusCode}`));
                    return;
                }
                res.pipe(fileStream);
                fileStream.on('finish', () => {
                    fileStream.close();
                    resolve();
                });
            }).on('error', (err) => {
                fs.unlink(destPath, () => {});
                reject(err);
            });
        };

        getWithRedirect(url);
    });
}

/**
 * Ensure git-notes binary exists or automatically download from GitHub releases
 */
export async function ensureGitNotesBinary(context: vscode.ExtensionContext): Promise<string> {
    const config = vscode.workspace.getConfiguration('git-notes');
    const configuredPath = config.get<string>('binaryPath', 'git-notes');

    // 1. If configured path is explicitly custom and works, use it
    if (configuredPath !== 'git-notes' && configuredPath !== 'gn') {
        if (await isBinaryExecutable(configuredPath)) {
            return configuredPath;
        }
    }

    // 2. Check if git-notes or gn is on system PATH
    if (await isBinaryExecutable('git-notes')) {
        return 'git-notes';
    }
    if (await isBinaryExecutable('gn')) {
        return 'gn';
    }

    // 3. Check extension global storage directory for previously downloaded binary
    const storageDir = context.globalStorageUri.fsPath;
    if (!fs.existsSync(storageDir)) {
        fs.mkdirSync(storageDir, { recursive: true });
    }

    const platform = os.platform();
    const arch = os.arch();
    let binaryName = 'git-notes';
    let assetName = '';

    if (platform === 'win32') {
        binaryName = 'git-notes.exe';
        assetName = 'git-notes.exe';
    } else if (platform === 'linux') {
        assetName = 'git-notes-linux-x86_64';
    } else if (platform === 'darwin') {
        assetName = arch === 'arm64' ? 'git-notes-macos-aarch64' : 'git-notes-macos-x86_64';
    }

    const localBinaryPath = path.join(storageDir, binaryName);
    if (fs.existsSync(localBinaryPath)) {
        if (await isBinaryExecutable(localBinaryPath)) {
            return localBinaryPath;
        }
    }

    // 4. Prompt user or auto-install
    const action = await vscode.window.showInformationMessage(
        'git-notes CLI binary not found on PATH. Would you like to automatically download it to enable gutter annotations and inline code reviews?',
        'Download & Install',
        'Configure Custom Path'
    );

    if (action === 'Configure Custom Path') {
        await vscode.commands.executeCommand('workbench.action.openSettings', 'git-notes.binaryPath');
        throw new Error('Please configure the path to git-notes in Settings.');
    }

    if (action !== 'Download & Install') {
        throw new Error('git-notes CLI binary is required for git-notes operations.');
    }

    // 5. Download the platform binary
    return await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: 'Downloading git-notes CLI binary...',
        cancellable: false
    }, async (progress) => {
        progress.report({ increment: 20, message: 'Fetching release assets...' });

        // Default direct GitHub Release URL fallback if API is rate limited
        let downloadUrl = `https://github.com/isaim0011/git-notes/releases/latest/download/${assetName}`;
        if (platform === 'win32') {
            downloadUrl = 'https://github.com/isaim0011/git-notes/releases/latest/download/git-notes.exe';
        }

        try {
            progress.report({ increment: 50, message: `Downloading ${assetName}...` });
            await downloadFile(downloadUrl, localBinaryPath);

            // Make executable on unix
            if (platform !== 'win32') {
                fs.chmodSync(localBinaryPath, 0o755);
            }

            progress.report({ increment: 30, message: 'Verifying binary...' });
            if (await isBinaryExecutable(localBinaryPath)) {
                vscode.window.showInformationMessage(`git-notes CLI binary successfully installed to ${localBinaryPath}!`);
                return localBinaryPath;
            } else {
                throw new Error('Downloaded binary verification failed.');
            }
        } catch (err: any) {
            vscode.window.showErrorMessage(`Failed to install git-notes CLI: ${err.message}`);
            throw err;
        }
    });
}
