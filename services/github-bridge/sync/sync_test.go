package sync

import (
	"context"
	"fmt"
	"testing"

	"github.com/git-notes/github-bridge/github"
	"github.com/git-notes/github-bridge/notes"
)

type mockClient struct {
	comments []github.PRComment
	err      error
}

func (m *mockClient) GetPRComments(ctx context.Context, owner, repo string, prNum int) ([]github.PRComment, error) {
	if m.err != nil {
		return nil, m.err
	}
	return m.comments, nil
}

type mockNotes struct {
	existing []notes.Note
	written  []notes.Note
	readErr  error
	writeErr error
}

func (m *mockNotes) ReadNotesJSON(namespace string) ([]notes.Note, error) {
	if m.readErr != nil {
		return nil, m.readErr
	}
	return m.existing, nil
}

func (m *mockNotes) WriteNote(note notes.Note) error {
	if m.writeErr != nil {
		return m.writeErr
	}
	m.written = append(m.written, note)
	return nil
}

func TestExtractCommentID(t *testing.T) {
	tests := []struct {
		body     string
		expected int64
		found    bool
	}{
		{"Hello world\n\n[github-comment-id: 12345]", 12345, true},
		{"Some comment [github-comment-id: 999]", 999, true},
		{"[github-comment-id:   42  ]", 42, true},
		{"No tag here", 0, false},
		{"[github-comment-id: invalid]", 0, false},
	}

	for _, tt := range tests {
		id, found := extractCommentID(tt.body)
		if found != tt.found {
			t.Errorf("extractCommentID(%q) found = %v, want %v", tt.body, found, tt.found)
		}
		if found && id != tt.expected {
			t.Errorf("extractCommentID(%q) id = %d, want %d", tt.body, id, tt.expected)
		}
	}
}

func TestStripMetaTag(t *testing.T) {
	body := "Fix this issue\n\n[github-comment-id: 100]"
	cleaned := stripMetaTag(body)
	if cleaned != "Fix this issue" {
		t.Errorf("stripMetaTag got %q, want %q", cleaned, "Fix this issue")
	}

	bodyWithoutTag := "No tag here"
	cleaned2 := stripMetaTag(bodyWithoutTag)
	if cleaned2 != "No tag here" {
		t.Errorf("stripMetaTag got %q, want %q", cleaned2, "No tag here")
	}
}

func TestSyncPRToNotes_Deduplication(t *testing.T) {
	ctx := context.Background()

	client := &mockClient{
		comments: []github.PRComment{
			{ID: 101, Path: "main.go", Line: 10, Body: "Fix typo"},
			{ID: 102, Path: "main.go", Line: 20, Body: "Refactor function"},
			{ID: 103, Path: "main.go", Line: 0, Body: "Zero line comment"}, // empty path/line test (Line 0)
			{ID: 104, Path: "", Line: 10, Body: "PR level comment"},        // empty path test
			{ID: 105, Path: "main.go", Line: 10, Body: "Fix typo"},         // duplicate fingerprint in same batch
		},
	}

	existingNotes := &mockNotes{
		existing: []notes.Note{
			{
				Body:      "Refactor function\n\n[github-comment-id: 102]",
				File:      "main.go",
				Line:      20,
				Namespace: "review",
			},
		},
	}

	err := SyncPRToNotes(ctx, client, existingNotes, "owner", "repo", 1)
	if err != nil {
		t.Fatalf("SyncPRToNotes failed: %v", err)
	}

	// Should only write 101 ("Fix typo").
	// 102 is skipped (comment ID match and fingerprint match)
	// 103 has line 0/empty logic? 103 line is 30, but path is main.go body is "".
	// 104 has path "" so skipped.
	// 105 has same fingerprint as 101 ("main.go:10:Fix typo"), so skipped.

	if len(existingNotes.written) != 1 {
		t.Fatalf("expected 1 written note, got %d", len(existingNotes.written))
	}

	expectedBody := "Fix typo\n\n[github-comment-id: 101]"
	if existingNotes.written[0].Body != expectedBody {
		t.Errorf("expected body %q, got %q", expectedBody, existingNotes.written[0].Body)
	}
}

func TestSyncPRToNotes_ClientError(t *testing.T) {
	ctx := context.Background()
	client := &mockClient{err: fmt.Errorf("network error")}
	notesStore := &mockNotes{}

	err := SyncPRToNotes(ctx, client, notesStore, "owner", "repo", 1)
	if err == nil {
		t.Error("expected error, got nil")
	}
}

func TestSyncPRToNotes_NotesReadError(t *testing.T) {
	ctx := context.Background()
	client := &mockClient{comments: []github.PRComment{{ID: 1, Path: "a.go", Line: 1, Body: "test"}}}
	notesStore := &mockNotes{readErr: fmt.Errorf("read error")}

	err := SyncPRToNotes(ctx, client, notesStore, "owner", "repo", 1)
	if err == nil {
		t.Error("expected error, got nil")
	}
}
