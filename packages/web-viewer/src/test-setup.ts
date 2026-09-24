import { GlobalWindow } from 'happy-dom';
import { cleanup } from '@testing-library/preact';
import { afterEach } from 'bun:test';

const window = new GlobalWindow({ url: 'http://localhost:3000' });
globalThis.window = window as unknown as Window & typeof globalThis;
globalThis.document = window.document as unknown as Document;
globalThis.navigator = window.navigator as unknown as Navigator;
globalThis.HTMLElement = window.HTMLElement as unknown as typeof HTMLElement;
globalThis.Element = window.Element as unknown as typeof Element;
globalThis.Node = window.Node as unknown as typeof Node;

afterEach(() => {
    cleanup();
});
