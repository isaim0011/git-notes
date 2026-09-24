import { describe, expect, it, mock } from 'bun:test';
import { render, fireEvent } from '@testing-library/preact';
import { h } from 'preact';
import { FileTree } from './FileTree';
import { Note } from '../types';

describe('FileTree', () => {
    const mockNotes: Note[] = [
        {
            id: '1',
            file_path: 'src/components/App.tsx',
            line_number: 10,
            namespace: 'reviews',
            author: 'Alice',
            timestamp: '2023-01-01T00:00:00Z',
            body: 'First note',
            status: 'open'
        },
        {
            id: '2',
            file_path: 'src/components/App.tsx',
            line_number: 20,
            namespace: 'reviews',
            author: 'Bob',
            timestamp: '2023-01-01T01:00:00Z',
            body: 'Second note on same file',
            status: 'open'
        },
        {
            id: '3',
            file_path: 'README.md',
            line_number: 1,
            namespace: 'general',
            author: 'Charlie',
            timestamp: '2023-01-01T02:00:00Z',
            body: 'Documentation note',
            status: 'resolved'
        }
    ];

    it('renders heading and empty list when no notes are provided', () => {
        const handleSelect = mock(() => {});
        const { getByText, container } = render(
            <FileTree notes={[]} selectedFile={null} onSelectFile={handleSelect} />
        );

        expect(getByText('Files')).toBeTruthy();
        const listItems = container.querySelectorAll('li');
        expect(listItems.length).toBe(0);
    });

    it('groups notes by file_path, displays counts, and sorts files alphabetically', () => {
        const handleSelect = mock(() => {});
        const { container } = render(
            <FileTree notes={mockNotes} selectedFile={null} onSelectFile={handleSelect} />
        );

        const listItems = container.querySelectorAll('li');
        expect(listItems.length).toBe(2);

        // Files sorted alphabetically: 'README.md' comes before 'src/components/App.tsx'
        const firstItem = listItems[0];
        const secondItem = listItems[1];

        // First item: README.md
        const firstSpan = firstItem.querySelector('span:first-child');
        const firstBadge = firstItem.querySelector('span:last-child');
        expect(firstSpan?.textContent).toBe('README.md');
        expect(firstSpan?.getAttribute('title')).toBe('README.md');
        expect(firstBadge?.textContent).toBe('1');

        // Second item: src/components/App.tsx
        const secondSpan = secondItem.querySelector('span:first-child');
        const secondBadge = secondItem.querySelector('span:last-child');
        expect(secondSpan?.textContent).toBe('App.tsx');
        expect(secondSpan?.getAttribute('title')).toBe('src/components/App.tsx');
        expect(secondBadge?.textContent).toBe('2');
    });

    it('highlights selected file correctly', () => {
        const handleSelect = mock(() => {});
        const { container } = render(
            <FileTree notes={mockNotes} selectedFile="README.md" onSelectFile={handleSelect} />
        );

        const listItems = container.querySelectorAll('li');
        const firstItem = listItems[0] as HTMLElement; // README.md
        const secondItem = listItems[1] as HTMLElement; // src/components/App.tsx

        expect(firstItem.style.background).toBe('#e0e0e0');
        expect(secondItem.style.background).toBe('transparent');
    });

    it('calls onSelectFile when a file item is clicked', () => {
        const handleSelect = mock((_file: string) => {});
        const { getByText } = render(
            <FileTree notes={mockNotes} selectedFile={null} onSelectFile={handleSelect} />
        );

        const appTsxItem = getByText('App.tsx');
        fireEvent.click(appTsxItem);

        expect(handleSelect).toHaveBeenCalledTimes(1);
        expect(handleSelect).toHaveBeenCalledWith('src/components/App.tsx');
    });
});
