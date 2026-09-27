import { exec } from 'child_process';
import * as vscode from 'vscode';

interface QueuedTask<T> {
    execute: () => Promise<T>;
    resolve: (value: T) => void;
    reject: (error: any) => void;
}

/**
 * Async Command Runner and Task Queue
 * Ensures git operations and CLI subprocesses are queued asynchronously,
 * debounced, and never freeze the VS Code UI thread or race with each other.
 */
export class AsyncRunner {
    private queue: QueuedTask<any>[] = [];
    private isRunning = false;
    private debounceTimers: Map<string, NodeJS.Timeout> = new Map();

    /**
     * Executes a CLI command asynchronously within the queue
     */
    public runCommand(cmd: string, cwd: string): Promise<string> {
        return this.enqueue(() => {
            return new Promise<string>((resolve, reject) => {
                exec(cmd, { cwd }, (error, stdout, stderr) => {
                    if (error) {
                        reject(new Error(stderr.trim() || stdout.trim() || error.message));
                    } else {
                        resolve(stdout.trim());
                    }
                });
            });
        });
    }

    /**
     * Debounces repetitive actions (like rapid saving or git head switches)
     */
    public debounce(key: string, fn: () => void, delayMs: number = 300) {
        if (this.debounceTimers.has(key)) {
            clearTimeout(this.debounceTimers.get(key)!);
        }
        this.debounceTimers.set(key, setTimeout(() => {
            this.debounceTimers.delete(key);
            fn();
        }, delayMs));
    }

    private enqueue<T>(execute: () => Promise<T>): Promise<T> {
        return new Promise<T>((resolve, reject) => {
            this.queue.push({ execute, resolve, reject });
            this.processNext();
        });
    }

    private async processNext() {
        if (this.isRunning || this.queue.length === 0) {
            return;
        }

        this.isRunning = true;
        const task = this.queue.shift();

        if (task) {
            try {
                const result = await task.execute();
                task.resolve(result);
            } catch (err) {
                task.reject(err);
            } finally {
                this.isRunning = false;
                this.processNext();
            }
        } else {
            this.isRunning = false;
        }
    }
}

export const runner = new AsyncRunner();
