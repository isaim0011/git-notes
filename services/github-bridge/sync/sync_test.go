package sync

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"reflect"
	"testing"

	"github.com/git-notes/github-bridge/github"
	"github.com/git-notes/github-bridge/notes"
	gh "github.com/google/go-github/v66/github"
)

func setupMockBridge(tb testing.TB, existingNotes []notes.Note, comments []*gh.PullRequestComment) (*github.Client, *notes.Reader) {
	tb.Helper()

	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/repos/owner/repo/pulls/1/comments" {
			w.Header().Set("Content-Type", "application/json")
			_ = json.NewEncoder(w).Encode(comments)
			return
		}
		http.NotFound(w, r)
	}))
	tb.Cleanup(ts.Close)

	baseURL := ts.URL + "/"
	ghClient, err := github.NewClientWithBaseURL("mock-token", baseURL)
	if err != nil {
		tb.Fatalf("failed to create gh client: %v", err)
	}

	tmpDir := tb.TempDir()
	notesFilePath := filepath.Join(tmpDir, "notes.json")
	notesData, err := json.Marshal(existingNotes)
	if err != nil {
		tb.Fatalf("failed to marshal existing notes: %v", err)
	}
	if err := os.WriteFile(notesFilePath, notesData, 0644); err != nil {
		tb.Fatalf("failed to write notes file: %v", err)
	}

	mockBinPath := filepath.Join(tmpDir, "mock-gn.sh")
	scriptContent := fmt.Sprintf(`#!/bin/sh
if [ "$1" = "list" ]; then
  cat "%s"
elif [ "$1" = "add" ]; then
  exit 0
fi
`, notesFilePath)

	if err := os.WriteFile(mockBinPath, []byte(scriptContent), 0755); err != nil {
		tb.Fatalf("failed to write mock binary: %v", err)
	}

	notesReader := notes.NewNotesReader(mockBinPath, tmpDir)
	return ghClient, notesReader
}

func TestSyncPRToNotes(t *testing.T) {
	ctx := context.Background()

	existingNotes := []notes.Note{
		{
			ID:   "note1",
			Body: "Existing review comment\n\n[github-comment-id: 101]",
			File: "main.go",
			Line: 10,
		},
	}

	comments := []*gh.PullRequestComment{
		{
			ID:   gh.Int64(101),
			Body: gh.String("Existing review comment"),
			Path: gh.String("main.go"),
			Line: gh.Int(10),
		},
		{
			ID:   gh.Int64(102),
			Body: gh.String("New review comment"),
			Path: gh.String("main.go"),
			Line: gh.Int(20),
		},
		{
			ID:   gh.Int64(103),
			Body: gh.String("PR level comment without line"),
			Path: gh.String(""),
			Line: gh.Int(0),
		},
	}

	ghClient, notesReader := setupMockBridge(t, existingNotes, comments)

	err := SyncPRToNotes(ctx, ghClient, notesReader, "owner", "repo", 1)
	if err != nil {
		t.Fatalf("SyncPRToNotes failed: %v", err)
	}
}

func TestParseCommentIDs(t *testing.T) {
	tests := []struct {
		name     string
		notes    []notes.Note
		expected map[int64]bool
	}{
		{
			name:     "empty notes",
			notes:    []notes.Note{},
			expected: map[int64]bool{},
		},
		{
			name: "single valid comment ID",
			notes: []notes.Note{
				{Body: "Fix this issue\n\n[github-comment-id: 12345]"},
			},
			expected: map[int64]bool{12345: true},
		},
		{
			name: "multiple comment IDs in multiple notes",
			notes: []notes.Note{
				{Body: "First note\n\n[github-comment-id: 101]"},
				{Body: "Second note\n\n[github-comment-id: 102]"},
			},
			expected: map[int64]bool{101: true, 102: true},
		},
		{
			name: "malformed or non-integer tags ignored",
			notes: []notes.Note{
				{Body: "[github-comment-id: abc]"},
				{Body: "[github-comment-id: ]"},
				{Body: "[github-comment-id: 999"},
			},
			expected: map[int64]bool{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := parseCommentIDs(tt.notes)
			if !reflect.DeepEqual(got, tt.expected) {
				t.Errorf("parseCommentIDs() = %v, want %v", got, tt.expected)
			}
		})
	}
}

func BenchmarkSyncPRToNotes(b *testing.B) {
	ctx := context.Background()

	const numExisting = 1000
	const numComments = 1000

	existingNotes := make([]notes.Note, numExisting)
	for i := 0; i < numExisting; i++ {
		existingNotes[i] = notes.Note{
			ID:   fmt.Sprintf("note-%d", i),
			Body: fmt.Sprintf("Comment body %d\n\n[github-comment-id: %d]", i, i),
			File: "main.go",
			Line: i + 1,
		}
	}

	comments := make([]*gh.PullRequestComment, numComments)
	for i := 0; i < numComments; i++ {
		commentID := int64(i)
		if i >= 500 {
			commentID = int64(1000 + i)
		}
		comments[i] = &gh.PullRequestComment{
			ID:   gh.Int64(commentID),
			Body: gh.String(fmt.Sprintf("Comment body %d", i)),
			Path: gh.String("main.go"),
			Line: gh.Int(i + 1),
		}
	}

	ghClient, notesReader := setupMockBridge(b, existingNotes, comments)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = SyncPRToNotes(ctx, ghClient, notesReader, "owner", "repo", 1)
	}
}
